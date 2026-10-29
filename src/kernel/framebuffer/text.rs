use core::fmt;

use spin::MutexGuard;

use super::{BUFFER, Color, OutOfBoundsError, ScreenBuffer, TEXT_BUFFER};

#[allow(dead_code)]
#[derive(Debug)]
pub struct TextBuffer {
    max_char_x: usize,
    max_char_y: usize,
    scale_factor: usize,
    padding_char_x: usize, // padding in pixels for a line of text
    padding_char_y: usize, // padding in pixels for a column of text
}

impl TextBuffer {
    #[expect(clippy::missing_panics_doc)]
    pub fn new(scale_factor: usize) -> Self {
        let buffer = BUFFER.get().expect("Buffer required").lock();
        let max_char_x = buffer.max_x() / (VGA_CHAR_SIZE.0 * scale_factor);
        let padding_char_x = (buffer.max_x() % (VGA_CHAR_SIZE.0 * scale_factor)) * VGA_CHAR_SIZE.1;
        let max_char_y = buffer.max_y() / (VGA_CHAR_SIZE.1 * scale_factor);
        let padding_char_y = (buffer.max_y() % (VGA_CHAR_SIZE.1 * scale_factor)) * VGA_CHAR_SIZE.0;
        Self {
            max_char_x,
            max_char_y,
            scale_factor,
            padding_char_x,
            padding_char_y,
        }
    }

    fn get_buffer() -> MutexGuard<'static, ScreenBuffer> {
        BUFFER.get().expect("Buffer required").lock()
    }

    const fn char_size_px(&self) -> (usize, usize) {
        (
            VGA_CHAR_SIZE.0 * self.scale_factor,
            VGA_CHAR_SIZE.1 * self.scale_factor,
        )
    }

    const fn char_coord_to_buffer_coord(&self, char_x: usize, char_y: usize) -> (usize, usize) {
        let (w, h) = self.char_size_px();
        (char_x * w, char_y * h)
    }

    #[expect(clippy::similar_names)]
    const fn char_offset_to_buffer_offset(&self, char_dx: isize, char_dy: isize) -> (isize, isize) {
        let (w, h) = self.char_size_px();
        // cast_signed() cause the char size is tiny except if you wanna write letters the size of the planet idk
        (char_dx * w.cast_signed(), char_dy * h.cast_signed())
    }

    // Assumes the slice is of length 16 (16 bytes = 8*16 pixels = char)
    #[allow(clippy::needless_range_loop)]
    fn draw_char_rect(
        &self,
        rect: &[u8],
        char_x: usize,
        char_y: usize,
        background_color: Color,
        foreground_color: Color,
    ) -> Result<(), OutOfBoundsError> {
        if char_x >= self.max_char_x || char_y >= self.max_char_y {
            return Err(OutOfBoundsError::new_char(
                char_x,
                char_y,
                self.max_char_x,
                self.max_char_y,
            ));
        }
        let mut buffer = Self::get_buffer();
        for char_pxl_x in 0..VGA_CHAR_SIZE.0 {
            for char_pxl_y in 0..VGA_CHAR_SIZE.1 {
                let char_pxl = (rect[char_pxl_y] >> (VGA_CHAR_SIZE.0 - 1 - char_pxl_x)) & 1;
                let color = if char_pxl == 0 {
                    background_color
                } else {
                    foreground_color
                };
                let (buffer_x, buffer_y) = self.char_coord_to_buffer_coord(char_x, char_y);
                for i in 0..self.scale_factor {
                    for j in 0..self.scale_factor {
                        buffer.write(
                            buffer_x + char_pxl_x * self.scale_factor + i,
                            buffer_y + char_pxl_y * self.scale_factor + j,
                            color,
                        )?;
                    }
                }
            }
        }
        Ok(())
    }

    fn write_char(
        &self,
        r#char: char,
        char_x: usize,
        char_y: usize,
        background_color: Color,
        foreground_color: Color,
    ) -> Result<(), OutOfBoundsError> {
        let char_rect = nth_char(char_to_font_idx(r#char));
        self.draw_char_rect(
            char_rect,
            char_x,
            char_y,
            background_color,
            foreground_color,
        )
    }

    /// # Errors
    /// This function will return an error if the text is out of bounds of the buffer.
    pub fn write_str(
        &mut self,
        string: &str,
        char_x: usize,
        char_y: usize,
        background_color: Color,
        foreground_color: Color,
    ) -> Result<(), OutOfBoundsError> {
        let mut char_x = char_x;
        let mut char_y = char_y;
        for char_i in string.chars() {
            if char_x >= self.max_char_x {
                char_x = 0;
                char_y += 1;
            }
            if char_i == '\n' {
                char_x = 0;
                char_y += 1;
            } else {
                self.write_char(char_i, char_x, char_y, background_color, foreground_color)?;
                char_x += 1;
            }
        }
        Ok(())
    }

    /// # Errors
    /// This function will return an error if the point is out of bounds of the buffer.
    #[expect(clippy::similar_names)]
    pub fn move_slice_text(
        &mut self,
        char_x: usize,
        char_y: usize,
        text_len: usize,
        char_dx: isize,
        char_dy: isize,
    ) -> Result<(), OutOfBoundsError> {
        let (x, y) = self.char_coord_to_buffer_coord(char_x, char_y);
        let (dx, dy) = self.char_offset_to_buffer_offset(char_dx, char_dy);
        let padding = self.padding_char_x * text_len / self.max_char_x;
        let len =
            text_len * VGA_CHAR_SIZE.0 * VGA_CHAR_SIZE.1 * self.scale_factor * self.scale_factor
                + padding;
        let mut buffer = Self::get_buffer();
        buffer.move_slice(x, y, len, dx, dy)
    }

    fn clear_line(&self, y: usize, background_color: Color) -> Result<(), OutOfBoundsError> {
        for x in 0..self.max_char_x {
            self.write_char(' ', x, y, background_color, background_color)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Writer {
    column_position: usize,
    background_code: Color,
    foreground_code: Color,
}

impl Default for Writer {
    fn default() -> Self {
        Self {
            column_position: 0,
            background_code: Color::new(0, 0, 0, 255),
            foreground_code: Color::new(255, 255, 255, 255),
        }
    }
}

impl Writer {
    fn get_text_buffer() -> MutexGuard<'static, TextBuffer> {
        TEXT_BUFFER.get().expect("Text buffer required").lock()
    }

    pub const fn change_background_color(&mut self, color: Color) {
        self.background_code = color;
    }

    pub const fn change_foreground_color(&mut self, color: Color) {
        self.foreground_code = color;
    }

    /// # Errors
    /// This function will return an error if the text is out of bounds of the buffer.
    pub fn write_str(&mut self, string: &str) -> Result<(), OutOfBoundsError> {
        let mut text_buffer = Self::get_text_buffer();
        for char in string.chars() {
            match char {
                '\n' => self.new_line(&mut text_buffer),
                char => {
                    if self.column_position >= text_buffer.max_char_x {
                        self.new_line(&mut text_buffer);
                    }
                    let max_char_y = text_buffer.max_char_y;
                    text_buffer.write_char(
                        char,
                        self.column_position,
                        max_char_y - 1,
                        self.background_code,
                        self.foreground_code,
                    )?;
                    self.column_position += 1;
                }
            }
        }
        Ok(())
    }

    fn clear_line(&mut self, text_buffer: &MutexGuard<'_, TextBuffer>) {
        let max_char_y = text_buffer.max_char_y;
        text_buffer
            .clear_line(max_char_y - 1, self.background_code)
            .expect("Is not out of bounds as it is a function called manually.");
        self.column_position = 0;
    }

    fn new_line(&mut self, text_buffer: &mut MutexGuard<'_, TextBuffer>) {
        let text_len = text_buffer.max_char_x * (text_buffer.max_char_y - 1);
        text_buffer
            .move_slice_text(0, 1, text_len, 0, -1)
            .expect("Is not out of bound as it is a function called manually.");
        self.clear_line(text_buffer);
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s).expect("Is not out of bounds");
        Ok(())
    }
}

////////////////////////////////////////////

const VGA_FONT: &[u8] = include_bytes!("../../../assets/VGA9.F16");
const VGA_CHAR_SIZE: (usize, usize) = (8, 16);

#[rustfmt::skip]
pub const CP437: [char; 256] = [
    '\0', '☺', '☻', '♥', '♦', '♣', '♠', '•', '◘', '○', '◙', '♂', '♀', '♪', '♫', '☼',
    '►',  '◄', '↕', '‼', '¶', '§', '▬', '↨', '↑', '↓', '→', '←', '∟', '↔', '▲', '▼',
    ' ',  '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/',
    '0',  '1', '2', '3', '4', '5', '6', '7', '8', '9', ':', ';', '<', '=', '>', '?',
    '@',  'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O',
    'P',  'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '[', '\\', ']', '^', '_',
    '`',  'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o',
    'p',  'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '{', '|', '}', '~', '⌂',
    'Ç',  'ü', 'é', 'â', 'ä', 'à', 'å', 'ç', 'ê', 'ë', 'è', 'ï', 'î', 'ì', 'Ä', 'Å',
    'É',  'æ', 'Æ', 'ô', 'ö', 'ò', 'û', 'ù', 'ÿ', 'Ö', 'Ü', '¢', '£', '¥', '₧', 'ƒ',
    'á',  'í', 'ó', 'ú', 'ñ', 'Ñ', 'ª', 'º', '¿', '⌐', '¬', '½', '¼', '¡', '«', '»',
    '░',  '▒', '▓', '│', '┤', '╡', '╢', '╖', '╕', '╣', '║', '╗', '╝', '╜', '╛', '┐',
    '└',  '┴', '┬', '├', '─', '┼', '╞', '╟', '╚', '╔', '╩', '╦', '╠', '═', '╬', '╧',
    '╨',  '╤', '╥', '╙', '╘', '╒', '╓', '╫', '╪', '┘', '┌', '█', '▄', '▌', '▐', '▀',
    'α',  'ß', 'Γ', 'π', 'Σ', 'σ', 'µ', 'τ', 'Φ', 'Θ', 'Ω', 'δ', '∞', 'φ', 'ε', '∩',
    '≡',  '±', '≥', '≤', '⌠', '⌡', '÷', '≈', '°', '∙', '·', '√', 'ⁿ', '²', '■', '\u{A0}',
];

fn nth_char(n: usize) -> &'static [u8] {
    &VGA_FONT[n * 16..(n + 1) * 16]
}

// TODO : transform this to a BTreeMap once there is a memory allocator
fn char_to_font_idx(c: char) -> usize {
    match c {
        ' '..='~' => c as usize,
        _ => CP437.iter().position(|&x| x == c).unwrap_or(178),
    }
}

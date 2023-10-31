use core::fmt;

use spin::MutexGuard;

use super::{Buffer, Color, OutOfBoundsError, BUFFER, TEXT_BUFFER};

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
    pub fn new(scale_factor: usize) -> Self {
        let buffer = BUFFER.get().expect("Buffer required").lock();
        let max_char_x = buffer.max_x() / (VGA_CHAR_SIZE.0 * scale_factor);
        let padding_char_x = (buffer.max_x() % (VGA_CHAR_SIZE.0 * scale_factor)) * VGA_CHAR_SIZE.1;
        let max_char_y = buffer.max_y() / (VGA_CHAR_SIZE.1 * scale_factor);
        let padding_char_y = (buffer.max_y() % (VGA_CHAR_SIZE.1 * scale_factor)) * VGA_CHAR_SIZE.0;
        TextBuffer {
            max_char_x,
            max_char_y,
            scale_factor,
            padding_char_x,
            padding_char_y,
        }
    }

    fn get_buffer() -> MutexGuard<'static, Buffer> {
        BUFFER.get().expect("Buffer required").lock()
    }

    fn char_coord_to_buffer_coord(&self, char_x: isize, char_y: isize) -> (isize, isize) {
        let real_char_x = char_x * (VGA_CHAR_SIZE.0 * self.scale_factor) as isize;
        let real_char_y = char_y * (VGA_CHAR_SIZE.1 * self.scale_factor) as isize;
        (real_char_x, real_char_y)
    }

    // Assumes the slice is of length 16 (16 bytes = 8*16 pixels = char)
    #[allow(clippy::needless_range_loop)]
    fn draw_char_rect(
        &mut self,
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
        for char_pxl_x in 0..VGA_CHAR_SIZE.0 {
            for char_pxl_y in 0..VGA_CHAR_SIZE.1 {
                let char_pxl = (rect[char_pxl_y] >> (VGA_CHAR_SIZE.0 - 1 - char_pxl_x)) & 1;
                let color = if char_pxl == 0 {
                    background_color
                } else {
                    foreground_color
                };
                let (buffer_x, buffer_y) =
                    self.char_coord_to_buffer_coord(char_x as isize, char_y as isize);
                for i in 0..self.scale_factor {
                    for j in 0..self.scale_factor {
                        let mut buffer = Self::get_buffer();
                        buffer.write(
                            buffer_x as usize + char_pxl_x * self.scale_factor + i,
                            buffer_y as usize + char_pxl_y * self.scale_factor + j,
                            color,
                        )?;
                    }
                }
            }
        }
        Ok(())
    }

    fn write_char(
        &mut self,
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
            match char_i {
                '\n' => {
                    char_x = 0;
                    char_y += 1;
                }
                _ => {
                    self.write_char(char_i, char_x, char_y, background_color, foreground_color)?;
                    char_x += 1;
                }
            }
        }
        Ok(())
    }

    pub fn move_slice_text(
        &mut self,
        char_x: usize,
        char_y: usize,
        text_len: usize,
        char_dx: isize,
        char_dy: isize,
    ) -> Result<(), OutOfBoundsError> {
        let (x, y) = self.char_coord_to_buffer_coord(char_x as isize, char_y as isize);
        let (dx, dy) = self.char_coord_to_buffer_coord(char_dx, char_dy);
        let padding = self.padding_char_x * text_len / self.max_char_x;
        let len =
            text_len * VGA_CHAR_SIZE.0 * VGA_CHAR_SIZE.1 * self.scale_factor * self.scale_factor
                + padding;
        let mut buffer = Self::get_buffer();
        buffer.move_slice(x as usize, y as usize, len, dx, dy)
    }

    fn clear_line(&mut self, y: usize, background_color: Color) -> Result<(), OutOfBoundsError> {
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
        Writer {
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

    pub fn change_background_color(&mut self, color: Color) {
        self.background_code = color;
    }

    pub fn change_foreground_color(&mut self, color: Color) {
        self.foreground_code = color;
    }

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

    fn clear_line(&mut self, text_buffer: &mut MutexGuard<'_, TextBuffer>) {
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

pub const VGA_TEST_SLICE: &str = " ☺☻♥♦♣♠•◘○◙♂♀♪♫☼►◄↕‼¶§▬↨↑↓→←∟↔▲▼ !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~⌂ÇüéâäàåçêëèïîìÄÅÉæÆôöòûùÿÖÜ¢£¥₧ƒáíóúñÑªº¿⌐¬½¼¡«»░▒▓│┤╡╢╖╕╣║╗╝╜╛┐└┴┬├─┼╞╟╚╔╩╦╠═╬╧╨╤╥╙╘╒╓╫╪┘┌█▄▌▐▀αßΓπΣσµτΦΘΩδ∞φε∩≡±≥≤⌠⌡÷≈°∙·√ⁿ²■ ";

fn nth_char(n: usize) -> &'static [u8] {
    &VGA_FONT[n * 16..(n + 1) * 16]
}

// TODO : transform this to a BTreeMap once there is a memory allocator
fn char_to_font_idx(c: char) -> usize {
    match c {
        //' ' => 0,
        '☺' => 1,
        '☻' => 2,
        '♥' => 3,
        '♦' => 4,
        '♣' => 5,
        '♠' => 6,
        '•' => 7,
        '◘' => 8,
        '○' => 9,
        '◙' => 10,
        '♂' => 11,
        '♀' => 12,
        '♪' => 13,
        '♫' => 14,
        '☼' => 15,
        '►' => 16,
        '◄' => 17,
        '↕' => 18,
        '‼' => 19,
        '¶' => 20,
        '§' => 21,
        '▬' => 22,
        '↨' => 23,
        '↑' => 24,
        '↓' => 25,
        '→' => 26,
        '←' => 27,
        '∟' => 28,
        '↔' => 29,
        '▲' => 30,
        '▼' => 31,
        ' ' => 32, // Space (again)
        '!' => 33,
        '"' => 34,
        '#' => 35,
        '$' => 36,
        '%' => 37,
        '&' => 38,
        '\'' => 39,
        '(' => 40,
        ')' => 41,
        '*' => 42,
        '+' => 43,
        ',' => 44,
        '-' => 45,
        '.' => 46,
        '/' => 47,
        '0' => 48,
        '1' => 49,
        '2' => 50,
        '3' => 51,
        '4' => 52,
        '5' => 53,
        '6' => 54,
        '7' => 55,
        '8' => 56,
        '9' => 57,
        ':' => 58,
        ';' => 59,
        '<' => 60,
        '=' => 61,
        '>' => 62,
        '?' => 63,
        '@' => 64,
        'A' => 65,
        'B' => 66,
        'C' => 67,
        'D' => 68,
        'E' => 69,
        'F' => 70,
        'G' => 71,
        'H' => 72,
        'I' => 73,
        'J' => 74,
        'K' => 75,
        'L' => 76,
        'M' => 77,
        'N' => 78,
        'O' => 79,
        'P' => 80,
        'Q' => 81,
        'R' => 82,
        'S' => 83,
        'T' => 84,
        'U' => 85,
        'V' => 86,
        'W' => 87,
        'X' => 88,
        'Y' => 89,
        'Z' => 90,
        '[' => 91,
        '\\' => 92,
        ']' => 93,
        '^' => 94,
        '_' => 95,
        '`' => 96,
        'a' => 97,
        'b' => 98,
        'c' => 99,
        'd' => 100,
        'e' => 101,
        'f' => 102,
        'g' => 103,
        'h' => 104,
        'i' => 105,
        'j' => 106,
        'k' => 107,
        'l' => 108,
        'm' => 109,
        'n' => 110,
        'o' => 111,
        'p' => 112,
        'q' => 113,
        'r' => 114,
        's' => 115,
        't' => 116,
        'u' => 117,
        'v' => 118,
        'w' => 119,
        'x' => 120,
        'y' => 121,
        'z' => 122,
        '{' => 123,
        '|' => 124,
        '}' => 125,
        '~' => 126,
        '⌂' => 127,
        'Ç' => 128,
        'ü' => 129,
        'é' => 130,
        'â' => 131,
        'ä' => 132,
        'à' => 133,
        'å' => 134,
        'ç' => 135,
        'ê' => 136,
        'ë' => 137,
        'è' => 138,
        'ï' => 139,
        'î' => 140,
        'ì' => 141,
        'Ä' => 142,
        'Å' => 143,
        'É' => 144,
        'æ' => 145,
        'Æ' => 146,
        'ô' => 147,
        'ö' => 148,
        'ò' => 149,
        'û' => 150,
        'ù' => 151,
        'ÿ' => 152,
        'Ö' => 153,
        'Ü' => 154,
        '¢' => 155,
        '£' => 156,
        '¥' => 157,
        '₧' => 158,
        'ƒ' => 159,
        'á' => 160,
        'í' => 161,
        'ó' => 162,
        'ú' => 163,
        'ñ' => 164,
        'Ñ' => 165,
        'ª' => 166,
        'º' => 167,
        '¿' => 168,
        '⌐' => 169,
        '¬' => 170,
        '½' => 171,
        '¼' => 172,
        '¡' => 173,
        '«' => 174,
        '»' => 175,
        '░' => 176,
        '▒' => 177,
        '▓' => 178,
        '│' => 179,
        '┤' => 180,
        '╡' => 181,
        '╢' => 182,
        '╖' => 183,
        '╕' => 184,
        '╣' => 185,
        '║' => 186,
        '╗' => 187,
        '╝' => 188,
        '╜' => 189,
        '╛' => 190,
        '┐' => 191,
        '└' => 192,
        '┴' => 193,
        '┬' => 194,
        '├' => 195,
        '─' => 196,
        '┼' => 197,
        '╞' => 198,
        '╟' => 199,
        '╚' => 200,
        '╔' => 201,
        '╩' => 202,
        '╦' => 203,
        '╠' => 204,
        '═' => 205,
        '╬' => 206,
        '╧' => 207,
        '╨' => 208,
        '╤' => 209,
        '╥' => 210,
        '╙' => 211,
        '╘' => 212,
        '╒' => 213,
        '╓' => 214,
        '╫' => 215,
        '╪' => 216,
        '┘' => 217,
        '┌' => 218,
        '█' => 219,
        '▄' => 220,
        '▌' => 221,
        '▐' => 222,
        '▀' => 223,
        'α' => 224,
        'ß' => 225,
        'Γ' => 226,
        'π' => 227,
        'Σ' => 228,
        'σ' => 229,
        'µ' => 230,
        'τ' => 231,
        'Φ' => 232,
        'Θ' => 233,
        'Ω' => 234,
        'δ' => 235,
        '∞' => 236,
        'φ' => 237,
        'ε' => 238,
        '∩' => 239,
        '≡' => 240,
        '±' => 241,
        '≥' => 242,
        '≤' => 243,
        '⌠' => 244,
        '⌡' => 245,
        '÷' => 246,
        '≈' => 247,
        '°' => 248,
        '∙' => 249,
        '·' => 250,
        '√' => 251,
        'ⁿ' => 252,
        '²' => 253,
        '■' => 254,
        ' ' => 255, // NBSP

        _ => 178,
    }
}

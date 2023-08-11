use core::fmt;
use volatile::Volatile;

use super::*;

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ScreenChar {
    pub ascii_character: u8,
    pub color_code: ColorCode8b,
}

impl ScreenChar {
    pub fn new(ascii_character: u8, color_code: ColorCode) -> Self {
        let color_code_byte = color_code.into();
        ScreenChar {
            ascii_character,
            color_code: color_code_byte,
        }
    }
}

#[repr(transparent)]
pub struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

impl Buffer {
    fn write(&mut self, row: usize, col: usize, screen_char: ScreenChar) {
        self.chars[row][col].write(screen_char);
    }

    fn read(&self, row: usize, col: usize) -> ScreenChar {
        self.chars[row][col].read()
    }
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn new(color_code: ColorCode, buffer: &'static mut Buffer) -> Self {
        Writer {
            column_position: 0,
            color_code,
            buffer,
        }
    }

    fn write_free(&mut self, row: usize, col: usize, byte: u8) {
        self.buffer
            .write(row, col, ScreenChar::new(byte, self.color_code));
    }

    #[cfg(test)]
    pub fn read_free(&self, row: usize, col: usize) -> ScreenChar {
        self.buffer.read(row, col)
    }

    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                self.write_free(row, col, byte);
                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let pxl = self.buffer.read(row, col);
                self.buffer.write(row - 1, col, pxl);
                if row == BUFFER_HEIGHT - 1 {
                    self.write_free(row, col, b' ');
                }
                self.column_position = 0;
            }
        }
    }

    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }

    pub fn clear(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                self.write_free(row, col, b' ');
            }
        }
        self.change_column_position(0);
    }

    #[allow(dead_code)]
    pub fn change_foreground_color(&mut self, foreground_color: Color4b) {
        self.color_code.change_foreground_color(foreground_color);
    }

    // TODO : Make it so that the background color can be changed with a Color4b if the blink is deactivated (vga::io_ports)
    #[allow(dead_code)]
    pub fn change_background_color(&mut self, background_color: Color3b) {
        self.color_code.change_background_color(background_color);
    }

    #[allow(dead_code)]
    pub fn change_blink_to(&mut self, blink: bool) {
        self.color_code.change_blink_to(blink);
    }

    #[allow(dead_code)]
    pub fn change_column_position(&mut self, column_position: usize) {
        self.column_position = column_position;
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use {super::Color4b::Black, super::*, crate::println};

    fn return_to_default() {
        let mut writer = WRITER.lock();
        writer.change_foreground_color(Black);
        writer.change_background_color(Color3b::LightGray);
        writer.change_blink_to(false);
        writer.clear();
    }

    #[test_case]
    fn writer_println_output() {
        let s = "Some test string that fits on a single line";
        x86_64::instructions::interrupts::without_interrupts(|| {
            return_to_default();

            println!("{}", s);

            let writer = WRITER.lock();

            for (i, c) in s.chars().enumerate() {
                let screen_char = writer.read_free(BUFFER_HEIGHT - 2, i);
                assert_eq!(char::from(screen_char.ascii_character), c);
            }
        });
    }

    #[test_case]
    fn writer_write_free() {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let mut writer = WRITER.lock();
            writer.write_free(12, 54, b'X');
            assert_eq!(writer.read_free(12, 54).ascii_character, b'X');
        });
    }

    #[test_case]
    fn writer_read_free_1() {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let mut writer = WRITER.lock();
            writer.write_free(12, 54, b'X');
            assert_eq!(writer.read_free(12, 54).ascii_character, b'X');
        });
    }

    #[test_case]
    fn writer_read_free_2() {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let mut writer = WRITER.lock();
            writer.write_free(12, 54, b'X');
            assert_eq!(writer.read_free(12, 54), writer.buffer.read(12, 54));
        });
    }

    #[test_case]
    fn writer_change_foreground_color() {
        x86_64::instructions::interrupts::without_interrupts(|| {
            return_to_default();

            let mut writer = WRITER.lock();
            writer.change_foreground_color(Color4b::LightBlue);
            writer.write_byte(b'X');

            assert_eq!(
                writer.read_free(BUFFER_HEIGHT - 1, 0).color_code.0,
                ColorCode8b::from(ColorCode::new(
                    Color4b::LightBlue,
                    Color3b::LightGray,
                    false
                ))
                .0
            );
        });
    }

    #[test_case]
    fn writer_change_background_color() {
        x86_64::instructions::interrupts::without_interrupts(|| {
            return_to_default();

            let mut writer = WRITER.lock();
            writer.change_background_color(Color3b::Blue);
            writer.write_byte(b'X');

            assert_eq!(
                writer.read_free(BUFFER_HEIGHT - 1, 0).color_code.0,
                ColorCode8b::from(ColorCode::new(Color4b::Black, Color3b::Blue, false)).0
            );
        });
    }

    #[test_case]
    fn writer_change_blink_to() {
        x86_64::instructions::interrupts::without_interrupts(|| {
            return_to_default();

            let mut writer = WRITER.lock();
            writer.change_blink_to(true);
            writer.change_column_position(0);
            writer.write_byte(b'X');

            assert_eq!(
                writer.read_free(BUFFER_HEIGHT - 1, 0).color_code.0,
                ColorCode8b::from(ColorCode::new(Color4b::Black, Color3b::LightGray, true)).0
            );
        });
    }

    #[test_case]
    fn writer_change_column_position() {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let mut writer = WRITER.lock();
            writer.change_column_position(10);
            writer.write_byte(b'X');
            assert_eq!(
                writer.read_free(BUFFER_HEIGHT - 1, 10).ascii_character,
                b'X'
            );
        });
    }

    #[test_case]
    fn writer_clear() {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let mut writer = WRITER.lock();
            writer.write_free(12, 54, b'X');
            writer.clear();
            for row in 0..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    assert_eq!(writer.read_free(row, col).ascii_character, b' ');
                }
            }
            assert_eq!(writer.column_position, 0);
        });
    }
}

// ENUMS

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color3b {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color4b {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

// STRUCTS

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorCode {
    foreground_color: Color4b,
    background_color: Color3b,
    blink: bool,
}

impl ColorCode {
    pub fn new(foreground_color: Color4b, background_color: Color3b, blink: bool) -> Self {
        ColorCode {
            foreground_color,
            background_color,
            blink,
        }
    }

    pub fn change_foreground_color(&mut self, foreground_color: Color4b) {
        self.foreground_color = foreground_color;
    }

    pub fn change_background_color(&mut self, background_color: Color3b) {
        self.background_color = background_color;
    }

    pub fn change_blink_to(&mut self, blink: bool) {
        self.blink = blink;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode8b(pub u8);

impl From<ColorCode> for ColorCode8b {
    fn from(color_code: ColorCode) -> Self {
        let blink_bit = (color_code.blink as u8) << 7;
        let background_bits = (color_code.background_color as u8) << 4;
        let forground_bits = color_code.foreground_color as u8;

        ColorCode8b(blink_bit | background_bits | forground_bits)
    }
}

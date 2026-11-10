pub mod screen;
pub mod text;

use spin::{Mutex, Once};

use crate::{
    boot,
    drivers::framebuffer::{screen::ScreenBuffer, text::Writer},
};

pub static BUFFER: Once<Mutex<ScreenBuffer>> = Once::new();
pub static WRITER: Once<Mutex<Writer>> = Once::new();

/// # Panics
/// This function will panic if the framebuffer tag is not present in the multiboot2 information.
pub fn init(scale_factor: usize) {
    let framebuffer_tag = boot::MULTIBOOT2_INFO
        .get()
        .expect("Multiboot info required")
        .framebuffer_tag()
        .expect("Framebuffer required")
        .expect("Framebuffer required");

    // TODO: Write combining on this framebuffer, through PAT, maybe, i don't really care tbh but memorial for the MTRR that were never used. No wonder why the osdev wiki was wrong on this. It's legacy

    BUFFER.call_once(|| Mutex::new(ScreenBuffer::new(framebuffer_tag)));
    WRITER.call_once(|| Mutex::new(Writer::new(scale_factor)));
}

#[expect(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum OutOfBoundsError {
    Point(PointOutOfBoundsError),
    Char(PointOutOfBoundsError),
    Slice(SliceOutOfBoundsError),
}
impl OutOfBoundsError {
    #[must_use]
    pub const fn new_point(x: usize, y: usize, max_x: usize, max_y: usize) -> Self {
        Self::Point(PointOutOfBoundsError::new(x, y, max_x, max_y))
    }

    #[must_use]
    pub const fn new_char(
        char_x: usize,
        char_y: usize,
        max_char_x: usize,
        max_char_y: usize,
    ) -> Self {
        Self::Char(PointOutOfBoundsError::new(
            char_x, char_y, max_char_x, max_char_y,
        ))
    }

    #[must_use]
    pub const fn new_slice(start: usize, end: usize, max: usize) -> Self {
        Self::Slice(SliceOutOfBoundsError::new(start, end, max))
    }
}

#[expect(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct PointOutOfBoundsError {
    x: usize,
    y: usize,
    max_x: usize,
    max_y: usize,
}

impl PointOutOfBoundsError {
    #[must_use]
    pub const fn new(x: usize, y: usize, max_x: usize, max_y: usize) -> Self {
        Self { x, y, max_x, max_y }
    }
}

#[expect(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct SliceOutOfBoundsError {
    start: usize,
    end: usize,
    max: usize,
}

impl SliceOutOfBoundsError {
    #[must_use]
    pub const fn new(start: usize, end: usize, max: usize) -> Self {
        Self { start, end, max }
    }
}

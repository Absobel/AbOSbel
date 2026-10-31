#![allow(dead_code)]

use core::fmt::{self, Write};

use spin::{Mutex, Once};

use crate::{MULTIBOOT2_INFO, io::SERIAL1, x86::without_interrupts};

use super::{ScreenBuffer, Writer};

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

#[derive(Debug, Clone, Copy)]
pub struct CharOutOfBoundsError {
    char_x: usize,
    char_y: usize,
    max_char_x: usize,
    max_char_y: usize,
}

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

//////////////////////////////////

pub static BUFFER: Once<Mutex<ScreenBuffer>> = Once::new();
pub static WRITER: Once<Mutex<Writer>> = Once::new();

/// # Panics
/// This function will panic if the framebuffer tag is not present in the multiboot2 information.
pub fn init_graphics() {
    let framebuffer_tag = MULTIBOOT2_INFO
        .get()
        .expect("Multiboot info required")
        .framebuffer_tag()
        .expect("Framebuffer required")
        .expect("Framebuffer required");

    // TODO: Write combining on this framebuffer, through PAT, maybe, i don't really care tbh but memorial for the MTRR that were never used. No wonder why the osdev wiki was wrong on this. It's legacy

    BUFFER.call_once(|| Mutex::new(ScreenBuffer::new(framebuffer_tag)));
    WRITER.call_once(|| Mutex::new(Writer::new(1)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    // Deactivating interrupts to avoid deadlocks
    without_interrupts(|| {
        if let Some(serial) = SERIAL1.get() {
            serial
                .lock()
                .write_fmt(args)
                .expect("Printing to serial failed");
        }
        if let Some(w) = WRITER.get() {
            w.lock()
                .write_fmt(args)
                .expect("Printing to Framebuffer failed");
        }
    });
}

///////////////

#[must_use]
pub const fn usize_plus_isize(u: usize, i: isize) -> usize {
    if i < 0 {
        u - i.wrapping_abs().cast_unsigned()
    } else {
        u + i.cast_unsigned()
    }
}

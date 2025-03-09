#![allow(dead_code)]

use core::fmt::{self, Write};

use lazy_static::lazy_static;
use spin::{Mutex, Once};

use crate::{MULTIBOOT2_INFO, x86::without_interrupts};

use super::{Buffer, TextBuffer, Writer};

#[derive(Debug, Clone, Copy)]
pub enum OutOfBoundsError {
    Point(PointOutOfBoundsError),
    Char(PointOutOfBoundsError),
    Slice(SliceOutOfBoundsError),
}

impl OutOfBoundsError {
    pub fn new_point(x: usize, y: usize, max_x: usize, max_y: usize) -> Self {
        OutOfBoundsError::Point(PointOutOfBoundsError::new(x, y, max_x, max_y))
    }

    pub fn new_char(char_x: usize, char_y: usize, max_char_x: usize, max_char_y: usize) -> Self {
        OutOfBoundsError::Char(PointOutOfBoundsError::new(
            char_x, char_y, max_char_x, max_char_y,
        ))
    }

    pub fn new_slice(start: usize, end: usize, max: usize) -> Self {
        OutOfBoundsError::Slice(SliceOutOfBoundsError::new(start, end, max))
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
    pub fn new(x: usize, y: usize, max_x: usize, max_y: usize) -> Self {
        PointOutOfBoundsError { x, y, max_x, max_y }
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
    pub fn new(start: usize, end: usize, max: usize) -> Self {
        SliceOutOfBoundsError { start, end, max }
    }
}

//////////////////////////////////

lazy_static! {
    pub static ref BUFFER: Once<Mutex<Buffer>> = Once::new();
    pub static ref TEXT_BUFFER: Once<Mutex<TextBuffer>> = Once::new();
    pub static ref WRITER: Once<Mutex<Writer>> = Once::new();
}

pub fn init_graphics() {
    let framebuffer_tag = MULTIBOOT2_INFO
        .get()
        .expect("Multiboot info required")
        .framebuffer_tag()
        .expect("Framebuffer required")
        .expect("Framebuffer required");

    // TODO : Causes GPF and is probably not needed
    // let height = framebuffer_tag.height() as usize;
    // let pitch = framebuffer_tag.pitch() as usize;
    // crate::x86::set_mtrr_wc(framebuffer_tag.address() as usize, height * pitch).expect("MTTR WC failed");

    BUFFER.call_once(|| Mutex::new(Buffer::new(framebuffer_tag)));
    TEXT_BUFFER.call_once(|| Mutex::new(TextBuffer::new(1)));
    WRITER.call_once(|| Mutex::new(Writer::default()));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    // Deactivating interrupts to avoid deadlocks
    without_interrupts(|| {
        WRITER
            .get()
            .expect("Writer required")
            .lock()
            .write_fmt(args)
            .expect("Printing to Framebuffer failed");
    });
}

///////////////

pub fn usize_plus_isize(u: usize, i: isize) -> usize {
    if i < 0 {
        u - i.wrapping_abs() as usize
    } else {
        u + i as usize
    }
}

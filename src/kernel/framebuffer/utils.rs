#![allow(dead_code)]

use core::fmt::{self, Write};

use crate::x86::set_mtrr_wc;
use spin::Mutex;

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

crate::sync_wrapper!(BUFFER, FrameBuffer, Mutex<Buffer>);
crate::sync_wrapper!(TEXT_BUFFER, OnceTextBuffer, Mutex<TextBuffer>);
crate::sync_wrapper!(WRITER, OnceWriter, Mutex<Writer>);

pub fn init_graphics() {
    let framebuffer_tag = crate::MULTIBOOT2_INFO
        .get()
        .expect("Multiboot info required")
        .framebuffer_tag()
        .expect("Framebuffer required")
        .expect("Framebuffer required");

    let height = framebuffer_tag.height() as usize;
    let pitch = framebuffer_tag.pitch() as usize;
    set_mtrr_wc(framebuffer_tag.address() as usize, height * pitch).expect("MTTR WC failed");

    BUFFER
        .set(Mutex::new(Buffer::new(framebuffer_tag)))
        .expect("Shouldn't be initialised");

    TEXT_BUFFER
        .set(Mutex::new(TextBuffer::new(1)))
        .expect("Shouldn't be initialised");

    WRITER
        .set(Mutex::new(Writer::default()))
        .expect("Shouldn't be initialised");
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    // Deactivating interrupts to avoid deadlocks
    x86_64::instructions::interrupts::without_interrupts(|| {
        WRITER
            .get()
            .expect("Writer required")
            .lock()
            .write_fmt(args)
            .expect("Printing to Framebuffer failed");
    });
}

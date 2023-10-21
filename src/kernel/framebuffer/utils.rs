#![allow(dead_code)]

use lazy_static::lazy_static;
use spin::Mutex;

use core::{
    cell::OnceCell,
    fmt::{self, Write},
};

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

crate::sync_wrapper!(FrameBuffer, Mutex<Buffer>);
pub static BUFFER: FrameBuffer = FrameBuffer(OnceCell::new());

pub fn init_buffer() {
    let framebuffer_tag = crate::MULTIBOOT2_INFO
        .get()
        .expect("Multiboot info required")
        .framebuffer_tag()
        .expect("Framebuffer required")
        .expect("Framebuffer required");

    BUFFER
        .0
        .set(Mutex::new(Buffer::new(framebuffer_tag)))
        .expect("Shouldn't be initialised");
}

// TODO : This shit is so awful i can't even, how do I modify the text buffer scale factor ? Dumbass
// Also this is so unfuture proof 'cauz it will be a pain to check what is initialized before what and
// welcome random panics
// This is just for debugging purposes though, I'll make a way better implementation after having gotten an allocator
lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new({
        let framebuffer_tag = crate::MULTIBOOT2_INFO
            .get()
            .expect("Multiboot info required")
            .framebuffer_tag()
            .expect("Framebuffer required")
            .expect("Framebuffer required");

        let buffer = Buffer::new(framebuffer_tag);
        let text_buffer = TextBuffer::new(buffer, 1);

        Writer::new(text_buffer)
    });
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    // Deactivating interrupts to avoid deadlocks
    x86_64::instructions::interrupts::without_interrupts(|| {
        WRITER
            .lock()
            .write_fmt(args)
            .expect("Printing to Framebuffer failed");
    });
}

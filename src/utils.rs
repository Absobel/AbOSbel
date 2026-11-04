use core::{
    arch::asm,
    fmt::{self, Write},
    panic::PanicInfo,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::{framebuffer::WRITER, io::EmergencySerialWriter, kernel::io::Port, print};

static PANICKING: AtomicBool = AtomicBool::new(false);

pub fn panic_report(info: &PanicInfo) {
    // Do not panic again if we are already panicking
    if !PANICKING.swap(true, Ordering::Relaxed) {
        let _ = writeln!(EmergencySerialWriter, "{info}");
        // Only prints the panic message to the framebuffer if it is initialized and available
        if let Some(writer) = WRITER.get()
            && let Some(mut w) = writer.try_lock()
        {
            let _ = writeln!(w, "{info}");
        }
    }
}

pub fn hlt_loop() -> ! {
    loop {
        flush_ring();
        unsafe {
            asm!("hlt");
        }
    }
}

pub fn flush_ring() {
    print!();
}

/// Qemu exits with (code << 1) | 1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10, // (0x10 << 1) | 1 = 33
    Failed = 0x11,  // (0x11 << 1) | 1 = 35
}

pub fn exit_qemu(exit_code: QemuExitCode) -> ! {
    const QEMU_EXIT_PORT: u16 = 0xf4;
    let debug_exit_port = Port::new(QEMU_EXIT_PORT);
    unsafe { debug_exit_port.outl(exit_code as u32) };
    hlt_loop()
}

#[must_use]
pub const fn align_up(addr: usize, align: usize) -> usize {
    usize::next_multiple_of(addr, align)
}

#[must_use]
pub const fn align_down_pow2(addr: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two());
    addr & !(align - 1)
}

// Ring

// TODO: Waste of space, could do with [len][bytes] record instead
#[derive(Debug, Clone, Copy)]
pub struct Message {
    text: [u8; 248],
    len: usize,
}

impl From<&str> for Message {
    fn from(s: &str) -> Self {
        let mut msg = Self::default();
        msg.append(s);
        msg
    }
}

impl Message {
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.text[..self.len]).unwrap_or("<invalid utf-8>")
    }

    fn append(&mut self, s: &str) {
        let bytes = s.as_bytes();
        let mut len = bytes.len().min(248 - self.len);
        while !s.is_char_boundary(len) {
            len -= 1;
        }
        self.text[self.len..self.len + len].copy_from_slice(&bytes[..len]);
        self.len += len;
    }
}

impl Default for Message {
    fn default() -> Self {
        Self {
            text: [0; 248],
            len: 0,
        }
    }
}

impl Write for Message {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.append(s);
        Ok(())
    }
}

pub struct RingBuffer<T: Copy + Default, const N: usize> {
    buffer: [T; N],
    head: usize,
    tail: usize,
    dropped: usize,
}

impl<T: Copy + Default, const N: usize> RingBuffer<T, N> {
    pub fn new() -> Self {
        Self {
            buffer: [T::default(); N],
            head: 0,
            tail: 0,
            dropped: 0,
        }
    }

    pub const fn push(&mut self, item: T) {
        let next_head = (self.head + 1) % N;
        if next_head == self.tail {
            self.tail = (self.tail + 1) % N; // Drop the oldest item
            self.dropped += 1;
        }
        self.buffer[self.head] = item;
        self.head = next_head;
    }

    pub const fn try_pop(&mut self) -> Option<T> {
        if self.head == self.tail {
            return None; // Buffer is empty
        }
        let item = self.buffer[self.tail];
        self.tail = (self.tail + 1) % N;
        Some(item)
    }

    pub const fn reset_drop_count(&mut self) -> usize {
        let d = self.dropped;
        self.dropped = 0;
        d
    }
}

impl<const N: usize> fmt::Write for RingBuffer<Message, N> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push(Message::from(s));
        Ok(())
    }

    fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> fmt::Result {
        let mut msg = Message::default();
        msg.write_fmt(args)?;
        self.push(msg);
        Ok(())
    }
}

impl<const N: usize> RingBuffer<Message, N> {
    pub fn print_all(&mut self, w: &mut impl Write) {
        let dropped = self.reset_drop_count();
        if dropped > 0 {
            writeln!(w, "Dropped {dropped} messages").expect("Writing to writer failed");
        }
        while let Some(msg) = self.try_pop() {
            write!(w, "{}", msg.as_str()).expect("Writing to writer failed");
        }
    }
}

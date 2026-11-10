pub mod macros;

use core::fmt::{self, Write};
use spin::Mutex;

use crate::{
    arch,
    drivers::{
        framebuffer::{BUFFER, WRITER},
        serial::SERIAL1,
    },
    ring::RingBuffer,
};

// PRINT HELPER FUNCTIONS //

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    // Deactivating interrupts to avoid deadlocks
    arch::without_interrupts(|| {
        if let Some(serial) = SERIAL1.get() {
            serial
                .lock()
                .write_fmt(args)
                .expect("Printing to serial failed");
        }
        if let Some(w) = WRITER.get()
            && let Some(b) = BUFFER.get()
        {
            let mut irq_buffer = IRQ_BUFFER.lock();
            if let Some(mut w) = w.try_lock()
                && !b.is_locked()
            {
                irq_buffer.print_all(&mut *w);
                w.write_fmt(args).expect("Printing to Framebuffer failed");
            } else {
                irq_buffer
                    .write_fmt(args)
                    .expect("Writing to IRQ buffer failed");
            }
        }
    });
}

#[doc(hidden)]
pub fn _serial_print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;

    // Deactivating interrupts to avoid deadlocks
    arch::without_interrupts(|| {
        if let Some(serial) = SERIAL1.get() {
            serial
                .lock()
                .write_fmt(args)
                .expect("Printing to serial failed");
        }
    });
}

// RING BUFFER FOR IRQ MESSAGES

pub static IRQ_BUFFER: Mutex<RingBuffer<Message, 16>> = Mutex::new(RingBuffer::new());

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

impl fmt::Write for Message {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.append(s);
        Ok(())
    }
}

impl<const N: usize> fmt::Write for RingBuffer<Message, N> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push_overwrite(Message::from(s));
        Ok(())
    }

    fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> fmt::Result {
        let mut msg = Message::default();
        msg.write_fmt(args)?;
        self.push_overwrite(msg);
        Ok(())
    }
}

impl<const N: usize> RingBuffer<Message, N> {
    pub fn print_all(&mut self, w: &mut impl fmt::Write) {
        let dropped = self.reset_drop_count();
        if dropped > 0 {
            writeln!(w, "Dropped {dropped} messages").expect("Writing to writer failed");
        }
        while let Some(msg) = self.try_pop() {
            write!(w, "{}", msg.as_str()).expect("Writing to writer failed");
        }
    }
}

use core::{
    arch::asm,
    error::Error,
    fmt::{self, Display, Formatter},
};

/// Methods taken straight from <https://wiki.osdev.org/Serial_Ports>
pub struct SerialPort {
    port: u16,
}

impl SerialPort {
    #[must_use]
    pub const unsafe fn new(port: u16) -> Self {
        Self { port }
    }

    /// # Errors
    /// This function will return an error if the serial port is faulty or doesn't exist.
    #[expect(clippy::identity_op)]
    pub fn init(&self) -> Result<(), LoopbackMismatch> {
        unsafe {
            outb(self.port + 1, 0x00); // Disable all interrupts
            outb(self.port + 3, 0x80); // Enable DLAB (set baud rate divisor)
            outb(self.port + 0, 0x03); // Set divisor to 3 (lo byte) 38400 baud
            outb(self.port + 1, 0x00); //                  (hi byte)
            outb(self.port + 3, 0x03); // 8 bits, no parity, one stop bit
            outb(self.port + 2, 0xC7); // Enable FIFO, clear them, with 14-byte threshold
            outb(self.port + 4, 0x0B); // IRQs enabled, RTS/DSR set
            outb(self.port + 4, 0x1E); // Set in loopback mode, test the serial chip
            outb(self.port + 0, 0xAE); // Test serial chip (send byte 0xAE and check if serial returns same 

            // Check if serial is faulty (i.e: not same byte as sent)
            let data = inb(self.port + 0);
            if data != 0xAE {
                return Err(LoopbackMismatch {
                    port: self.port,
                    data,
                });
            }

            // If serial is not faulty set it in normal operation mode
            // (not-loopback with IRQs enabled and OUT#1 and OUT#2 bits enabled)
            outb(self.port + 4, 0x0F);

            Ok(())
        }
    }

    #[must_use]
    pub fn byte_recieved(&self) -> bool {
        unsafe { inb(self.port + 5) & 1 != 0 }
    }

    #[must_use]
    pub fn read_byte(&self) -> u8 {
        while !self.byte_recieved() {}
        unsafe { inb(self.port) }
    }

    #[must_use]
    pub fn is_transmit_empty(&self) -> bool {
        unsafe { inb(self.port + 5) & 0x20 != 0 }
    }

    pub fn write_byte(&self, data: u8) {
        while !self.is_transmit_empty() {}
        unsafe { outb(self.port, data) };
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}

#[must_use]
pub unsafe fn inb(port: u16) -> u8 {
    let data;
    unsafe { asm!("in al, dx", in("dx") port, out("al") data) };
    data
}

pub unsafe fn outb(port: u16, data: u8) {
    unsafe { asm!("out dx, al", in("dx") port, in("al") data) };
}

#[must_use]
pub unsafe fn inw(port: u16) -> u16 {
    let data;
    unsafe { asm!("in ax, dx", in("dx") port, out("ax") data) };
    data
}

pub unsafe fn outw(port: u16, data: u16) {
    unsafe { asm!("out dx, ax", in("dx") port, in("ax") data) };
}

#[derive(Debug)]
pub struct LoopbackMismatch {
    port: u16,
    data: u8,
}
impl Error for LoopbackMismatch {}
impl Display for LoopbackMismatch {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Read 0x{:02X} from port 0x{:04X} in loopback test, expected 0xAE",
            self.data, self.port
        )
    }
}

///////////////////////////////

pub const PS2_KEYBOARD_IN: u16 = 0x60;
pub const PS2_KEYBOARD_OUT: u16 = 0x64;

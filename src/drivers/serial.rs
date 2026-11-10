use core::{
    error::Error,
    fmt::{self, Debug, Display, Formatter},
};

use spin::{Mutex, Once};

use crate::arch::port::Port;

pub static SERIAL1: Once<Mutex<Uart16550>> = Once::new();

const SERIAL1_PORT: u16 = 0x3F8;

/// # Errors
/// This function will return an error if the serial port is faulty/doesn't exist
pub fn init() -> Result<(), LoopbackMismatch> {
    let serial_port = unsafe { Uart16550::new(SERIAL1_PORT)? };
    SERIAL1.call_once(|| Mutex::new(serial_port));
    Ok(())
}

/// Name of the ports are based on what i use them for. They have more uses than jsut that
/// Methods taken straight from <https://wiki.osdev.org/Serial_Ports>
#[derive(Debug)]
pub struct Uart16550 {
    /// Data port
    /// Low byte of the divisor (when DLAB=1)
    data_low_div: Port<u8>,
    /// Interrupt enable
    /// High byte of the divisor (when DLAB=1)
    interrupt_high_div: Port<u8>,
    /// FIFO control (amongst other things)
    fifo_control: Port<u8>,
    /// DLAB control (amongst other thing)
    line_control: Port<u8>,
    /// Line/IRQ/Loopback control (amongst other things)
    modem_control: Port<u8>,
    /// Line status
    line_status: Port<u8>,
}

impl Uart16550 {
    /// # Errors
    /// This function will return an error if the serial port is faulty or doesn't exist.
    #[expect(clippy::identity_op)]
    pub unsafe fn new(data_port: u16) -> Result<Self, LoopbackMismatch> {
        let port = Self {
            data_low_div: Port::new(data_port + 0),
            interrupt_high_div: Port::new(data_port + 1),
            fifo_control: Port::new(data_port + 2),
            line_control: Port::new(data_port + 3),
            modem_control: Port::new(data_port + 4),
            line_status: Port::new(data_port + 5),
        };

        unsafe {
            port.interrupt_high_div.outb(0x00); // Disable all interrupts
            port.line_control.outb(0x80); // Enable DLAB (set baud rate divisor)
            port.data_low_div.outb(0x03); // Set divisor to 3 (lo byte) 38400 baud
            port.interrupt_high_div.outb(0x00); //            (hi byte)
            port.line_control.outb(0x03); // 8 bits, no parity, one stop bit
            port.fifo_control.outb(0xC7); // Enable FIFO, clear them, with 14-byte threshold
            port.modem_control.outb(0x0B); // IRQs enabled, RTS/DSR set
            port.modem_control.outb(0x1E); // Set in loopback mode, test the serial chip
            port.data_low_div.outb(0xAE); // Test serial chip (send byte 0xAE and check if serial returns same 

            // Check if serial is faulty (i.e: not same byte as sent)
            let data = port.data_low_div.inb();
            if data != 0xAE {
                return Err(LoopbackMismatch {
                    port_addr: data_port,
                    data,
                });
            }

            // If serial is not faulty set it in normal operation mode
            // (not-loopback with IRQs enabled and OUT#1 and OUT#2 bits enabled)
            port.modem_control.outb(0x0F);
        }

        Ok(port)
    }

    #[expect(dead_code)]
    #[must_use]
    pub fn byte_received(&self) -> bool {
        unsafe { self.line_status.inb() & 1 != 0 }
    }

    #[expect(dead_code)]
    #[must_use]
    pub fn read_byte(&self) -> u8 {
        while !self.byte_received() {
            core::hint::spin_loop();
        }
        unsafe { self.data_low_div.inb() }
    }

    #[must_use]
    pub fn is_transmit_empty(&self) -> bool {
        unsafe { self.line_status.inb() & 0x20 != 0 }
    }

    pub fn write_byte(&self, data: u8) {
        while !self.is_transmit_empty() {
            core::hint::spin_loop();
        }
        unsafe { self.data_low_div.outb(data) };
    }
}

impl fmt::Write for Uart16550 {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct LoopbackMismatch {
    port_addr: u16,
    data: u8,
}
impl Error for LoopbackMismatch {}
impl Display for LoopbackMismatch {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Read 0x{:02X} from port 0x{:04X} in loopback test, expected 0xAE",
            self.data, self.port_addr
        )
    }
}

pub struct EmergencySerialWriter;
impl fmt::Write for EmergencySerialWriter {
    // Just rewrote the write_str function but raw to the serial port without Mutex
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let data_port = Port::new(SERIAL1_PORT);
        let line_status = Port::<u8>::new(SERIAL1_PORT + 5);
        for b in s.bytes() {
            while !unsafe { line_status.inb() & 0x20 != 0 } {
                core::hint::spin_loop();
            }
            unsafe { data_port.outb(b) };
        }
        Ok(())
    }
}

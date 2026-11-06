use crate::kernel::{
    io::{self, Port},
    x86,
};

const MASTER_COMMAND: Port<u8> = Port::new(0x20);
const MASTER_DATA: Port<u8> = Port::new(0x21);
const SLAVE_COMMAND: Port<u8> = Port::new(0xA0);
const SLAVE_DATA: Port<u8> = Port::new(0xA1);

/// Initialization Bit
const ICW1_INIT: u8 = 0x10;
/// Indicate that the ICW4 will be present
const ICW1_ICW4: u8 = 0x01;
/// Offset of the master PIC's interrupt vector
const MASTER_OFFSET: u8 = 0x20;
/// Offset of the slave PIC's interrupt vector
const SLAVE_OFFSET: u8 = 0x28;
/// Number of the IRQ line allowing cascading from the slave PIC to the master PIC
const CASCADE_IRQ: u8 = 2;
/// 8086/88 (MCS-80/85) mode
const ICW4_8086: u8 = 0x01;

#[derive(Debug, Clone, Copy)]
enum Interrupts {
    Timer = 0,
    Keyboard = 1,
    Cascade = 2,
    // ...
    // Mouse = 12,
}

pub fn init() {
    unsafe {
        // Init bits
        MASTER_COMMAND.outb(ICW1_INIT | ICW1_ICW4);
        io::wait();
        SLAVE_COMMAND.outb(ICW1_INIT | ICW1_ICW4);
        io::wait();
        // Vector offsets
        MASTER_DATA.outb(MASTER_OFFSET);
        io::wait();
        SLAVE_DATA.outb(SLAVE_OFFSET);
        io::wait();
        // Cascade configuration
        MASTER_DATA.outb(1 << CASCADE_IRQ);
        io::wait();
        SLAVE_DATA.outb(CASCADE_IRQ);
        io::wait();
        // Use the 8086/88 mode
        MASTER_DATA.outb(ICW4_8086);
        io::wait();
        SLAVE_DATA.outb(ICW4_8086);
        io::wait();
        // Mask all interrupts
        MASTER_DATA.outb(0xFF);
        io::wait();
        SLAVE_DATA.outb(0xFF);
        io::wait();
    }

    // mask_irq(Mask::Clear, Interrupts::Timer);
    mask_irq(Mask::Clear, Interrupts::Cascade);
    mask_irq(Mask::Clear, Interrupts::Keyboard);

    // enable interrupts
    // mask_irq(Mask::Set, Interrupts::Timer);
    x86::enable_interrupts();
}

#[expect(dead_code)]
#[derive(Debug, Clone, Copy)]
enum Mask {
    Clear,
    Set,
}

fn mask_irq(mode: Mask, irq: Interrupts) {
    let irq = irq as u8;
    let (port, mask) = if irq < 8 {
        (MASTER_DATA, 1 << irq)
    } else {
        (SLAVE_DATA, 1 << (irq - 8))
    };

    unsafe {
        let current_mask = port.inb();
        match mode {
            Mask::Clear => port.outb(current_mask & !mask),
            Mask::Set => port.outb(current_mask | mask),
        }
        io::wait();
    }
}

fn send_eoi(irq: Interrupts) {
    const EOI: u8 = 0x20;

    let irq = irq as u8;
    if irq >= 8 {
        unsafe { SLAVE_COMMAND.outb(EOI) };
    }
    unsafe { MASTER_COMMAND.outb(EOI) };
}

// Handlers

macro_rules! pic_interrupt {
    ($name:ident, $txt:literal, $irq:expr) => {
        pub extern "x86-interrupt" fn $name(_stack_frame: InterruptStackFrame) {
            // TODO: Translate the scan code
            let scan_code = unsafe { $crate::kernel::pic::PS2_DATA_PORT.inb() };
            println!("Interrupt: {}, Scan Code: {scan_code:#X}", $txt);
            crate::kernel::pic::send_eoi($irq);
        }
    };
}

pub mod handlers {
    use crate::{
        kernel::{interrupts::InterruptStackFrame, pic::Interrupts},
        println,
    };

    pic_interrupt!(timer, "TIMER", Interrupts::Timer);
    pic_interrupt!(keyboard, "KEYBOARD", Interrupts::Keyboard);
}

// PS/2 Controller

const PS2_DATA_PORT: Port<u8> = Port::new(0x60);
const PS2_COMMAND_PORT: Port<u8> = Port::new(0x64);


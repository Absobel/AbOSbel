use crate::arch::{
    idt::InterruptStackFrame,
    port::{self, Port},
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
pub enum Interrupts {
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
        port::wait();
        SLAVE_COMMAND.outb(ICW1_INIT | ICW1_ICW4);
        port::wait();
        // Vector offsets
        MASTER_DATA.outb(MASTER_OFFSET);
        port::wait();
        SLAVE_DATA.outb(SLAVE_OFFSET);
        port::wait();
        // Cascade configuration
        MASTER_DATA.outb(1 << CASCADE_IRQ);
        port::wait();
        SLAVE_DATA.outb(CASCADE_IRQ);
        port::wait();
        // Use the 8086/88 mode
        MASTER_DATA.outb(ICW4_8086);
        port::wait();
        SLAVE_DATA.outb(ICW4_8086);
        port::wait();
        // Mask all interrupts
        MASTER_DATA.outb(0xFF);
        port::wait();
        SLAVE_DATA.outb(0xFF);
        port::wait();
    }

    mask_irq(Mask::Clear, Interrupts::Timer);
    mask_irq(Mask::Clear, Interrupts::Cascade);
    mask_irq(Mask::Clear, Interrupts::Keyboard);
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
        port::wait();
    }
}

pub fn send_eoi(irq: Interrupts) {
    const EOI: u8 = 0x20;

    let irq = irq as u8;
    if irq >= 8 {
        unsafe { SLAVE_COMMAND.outb(EOI) };
    }
    unsafe { MASTER_COMMAND.outb(EOI) };
}

// Handlers

pub extern "x86-interrupt" fn timer(_stack_frame: InterruptStackFrame) {
    send_eoi(Interrupts::Timer);
}

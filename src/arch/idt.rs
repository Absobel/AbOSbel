use core::{arch::asm, ops::Deref};
use spin::Once;

use crate::{
    arch::{exceptions, gdt::KERNEL_CS},
    drivers::{keyboard::keyboard, pic},
};

static IDT_ARR: Once<IdtArr> = Once::new();

const NO_IST: u8 = 0;
const DOUBLE_FAULT_IST: u8 = 1;

pub fn init() {
    let idt_arr: &'static IdtArr = IDT_ARR.call_once(|| {
        let mut idt_arr = IdtArr::new();

        idt_arr.set(0x0, exceptions::divide_error, NO_IST);
        idt_arr.set(0x1, exceptions::debug, NO_IST);
        idt_arr.set(0x2, exceptions::nmi, NO_IST);
        idt_arr.set(0x3, exceptions::breakpoint, NO_IST);
        idt_arr.set(0x4, exceptions::overflow, NO_IST);
        idt_arr.set(0x5, exceptions::bound_range_exceeded, NO_IST);
        idt_arr.set(0x6, exceptions::invalid_opcode, NO_IST);
        idt_arr.set(0x7, exceptions::device_not_available, NO_IST);
        idt_arr.set_err(0x8, exceptions::double_fault, DOUBLE_FAULT_IST);
        idt_arr.set_err(0xA, exceptions::invalid_tss, NO_IST);
        idt_arr.set_err(0xB, exceptions::segment_not_present, NO_IST);
        idt_arr.set_err(0xC, exceptions::stack_segment_fault, NO_IST);
        idt_arr.set_err(0xD, exceptions::general_protection_fault, NO_IST);
        idt_arr.set_err(0xE, exceptions::page_fault, NO_IST);
        idt_arr.set(0x10, exceptions::x87_floating_point, NO_IST);
        idt_arr.set_err(0x11, exceptions::alignment_check, NO_IST);
        idt_arr.set(0x12, exceptions::machine_check, NO_IST);
        idt_arr.set(0x13, exceptions::simd_floating_point, NO_IST);
        idt_arr.set(0x14, exceptions::virtualization, NO_IST);
        idt_arr.set_err(0x15, exceptions::control_protection, NO_IST);

        idt_arr.set(0x20, pic::timer, NO_IST);
        idt_arr.set(0x21, keyboard, NO_IST);

        idt_arr
    });

    Idt::new(idt_arr).load();
}

///// IDT

// This struct represents what will be loaded into the IDTR register with `lidt`
#[repr(C, packed)]
struct Idt {
    limit: u16, // The size of the IDT - 1
    base: u64,  // The address of the IDT
}

impl Idt {
    fn new(idt: &IdtArr) -> Self {
        Self {
            limit: (u16::try_from(idt.len() * core::mem::size_of::<IDTEntry>() - 1)
                .expect("IDT too large")),
            base: idt.as_ptr() as u64,
        }
    }

    fn load(&self) {
        unsafe {
            asm!("lidt [{}]", in(reg) self);
        }
    }
}

type Handler = extern "x86-interrupt" fn(InterruptStackFrame);
type HandlerErr = extern "x86-interrupt" fn(InterruptStackFrame, u64);

struct IdtArr {
    entries: [IDTEntry; 256],
}

impl IdtArr {
    const fn new() -> Self {
        Self {
            entries: [IDTEntry::null(); 256],
        }
    }

    fn add_entry(&mut self, index: u8, handler: *const (), ist: u8) {
        self.entries[index as usize] = IDTEntry::new(
            handler as u64,
            KERNEL_CS,
            ist,
            IDTGateType::InterruptGate,
            0,
        );
    }

    fn set(&mut self, index: u8, h: Handler, ist: u8) {
        self.add_entry(index, h as *const (), ist);
    }
    fn set_err(&mut self, index: u8, h: HandlerErr, ist: u8) {
        self.add_entry(index, h as *const (), ist);
    }
}

impl Deref for IdtArr {
    type Target = [IDTEntry; 256];

    fn deref(&self) -> &Self::Target {
        &self.entries
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct IDTEntry {
    offset_low: u16,
    selector: u16,
    ist: u8, // ist 3bit + reserved
    flags: u8,
    offset_mid: u16,
    offset_high: u32,
    _reserved: u32,
}

impl IDTEntry {
    // offset : address of the interrupt handler
    // selector : code segment selector
    // ist : interrupt stack table offset
    // gate_type : type of the gate (interrupt or trap)
    // dpl : descriptor privilege level
    const fn new(offset: u64, selector: u16, ist: u8, gate_type: IDTGateType, dpl: u8) -> Self {
        Self {
            offset_low: offset as u16,
            selector,
            ist,
            flags: (1 << 7) | (dpl << 5) | (gate_type as u8),
            offset_mid: (offset >> 16) as u16,
            offset_high: (offset >> 32) as u32,
            _reserved: 0,
        }
    }

    const fn null() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            ist: 0,
            flags: 0,
            offset_mid: 0,
            offset_high: 0,
            _reserved: 0,
        }
    }
}

#[allow(dead_code)]
enum IDTGateType {
    InterruptGate = 0b1110,
    TrapGate = 0b1111,
}

#[derive(Debug)]
#[repr(C)]
pub struct InterruptStackFrame {
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

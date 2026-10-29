#![allow(clippy::fn_to_numeric_cast)]

use core::{arch::asm, ops::Deref};
use spin::Once;

use crate::KERNEL_CS;

static IDT_ARR: Once<IdtArr> = Once::new();

macro_rules! exception {
    ($name:ident, $txt:literal) => {
        pub(super) extern "x86-interrupt" fn $name(stack_frame: InterruptStackFrame) {
            panic!("{} at address {:#x}", $txt, stack_frame.rip)
        }
    };
    ($name:ident, $txt:literal, $err:ident) => {
        pub(super) extern "x86-interrupt" fn $name(stack_frame: InterruptStackFrame, $err: u64) {
            panic!(
                "{} at address {:#x} with error code: {}",
                $txt, stack_frame.rip, $err
            )
        }
    };
}

mod handlers {
    use core::arch::asm;

    use super::InterruptStackFrame;
    use crate::println;

    exception!(divide_error, "DIVISION ERROR");
    exception!(debug, "DEBUG EXCEPTION");
    exception!(nmi, "NON-MASKABLE INTERRUPT");
    exception!(overflow, "OVERFLOW");
    exception!(bound_range_exceeded, "BOUND RANGE EXCEEDED");
    exception!(invalid_opcode, "INVALID OPCODE");
    exception!(device_not_available, "DEVICE NOT AVAILABLE");
    exception!(double_fault, "DOUBLE FAULT", error_code);
    exception!(invalid_tss, "INVALID TSS", error_code);
    exception!(segment_not_present, "SEGMENT NOT PRESENT", error_code);
    exception!(stack_segment_fault, "STACK SEGMENT FAULT", error_code);
    exception!(
        general_protection_fault,
        "GENERAL PROTECTION FAULT",
        error_code
    );
    exception!(x87_floating_point, "x87 FLOATING POINT EXCEPTION");
    exception!(alignment_check, "ALIGNMENT CHECK", error_code);
    exception!(machine_check, "MACHINE CHECK");
    exception!(simd_floating_point, "SIMD FLOATING POINT EXCEPTION");
    exception!(virtualization, "VIRTUALIZATION EXCEPTION");
    exception!(
        control_protection,
        "CONTROL PROTECTION EXCEPTION",
        error_code
    );

    // TODO: can deadlock if the writer locks are held. Whenever IRQs are enabled, keep the text to write after the lock is released
    pub(super) extern "x86-interrupt" fn breakpoint(stack_frame: InterruptStackFrame) {
        println!("BREAKPOINT at address {:#x}", stack_frame.rip);
    }

    // TODO: decode error code
    pub(super) extern "x86-interrupt" fn page_fault(
        stack_frame: InterruptStackFrame,
        error_code: u64,
    ) {
        let cr2: u64;
        unsafe { asm!("mov {}, cr2", out(reg) cr2) };
        panic!(
            "PAGE FAULT at address {:#x} with error code: {}\n\
                Trying to access address: {:#x}",
            stack_frame.rip, error_code, cr2
        );
    }
}

const NO_IST: u8 = 0;
const DOUBLE_FAULT_IST: u8 = 1;

pub fn init_idt() {
    let idt_arr: &'static IdtArr = IDT_ARR.call_once(|| {
        let mut idt_arr = IdtArr::new();


        idt_arr.set(0x0, handlers::divide_error, NO_IST);
        idt_arr.set(0x1, handlers::debug, NO_IST);
        idt_arr.set(0x2, handlers::nmi, NO_IST);
        idt_arr.set(0x3, handlers::breakpoint, NO_IST);
        idt_arr.set(0x4, handlers::overflow, NO_IST);
        idt_arr.set(0x5, handlers::bound_range_exceeded, NO_IST);
        idt_arr.set(0x6, handlers::invalid_opcode, NO_IST);
        idt_arr.set(0x7, handlers::device_not_available, NO_IST);
        idt_arr.set_err(0x8, handlers::double_fault, DOUBLE_FAULT_IST);
        idt_arr.set_err(0xA, handlers::invalid_tss, NO_IST);
        idt_arr.set_err(0xB, handlers::segment_not_present, NO_IST);
        idt_arr.set_err(0xC, handlers::stack_segment_fault, NO_IST);
        idt_arr.set_err(0xD, handlers::general_protection_fault, NO_IST);
        idt_arr.set_err(0xE, handlers::page_fault, NO_IST);
        idt_arr.set(0x10, handlers::x87_floating_point, NO_IST);
        idt_arr.set_err(0x11, handlers::alignment_check, NO_IST);
        idt_arr.set(0x12, handlers::machine_check, NO_IST);
        idt_arr.set(0x13, handlers::simd_floating_point, NO_IST);
        idt_arr.set(0x14, handlers::virtualization, NO_IST);
        idt_arr.set_err(0x15, handlers::control_protection, NO_IST);

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
        self.entries[index as usize] =
            IDTEntry::new(handler as u64, KERNEL_CS, ist, IDTGateType::InterruptGate, 0);
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

// TODO : use bitflags
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
struct InterruptStackFrame {
    rip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64,
}

#![allow(clippy::fn_to_numeric_cast)]

use core::{arch::asm, ops::Deref};
use lazy_static::lazy_static;

// The fact that this work is truly an example of the might humanity is capable of.

lazy_static! {
    static ref sIDT: Idt = {
        // HANDLERS

// 0x00: Division by Zero
extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) -> ! {
    panic!("DIVISION ERROR at address {:#x}", stack_frame.rip);
}

// 0x01: Debug Exception
extern "x86-interrupt" fn debug_handler(stack_frame: InterruptStackFrame) -> ! {
    panic!("DEBUG EXCEPTION at address {:#x}", stack_frame.rip);
}

// 0x02: Non-Maskable Interrupt
extern "x86-interrupt" fn nmi_handler(stack_frame: InterruptStackFrame) -> ! {
    panic!("NON-MASKABLE INTERRUPT at address {:#x}", stack_frame.rip);
}

// 0x03: Breakpoint
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    panic!("BREAKPOINT at address {:#x}", stack_frame.rip);
}

// 0x04: Overflow
extern "x86-interrupt" fn overflow_handler(stack_frame: InterruptStackFrame) -> ! {
    panic!("OVERFLOW at address {:#x}", stack_frame.rip);
}

// 0x05: Bound Range Exceeded
extern "x86-interrupt" fn bound_range_exceeded_handler(stack_frame: InterruptStackFrame) -> ! {
    panic!("BOUND RANGE EXCEEDED at address {:#x}", stack_frame.rip);
}

// 0x06: Invalid Opcode
extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) -> ! {
    panic!("INVALID OPCODE at address {:#x}", stack_frame.rip);
}

// 0x07: Device Not Available
extern "x86-interrupt" fn device_not_available_handler(stack_frame: InterruptStackFrame) -> ! {
    panic!("DEVICE NOT AVAILABLE at address {:#x}", stack_frame.rip);
}

// 0x08: Double Fault
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    panic!(
        "DOUBLE FAULT at address {:#x} with error code: {}",
        stack_frame.rip, error_code
    );
}

// 0x0A: Invalid TSS
extern "x86-interrupt" fn invalid_tss_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    panic!(
        "INVALID TSS at address {:#x} with error code: {}",
        stack_frame.rip, error_code
    );
}

// 0x0B: Segment Not Present
extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    panic!(
        "SEGMENT NOT PRESENT at address {:#x} with error code: {}",
        stack_frame.rip, error_code
    );
}

// 0x0C: Stack Segment Fault
extern "x86-interrupt" fn stack_segment_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    panic!(
        "STACK SEGMENT FAULT at address {:#x} with error code: {}",
        stack_frame.rip, error_code
    );
}

// 0x0D: General Protection Fault
extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    panic!(
        "GENERAL PROTECTION FAULT at address {:#x} with error code: {}",
        stack_frame.rip, error_code
    );
}

// 0x0E: Page Fault
extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    // error_code: u64,
) -> ! {
    panic!("PAGE FAULT at address {:#x}", stack_frame.rip);
}

// 0x10: x87 Floating Point Exception
extern "x86-interrupt" fn x87_floating_point_handler(stack_frame: InterruptStackFrame) -> ! {
    panic!(
        "x87 FLOATING POINT EXCEPTION at address {:#x}",
        stack_frame.rip
    );
}

// 0x11: Alignment Check
extern "x86-interrupt" fn alignment_check_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    panic!(
        "ALIGNMENT CHECK at address {:#x} with error code: {}",
        stack_frame.rip, error_code
    );
}

// 0x12: Machine Check
extern "x86-interrupt" fn machine_check_handler(stack_frame: InterruptStackFrame) -> ! {
    panic!("MACHINE CHECK at address {:#x}", stack_frame.rip);
}

// 0x13: SIMD Floating Point Exception
extern "x86-interrupt" fn simd_floating_point_handler(stack_frame: InterruptStackFrame) -> ! {
    panic!(
        "SIMD FLOATING POINT EXCEPTION at address {:#x}",
        stack_frame.rip
    );
}

// 0x14: Virtualization Exception
extern "x86-interrupt" fn virtualization_handler(stack_frame: InterruptStackFrame) -> ! {
    panic!("VIRTUALIZATION EXCEPTION at address {:#x}", stack_frame.rip);
}

// 0x15: Control Protection Exception
extern "x86-interrupt" fn control_protection_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    panic!(
        "CONTROL PROTECTION EXCEPTION at address {:#x} with error code: {}",
        stack_frame.rip, error_code
    );
}


        let mut idt = IdtArr::new();

        idt.set_entry(
            0x0,
            IDTEntry::new(
                divide_error_handler as u64,
                0x08,
                0,
                IDTGateType::InterruptGate,
                0,
            ),
        );
        idt.set_entry(
            0x1,
            IDTEntry::new(debug_handler as u64, 0x08, 0, IDTGateType::InterruptGate, 0),
        );
        idt.set_entry(
            0x2,
            IDTEntry::new(nmi_handler as u64, 0x08, 0, IDTGateType::InterruptGate, 0),
        );
        idt.set_entry(
            0x3,
            IDTEntry::new(
                breakpoint_handler as u64,
                0x08,
                0,
                IDTGateType::InterruptGate,
                0,
            ),
        );
        idt.set_entry(
            0x4,
            IDTEntry::new(
                overflow_handler as u64,
                0x08,
                0,
                IDTGateType::InterruptGate,
                0,
            ),
        );
        idt.set_entry(
            0x5,
            IDTEntry::new(
                bound_range_exceeded_handler as u64,
                0x08,
                0,
                IDTGateType::InterruptGate,
                0,
            ),
        );
        idt.set_entry(
            0x6,
            IDTEntry::new(
                invalid_opcode_handler as u64,
                0x08,
                0,
                IDTGateType::InterruptGate,
                0,
            ),
        );
        idt.set_entry(
            0x7,
            IDTEntry::new(
                device_not_available_handler as u64,
                0x08,
                0,
                IDTGateType::InterruptGate,
                0,
            ),
        );
        idt.set_entry(
            0x8,
            IDTEntry::new(
                double_fault_handler as u64,
                0x08,
                0,
                IDTGateType::InterruptGate,
                0,
            ),
        );
        idt.set_entry(
            0xA,
            IDTEntry::new(
                invalid_tss_handler as u64,
                0x08,
                0,
                IDTGateType::InterruptGate,
                0,
            ),
        );
        idt.set_entry(
            0xB,
            IDTEntry::new(
                segment_not_present_handler as u64,
                0x08,
                0,
                IDTGateType::InterruptGate,
                0,
            ),
        );
        idt.set_entry(
            0xC,
            IDTEntry::new(
                stack_segment_fault_handler as u64,
                0x08,
                0,
                IDTGateType::InterruptGate,
                0,
            ),
        );
        idt.set_entry(
            0xD,
            IDTEntry::new(
                general_protection_fault_handler as u64,
                0x08,
                0,
                IDTGateType::InterruptGate,
                0,
            ),
        );
        idt.set_entry(
            0xE,
            IDTEntry::new(
                page_fault_handler as u64,
                0x08,
                0,
                IDTGateType::InterruptGate,
                0,
            ),
        );
        idt.set_entry(
            0x10,
            IDTEntry::new(
                x87_floating_point_handler as u64,
                0x08,
                0,
                IDTGateType::InterruptGate,
                0,
            ),
        );
        idt.set_entry(
            0x11,
            IDTEntry::new(
                alignment_check_handler as u64,
                0x08,
                0,
                IDTGateType::InterruptGate,
                0,
            ),
        );
        idt.set_entry(
            0x12,
            IDTEntry::new(
                machine_check_handler as u64,
                0x08,
                0,
                IDTGateType::InterruptGate,
                0,
            ),
        );
        idt.set_entry(
            0x13,
            IDTEntry::new(
                simd_floating_point_handler as u64,
                0x08,
                0,
                IDTGateType::InterruptGate,
                0,
            ),
        );
        idt.set_entry(
            0x14,
            IDTEntry::new(
                virtualization_handler as u64,
                0x08,
                0,
                IDTGateType::InterruptGate,
                0,
            ),
        );
        idt.set_entry(
            0x15,
            IDTEntry::new(
                control_protection_handler as u64,
                0x08,
                0,
                IDTGateType::InterruptGate,
                0,
            ),
        );

        Idt::new(idt)
    };
}

pub fn init_idt() {
    sIDT.load();
}

///// IDT

// This struct represents what will be loaded into the IDTR register with `lidt`
#[repr(C, packed)]
struct Idt {
    limit: u16, // The size of the IDT - 1
    base: u64,  // The address of the IDT
}

impl Idt {
    fn new(idt: IdtArr) -> Self {
        Idt {
            limit: (idt.len() * core::mem::size_of::<IDTEntry>() - 1) as u16,
            base: idt.as_ptr() as u64,
        }
    }

    fn load(&self) {
        unsafe {
            asm!("lidt [{}]", in(reg) self);
        }
    }
}

struct IdtArr {
    entries: [IDTEntry; 256],
}

impl IdtArr {
    fn new() -> Self {
        IdtArr {
            entries: [IDTEntry::null(); 256],
        }
    }

    fn set_entry(&mut self, index: usize, entry: IDTEntry) {
        self.entries[index] = entry;
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
    fn new(offset: u64, selector: u16, ist: u8, gate_type: IDTGateType, dpl: u8) -> Self {
        IDTEntry {
            offset_low: offset as u16,
            selector,
            ist,
            flags: (1 << 7) | (dpl << 5) | (gate_type as u8),
            offset_mid: (offset >> 16) as u16,
            offset_high: (offset >> 32) as u32,
            _reserved: 0,
        }
    }

    fn null() -> Self {
        IDTEntry {
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

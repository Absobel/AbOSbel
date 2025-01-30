use core::{arch::asm, ops::Deref};
use lazy_static::lazy_static;

use crate::serial_println;

// The fact that this work is truly an example of the might humanity is capable of.

// This struct represents what will be loaded into the IDTR register with `lidt`
#[repr(C, packed)]
struct Idt {
    limit: u16, // The size of the IDT - 1
    base: u64, // The address of the IDT
}

impl Idt {
    fn new(idt: IDTarr) -> Self {
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

struct IDTarr {
    entries: [IDTEntry; 256],
}

impl IDTarr {
    fn new() -> Self {
        IDTarr {
            entries: [IDTEntry::null(); 256],
        }
    }

    fn set_entry(&mut self, index: usize, entry: IDTEntry) {
        self.entries[index] = entry;
    }
}

impl Deref for IDTarr {
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

fn double_fault_handler() {
    serial_println!("Double fault!");
    panic!("Double fault!");
}

// This is the IDT that will be loaded into the IDTR register
lazy_static! {
    static ref sIDT: Idt = {
        let mut idt = IDTarr::new();

        #[allow(clippy::fn_to_numeric_cast)]
        idt.set_entry(8, IDTEntry::new(double_fault_handler as u64, 0x08, 0, IDTGateType::InterruptGate, 0));

        Idt::new(idt)
    };
}

pub fn init_idt() {
    sIDT.load();
}
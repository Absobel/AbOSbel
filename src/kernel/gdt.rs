use core::arch::asm;

use lazy_static::lazy_static;

//// Initialization

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::empty();
        tss.iopb_offset = core::mem::size_of::<TaskStateSegment>() as u16; // essentially disabling IOPB
        tss
    };

    static ref sGDT: Gdt = {
        let mut gdt = GdtArr::new();
        gdt.tss_descriptor.change_base(&TSS as *const _ as u64);
        Gdt::new(gdt)
    };
}

pub fn init() {
    sGDT.load();
}

/*
static mut TSS: TaskStateSegment = TaskStateSegment::empty();

unsafe fn reload_cs() {
    unsafe{asm!(
        "mov ax, 0x10",
        "mov ds, ax",
        "mov es, ax",
        "mov fs, ax",
        "mov gs, ax",
        "mov ss, ax",
    )}
}

pub fn init() {
    crate::x86::without_interrupts(|| unsafe {
        // compile_error!("TODO : Implement GDT init");

        let tss_ptr = &raw const TSS as u64;
        #[allow(static_mut_refs)] // TODO : do that better
        GDT.change_tss_base(tss_ptr);

        // Load the GDT
        asm!(
            "push 0x08", // TODO : tf is that
            "lea rax, [rip + {rcs}]",
            "push rax",
            "retfq",
            rcs = sym reload_cs,
        )
    })
}

*/

// TODO : use bitflags everywhere

///// GDT

#[repr(C, packed)]
struct Gdt {
    limit: u16,
    base: u64,
}

impl Gdt {
    fn new(gdt: GdtArr) -> Self {
        Gdt {
            limit: (core::mem::size_of::<GdtArr>() - 1) as u16,
            base: &gdt as *const GdtArr as u64,
        }
    }

    fn load(&self) {
        unsafe {
            asm!("lgdt [{}]", in(reg) self);
        }
    }
}

#[repr(C, packed)]
struct GdtArr {
    null_descriptor: GdtNormalDescriptor,
    kernel_code_descriptor: GdtNormalDescriptor,
    kernel_data_descriptor: GdtNormalDescriptor,
    tss_descriptor: GdtSystemDescriptor,
}

impl GdtArr {
    fn new() -> Self {
        let kernel_code_descriptor = GdtNormalDescriptor::new(
            0x00400000,
            0x003FFFFF,
            GdtNormalAccess::from_u8(0x9A),
            GdtFlags::from_u8(0xC),
        );
        let kernel_data_descriptor = GdtNormalDescriptor::new(
            0x00800000,
            0x003FFFFF,
            GdtNormalAccess::from_u8(0x92),
            GdtFlags::from_u8(0xC),
        );
        let tss_descriptor = GdtSystemDescriptor::new(
            0, // Base will be changed later
            core::mem::size_of::<GdtSystemDescriptor>() as u32,
            GdtSystemAccess::from_u8(0x89),
            GdtFlags::from_u8(0x0),
        );

        GdtArr {
            null_descriptor: GdtNormalDescriptor::null_descriptor(),
            kernel_code_descriptor,
            kernel_data_descriptor,
            tss_descriptor,
        }
    }
}

// The actual GDT entries

// Normal GDT entry

#[repr(C, packed)]
struct GdtNormalDescriptor {
    limit_low: u16,
    base_low: u16,
    base_mid: u8,
    access: u8,
    flags_limit_high: u8,
    base_high: u8,
}

impl GdtNormalDescriptor {
    const fn new(base: u32, limit: u32, access: GdtNormalAccess, flags: GdtFlags) -> Self {
        GdtNormalDescriptor {
            base_high: ((base >> 24) & 0xff) as u8,
            flags_limit_high: ((flags.value() & 0xf) << 4) | (((limit >> 16) & 0xf) as u8),
            access: access.value(),
            base_mid: ((base >> 16) & 0xff) as u8,
            base_low: (base & 0xffff) as u16,
            limit_low: (limit & 0xffff) as u16,
        }
    }

    pub const fn null_descriptor() -> Self {
        Self::new(
            0,
            0,
            GdtNormalAccess {
                present: false,
                privilege: 0,
                descriptor_type: 0,
                executable: false,
                conforming: 0,
                readable_writable: 0,
                accessed: false,
            },
            GdtFlags {
                granularity: 0,
                size: 0,
                long_mode_code: 0,
            },
        )
    }
}

// System GDT entry

#[repr(C, packed)]
struct GdtSystemDescriptor {
    limit_low: u16,
    base_low: u16,
    base_mid: u8,
    access: u8,
    flags_limit_high: u8,
    base_high: u8,
    base_very_high: u32,
    _reserved: u32,
}

impl GdtSystemDescriptor {
    const fn new(base: u64, limit: u32, access: GdtSystemAccess, granularity: GdtFlags) -> Self {
        GdtSystemDescriptor {
            _reserved: 0,
            base_very_high: ((base >> 32) & 0xffffffff) as u32,
            base_high: ((base >> 24) & 0xff) as u8,
            flags_limit_high: ((granularity.value() & 0xf) << 4) & (((limit >> 16) & 0xf) as u8),
            access: access.value(),
            base_mid: ((base >> 16) & 0xff) as u8,
            base_low: (base & 0xffff) as u16,
            limit_low: (limit & 0xffff) as u16,
        }
    }

    fn change_base(&mut self, base: u64) {
        self.base_very_high = ((base >> 32) & 0xffffffff) as u32;
        self.base_high = ((base >> 24) & 0xff) as u8;
        self.base_mid = ((base >> 16) & 0xff) as u8;
        self.base_low = (base & 0xffff) as u16;
    }
}

///// Other random things

/// Normal Descriptor Access Byte
struct GdtNormalAccess {
    present: bool,       // 1 bit : true if valid segment
    privilege: u8,       // 2 bits : 0-3 ring level
    descriptor_type: u8, // 1 bit : 0 for system, 1 for code/data
    executable: bool,    // 1 bit : true for code segment, false for data segment
    // 1 bit
    // Data segment : 0 for growing up, 1 for growing down
    // Code segment : 0 exact privilege level, 1 for also lower privilege level
    conforming: u8,
    // 1 bit
    // Data segment : 0 for read only, 1 for read/write
    // Code segment : 0 for execute only, 1 for execute/read
    readable_writable: u8,
    accessed: bool, // 1 bit : set by CPU when segment is accessed, best to set to 1 to avoid page fault
}

impl GdtNormalAccess {
    const fn value(&self) -> u8 {
        let mut value = 0;
        value |= if self.present { 1 << 7 } else { 0 };
        value |= (self.privilege & 0b11) << 5;
        value |= if self.descriptor_type != 0 { 1 << 4 } else { 0 };
        value |= if self.executable { 1 << 3 } else { 0 };
        value |= (self.conforming & 0b1) << 2;
        value |= (self.readable_writable & 0b1) << 1;
        value |= if self.accessed { 1 } else { 0 };
        value
    }

    const fn from_u8(value: u8) -> Self {
        GdtNormalAccess {
            present: (value & (1 << 7)) != 0,
            privilege: (value >> 5) & 0b11,
            descriptor_type: (value >> 4) & 1,
            executable: (value & (1 << 3)) != 0,
            conforming: (value >> 2) & 1,
            readable_writable: (value >> 1) & 1,
            accessed: (value & 1) != 0,
        }
    }
}

struct GdtSystemAccess {
    present: bool,       // 1 bit : true if valid segment
    privilege: u8,       // 2 bits : 0-3 ring level
    descriptor_type: u8, // 1 bit : 0 for system, 1 for code/data
    // 4 bits
    // In long mode :
    // 0x2: LDT
    // 0x9: 64-bit TaskStateSegment (Available)
    // 0xB: 64-bit TaskStateSegment (Busy)
    segment_type: u8,
}

impl GdtSystemAccess {
    const fn value(&self) -> u8 {
        let mut value = 0;
        value |= if self.present { 1 << 7 } else { 0 };
        value |= (self.privilege & 0b11) << 5;
        value |= if self.descriptor_type != 0 { 1 << 4 } else { 0 };
        value |= self.segment_type & 0xf;
        value
    }

    const fn from_u8(value: u8) -> Self {
        GdtSystemAccess {
            present: (value & (1 << 7)) != 0,
            privilege: (value >> 5) & 0b11,
            descriptor_type: (value >> 4) & 1,
            segment_type: value & 0xf,
        }
    }
}

// 4bit flags for GDT entries

struct GdtFlags {
    granularity: u8, // 1 bit : 0 limit in bytes, 1 limit in 4KB pages
    size: u8,        // 1 bit : 0 16 bit, 1 32 bit, MUST BE CLEARED FOR LONG MODE CODE SEGMENTS
    long_mode_code: u8, // 1 bit : 0 for 16/32 bit or data segment, 1 for long mode code segment
                     // 1 bit : reserved
}

impl GdtFlags {
    const fn value(&self) -> u8 {
        let mut value = 0;
        value |= (self.granularity & 0b1) << 3;
        value |= (self.size & 0b1) << 2;
        value |= (self.long_mode_code & 0b1) << 1;
        value
    }

    const fn from_u8(value: u8) -> Self {
        GdtFlags {
            granularity: (value >> 3) & 1,
            size: (value >> 2) & 1,
            long_mode_code: (value >> 1) & 1,
        }
    }
}

// Task State Segment

#[allow(dead_code)]
#[repr(C, packed)]
struct TaskStateSegment {
    _reserved1: u32,
    rsp0: u64,
    rsp1: u64,
    rsp2: u64,
    _reserved2: u64,
    ist1: u64,
    ist2: u64,
    ist3: u64,
    ist4: u64,
    ist5: u64,
    ist6: u64,
    ist7: u64,
    _reserved3: u64,
    _reserved4: u16,
    iopb_offset: u16,
}

impl TaskStateSegment {
    const fn empty() -> Self {
        TaskStateSegment {
            _reserved1: 0,
            rsp0: 0,
            rsp1: 0,
            rsp2: 0,
            _reserved2: 0,
            ist1: 0,
            ist2: 0,
            ist3: 0,
            ist4: 0,
            ist5: 0,
            ist6: 0,
            ist7: 0,
            _reserved3: 0,
            _reserved4: 0,
            iopb_offset: 0,
        }
    }
}

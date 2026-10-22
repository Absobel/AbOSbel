use core::arch::asm;

use spin::Once;

// Initialization

static TSS: Once<TaskStateSegment> = Once::new();
static GDT_ARR: Once<GdtDescriptors> = Once::new();

pub fn init() {
    let tss: &'static TaskStateSegment = TSS.call_once(|| {
        let mut tss = TaskStateSegment::empty();
        tss.iopb_offset = core::mem::size_of::<TaskStateSegment>() as u16; // essentially disabling IOPB
        tss
    });

    let gdt_arr: &'static GdtDescriptors = GDT_ARR.call_once(|| {
        let mut gdt_arr = GdtDescriptors::new();
        gdt_arr.tss.change_base(core::ptr::from_ref(tss) as u64);
        gdt_arr
    });

    let mut gdt = Gdt::empty();
    gdt.set_base(gdt_arr);
    gdt.load();
}

// TODO : use bitflags everywhere

///// GDT

#[repr(C, packed)]
struct Gdt {
    limit: u16,
    base: u64,
}

impl Gdt {
    const fn empty() -> Self {
        Self {
            limit: (core::mem::size_of::<GdtDescriptors>() - 1) as u16,
            base: 0,
        }
    }

    fn set_base(&mut self, gdt_arr: &GdtDescriptors) {
        self.base = core::ptr::from_ref::<GdtDescriptors>(gdt_arr) as u64;
    }

    fn load(&self) {
        unsafe {
            // readonly: lgdt only reads the GdtArr through the pointer then just pushes to a register, no writing memory
            // nostack: lgdt doesn't touch the stack
            // preserves_flags: no arithmetic
            asm!("lgdt [{}]", in(reg) self, options(readonly, nostack, preserves_flags));
            // Reload code segment register
            asm!(
                "push 0x08", // Refers to the 8th byte of the GDT, which is the kernel code segment descriptor
                "lea {tmp}, [rip + 2f]",
                "push {tmp}",
                "retfq",
                "2:",
                "mov ax, 0x10",
                "mov ds, ax",
                "mov es, ax",
                "mov fs, ax",
                "mov gs, ax",
                "mov ss, ax",
                tmp = out(reg) _,
            );
        }
    }
}

#[repr(C, packed)]
struct GdtDescriptors {
    null: GdtNormalDescriptor,
    kernel_code: GdtNormalDescriptor,
    kernel_data: GdtNormalDescriptor,
    tss: GdtSystemDescriptor,
}

impl GdtDescriptors {
    const fn new() -> Self {
        // In 64bit mode, base and limit are ignored for data and code segments
        let kernel_code = GdtNormalDescriptor::new(
            0x0,
            0x0,
            &GdtNormalAccess::from_u8(0x9A),
            &GdtFlags::from_u8(0b1010),
        );
        let kernel_data = GdtNormalDescriptor::new(
            0x0,
            0x0,
            &GdtNormalAccess::from_u8(0x92),
            &GdtFlags::from_u8(0b1000),
        );
        let tss = GdtSystemDescriptor::new(
            0, // Base will be changed later
            core::mem::size_of::<TaskStateSegment>() as u32 - 1,
            &GdtSystemAccess::from_u8(0x89),
            &GdtFlags::from_u8(0x0),
        );

        Self {
            null: GdtNormalDescriptor::null_descriptor(),
            kernel_code,
            kernel_data,
            tss,
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
    const fn new(base: u32, limit: u32, access: &GdtNormalAccess, flags: &GdtFlags) -> Self {
        Self {
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
            &GdtNormalAccess {
                present: false,
                privilege: 0,
                descriptor_type: 0,
                executable: false,
                conforming: 0,
                readable_writable: 0,
                accessed: false,
            },
            &GdtFlags {
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
    const fn new(base: u64, limit: u32, access: &GdtSystemAccess, granularity: &GdtFlags) -> Self {
        Self {
            _reserved: 0,
            base_very_high: ((base >> 32) & 0xffff_ffff) as u32,
            base_high: ((base >> 24) & 0xff) as u8,
            flags_limit_high: ((granularity.value() & 0xf) << 4) | (((limit >> 16) & 0xf) as u8),
            access: access.value(),
            base_mid: ((base >> 16) & 0xff) as u8,
            base_low: (base & 0xffff) as u16,
            limit_low: (limit & 0xffff) as u16,
        }
    }

    const fn change_base(&mut self, base: u64) {
        self.base_very_high = ((base >> 32) & 0xffff_ffff) as u32;
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
        Self {
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
        Self {
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
        Self {
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
        Self {
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

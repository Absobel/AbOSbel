// Access Byte for an entry in the GDT
// TODO : Enums or const for the flags or smth
// REWRITE FUCKING EVERYTHING

/// Normal Descriptor Access Byte
pub struct GdtNormalAccess {
    pub present: bool,       // 1 bit : true if valid segment
    pub privilege: u8,       // 2 bits : 0-3 ring level
    pub descriptor_type: u8, // 1 bit : 0 for system, 1 for code/data
    pub executable: bool,    // 1 bit : true for code segment, false for data segment
    // 1 bit
    // Data segment : 0 for growing up, 1 for growing down
    // Code segment : 0 exact privilege level, 1 for also lower privilege level
    pub conforming: u8,
    // 1 bit
    // Data segment : 0 for read only, 1 for read/write
    // Code segment : 0 for execute only, 1 for execute/read
    pub readable_writable: u8,
    pub accessed: bool, // 1 bit : set by CPU when segment is accessed, best to set to 1 to avoid page fault
}

impl GdtNormalAccess {
    pub const fn value(&self) -> u8 {
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
}

// System Descriptor

pub struct GdtSystemAccess {
    pub present: bool,       // 1 bit : true if valid segment
    pub privilege: u8,       // 2 bits : 0-3 ring level
    pub descriptor_type: u8, // 1 bit : 0 for system, 1 for code/data
    // 4 bits
    // In long mode :
    // 0x2: LDT
    // 0x9: 64-bit TaskStateSegment (Available)
    // 0xB: 64-bit TaskStateSegment (Busy)
    pub segment_type: u8,
}

impl GdtSystemAccess {
    pub const fn value(&self) -> u8 {
        let mut value = 0;
        value |= if self.present { 1 << 7 } else { 0 };
        value |= (self.privilege & 0b11) << 5;
        value |= if self.descriptor_type != 0 { 1 << 4 } else { 0 };
        value |= self.segment_type & 0xf;
        value
    }
}

// 4bit flags for GDT entries

pub struct GdtFlags {
    pub granularity: u8, // 1 bit : 0 limit in bytes, 1 limit in 4KB pages
    pub size: u8,        // 1 bit : 0 16 bit, 1 32 bit, MUST BE CLEARED FOR LONG MODE CODE SEGMENTS
    pub long_mode_code: u8, // 1 bit : 0 for 16/32 bit or data segment, 1 for long mode code segment
                         // 1 bit : reserved
}

impl GdtFlags {
    pub const fn value(&self) -> u8 {
        let mut value = 0;
        value |= (self.granularity & 0b1) << 3;
        value |= (self.size & 0b1) << 2;
        value |= (self.long_mode_code & 0b1) << 1;
        value
    }
}

// The actual GDT entries

// Normal GDT entry

#[repr(C, packed)]
pub struct GdtNormalDescriptor {
    base_high: u8,
    flags_limit_high: u8,
    access: u8,
    base_mid: u8,
    base_low: u16,
    limit_low: u16,
}

impl GdtNormalDescriptor {
    pub const fn new(
        base: u32,
        limit: u32,
        access: GdtNormalAccess,
        granularity: GdtFlags,
    ) -> Self {
        GdtNormalDescriptor {
            base_high: ((base >> 24) & 0xff) as u8,
            flags_limit_high: ((granularity.value() & 0xf) << 4) & (((limit >> 16) & 0xf) as u8),
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
pub struct GdtSystemDescriptor {
    _reserved: u32,
    base_very_high: u32,
    base_high: u8,
    flags_limit_high: u8,
    access: u8,
    base_mid: u8,
    base_low: u16,
    limit_low: u16,
}

impl GdtSystemDescriptor {
    pub const fn new(
        base: u64,
        limit: u32,
        access: GdtSystemAccess,
        granularity: GdtFlags,
    ) -> Self {
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

    pub fn change_base(&mut self, base: u64) {
        self.base_very_high = ((base >> 32) & 0xffffffff) as u32;
        self.base_high = ((base >> 24) & 0xff) as u8;
        self.base_mid = ((base >> 16) & 0xff) as u8;
        self.base_low = (base & 0xffff) as u16;
    }
}

// Task State Segment

#[repr(C, packed)]
pub struct TaskStateSegment {
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
    iopb: u16,
}

#[allow(clippy::too_many_arguments)]
impl TaskStateSegment {
    pub const fn new(
        rsp0: u64,
        rsp1: u64,
        rsp2: u64,
        ist1: u64,
        ist2: u64,
        ist3: u64,
        ist4: u64,
        ist5: u64,
        ist6: u64,
        ist7: u64,
    ) -> TaskStateSegment {
        TaskStateSegment {
            _reserved1: 0,
            rsp0,
            rsp1,
            rsp2,
            _reserved2: 0,
            ist1,
            ist2,
            ist3,
            ist4,
            ist5,
            ist6,
            ist7,
            _reserved3: 0,
            _reserved4: 0,
            iopb: 104, // IOPB offset, set to size of TaskStateSegment (104 bytes) to disable
        }
    }

    pub const fn empty() -> TaskStateSegment {
        TaskStateSegment::new(0, 0, 0, 0, 0, 0, 0, 0, 0, 0)
    }
}

use core::marker::PhantomData;

use crate::memory::{self, PhysicalAddress};

#[repr(transparent)]
#[derive(Clone, Copy)]
struct RawEntry(u64);

/// Level 2-4 page table entry
#[expect(clippy::struct_excessive_bools)]
struct TableEntry {
    present: bool,
    writable: bool,        // or read-only
    user_accessible: bool, // or supervisor-only
    /// Goes to both the cache and directly to memory
    write_through: bool,
    /// Disables the cache entirely
    cache_disabled: bool,
    /// Written by the CPU to say this page has been accessed
    accessed: bool,
    physical_address: PhysicalAddress,
    disabled_execute: bool,
    /// Available to me to use whatever i want :shrug:
    /// bits 6, 8, 9-11 && 52-62
    free_territory_baby: u16,
}

impl From<TableEntry> for RawEntry {
    fn from(entry: TableEntry) -> Self {
        let mut raw = 0u64;
        if entry.present {
            raw |= 1 << 0;
        }
        if entry.writable {
            raw |= 1 << 1;
        }
        if entry.user_accessible {
            raw |= 1 << 2;
        }
        if entry.write_through {
            raw |= 1 << 3;
        }
        if entry.cache_disabled {
            raw |= 1 << 4;
        }
        if entry.accessed {
            raw |= 1 << 5;
        }
        // bits 12 .. (MAXPHYADDR -1)
        raw |= entry.physical_address.0 as u64 & ((1 << memory::MAXPHYADDR) - 1) & !0xFFF;
        if entry.disabled_execute {
            raw |= 1 << 63;
        }
        let free = u64::from(entry.free_territory_baby);
        raw |= (free & 0x1) << 6;
        raw |= ((free >> 1) & 0xF) << 8;
        raw |= ((free >> 5) & 0x7FF) << 52;
        Self(raw)
    }
}

impl From<RawEntry> for TableEntry {
    fn from(raw: RawEntry) -> Self {
        Self {
            present: raw.0 & (1 << 0) != 0,
            writable: raw.0 & (1 << 1) != 0,
            user_accessible: raw.0 & (1 << 2) != 0,
            write_through: raw.0 & (1 << 3) != 0,
            cache_disabled: raw.0 & (1 << 4) != 0,
            accessed: raw.0 & (1 << 5) != 0,
            physical_address: PhysicalAddress(
                raw.0 as usize & ((1 << memory::MAXPHYADDR) - 1) & !0xFFF,
            ),
            disabled_execute: raw.0 & (1 << 63) != 0,
            free_territory_baby: (((raw.0 >> 6) & 0x1)
                | (((raw.0 >> 8) & 0xF) << 1)
                | (((raw.0 >> 52) & 0x7FF) << 5)) as u16,
        }
    }
}

/// Level 1 page table entry
#[expect(clippy::struct_excessive_bools)]
struct PageEntry {
    present: bool,
    writable: bool,        // or read-only
    user_accessible: bool, // or supervisor-only
    /// PWT: Goes to both the cache and directly to memory
    write_through: bool,
    /// PCD: Disables the cache entirely
    cache_disabled: bool,
    /// Written by the CPU to say this page has been accessed
    accessed: bool,
    /// Written by the CPU to say this page has been written to
    dirty: bool,
    /// TODO: write comment and use it for WC in framebuffer
    pat: bool,
    /// Tells the CPU to keep this page's TLB entry when CR3 is reloaded.
    /// CR4:0b7 (PGE) must be set for global pages to work
    global: bool,
    physical_address: PhysicalAddress,
    disabled_execute: bool,
    // TODO: Add warning when the top two bits are set when they're not used
    /// Available to me to use whatever i want :shrug:
    /// bits 9-11 && bits 52-62 (well there's Protection Key at 59-62 but unused for now)
    free_territory_baby: u16,
}

impl From<PageEntry> for RawEntry {
    fn from(entry: PageEntry) -> Self {
        let mut raw = 0u64;
        if entry.present {
            raw |= 1 << 0;
        }
        if entry.writable {
            raw |= 1 << 1;
        }
        if entry.user_accessible {
            raw |= 1 << 2;
        }
        if entry.write_through {
            raw |= 1 << 3;
        }
        if entry.cache_disabled {
            raw |= 1 << 4;
        }
        if entry.accessed {
            raw |= 1 << 5;
        }
        if entry.dirty {
            raw |= 1 << 6;
        }
        if entry.pat {
            raw |= 1 << 7;
        }
        if entry.global {
            raw |= 1 << 8;
        }
        // bits 12 .. (MAXPHYADDR -1)
        raw |= entry.physical_address.0 as u64 & ((1 << memory::MAXPHYADDR) - 1) & !0xFFF;
        if entry.disabled_execute {
            raw |= 1 << 63;
        }
        let free = u64::from(entry.free_territory_baby);
        raw |= (free & 0x7) << 9;
        raw |= ((free >> 3) & 0x7FF) << 52;
        Self(raw)
    }
}

impl From<RawEntry> for PageEntry {
    fn from(raw: RawEntry) -> Self {
        Self {
            present: raw.0 & (1 << 0) != 0,
            writable: raw.0 & (1 << 1) != 0,
            user_accessible: raw.0 & (1 << 2) != 0,
            write_through: raw.0 & (1 << 3) != 0,
            cache_disabled: raw.0 & (1 << 4) != 0,
            accessed: raw.0 & (1 << 5) != 0,
            dirty: raw.0 & (1 << 6) != 0,
            pat: raw.0 & (1 << 7) != 0,
            global: raw.0 & (1 << 8) != 0,
            physical_address: PhysicalAddress(
                raw.0 as usize & ((1 << memory::MAXPHYADDR) - 1) & !0xFFF,
            ),
            disabled_execute: raw.0 & (1 << 63) != 0,
            free_territory_baby: (((raw.0 >> 9) & 0x7) | (((raw.0 >> 52) & 0x7FF) << 3)) as u16,
        }
    }
}

#[repr(C, align(4096))]
struct PageTable<L: Level> {
    entries: [RawEntry; 512],
    _level: PhantomData<L>,
}

const _: () = assert!(size_of::<PageTable<L4>>() == 4096);

trait Level {
    const SHIFT: usize;
    type Entry: Into<RawEntry> + From<RawEntry>;
}

trait HasNext: Level {
    type Next: Level;
}

struct L4;
struct L3;
struct L2;
struct L1;

impl Level for L4 {
    const SHIFT: usize = 39;
    type Entry = TableEntry;
}
impl Level for L3 {
    const SHIFT: usize = 30;
    type Entry = TableEntry;
}
impl Level for L2 {
    const SHIFT: usize = 21;
    type Entry = TableEntry;
}
impl Level for L1 {
    const SHIFT: usize = 12;
    type Entry = PageEntry;
}

impl HasNext for L4 {
    type Next = L3;
}
impl HasNext for L3 {
    type Next = L2;
}
impl HasNext for L2 {
    type Next = L1;
}

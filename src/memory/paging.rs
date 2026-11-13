use core::{
    arch::asm, marker::PhantomData, sync::atomic::{AtomicUsize, Ordering},
};

use spin::{Mutex, Once};

use crate::{
    arch::{cpuid, msr::MsrRegister},
    memory::{
        self, PhysicalAddress, VirtualAddress,
        frame::{END_ADDRESS, FRAME_ALLOCATOR},
    },
};

const MSR_PAT: MsrRegister = MsrRegister::new(0x277);
const MSR_EFER: MsrRegister = MsrRegister::new(0xC000_0080);
const MAX_ADDR_FEATURE: cpuid::Feature = cpuid::Feature::new(0x8000_0008, 0, 0, cpuid::Reg::Eax);

pub static L4_TABLE: Once<Mutex<&'static mut PageTable<L4>>> = Once::new();
pub static MAXPHYADDR: AtomicUsize = AtomicUsize::new(0);

/// # Panics
/// Panics if there's no more memory. I don't think it can panic from other things since it's a new page table
pub fn init() {
    MAXPHYADDR.store((MAX_ADDR_FEATURE.read() & 0xFF) as usize, Ordering::Relaxed);

    // Set the MSR EFER.NXE bit for non executable pages
    let efer = unsafe { MSR_EFER.read_msr() };
    unsafe { MSR_EFER.write_msr(efer | (1 << 11)) };

    // Change the MSR PAT stuff to be able to make the framebuffer write combining
    cpuid::Feature::new(0x1, 0, 16, cpuid::Reg::Edx)
        .cpu_has_feature()
        .unwrap_or_else(|e| panic!("{e}"));
    Pat::write_to_msr(MSR_PAT_ARRAY);

    let l4_frame = FRAME_ALLOCATOR
        .lock()
        .alloc()
        .expect("No more memory for page tables");
    l4_frame.zero();
    let l4_table_ptr =
        VirtualAddress::phys_to_virt(l4_frame.address()).as_mut_ptr::<PageTable<L4>>();
    let l4_table = unsafe { &mut *l4_table_ptr };
    L4_TABLE.call_once(|| Mutex::new(l4_table));
    let mut l4_table = L4_TABLE.get().unwrap().lock();

    // Identity map the kernel so that everything still runs after switching to the new page tables

    let (kernel_start, kernel_end) = (memory::kernel_start_addr(), memory::kernel_end_addr());
    unsafe {
        map_range::<Size4K>(
            &mut l4_table,
            VirtualAddress::identity_map(kernel_start),
            kernel_start,
            kernel_end.0 - kernel_start.0,
            PageEntryParameters {
                writable: true,
                user_accessible: false,
                executable: true,
                pat: Pat::WriteBack,
            }
        )
        .unwrap();
    };

    // Map the whole physical memory to +OFFSET
    let phys_mem_start = PhysicalAddress(0);
    let phys_mem_end = PhysicalAddress(memory::align_up(
        END_ADDRESS.load(Ordering::Relaxed),
        1 << Size2M::SHIFT,
    ));
    unsafe {
        map_range::<Size2M>(
            &mut l4_table,
            VirtualAddress::phys_to_virt(phys_mem_start),
            phys_mem_start,
            phys_mem_end.0 - phys_mem_start.0,
            PageEntryParameters {
                writable: true,
                user_accessible: false,
                executable: false,
                pat: Pat::WriteBack,
            }
        )
        .unwrap();
    };

    unsafe {
        asm!(
            "mov cr3, {0}",
            in(reg) l4_frame.address().0,
            options(nostack)
        );
    }

    memory::END_OF_PAGING.store(phys_mem_end.0, Ordering::Relaxed);
}

// MSR PAT //

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pat {
    /// All accesses are uncacheable. Write combining is not allowed. Speculative accesses are not allowed.
    Uncacheable = 0b000,
    /// All accesses are uncacheable. Write combining is allowed. Speculative reads are allowed.
    WriteCombining = 0b001,
    /// Reads allocate cache lines on a cache miss. Cache lines are not allocated on a write miss.
    /// Write hits update the cache and main memory.
    WriteThrough = 0b100,
    /// Reads allocate cache lines on a cache miss. All writes update main memory.
    /// Cache lines are not allocated on a write miss. Write hits invalidate the cache line and update main memory.
    WriteProtected = 0b101,
    /// Reads allocate cache lines on a cache miss, and can allocate to either the shared, exclusive, or modified state. Writes allocate to the modified state on a cache miss.
    WriteBack = 0b110,
    /// Same as uncacheable, except that this can be overriden by Write-Combining MTRRs.
    Uncached = 0b111,
}

impl Pat {
    fn write_to_msr(pat_arr: [Self; 8]) {
        let mut msr_value = 0u64;
        for (i, pat) in pat_arr.iter().enumerate() {
            msr_value |= (*pat as u64) << (i * 8);
        }
        unsafe { MSR_PAT.write_msr(msr_value as usize) };
    }

    fn to_msr_arr_index(self, pat_arr: [Self; 8]) -> u64 {
        let index = pat_arr.iter().position(|&p| p == self).expect("Invalid PAT value");
        (index as u64) & 0b111
    }
}

const MSR_PAT_ARRAY: [Pat; 8] = [
    Pat::WriteBack,
    Pat::WriteThrough,
    Pat::Uncached,
    Pat::Uncacheable,
    Pat::WriteCombining,
    Pat::WriteProtected,
    Pat::Uncached,
    Pat::Uncached,
];

// ENTRIES //

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct RawEntry(u64);

/// Level 2-4 page table entry
#[expect(clippy::struct_excessive_bools)]
pub struct TableEntry {
    writable: bool,        // or read-only
    user_accessible: bool, // or supervisor-only
    /// Written by the CPU to say this page has been accessed
    accessed: bool,
    physical_address: PhysicalAddress,
    disabled_execute: bool,
    /// Available to me to use whatever i want :shrug:
    /// bits 6, 8, 9-11 && 52-62
    free_territory_baby: u16,
}

impl TableEntry {
    const fn pointing_to(table: PhysicalAddress) -> Self {
        Self {
            writable: true,
            user_accessible: true,
            accessed: false,
            physical_address: table,
            disabled_execute: false,
            free_territory_baby: 0,
        }
    }
}

impl From<TableEntry> for RawEntry {
    fn from(entry: TableEntry) -> Self {
        let mut raw = 0u64;
        // present bit is always set for valid entries
        raw |= 1 << 0;
        if entry.writable {
            raw |= 1 << 1;
        }
        if entry.user_accessible {
            raw |= 1 << 2;
        }
        if entry.accessed {
            raw |= 1 << 5;
        }
        // bits 12 .. (MAXPHYADDR -1)
        assert!(entry.physical_address.0 < (1 << MAXPHYADDR.load(Ordering::Relaxed)));
        raw |= entry.physical_address.0 as u64
            & ((1 << MAXPHYADDR.load(Ordering::Relaxed)) - 1)
            & !0xFFF;
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
            writable: raw.0 & (1 << 1) != 0,
            user_accessible: raw.0 & (1 << 2) != 0,
            accessed: raw.0 & (1 << 5) != 0,
            physical_address: PhysicalAddress(
                raw.0 as usize & ((1 << MAXPHYADDR.load(Ordering::Relaxed)) - 1) & !0xFFF,
            ),
            disabled_execute: raw.0 & (1 << 63) != 0,
            free_territory_baby: (((raw.0 >> 6) & 0x1)
                | (((raw.0 >> 8) & 0xF) << 1)
                | (((raw.0 >> 52) & 0x7FF) << 5)) as u16,
        }
    }
}

/// Level 1 entries + Level 2 and 3 huge table entries
#[expect(clippy::struct_excessive_bools)]
pub struct PageEntry<S: PageSize> {
    writable: bool,        // or read-only
    user_accessible: bool, // or supervisor-only
    /// Written by the CPU to say this page has been accessed
    accessed: bool,
    /// Written by the CPU to say this page has been written to
    dirty: bool,
    /// Tells the CPU to keep this page's TLB entry when CR3 is reloaded.
    /// CR4:0b7 (PGE) must be set for global pages to work
    global: bool,
    physical_address: PhysicalAddress,
    disabled_execute: bool,
    pat: Pat,
    // TODO: Add warning when the top two bits are set when they're not used
    /// Available to me to use whatever i want :shrug:
    /// bits 9-11 && bits 52-62 (well there's Protection Key at 59-62 but unused for now)
    free_territory_baby: u16,
    _size: PhantomData<S>,
}

impl<S: PageSize> PageEntry<S> {
    const fn pointing_to(
        frame: PhysicalAddress,
        param: PageEntryParameters
    ) -> Self {
        Self {
            writable: param.writable,
            user_accessible: param.user_accessible,
            accessed: false,
            dirty: false,
            pat: param.pat,
            global: false,
            physical_address: frame,
            disabled_execute: !param.executable,
            free_territory_baby: 0,
            _size: PhantomData,
        }
    }
}

impl From<PageEntry<Size4K>> for RawEntry {
    fn from(entry: PageEntry<Size4K>) -> Self {
        let mut raw = 0u64;
        let pat_bits = entry.pat.to_msr_arr_index(MSR_PAT_ARRAY);
        // present bit is always set for valid entries
        raw |= 1 << 0;
        if entry.writable {
            raw |= 1 << 1;
        }
        if entry.user_accessible {
            raw |= 1 << 2;
        }
        raw |= (pat_bits & 0x1) << 3;
        raw |= ((pat_bits >> 1) & 0x1) << 4;
        if entry.accessed {
            raw |= 1 << 5;
        }
        if entry.dirty {
            raw |= 1 << 6;
        }
        raw |= ((pat_bits >> 2) & 0x1) << 7;
        if entry.global {
            raw |= 1 << 8;
        }
        // bits 12 .. (MAXPHYADDR -1)
        assert!(entry.physical_address.0 < (1 << MAXPHYADDR.load(Ordering::Relaxed)));
        raw |= entry.physical_address.0 as u64
            & ((1 << MAXPHYADDR.load(Ordering::Relaxed)) - 1)
            & !((1 << Size4K::SHIFT) - 1);
        if entry.disabled_execute {
            raw |= 1 << 63;
        }
        let free = u64::from(entry.free_territory_baby);
        raw |= (free & 0x7) << 9;
        raw |= ((free >> 3) & 0x7FF) << 52;
        Self(raw)
    }
}

impl From<RawEntry> for PageEntry<Size4K> {
    fn from(raw: RawEntry) -> Self {
        let pat_bits = ((raw.0 >> 3) & 0x1) | (((raw.0 >> 4) & 0x1) << 1) | (((raw.0 >> 7) & 0x1) << 2);
        Self {
            writable: raw.0 & (1 << 1) != 0,
            user_accessible: raw.0 & (1 << 2) != 0,
            accessed: raw.0 & (1 << 5) != 0,
            dirty: raw.0 & (1 << 6) != 0,
            pat: MSR_PAT_ARRAY[pat_bits as usize],
            global: raw.0 & (1 << 8) != 0,
            physical_address: PhysicalAddress(
                raw.0 as usize
                    & ((1 << MAXPHYADDR.load(Ordering::Relaxed)) - 1)
                    & !((1 << Size4K::SHIFT) - 1),
            ),
            disabled_execute: raw.0 & (1 << 63) != 0,
            free_territory_baby: (((raw.0 >> 9) & 0x7) | (((raw.0 >> 52) & 0x7FF) << 3)) as u16,
            _size: PhantomData,
        }
    }
}

impl<S: HugeSize> From<PageEntry<S>> for RawEntry {
    fn from(entry: PageEntry<S>) -> Self {
        assert_eq!(entry.physical_address.0 % (1 << S::SHIFT), 0);
        let mut raw = 0u64;
        let pat_bits = entry.pat.to_msr_arr_index(MSR_PAT_ARRAY);
        // present bit is always set for valid entries
        raw |= 1 << 0;
        if entry.writable {
            raw |= 1 << 1;
        }
        if entry.user_accessible {
            raw |= 1 << 2;
        }
        raw |= (pat_bits & 0x1) << 3;
        raw |= ((pat_bits >> 1) & 0x1) << 4;
        if entry.accessed {
            raw |= 1 << 5;
        }
        if entry.dirty {
            raw |= 1 << 6;
        }
        // Set the huge bit
        raw |= 1 << 7;
        if entry.global {
            raw |= 1 << 8;
        }
        raw |= ((pat_bits >> 2) & 0x1) << 12;
        // bits S::SHIFT .. (MAXPHYADDR -1)
        assert!(entry.physical_address.0 < (1 << MAXPHYADDR.load(Ordering::Relaxed)));
        raw |= entry.physical_address.0 as u64
            & ((1 << MAXPHYADDR.load(Ordering::Relaxed)) - 1)
            & !((1 << S::SHIFT) - 1);
        if entry.disabled_execute {
            raw |= 1 << 63;
        }
        let free = u64::from(entry.free_territory_baby);
        raw |= (free & 0x7) << 9;
        raw |= ((free >> 3) & 0x7FF) << 52;
        Self(raw)
    }
}

impl<S: HugeSize> From<RawEntry> for PageEntry<S> {
    fn from(raw: RawEntry) -> Self {
        assert!(raw.0 & (1 << 7) != 0);
        let pat_bits = ((raw.0 >> 3) & 0x1) | (((raw.0 >> 4) & 0x1) << 1) | (((raw.0 >> 12) & 0x1) << 2);
        Self {
            writable: raw.0 & (1 << 1) != 0,
            user_accessible: raw.0 & (1 << 2) != 0,
            accessed: raw.0 & (1 << 5) != 0,
            dirty: raw.0 & (1 << 6) != 0,
            pat: MSR_PAT_ARRAY[pat_bits as usize],
            global: raw.0 & (1 << 8) != 0,
            physical_address: PhysicalAddress(
                raw.0 as usize
                    & ((1 << MAXPHYADDR.load(Ordering::Relaxed)) - 1)
                    & !((1 << S::SHIFT) - 1),
            ),
            disabled_execute: raw.0 & (1 << 63) != 0,
            free_territory_baby: (((raw.0 >> 9) & 0x7) | (((raw.0 >> 52) & 0x7FF) << 3)) as u16,
            _size: PhantomData,
        }
    }
}

pub enum EntryType<S: HugeSize> {
    Table(TableEntry),
    Page(PageEntry<S>),
}

impl<S: HugeSize> From<EntryType<S>> for RawEntry {
    fn from(entry: EntryType<S>) -> Self {
        match entry {
            EntryType::Table(table_entry) => table_entry.into(),
            EntryType::Page(page_entry) => page_entry.into(),
        }
    }
}

impl<S: HugeSize> From<RawEntry> for EntryType<S> {
    fn from(raw: RawEntry) -> Self {
        // If bit 7 (the huge bit) is set, it's a page entry, otherwise it's a table entry
        if raw.0 & (1 << 7) != 0 {
            Self::Page(raw.into())
        } else {
            Self::Table(raw.into())
        }
    }
}

pub trait CanHaveNext {
    fn as_table_entry(&self) -> Option<&TableEntry>;
    fn pointing_to(table: PhysicalAddress) -> Self;
}
impl CanHaveNext for TableEntry {
    fn as_table_entry(&self) -> Option<&TableEntry> {
        Some(self)
    }

    fn pointing_to(table: PhysicalAddress) -> Self {
        Self::pointing_to(table)
    }
}
impl<S: HugeSize> CanHaveNext for EntryType<S> {
    fn as_table_entry(&self) -> Option<&TableEntry> {
        match self {
            Self::Table(table_entry) => Some(table_entry),
            Self::Page(_) => None,
        }
    }

    fn pointing_to(table: PhysicalAddress) -> Self {
        Self::Table(TableEntry::pointing_to(table))
    }
}

#[repr(C, align(4096))]
pub struct PageTable<L: Level> {
    entries: [RawEntry; 512],
    _level: PhantomData<L>,
}

impl<L: Level> PageTable<L> {
    fn add_entry(&mut self, index: usize, entry: L::Entry) -> Result<(), PagingError> {
        assert!(index < 512);
        let raw_entry = entry.into();
        if self.get_entry(index).is_some() {
            return Err(PagingError::EntryAlreadyPresent {
                level: L::NUMBER,
                index,
            });
        }
        self.entries[index] = raw_entry;
        Ok(())
    }

    fn get_entry(&self, index: usize) -> Option<L::Entry> {
        assert!(index < 512);
        let raw_entry = self.entries[index];
        if raw_entry.0 & 1 == 0 {
            return None;
        }
        Some(raw_entry.into())
    }
}

impl<L: HasNext> PageTable<L> {
    #[expect(clippy::needless_pass_by_ref_mut)]
    fn next_table(&mut self, index: usize) -> Result<&mut PageTable<L::Next>, PagingError> {
        assert!(index < 512);
        let entry = self.get_entry(index).ok_or(PagingError::EntryNotPresent {
            level: L::NUMBER,
            index,
        })?;
        let table_entry = entry.as_table_entry().ok_or(PagingError::EntryNotTable {
            level: L::NUMBER,
            index,
        })?;
        let virt = VirtualAddress::phys_to_virt(table_entry.physical_address);
        let next_table_ptr = virt.as_mut_ptr::<PageTable<L::Next>>();
        unsafe { Ok(&mut *next_table_ptr) }
    }

    fn next_table_or_create(
        &mut self,
        index: usize,
    ) -> Result<&mut PageTable<L::Next>, PagingError> {
        assert!(index < 512);
        if self.get_entry(index).is_none() {
            let frame = FRAME_ALLOCATOR
                .lock()
                .alloc()
                .ok_or(PagingError::NoMoreMemory)?;
            frame.zero();
            self.add_entry(index, L::Entry::pointing_to(frame.address()))?;
        }
        self.next_table(index)
    }
}

const _: () = assert!(size_of::<PageTable<L4>>() == 4096);

pub struct L4;
pub struct L3;
pub struct L2;
pub struct L1;

pub trait Level {
    const SHIFT: usize;
    const NUMBER: usize;
    type Entry: Into<RawEntry> + From<RawEntry>;
}

impl Level for L4 {
    const SHIFT: usize = 39;
    const NUMBER: usize = 4;
    type Entry = TableEntry;
}
impl Level for L3 {
    const SHIFT: usize = 30;
    const NUMBER: usize = 3;
    type Entry = EntryType<Size1G>;
}
impl Level for L2 {
    const SHIFT: usize = 21;
    const NUMBER: usize = 2;
    type Entry = EntryType<Size2M>;
}
impl Level for L1 {
    const SHIFT: usize = 12;
    const NUMBER: usize = 1;
    type Entry = PageEntry<Size4K>;
}

pub trait HasNext: Level<Entry: CanHaveNext> {
    type Next: Level;
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

/// L1 Entry
pub struct Size4K;
/// L2 Huge Entry
pub struct Size2M;
/// L3 Huge Entry
pub struct Size1G;
pub trait PageSize {
    const SHIFT: usize;
}
impl PageSize for Size4K {
    const SHIFT: usize = 12;
}
impl PageSize for Size2M {
    const SHIFT: usize = 21;
}
impl PageSize for Size1G {
    const SHIFT: usize = 30;
}
pub trait HugeSize: PageSize {}
impl HugeSize for Size2M {}
impl HugeSize for Size1G {}

// TODO: sort that code

#[derive(Debug, Clone, Copy)]
pub struct PageEntryParameters {
    pub writable: bool,
    pub user_accessible: bool,
    pub executable: bool,
    pub pat: Pat,
}

unsafe fn map(
    l4_table: &mut PageTable<L4>,
    virt: VirtualAddress,
    phys: PhysicalAddress,
    param: PageEntryParameters,
) -> Result<(), PagingError> {
    assert_eq!(virt.0 % (1 << Size4K::SHIFT), 0);
    let l3_table = l4_table.next_table_or_create((virt.0 >> L4::SHIFT) & 0x1FF)?;
    let l2_table = l3_table.next_table_or_create((virt.0 >> L3::SHIFT) & 0x1FF)?;
    let l1_table = l2_table.next_table_or_create((virt.0 >> L2::SHIFT) & 0x1FF)?;
    let l1_entry = PageEntry::pointing_to(phys, param);
    l1_table.add_entry((virt.0 >> L1::SHIFT) & 0x1FF, l1_entry)?;

    Ok(())
}

unsafe fn map_huge_2m(
    l4_table: &mut PageTable<L4>,
    virt: VirtualAddress,
    phys: PhysicalAddress,
    param: PageEntryParameters,
) -> Result<(), PagingError> {
    assert_eq!(virt.0 % (1 << Size2M::SHIFT), 0);
    let l3_table = l4_table.next_table_or_create((virt.0 >> L4::SHIFT) & 0x1FF)?;
    let l2_table = l3_table.next_table_or_create((virt.0 >> L3::SHIFT) & 0x1FF)?;
    let l2_entry = PageEntry::pointing_to(phys, param);
    l2_table.add_entry((virt.0 >> L2::SHIFT) & 0x1FF, EntryType::Page(l2_entry))?;
    Ok(())
}

pub unsafe fn map_range<S: PageSize>(
    l4_table: &mut PageTable<L4>,
    virt_start: VirtualAddress,
    phys_start: PhysicalAddress,
    len: usize,
    param: PageEntryParameters
) -> Result<(), PagingError> {
    assert_eq!(virt_start.0 % (1 << S::SHIFT), 0);
    assert_eq!(phys_start.0 % (1 << S::SHIFT), 0);
    assert_eq!(len % (1 << S::SHIFT), 0);

    let num_pages = len / (1 << S::SHIFT);
    for i in 0..num_pages {
        let virt = VirtualAddress(virt_start.0 + i * (1 << S::SHIFT));
        let phys = PhysicalAddress(phys_start.0 + i * (1 << S::SHIFT));
        match S::SHIFT {
            Size4K::SHIFT => {
                unsafe { map(l4_table, virt, phys, param) }?;
            }
            Size2M::SHIFT => {
                unsafe {
                    map_huge_2m(l4_table, virt, phys, param)
                }?;
            }
            _ => unimplemented!("Mapping for page size {} is not implemented", S::SHIFT),
        }
    }

    Ok(())
}

/// To use only for MMIO mapping
pub fn map_mmio(
    l4_table: &mut PageTable<L4>,
    phys_start: PhysicalAddress,
    len: usize,
    param: PageEntryParameters
) -> VirtualAddress {
    assert_eq!(phys_start.0 % (1 << Size4K::SHIFT), 0);
    assert_eq!(len % (1 << Size4K::SHIFT), 0);

    let virt_start = VirtualAddress::new(phys_start.0 + memory::VIRTUAL_ADDRESS_OFFSET);
    unsafe {
        map_range::<Size4K>(
            l4_table,
            virt_start,
            phys_start,
            len,
            param,
        )
        .unwrap();
    }

    virt_start
}

#[expect(dead_code)]
fn vm_map(
    l4_table: &mut PageTable<L4>,
    virt: VirtualAddress,
    param: PageEntryParameters,
) -> Result<(), PagingError> {
    let frame = FRAME_ALLOCATOR
        .lock()
        .alloc()
        .ok_or(PagingError::NoMoreMemory)?;
    frame.zero();
    unsafe {
        map(
            l4_table,
            virt,
            frame.address(),
            param,
        )
    }
}

// TODO: Unnap (TLB flush)

#[expect(dead_code)]
#[derive(Debug)]
pub enum PagingError {
    NoMoreMemory,
    EntryAlreadyPresent { level: usize, index: usize },
    EntryNotPresent { level: usize, index: usize },
    EntryNotTable { level: usize, index: usize },
}

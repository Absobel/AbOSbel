use core::{
    fmt::LowerHex,
    ops::Add,
    sync::atomic::{AtomicUsize, Ordering},
};

pub mod frame;
pub mod paging;

/// At boot, the first 512 GiB of physical memory are mapped to the higher half of the virtual space
/// After mapping, it's the whole physical memory
const VIRTUAL_ADDRESS_OFFSET: usize = 0xffff_8000_0000_0000;
static END_OF_PAGING: AtomicUsize = AtomicUsize::new(1 << 39); // 512 GiB mapped at boot

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalAddress(pub usize);

impl LowerHex for PhysicalAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        LowerHex::fmt(&self.0, f)
    }
}

impl Add<usize> for PhysicalAddress {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtualAddress(usize);

impl VirtualAddress {
    /// # Panics
    /// The address should be a valid virtual address: bits 48-63 should be the same as bit 47
    pub const fn new(addr: usize) -> Self {
        let extended_bits = addr >> 47;
        assert!(extended_bits == 0 || extended_bits == 0x1FFFF);
        Self(addr)
    }

    pub fn phys_to_virt(phys: PhysicalAddress) -> Self {
        assert!(phys.0 < END_OF_PAGING.load(Ordering::Relaxed));
        Self(phys.0 + VIRTUAL_ADDRESS_OFFSET)
    }

    pub fn phys_window_to_phys(self) -> PhysicalAddress {
        assert!(
            (VIRTUAL_ADDRESS_OFFSET
                ..VIRTUAL_ADDRESS_OFFSET + END_OF_PAGING.load(Ordering::Relaxed))
                .contains(&self.0)
        );
        PhysicalAddress(self.0 - VIRTUAL_ADDRESS_OFFSET)
    }

    /// # Safety
    /// The caller must ensure that the virtual address is valid
    pub const unsafe fn identity_map(phys: PhysicalAddress) -> Self {
        Self(phys.0)
    }

    pub const fn as_ptr<T>(&self) -> *const T {
        self.0 as *const T
    }

    pub const fn as_mut_ptr<T>(&self) -> *mut T {
        self.0 as *mut T
    }
}

unsafe extern "C" {
    static kernel_start: u8;
}
unsafe extern "C" {
    static kernel_end: u8;
}
#[must_use]
pub fn kernel_start_addr() -> PhysicalAddress {
    let addr = &raw const kernel_start as usize;
    assert_eq!(addr % 4096, 0, "kernel_start is not page aligned");
    PhysicalAddress(addr)
}
#[must_use]
pub fn kernel_end_addr() -> PhysicalAddress {
    let addr = &raw const kernel_end as usize;
    assert_eq!(addr % 4096, 0, "kernel_end is not page aligned");
    PhysicalAddress(addr)
}

#[must_use]
pub const fn align_up(addr: usize, align: usize) -> usize {
    usize::next_multiple_of(addr, align)
}

#[must_use]
pub const fn align_down_pow2(addr: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two());
    addr & !(align - 1)
}

// TODO: heap allocator, for later

// use core::ptr::null_mut;

// use alloc::alloc::{GlobalAlloc, Layout};

// pub struct Dummy;

// unsafe impl GlobalAlloc for Dummy {
//     unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
//         null_mut()
//     }

//     unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
//         panic!("dealloc should be never called")
//     }
// }

// #[global_allocator]
// static ALLOCATOR: Dummy = Dummy;

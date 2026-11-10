use core::{fmt::LowerHex, ops::Add};

pub mod frame;

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

unsafe extern "C" {
    static kernel_end: u8;
}
#[must_use]
pub fn kernel_end_addr() -> PhysicalAddress {
    PhysicalAddress(&raw const kernel_end as usize)
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

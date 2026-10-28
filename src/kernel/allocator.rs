use core::ops::Range;

use multiboot2::{MemoryArea, MemoryAreaType};
use spin::{Mutex, Once};

use crate::{MULTIBOOT2_INFO, align_down_pow2, align_up, kernel_end_addr};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalAddress(pub usize);

// TODO: find a way to do mutexes without_interrupt
pub static FRAME_ALLOCATOR: Once<Mutex<FrameAllocator>> = Once::new();

pub fn init() {
    FRAME_ALLOCATOR.call_once(|| Mutex::new(FrameAllocator::new()));
}

// Naive bump allocator for now
pub struct FrameAllocator {
    memory_map: &'static [MemoryArea],
    mbi_addr: Range<PhysicalAddress>,
    current_area_idx: usize,
    current: PhysicalAddress,
}

impl FrameAllocator {
    const FRAME_SIZE: usize = 4096;

    fn new() -> Self {
        let boot_info = MULTIBOOT2_INFO.get().expect("Multiboot info required");
        let memory_map = boot_info
            .memory_map_tag()
            .expect("Memory map tag required")
            .memory_areas();
        let mbi_addr = Range {
            start: PhysicalAddress(align_down_pow2(boot_info.start_address(), Self::FRAME_SIZE)),
            end: PhysicalAddress(align_up(boot_info.end_address(), Self::FRAME_SIZE)),
        };

        Self {
            memory_map,
            mbi_addr,
            current_area_idx: 0,
            current: kernel_end_addr(),
        }
    }
}

impl Iterator for FrameAllocator {
    type Item = PhysicalAddress;

    // TODO: assume unsorted memory map for next time
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let area = self.memory_map.get(self.current_area_idx)?;
            let start = PhysicalAddress(align_up(area.start_address() as usize, Self::FRAME_SIZE));
            if self.current < start {
                self.current = start;
            }
            let current = self.current;

            if area.typ() != MemoryAreaType::Available
                || current.0 + Self::FRAME_SIZE > area.end_address() as usize
            {
                self.current_area_idx += 1;
                continue;
            }
            if self.mbi_addr.contains(&current) {
                self.current = self.mbi_addr.end;
                continue;
            }

            self.current = PhysicalAddress(current.0 + Self::FRAME_SIZE);
            return Some(current);
        }
    }
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

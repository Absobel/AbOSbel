use core::ops::Range;

use multiboot2::{MemoryArea, MemoryAreaType};
use spin::{Mutex, Once};

use crate::{
    boot,
    memory::{PhysicalAddress, kernel_end_addr, align_down_pow2, align_up},
};

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
        let boot_info = boot::MULTIBOOT2_INFO
            .get()
            .expect("Multiboot info required");
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
                || current + Self::FRAME_SIZE > PhysicalAddress(area.end_address() as usize)
            {
                self.current_area_idx += 1;
                continue;
            }
            if self.mbi_addr.contains(&current) {
                self.current = self.mbi_addr.end;
                continue;
            }

            self.current = current + Self::FRAME_SIZE;
            return Some(current);
        }
    }
}

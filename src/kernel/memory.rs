use core::slice::Iter;

use multiboot2::MemoryArea;

const PAGE_SIZE: usize = 0x1000;

struct Frame {
    number: usize,
}

impl Frame {
    fn from_address(address: usize) -> Self {
        Frame { number: address / PAGE_SIZE }
    }
}

trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame>;
    fn deallocate_frame(&mut self, frame: Frame);
}

struct AreaFrameAllocator {
    next_free_frame: Frame,
    current_area: Option<&'static MemoryArea>,
    areas: Iter<'static, MemoryArea>,
    kernel_start: Frame,
    kernel_end: Frame,
    multiboot_start: Frame,
    multiboot_end: Frame,
}


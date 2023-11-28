use core::slice::Iter;

use multiboot2::MemoryArea;
use spin::Mutex;

const PAGE_SIZE: usize = 0x1000;

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct Frame {
    number: usize,
}

impl Frame {
    fn from_address(address: usize) -> Self {
        Frame {
            number: address / PAGE_SIZE,
        }
    }
}

pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame>;
    fn deallocate_frame(&mut self, frame: Frame);
}

#[derive(Debug)]
pub struct AreaFrameAllocator {
    next_free_frame: Frame,
    current_area: Option<&'static MemoryArea>,
    areas: Iter<'static, MemoryArea>,
    kernel_start: Frame,
    kernel_end: Frame,
    multiboot_start: Frame,
    multiboot_end: Frame,
}

impl FrameAllocator for AreaFrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame> {
        if let Some(area) = self.current_area {
            let frame = Frame {
                number: self.next_free_frame.number,
            };

            let current_area_last_frame = {
                let address = area.end_address() - 1;
                Frame::from_address(address as usize)
            };

            if frame > current_area_last_frame {
                self.choose_next_area();
            } else if frame >= self.kernel_start && frame <= self.kernel_end {
                self.next_free_frame = Frame {
                    number: self.kernel_end.number + 1,
                };
            } else if frame >= self.multiboot_start && frame <= self.multiboot_end {
                self.next_free_frame = Frame {
                    number: self.multiboot_end.number + 1,
                };
            } else {
                self.next_free_frame.number += 1;
                return Some(frame);
            }
            self.allocate_frame()
        } else {
            None // No free frames left
        }
    }

    fn deallocate_frame(&mut self, frame: Frame) {
        todo!()
    }
}

impl AreaFrameAllocator {
    pub fn new(
        kernel_start: usize,
        kernel_end: usize,
        multiboot_start: usize,
        multiboot_end: usize,
        memory_areas: Iter<'static, MemoryArea>,
    ) -> AreaFrameAllocator {
        let mut allocator = AreaFrameAllocator {
            next_free_frame: Frame::from_address(0),
            current_area: None,
            areas: memory_areas,
            kernel_start: Frame::from_address(kernel_start),
            kernel_end: Frame::from_address(kernel_end),
            multiboot_start: Frame::from_address(multiboot_start),
            multiboot_end: Frame::from_address(multiboot_end),
        };
        allocator.choose_next_area();
        allocator
    }

    fn choose_next_area(&mut self) {
        self.current_area = self
            .areas
            .clone()
            .filter(|area| {
                let address = area.end_address() - 1;
                Frame::from_address(address as usize) >= self.next_free_frame
            })
            .min_by_key(|area| area.start_address());

        if let Some(area) = self.current_area {
            let start_frame = Frame::from_address(area.start_address() as usize);
            if self.next_free_frame < start_frame {
                self.next_free_frame = start_frame;
            }
        }
    }
}

////////////// UTILS //////////////

crate::sync_wrapper!(
    FRAME_ALLOCATOR,
    OnceFrameAllocator,
    Mutex<AreaFrameAllocator>
);

pub fn init_frame_alloc() {
    let boot_info = crate::MULTIBOOT2_INFO
        .get()
        .expect("Multiboot info required");

    let memory_map_tag = boot_info.memory_map_tag().expect("Memory map required");
    let elf_sections = boot_info.elf_sections().expect("Elf sections required");

    let (kernel_start, kernel_end) = elf_sections.fold((usize::MAX, 0), |(start, end), section| {
        let new_start = start.min(section.start_address() as usize);
        let new_end = end.max(section.end_address() as usize);
        (new_start, new_end)
    });

    let multiboot_start = boot_info.start_address();
    let multiboot_end = boot_info.end_address();

    let frame_allocator = AreaFrameAllocator::new(
        kernel_start,
        kernel_end,
        multiboot_start,
        multiboot_end,
        memory_map_tag.memory_areas().iter(),
    );

    FRAME_ALLOCATOR
        .set(Mutex::new(frame_allocator))
        .expect("Shouldn't be initialised");
}

use crate::MULTIBOOT2_INFO;

pub const PAGE_SIZE: usize = 4096;

pub fn total_mem() -> usize {
    let boot_info = MULTIBOOT2_INFO.get().expect("Multiboot Info Required");

    boot_info
        .memory_map_tag()
        .expect("MemoryMapTag required")
        .memory_areas()
        .iter()
        .filter(|area| area.typ() == multiboot2::MemoryAreaType::Available)
        .map(|area| area.size() as usize)
        .sum()
}

pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame>;
    fn deallocate_frame(&mut self, frame: Frame);
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    pub number: usize,
}

impl Frame {
    pub fn containing_address(address: usize) -> Frame {
        Frame {
            number: address / PAGE_SIZE,
        }
    }
}

// TODO: redo this function
pub fn frame_allocator() -> impl FrameAllocator {
    let boot_info = MULTIBOOT2_INFO.get().expect("Multiboot info required");

    let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");

    let elf_sections = boot_info.elf_sections().expect("Elf sections required");

    let kernel_start = elf_sections
        .clone()
        .min_by_key(|s| s.start_address())
        .expect("At least one elf section required")
        .start_address();
    let kernel_end = elf_sections
        .map(|s| s.end_address())
        .max()
        .expect("At least one elf section required");

    let multiboot_start = boot_info.start_address();
    let multiboot_end = multiboot_start + boot_info.total_size();

    super::AreaFrameAllocator::new(
        kernel_start as usize,
        kernel_end as usize,
        multiboot_start,
        multiboot_end,
        memory_map_tag.memory_areas(),
    )
}

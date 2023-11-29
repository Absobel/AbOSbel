use spin::Mutex;

use super::AreaFrameAllocator;

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

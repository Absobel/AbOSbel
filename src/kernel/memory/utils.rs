use multiboot2::{BootInformation, BootInformationHeader};

pub const PAGE_SIZE: usize = 4096;

pub static mut MULTIBOOT2_INFO: Option<BootInformation> = None;

#[allow(clippy::missing_safety_doc)]
pub unsafe fn load_multiboot(multiboot_info_addr: usize) {
    if MULTIBOOT2_INFO.is_none() {
        MULTIBOOT2_INFO =
            BootInformation::load(multiboot_info_addr as *const BootInformationHeader).ok();
    }
}

pub fn total_mem() -> Result<usize, &'static str> {
    let maybe_boot_info = unsafe { MULTIBOOT2_INFO.as_ref() };

    match maybe_boot_info {
        Some(boot_info) => Ok(boot_info
            .memory_map_tag()
            .unwrap()
            .memory_areas()
            .iter()
            .filter(|area| area.typ() == multiboot2::MemoryAreaType::Available)
            .map(|area| area.size() as usize)
            .sum()),
        None => Err("Boot information not loaded"),
    }
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

pub fn frame_allocator() -> Result<impl FrameAllocator, &'static str> {
    let boot_info = unsafe { MULTIBOOT2_INFO.as_ref() }.ok_or("Boot information not loaded")?;

    let memory_map_tag = boot_info.memory_map_tag().unwrap();

    let elf_sections = boot_info.elf_sections().unwrap();

    let kernel_start = elf_sections
        .clone()
        .min_by_key(|s| s.start_address())
        .unwrap()
        .start_address();
    let kernel_end = elf_sections.map(|s| s.end_address()).max().unwrap();

    let multiboot_start = boot_info.start_address();
    let multiboot_end = multiboot_start + boot_info.total_size();

    Ok(super::AreaFrameAllocator::new(
        kernel_start as usize,
        kernel_end as usize,
        multiboot_start,
        multiboot_end,
        memory_map_tag.memory_areas(),
    ))
}

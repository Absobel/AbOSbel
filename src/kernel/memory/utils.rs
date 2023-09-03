use multiboot2::{BootInformation, BootInformationHeader, MbiLoadError};

pub const PAGE_SIZE: usize = 4096;

pub static mut MULTIBOOT2_INFO: Option<BootInformation> = None;

#[allow(clippy::missing_safety_doc)]
pub unsafe fn load_multiboot(multiboot_info_addr: usize) -> Result<(), MbiLoadError> {
    if MULTIBOOT2_INFO.is_none() {
        MULTIBOOT2_INFO = Some(BootInformation::load(
            multiboot_info_addr as *const BootInformationHeader,
        )?);
    }
    Ok(())
}

pub fn total_mem() -> usize {
    let boot_info = unsafe { MULTIBOOT2_INFO.as_ref().expect("Multiboot info required") };

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
    let boot_info = unsafe { MULTIBOOT2_INFO.as_ref().expect("Multiboot info required") };

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

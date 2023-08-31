use multiboot2::{BootInformation, BootInformationHeader};

pub const PAGE_SIZE: usize = 4096;

static mut MULTIBOOT2_INFO: Option<BootInformation> = None;

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

pub fn frame_allocator(multiboot_info_addr: usize) -> impl FrameAllocator {
    unsafe {
        MULTIBOOT2_INFO = BootInformation::load(multiboot_info_addr as *const BootInformationHeader).ok();
    };

    let boot_info = unsafe { MULTIBOOT2_INFO.as_ref().unwrap() };

    let memory_map_tag = boot_info.memory_map_tag().unwrap();

    let elf_sections = boot_info.elf_sections().unwrap();

    let kernel_start = elf_sections
        .clone()
        .min_by_key(|s| s.start_address())
        .unwrap()
        .start_address();
    let kernel_end = elf_sections.map(|s| s.end_address()).max().unwrap();

    let multiboot_start = multiboot_info_addr;
    let multiboot_end = multiboot_start + boot_info.total_size();

    super::AreaFrameAllocator::new(
        kernel_start as usize,
        kernel_end as usize,
        multiboot_start,
        multiboot_end,
        memory_map_tag.memory_areas(),
    )
}

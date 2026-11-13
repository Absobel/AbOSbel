use multiboot2::MemoryAreaType;
use spin::Mutex;

use crate::{
    boot,
    memory::{PhysicalAddress, VirtualAddress, align_down_pow2, align_up, kernel_end_addr},
};

// TODO: find a way to do mutexes without_interrupt
// Also check that this lock is taken as few times as possible, as high as possible (which is not the case right now)
pub static FRAME_ALLOCATOR: Mutex<FrameAllocator> = Mutex::new(FrameAllocator::empty());

pub fn init() {
    FRAME_ALLOCATOR.lock().init();
}

pub struct AllocatedFrame(PhysicalAddress);

impl AllocatedFrame {
    pub unsafe fn new(frame: PhysicalAddress) -> Self {
        assert_eq!(frame.0 % FrameAllocator::FRAME_SIZE, 0);
        Self(frame)
    }

    pub const fn address(&self) -> PhysicalAddress {
        self.0
    }

    pub fn zero(&self) {
        let virt = VirtualAddress::phys_to_virt(self.0);
        let ptr = virt.as_mut_ptr::<Frame>();
        unsafe {
            (*ptr).0.fill(0);
        }
    }
}

#[repr(transparent)]
struct Frame ([u8; FrameAllocator::FRAME_SIZE]);

// Random number rn ngl
const TOTAL_FRAMES: usize = 1024 * 1024; // 4 GiB of memory
pub struct FrameAllocator {
    /// if frame n is allocated, then bit n is set to 0, otherwise 1
    bitmap: [u8; TOTAL_FRAMES / 8],
    latest_allocated_frame: usize,
}

impl FrameAllocator {
    const FRAME_SIZE: usize = 4096;

    #[expect(clippy::large_stack_arrays)]
    const fn empty() -> Self {
        Self {
            bitmap: [0; TOTAL_FRAMES / 8],
            latest_allocated_frame: 0,
        }
    }

    // TODO: assume unsorted memory map for next time
    fn init(&mut self) {
        let boot_info = boot::MULTIBOOT2_INFO
            .get()
            .expect("Multiboot info required");
        let memory_map = boot_info
            .memory_map_tag()
            .expect("Memory map tag required")
            .memory_areas();
        let (boot_info_start, boot_info_end) = (
            VirtualAddress::new(boot_info.start_address())
                .phys_window_to_phys()
                .0,
            VirtualAddress::new(boot_info.end_address())
                .phys_window_to_phys()
                .0,
        );
        let (mbi_addr_start, mbi_addr_end) = (
            align_down_pow2(boot_info_start, Self::FRAME_SIZE) / Self::FRAME_SIZE,
            align_up(boot_info_end, Self::FRAME_SIZE) / Self::FRAME_SIZE,
        );
        let aligned_kernel_end = align_up(kernel_end_addr().0, Self::FRAME_SIZE) / Self::FRAME_SIZE;
        for area in memory_map {
            if area.typ() == MemoryAreaType::Available {
                let start_frame = align_up(area.start_address() as usize, Self::FRAME_SIZE * 8)
                    / Self::FRAME_SIZE;
                let end_frame = align_down_pow2(area.end_address() as usize, Self::FRAME_SIZE)
                    / Self::FRAME_SIZE;
                if end_frame > TOTAL_FRAMES || start_frame >= end_frame {
                    continue;
                }
                self.bitmap[start_frame / 8..end_frame / 8]
                    .iter_mut()
                    .for_each(|b| *b = 0xFF);
            }
        }
        self.bitmap[mbi_addr_start / 8..=mbi_addr_end / 8]
            .iter_mut()
            .for_each(|b| *b = 0);
        self.bitmap[0..=aligned_kernel_end / 8]
            .iter_mut()
            .for_each(|b| *b = 0);
    }

    #[expect(clippy::needless_pass_by_value)]
    pub const fn free(&mut self, frame: AllocatedFrame) {
        let frame_index = frame.0.0 / Self::FRAME_SIZE;
        let byte_index = frame_index / 8;
        let bit_index = frame_index % 8;
        self.bitmap[byte_index] |= 1 << bit_index;
    }

    pub fn alloc(&mut self) -> Option<AllocatedFrame> {
        for frame in
            ((self.latest_allocated_frame + 1)..TOTAL_FRAMES).chain(0..=self.latest_allocated_frame)
        {
            let byte_index = frame / 8;
            let bit_index = frame % 8;
            if (self.bitmap[byte_index] & (1 << bit_index)) != 0 {
                self.bitmap[byte_index] &= !(1 << bit_index);
                self.latest_allocated_frame = frame;
                return Some(AllocatedFrame(PhysicalAddress(frame * Self::FRAME_SIZE)));
            }
        }
        None
    }
}

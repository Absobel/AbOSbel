use alloc::alloc::{GlobalAlloc, Layout};
use spin::Mutex;

use crate::memory::{
    VirtualAddress, align_up,
    paging::{self, L4_TABLE},
};

const HEAP_START: VirtualAddress = VirtualAddress(0xffff_c000_0000_0000); // index 384 of the l4 table, and above OFFSET + 2^MAXADDRPHYS (up to 46 bits)
const HEAP_SIZE: usize = 1024 * 1024; // 1 MiB

#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator::new();

/// # Panics
/// No idea ngl, i'll wait till it bites me in the ass idc
pub fn init() {
    let mut l4_table = L4_TABLE.get().expect("L4_TABLE required").lock();
    paging::vm_map_range(
        &mut l4_table,
        HEAP_START,
        HEAP_SIZE,
        paging::PageEntryParameters {
            writable: true,
            user_accessible: false,
            executable: false,
            pat: paging::Pat::WriteBack,
        },
    )
    .unwrap();

    unsafe {
        ALLOCATOR.init(HEAP_START, HEAP_SIZE);
    }
}

struct AllocParam {
    heap_start: VirtualAddress,
    heap_end: VirtualAddress,
    next: VirtualAddress,
    allocations: usize,
}

pub struct BumpAllocator {
    param: Mutex<AllocParam>,
}

impl BumpAllocator {
    pub const fn new() -> Self {
        Self {
            param: Mutex::new(AllocParam {
                heap_start: VirtualAddress(0),
                heap_end: VirtualAddress(0),
                next: VirtualAddress(0),
                allocations: 0,
            }),
        }
    }

    pub unsafe fn init(&self, heap_start: VirtualAddress, heap_size: usize) {
        let param = &mut *self.param.lock();
        param.heap_start = heap_start;
        param.heap_end = heap_start + heap_size;
        param.next = heap_start;
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let param = &mut *self.param.lock();
        let alloc_start = VirtualAddress(align_up(param.next.0, layout.align()));
        let alloc_end = match alloc_start.0.checked_add(layout.size()) {
            Some(end) => VirtualAddress(end),
            None => return core::ptr::null_mut(),
        };

        if alloc_end > param.heap_end {
            core::ptr::null_mut() // Out of memory
        } else {
            param.next = alloc_start + layout.size();
            param.allocations += 1;
            alloc_start.as_mut_ptr()
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        let param = &mut *self.param.lock();

        param.allocations -= 1;
        if param.allocations == 0 {
            param.next = param.heap_start; // Reset the allocator when all allocations are freed
        }
    }
}

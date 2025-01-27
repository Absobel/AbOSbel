use core::arch::asm;

use const_field_count::FieldCount;


/// Long mode Global Descriptor Table
// TODO : Transform this into folder with enums for flags and all
// TODO : maybe use crate amplify_num ? or bitflags ?
use super::{
    GdtFlags, GdtNormalAccess, GdtNormalDescriptor, GdtSystemAccess, GdtSystemDescriptor,
    TaskStateSegment,
};

#[derive(FieldCount)]
#[allow(dead_code)]
struct Gdt {
    null_descriptor: GdtNormalDescriptor,
    kernel_code_descriptor: GdtNormalDescriptor,
    kernel_data_descriptor: GdtNormalDescriptor,
    tss_descriptor: GdtSystemDescriptor,
}

impl Gdt {
    fn change_tss_base(&mut self, base: u64) {
        self.tss_descriptor.change_base(base);
    }
}

static mut GDT: Gdt = {
    let null_descriptor =
            // Null descriptor
            GdtNormalDescriptor::null_descriptor();

    let kernel_code_descriptor =

        // Code segment
        GdtNormalDescriptor::new(
            0x00400000,
            0x003FFFFF,
            GdtNormalAccess {
                present: true,
                privilege: 0,
                descriptor_type: 1,
                executable: true,
                conforming: 0,
                readable_writable: 1,
                accessed: true,
            },
            GdtFlags {
                granularity: 1,
                size: 0,
                long_mode_code: 1,
            },
        );

    let kernel_data_descriptor =
        // Data segment
        GdtNormalDescriptor::new(
           0x00800000,
            0x003FFFFF,
            GdtNormalAccess {
                present: true,
                privilege: 0,
                descriptor_type: 1,
                executable: false,
                conforming: 0,
                readable_writable: 1,
                accessed: true,
            },
            GdtFlags {
                granularity: 1,
                size: 1,
                long_mode_code: 0,
            },
        );

    let tss_descriptor = GdtSystemDescriptor::new(
        0u64, // tss address being loaded at runtime
        core::mem::size_of::<TaskStateSegment>() as u32 - 1,
        GdtSystemAccess {
            present: true,
            privilege: 0,
            descriptor_type: 0,
            segment_type: 0x9,
        },
        GdtFlags {
            granularity: 0,
            size: 0,
            long_mode_code: 0,
        },
    );

    Gdt {
        null_descriptor,
        kernel_code_descriptor,
        kernel_data_descriptor,
        tss_descriptor,
    }
};

static mut TSS: TaskStateSegment = TaskStateSegment::empty();

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

unsafe fn reload_cs() {
    unsafe{asm!(
        "mov ax, 0x10",
        "mov ds, ax",
        "mov es, ax",
        "mov fs, ax",
        "mov gs, ax",
        "mov ss, ax",
    )}
}

pub fn init() {
    crate::x86::without_interrupts(|| unsafe {
        // compile_error!("TODO : Implement GDT init");

        
        let tss_ptr = &raw const TSS as u64;
        #[allow(static_mut_refs)] // TODO : do that better
        GDT.change_tss_base(tss_ptr);
        
        // Load the GDT
        asm!(
            "push 0x08", // TODO : tf is that
            "lea rax, [rip + {rcs}]",
            "push rax",
            "retfq",
            rcs = sym reload_cs,
        )
    })
}

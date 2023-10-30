use core::arch::asm;

pub struct Feature {
    leaf: usize,
    bit: u8,
}

// TODO : Make a safe abstraction
// For now : safe as long as the user (me) is using the constants defined in this file
pub fn cpu_has_feature(feature: Feature) -> bool {
    let edx: usize;
    unsafe {
        asm!(
            "cpuid",
            in("eax") feature.leaf,
            out("edx") edx,
        );
    }
    edx & (1 << feature.bit) != 0
}

// FEATURES

pub const MSR_FEATURE: Feature = Feature { leaf: 1, bit: 12 };

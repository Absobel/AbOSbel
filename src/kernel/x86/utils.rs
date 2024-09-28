use core::arch::asm;

pub struct Feature {
    leaf: usize,
    bit: u8,
}

impl Feature {
    pub fn cpu_has_feature(&self) -> bool {
        let edx: usize;
        unsafe {
            asm!(
                "cpuid",
                in("eax") self.leaf,
                out("edx") edx,
            );
        }
        edx & (1 << self.bit) != 0
    }
}

// FEATURES

pub const MSR_FEATURE: Feature = Feature { leaf: 1, bit: 12 };

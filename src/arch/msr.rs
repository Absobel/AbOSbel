use core::arch::asm;

use crate::arch::cpuid;

/// # Panics
/// If the CPU doesn't support MSR
pub fn init() {
    // Low key useless cause the bootstrap uses MSRs already
    let msr_feature = cpuid::Feature::new(0x1, 0, 5, cpuid::Reg::Edx);
    msr_feature
        .cpu_has_feature()
        .unwrap_or_else(|e| panic!("{e}"));
}

pub struct MsrRegister {
    reg: usize,
}

impl MsrRegister {
    pub const fn new(reg: usize) -> Self {
        Self { reg }
    }

    pub unsafe fn read_msr(&self) -> usize {
        let (eax, edx): (u32, u32);

        unsafe {
            asm!(
                "rdmsr",
                in("ecx") self.reg,
                out("eax") eax,
                out("edx") edx,
            );
        };
        ((edx as usize) << 32) | eax as usize
    }

    pub unsafe fn write_msr(&self, value: usize) {
        let eax = value as u32;
        let edx = (value >> 32) as u32;
        unsafe {
            asm!(
                "wrmsr",
                in("ecx") self.reg,
                in("eax") eax,
                in("edx") edx,
            );
        };
    }
}

use core::arch::asm;

use crate::arch::cpuid;

const MSR: cpuid::Feature = cpuid::Feature::new(0x1, 0, 5, cpuid::Reg::Edx);
#[expect(dead_code)]
const IA32_PAT_MSR: MsrRegister = MsrRegister::new(0x277);

/// # Panics
/// If the CPU doesn't support MSR
pub fn init() {
    // TODO: i guess support when there's no msr? but like?
    MSR.cpu_has_feature().unwrap_or_else(|e| panic!("{e}"));
}

pub struct MsrRegister {
    reg: usize,
}

#[expect(dead_code)]
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

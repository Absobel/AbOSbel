use core::arch::asm;

// CPUID ///

pub struct Feature {
    leaf: usize,
    bit: u8,
}

impl Feature {
    #[must_use]
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

pub const MTTR_FEATURE: Feature = Feature { leaf: 1, bit: 12 };

// INTERRUPTS ///

pub fn without_interrupts<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let r: R;
    unsafe {
        let rflags: usize;
        asm!("pushfq; pop {}", out(reg) rflags, options(nomem, preserves_flags));
        let enabled = rflags & (1 << 9) != 0;
        if enabled {
            asm!("cli", options(nostack, preserves_flags));
        }
        r = f();
        if enabled {
            asm!("sti", options(nostack, preserves_flags));
        }
    }
    r
}

use core::{
    arch::{asm, x86_64::CpuidResult},
    error::Error,
    fmt::Display,
};

use spin::Once;

// CPUID ///

static MAX_CPUID_LEAF: Once<u32> = Once::new();

#[derive(Debug, Clone, Copy)]
pub struct UnsupportedCpuVendor([u8; 12]);
impl Error for UnsupportedCpuVendor {}
impl Display for UnsupportedCpuVendor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Unsupported CPU vendor: {}",
            core::str::from_utf8(&self.0).unwrap_or("Unknown")
        )
    }
}

/// # Errors
/// This function will return an error if the CPU vendor is not supported
pub fn init_cpuid() -> Result<(), UnsupportedCpuVendor> {
    let result = core::arch::x86_64::__cpuid(0);
    MAX_CPUID_LEAF.call_once(|| result.eax);

    let mut vendor = [0; 12];
    vendor[0..4].copy_from_slice(&result.ebx.to_le_bytes());
    vendor[4..8].copy_from_slice(&result.edx.to_le_bytes());
    vendor[8..12].copy_from_slice(&result.ecx.to_le_bytes());

    match &vendor {
        b"GenuineIntel" | b"AuthenticAMD" => (),
        _ => return Err(UnsupportedCpuVendor(vendor)),
    }
    Ok(())
}

#[expect(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum Reg {
    Eax,
    Ebx,
    Ecx,
    Edx,
}

impl Reg {
    const fn get(self, r: &CpuidResult) -> u32 {
        match self {
            Self::Eax => r.eax,
            Self::Ebx => r.ebx,
            Self::Ecx => r.ecx,
            Self::Edx => r.edx,
        }
    }
}

/// cpuid is always available cause it's checked for in the asm bootstrap
#[expect(dead_code)]
pub struct Feature {
    leaf: u32,
    subleaf: u32,
    bit: u8,
    reg: Reg,
}

impl Feature {
    #[expect(dead_code)]
    pub const fn new(leaf: u32, subleaf: u32, bit: u8, reg: Reg) -> Self {
        Self {
            leaf,
            subleaf,
            bit,
            reg,
        }
    }

    #[expect(dead_code)]
    #[must_use]
    pub fn cpu_has_feature(&self) -> bool {
        // Will always panic for extended leaves, but those are not used yet, so it's fine
        debug_assert!(
            self.leaf <= MAX_CPUID_LEAF.get().copied().expect("Init CPUID first"),
            "CPUID leaf {} is not supported by this CPU",
            self.leaf
        );
        self.reg
            .get(&core::arch::x86_64::__cpuid_count(self.leaf, self.subleaf))
            & (1 << self.bit)
            != 0
    }
}

// INTERRUPTS //

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

// MSR //

#[expect(dead_code)]
unsafe fn read_msr(reg: usize) -> usize {
    let (eax, edx): (u32, u32);

    unsafe {
        asm!(
            "rdmsr",
            in("ecx") reg,
            out("eax") eax,
            out("edx") edx,
        );
    };
    ((edx as usize) << 32) | eax as usize
}

#[expect(dead_code)]
unsafe fn write_msr(reg: usize, value: usize) {
    let eax = value as u32;
    let edx = (value >> 32) as u32;
    unsafe {
        asm!(
            "wrmsr",
            in("ecx") reg,
            in("eax") eax,
            in("edx") edx,
        );
    };
}

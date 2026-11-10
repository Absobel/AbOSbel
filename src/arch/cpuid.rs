// CPUID ///

use core::{arch::x86_64::CpuidResult, error::Error, fmt::Display};

use spin::Once;

/// # Errors
/// This function will return an error if the CPU vendor is not supported
pub fn init() -> Result<(), UnsupportedCpuVendor> {
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
#[derive(Debug, Clone, Copy)]
pub struct Feature {
    leaf: u32,
    subleaf: u32,
    bit: u8,
    reg: Reg,
}

impl Feature {
    pub const fn new(leaf: u32, subleaf: u32, bit: u8, reg: Reg) -> Self {
        Self {
            leaf,
            subleaf,
            bit,
            reg,
        }
    }

    pub fn cpu_has_feature(&self) -> Result<(), UnsupportedFeature> {
        // Will always panic for extended leaves, but those are not used yet, so it's fine
        debug_assert!(
            self.leaf <= MAX_CPUID_LEAF.get().copied().expect("Init CPUID first"),
            "CPUID leaf {} is not supported by this CPU",
            self.leaf
        );
        let is_available = self
            .reg
            .get(&core::arch::x86_64::__cpuid_count(self.leaf, self.subleaf))
            & (1 << self.bit)
            != 0;
        if !is_available {
            return Err(UnsupportedFeature::new(*self));
        }
        Ok(())
    }
}

// ERRORS //

#[derive(Debug, Clone, Copy)]
pub struct UnsupportedFeature {
    feature: Feature,
}
impl UnsupportedFeature {
    pub const fn new(feature: Feature) -> Self {
        Self { feature }
    }
}
impl Error for UnsupportedFeature {}
impl Display for UnsupportedFeature {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Unsupported CPU feature: leaf {}, subleaf {}, bit {}, reg {:?}",
            self.feature.leaf, self.feature.subleaf, self.feature.bit, self.feature.reg
        )
    }
}

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

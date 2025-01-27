use core::{arch::asm, ops::RangeInclusive};

use super::{utils::MSR_FEATURE, without_interrupts};

// TODO : bitflags crate looks perfect for this and i already use it elsewhere ?

////////////////////////////////

const WC_MEMORY_TYPE: usize = 1;
// TODO : Find this through cpuid or smth
const ADDRESS_WIDTH: usize = 48;

///////////////////////////////

unsafe fn readmsr_byte(reg: usize) -> usize {
    let (eax, edx): (u32, u32);

    unsafe{asm!(
        "rdmsr",
        in("ecx") reg,
        out("eax") eax,
        out("edx") edx,
    )};
    ((edx as usize) << 32) | eax as usize
}

unsafe fn writemsr_byte(reg: usize, value: usize) {
    let eax = value as u32;
    let edx = (value >> 32) as u32;
    unsafe{asm!(
        "wrmsr",
        in("ecx") reg,
        in("eax") eax,
        in("edx") edx,
    )};
}

unsafe fn readmsr(reg: usize, bits: RangeInclusive<usize>) -> usize {
    let byte = unsafe{readmsr_byte(reg)};
    let mask = (1 << (bits.end() + 1)) - (1 << bits.start());
    (byte & mask) >> bits.start()
}

unsafe fn writemsr(reg: usize, bits: RangeInclusive<usize>, value: usize) -> Result<(), MsrError> {
    if value >> (bits.end() - bits.start() + 1) != 0 {
        return Err(MsrError::ValueExceedsBitRange);
    }
    let byte = unsafe {readmsr_byte(reg)};
    let mask = (1 << (bits.end() + 1)) - (1 << bits.start());
    let new_byte = (byte & !mask) | ((value << bits.start()) & mask);
    unsafe{writemsr_byte(reg, new_byte)};
    Ok(())
}

///////////////////////////////

#[derive(Debug)]
pub enum MsrError {
    NoMsrSupport,
    NoWCTypeSupport,
    NoFreeMtrPair,
    ValueExceedsBitRange,
}
trait ConstMsr {
    const REG: usize;
}

struct Ia32MtrrCap;

impl ConstMsr for Ia32MtrrCap {
    const REG: usize = 0xFE;
}

impl Ia32MtrrCap {
    fn has_wc_type_support() -> bool {
        unsafe { readmsr(Self::REG, 10..=10) == 1 }
    }

    fn mtrr_pair_reg_nb() -> usize {
        unsafe { readmsr(Self::REG, 0..=7) }
    }
}

struct Ia32MtrrDefType;

impl ConstMsr for Ia32MtrrDefType {
    const REG: usize = 0x2FF;
}

impl Ia32MtrrDefType {
    fn enable_mtrr() {
        unsafe {
            writemsr(Self::REG, 11..=11, 1).unwrap();
        }
    }
}

struct MtrrPhysPair {
    base: usize,
}

impl MtrrPhysPair {
    fn base_reg(&self) -> usize {
        self.base
    }
    fn mask_reg(&self) -> usize {
        self.base + 1
    }

    fn free_mtrr_pair() -> Option<MtrrPhysPair> {
        let mttr_pair_reg_nb = Ia32MtrrCap::mtrr_pair_reg_nb();
        let mut mttr_pair_reg = None;
        for i in 0..mttr_pair_reg_nb {
            let mask_reg = 0x201 + i * 2;
            let valid_bit = unsafe { readmsr(mask_reg, 11..=11) };
            if valid_bit == 0 {
                mttr_pair_reg = Some(MtrrPhysPair { base: mask_reg - 1 });
                break;
            }
        }
        mttr_pair_reg
    }

    fn set_memory_type(
        &self,
        addr: usize,
        size: usize,
        memory_type: usize,
    ) -> Result<(), MsrError> {
        // size must be aligned to a boundary of a power of two and not be bigger than NB_PHYSYCAL_ADDRESS_BITS bits
        let mask = !(size.next_power_of_two() - 1) & ((1 << ADDRESS_WIDTH) - 1);

        without_interrupts(move || unsafe {
            writemsr(self.base_reg(), 12..=(ADDRESS_WIDTH - 1), addr >> 12)?; // Set the base address
            writemsr(self.base_reg(), 0..=7, memory_type)?; // Set the memory type
            writemsr(self.mask_reg(), 12..=(ADDRESS_WIDTH - 1), mask >> 12)?; // Set the mask
            writemsr(self.mask_reg(), 11..=11, 1)?; // Set the valid bit
            Ok(())
        })?;

        Ok(())
    }
}

///////////////////////////////

// TODO : Think of a better way than .next_power_of_two() to align the size (several MTRRs can be used)
pub fn set_mtrr_wc(addr: usize, size: usize) -> Result<(), MsrError> {
    if !MSR_FEATURE.cpu_has_feature() {
        return Err(MsrError::NoMsrSupport);
    }
    if !Ia32MtrrCap::has_wc_type_support() {
        return Err(MsrError::NoWCTypeSupport);
    }

    Ia32MtrrDefType::enable_mtrr();

    let mtrr_phys_pair = MtrrPhysPair::free_mtrr_pair().ok_or(MsrError::NoFreeMtrPair)?;
    mtrr_phys_pair.set_memory_type(addr, size, WC_MEMORY_TYPE)?;

    Ok(())
}

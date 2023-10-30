use core::{arch::asm, ops::RangeInclusive};

use crate::serial_println;

use super::utils::{cpu_has_feature, MSR_FEATURE};

// TODO : add safe abstractions

////////////////////////////////

const IA32_MTRRCAP: usize = 0xFE;
const IA32_MTRR_DEF_TYPE: usize = 0x2FF;

const WC_MEMORY_TYPE: usize = 1;

////////////////////////////////

#[derive(Debug)]
pub enum MsrError {
    NoMsrSupport,
    NoWCTypeSupport,
    NoFreeMtrPair,
    ValueExceedsBitRange,
}

///////////////////////////////

unsafe fn readmsr_byte(reg: usize) -> usize {
    let (eax, edx): (u32, u32);

    asm!(
        "rdmsr",
        in("ecx") reg,
        out("eax") eax,
        out("edx") edx,
    );
    ((edx as usize) << 32) | eax as usize
}

unsafe fn writemsr_byte(reg: usize, value: usize) {
    let eax = value as u32;
    let edx = (value >> 32) as u32;
    asm!(
        "wrmsr",
        in("ecx") reg,
        in("eax") eax,
        in("edx") edx,
    );
}

unsafe fn readmsr(reg: usize, bits: RangeInclusive<usize>) -> usize {
    let byte = readmsr_byte(reg);
    let mask = (1 << (bits.end() + 1)) - (1 << bits.start());
    (byte & mask) >> bits.start()
}

// TODO : Check that the value is in the range
unsafe fn writemsr(reg: usize, bits: RangeInclusive<usize>, value: usize) -> Result<(), MsrError> {
    if value >> (bits.end() - bits.start() + 1) != 0 {
        return Err(MsrError::ValueExceedsBitRange);
    }
    serial_println!("reg = {:#x}", reg); // DEBUG
    serial_println!("value = {:#x}", value); // DEBUG
    serial_println!("bits = {:?}", bits); // DEBUG
    let byte = readmsr_byte(reg);
    serial_println!("byte = {:#x}", byte); // DEBUG
    let mask = (1 << (bits.end() + 1)) - (1 << bits.start());
    let new_byte = (byte & !mask) | ((value << bits.start()) & mask);
    serial_println!("new_byte = {:#x}", new_byte); // DEBUG
    writemsr_byte(reg, new_byte);
    Ok(())
}

///////////////////////////////

fn has_msr_support() -> bool {
    cpu_has_feature(MSR_FEATURE)
}

fn has_wc_type_support() -> bool {
    unsafe { readmsr(IA32_MTRRCAP, 10..=10) == 1 }
}

fn mtrr_pair_reg_nb() -> usize {
    unsafe { readmsr(IA32_MTRRCAP, 0..=7) }
}

fn enable_mtrr() {
    unsafe {
        writemsr(IA32_MTRR_DEF_TYPE, 11..=11, 1).unwrap();
    }
}

///////////////////////////////

fn free_mtrr_pair() -> Option<(usize, usize)> {
    let mttr_pair_reg_nb = mtrr_pair_reg_nb();
    let mut mttr_pair_reg = None;
    for i in 0..mttr_pair_reg_nb {
        let mask_reg = 0x201 + i * 2;
        let valid_bit = unsafe { readmsr(mask_reg, 11..=11) };
        if valid_bit == 0 {
            mttr_pair_reg = Some((mask_reg - 1, mask_reg));
            break;
        }
    }
    mttr_pair_reg
}

// TODO : Think of a better way than .next_power_of_two() to align the size
pub fn set_mtrr_wc(addr: usize, size: usize) -> Result<(), MsrError> {
    if !has_msr_support() {
        return Err(MsrError::NoMsrSupport);
    }
    if !has_wc_type_support() {
        return Err(MsrError::NoWCTypeSupport);
    }

    serial_println!("addr = {:#x}", addr); // DEBUG
    serial_println!("size = {:#x}", size.next_power_of_two()); // DEBUG

    serial_println!("/// Enabling MTRR..."); // DEBUG
    enable_mtrr();

    // Use the MTTR pair to set the WC memory type to the given address range
    let (base_reg, mask_reg) = free_mtrr_pair().ok_or(MsrError::NoFreeMtrPair)?;
    // size must be aligned to a boundary of a power of two and not be bigger than 52 bits
    let mask = !(size.next_power_of_two() - 1) & ((1 << 52) - 1);

    x86_64::instructions::interrupts::without_interrupts(move || unsafe {
        serial_println!("/// Setting the base address..."); // DEBUG
        writemsr(base_reg, 12..=51, addr >> 12).unwrap(); // Set the base address
        serial_println!("/// Setting the memory type..."); // DEBUG
        writemsr(base_reg, 0..=7, WC_MEMORY_TYPE).unwrap(); // Set the memory type
        serial_println!("/// Setting the mask..."); // DEBUG
        writemsr(mask_reg, 12..=51, mask >> 12).unwrap(); // Set the mask
        serial_println!("/// Setting the valid bit..."); // DEBUG
        writemsr(mask_reg, 11..=11, 1).unwrap(); // Set the valid bit
    });

    serial_println!("Done !"); // DEBUG

    Ok(())
}

#![allow(dead_code)]

use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::port::Port;

// CONSTS

lazy_static! {
    pub static ref INDEX_STATUS_PORT: Mutex<Port<u8>> = Mutex::new(Port::new(0x3DA));
    pub static ref INDEX_WDATA_PORT: Mutex<Port<u8>> = Mutex::new(Port::new(0x3C0));
    pub static ref RDATA_PORT: Mutex<Port<u8>> = Mutex::new(Port::new(0x3C1));
}

const ACTIVATED: u8 = 0b1111_1111;
const DEACTIVATED: u8 = 0b0000_0000;

#[allow(non_snake_case)]
const fn MODE_CTRL_IDX_NORMAL() -> AdressRegister {
    AdressRegister {
        operation: Operation::Normal,
        raw_index: IDX::MODE_CTRL,
    }
}

// ENUMS

#[repr(u8)]
enum Operation {
    Palette = 0b0000_0000,
    Normal = 0b0010_0000,
}

#[allow(non_camel_case_types)]
#[repr(u8)]
enum RawRegisterIndex {
    MODE_CTRL = 0x10,
}
use RawRegisterIndex as IDX;

#[allow(clippy::upper_case_acronyms)]
#[repr(u8)]
enum AttributeModeCtrlMask {
    ATGE = 0b0000_0001,
    MONO = 0b0000_0010,
    LGE = 0b0000_0100,
    BLINK = 0b0000_1000,
    PPM = 0b0010_0000,
    N8BIT = 0b0100_0000,
    P54S = 0b1000_0000,
}
use AttributeModeCtrlMask as MC_MASK;

// STRUCTS

struct AdressRegister {
    operation: Operation,
    raw_index: RawRegisterIndex,
}

impl From<AdressRegister> for u8 {
    fn from(adress_register: AdressRegister) -> Self {
        (adress_register.operation as u8) | (adress_register.raw_index as u8)
    }
}

// HELPER FUNCTIONS

fn format_write_data(mask: u8, old_data: u8, new_data: u8) -> u8 {
    (old_data & !mask) | (new_data & mask)
}

// INTERN API

unsafe fn write_data_attribute_register(index: u8, mask: u8, data: u8) {
    let mut idx_wdata_port = INDEX_WDATA_PORT.lock();

    let _ = INDEX_STATUS_PORT.lock().read(); // sets the index_data register to index mode
    idx_wdata_port.write(index); // sets the data register to the subregister of index $index

    let old_data_value = RDATA_PORT.lock().read();
    idx_wdata_port.write(format_write_data(mask, old_data_value, data));
}

unsafe fn read_data_attribute_register(index: u8, mask: u8) -> u8 {
    let _ = INDEX_STATUS_PORT.lock().read(); // sets the index_data register to index mode
    INDEX_WDATA_PORT.lock().write(index); // sets the data register to the subregister of index $index

    let data = RDATA_PORT.lock().read();
    data & mask
}

unsafe fn toggle_data_attribute_register(index: u8, mask: u8) {
    let mut idx_wdata_port = INDEX_WDATA_PORT.lock();

    let _ = INDEX_STATUS_PORT.lock().read(); // sets the index_data register to index mode
    idx_wdata_port.write(index); // sets the data register to the subregister of index $index

    let old_data_value = RDATA_PORT.lock().read();
    idx_wdata_port.write(old_data_value ^ mask);
}

// PUBLIC API

pub fn enable_blink_mode() {
    unsafe {
        write_data_attribute_register(
            MODE_CTRL_IDX_NORMAL().into(),
            MC_MASK::BLINK as u8,
            ACTIVATED,
        )
    };
}

pub fn disable_blink_mode() {
    unsafe {
        write_data_attribute_register(
            MODE_CTRL_IDX_NORMAL().into(),
            MC_MASK::BLINK as u8,
            DEACTIVATED,
        )
    };
}

pub fn toggle_blink_mode() {
    unsafe { toggle_data_attribute_register(MODE_CTRL_IDX_NORMAL().into(), MC_MASK::BLINK as u8) };
}

pub fn is_blink_mode_enabled() -> bool {
    unsafe {
        read_data_attribute_register(MODE_CTRL_IDX_NORMAL().into(), MC_MASK::BLINK as u8)
            == ACTIVATED & MC_MASK::BLINK as u8
    }
}

// TESTS

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn enable_blink_mode_test() {
        enable_blink_mode();
        assert_eq!(
            unsafe {
                read_data_attribute_register(MODE_CTRL_IDX_NORMAL().into(), MC_MASK::BLINK as u8)
            },
            ACTIVATED & MC_MASK::BLINK as u8
        );
    }

    #[test_case]
    fn disable_blink_mode_test() {
        disable_blink_mode();
        assert_eq!(
            unsafe {
                read_data_attribute_register(MODE_CTRL_IDX_NORMAL().into(), MC_MASK::BLINK as u8)
            },
            DEACTIVATED & MC_MASK::BLINK as u8
        );
    }

    #[test_case]
    fn toggle_blink_mode_test() {
        enable_blink_mode();
        toggle_blink_mode();
        assert_eq!(
            unsafe {
                read_data_attribute_register(MODE_CTRL_IDX_NORMAL().into(), MC_MASK::BLINK as u8)
            },
            DEACTIVATED & MC_MASK::BLINK as u8
        );
        toggle_blink_mode();
        assert_eq!(
            unsafe {
                read_data_attribute_register(MODE_CTRL_IDX_NORMAL().into(), MC_MASK::BLINK as u8)
            },
            ACTIVATED & MC_MASK::BLINK as u8
        );
    }
}

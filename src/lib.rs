#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(error_in_core)]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(clippy::missing_safety_doc)]

mod kernel;
mod tests_lib;
mod utils;

use core::arch::global_asm;

pub use kernel::*;
pub use tests_lib::*;
pub use utils::*;

// BOOTSTRAP

global_asm!(include_str!("preliminary/multiboot.s"), options(raw));
global_asm!(include_str!("preliminary/boot.s"), options(raw));

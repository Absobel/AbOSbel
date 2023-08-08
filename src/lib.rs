// Purpose: Library for integration tests.

#![no_std]
#![feature(abi_x86_interrupt)]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod kernel;
mod utils;
mod tests_lib;

pub use kernel::*;
pub use utils::*;
pub use tests_lib::*;
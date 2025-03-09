#![no_std]
#![feature(abi_x86_interrupt)]
#![allow(clippy::missing_safety_doc)]

mod kernel;
mod utils;

use core::arch::global_asm;

pub use kernel::*;
pub use utils::*;

// BOOTSTRAP

global_asm!(include_str!("preliminary/multiboot.s"), options(raw));
global_asm!(include_str!("preliminary/boot.s"), options(raw));

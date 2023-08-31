use crate::*;

//// TESTS

// CORE FOR TESTS

#[cfg(test)]
#[no_mangle]
pub extern "C" fn main(_multiboot_info_addr: usize) -> ! {  
    init();
    test_main();
    hlt_loop()
}

// PANIC FOR TESTS

use core::panic::PanicInfo;

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);

    hlt_loop()
}

// TEST RUNNER

pub trait Testable {
    fn run(&self);
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

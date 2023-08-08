use crate::println;
use ab_os_bel::{serial_println, vga::WRITER};

#[allow(dead_code)]
pub fn main() {
    WRITER.lock().clear();
    println!("Hello World{}", "!");

    // triggering a stack overflow exception
    fn stack_overflow(i: usize) -> usize {
        serial_println!("stack overflow {}", i);
        stack_overflow(i + 1)
    }

    stack_overflow(0);

    println!("It did not crash!");
}

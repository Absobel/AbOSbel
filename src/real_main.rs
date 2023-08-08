use crate::println;
use ab_os_bel::vga::WRITER;

#[allow(dead_code)]
pub fn main() {
    WRITER.lock().clear();
    println!("Hello World{}", "!");
    x86_64::instructions::interrupts::int3();
    println!("It did not crash!");
}

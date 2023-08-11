use crate::println;
use ab_os_bel::vga::WRITER;

#[allow(dead_code)]
pub fn main() {
    WRITER.lock().clear();
    println!("Hello World{}", "!");

    WRITER.lock().clear();

    println!("It did not crash!");
}

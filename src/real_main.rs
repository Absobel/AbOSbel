use crate::println;
use ab_os_bel::vga::WRITER;

#[allow(dead_code)]
pub fn main() {
    WRITER.lock().clear();
    println!("Hello World{}", "!");
    panic!("Some panic message");
}

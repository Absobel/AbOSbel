use crate::println;
use crate::vga_buffer::WRITER;

#[allow(dead_code)]
pub fn main() {
    WRITER.lock().clear();
    println!("Hello World{}", "!");
    panic!("Some panic message");
}

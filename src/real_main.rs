use crate::println;
use ab_os_bel::vga::WRITER;
use x86_64::registers::control::Cr3;

#[allow(dead_code)]
pub fn main() {
    WRITER.lock().clear();
    println!("Hello World{}", "!");

    //let (level_4_page_table, _) = Cr3::read();
    //println!("Level 4 page table at: {:?}", level_4_page_table.start_address());

    println!("It did not crash!");
}

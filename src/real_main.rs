use crate::println;
use ab_os_bel::vga::WRITER;

#[allow(dead_code)]
pub fn main() {
    WRITER.lock().clear();
    println!("Hello World{}", "!");
    println!("Blink mode ? {}", ab_os_bel::vga::io_ports::is_blink_mode_enabled());
    ab_os_bel::vga::io_ports::disable_blink_mode();
    println!("Blink mode ? {}", ab_os_bel::vga::io_ports::is_blink_mode_enabled());
    WRITER.lock().change_blink(true);
    println!("Hello Blinky World{}", "!");
}

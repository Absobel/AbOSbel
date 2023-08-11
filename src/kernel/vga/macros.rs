#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test_case]
    fn no_panic_print() {
        println!("test_println_simple output");
    }

    #[test_case]
    fn no_panic_overflow_print() {
        for _ in 0..100 {
            println!("test_println_overflow output");
        }
    }
}

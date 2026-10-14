use spin::Mutex;
use uart_16550::SerialPort;

use crate::x86::without_interrupts;

pub static SERIAL1: Mutex<SerialPort> = Mutex::new(unsafe { SerialPort::new(0x3F8) });

pub fn init_serial() {
    SERIAL1.lock().init();
}

// MACROS

#[doc(hidden)]
pub fn _serial_print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;

    // Deactivating interrupts to avoid deadlocks
    without_interrupts(|| {
        SERIAL1
            .lock()
            .write_fmt(args)
            .expect("Printing to serial failed");
    });
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::io::serial::_serial_print(format_args!($($arg)*))
    };
}

/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}

#[macro_export]
macro_rules! serial_dbg {
    () => {
        $crate::serial_println!("[{}:{}]", $crate::file!(), $crate::line!())
    };
    ($val:expr $(,)?) => {
        match $val {
            tmp => {
                $crate::serial_println!("[{}:{}] {} = {:#?}",
                    file!(), line!(), stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::serial_dbg!($val)),+,)
    };
}

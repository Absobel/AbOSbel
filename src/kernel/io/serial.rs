use spin::{Mutex, Once};

use crate::{
    io::{LoopbackMismatch, SerialPort},
    x86::without_interrupts,
};

pub static SERIAL1: Once<Mutex<SerialPort>> = Once::new();

/// # Errors
/// This function will return an error if the serial port is faulty/doesn't exist
pub fn init_serial() -> Result<(), LoopbackMismatch> {
    let serial_port = unsafe { SerialPort::new(0x3F8) };
    serial_port.init()?;
    SERIAL1.call_once(|| Mutex::new(serial_port));
    Ok(())
}

// MACROS

#[doc(hidden)]
pub fn _serial_print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;

    // Deactivating interrupts to avoid deadlocks
    without_interrupts(|| {
        if let Some(serial) = SERIAL1.get() {
            serial
                .lock()
                .write_fmt(args)
                .expect("Printing to serial failed");
        }
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
    ($($arg:tt)*) => ($crate::serial_print!("{}\n", format_args!($($arg)*)));

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

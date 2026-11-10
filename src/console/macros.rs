#[macro_export]
macro_rules! print {
    () => ($crate::_print(format_args!("")));
    ($($arg:tt)*) => ($crate::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! dbg {
    () => {
        $crate::println!("[{}:{}]\n", $crate::file!(), $crate::line!())
    };
    ($val:expr $(,)?) => {
        match $val {
            tmp => {
                $crate::println!("[{}:{}] {} = \n{:#?}",
                    file!(), line!(), stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::_serial_print(format_args!($($arg)*))
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

//! Kernel output mirrored to the screen and QEMU's COM1 serial device.

use core::fmt;
use spin::Mutex;

static OUTPUT_LOCK: Mutex<()> = Mutex::new(());

pub fn init() {
    let _lock = OUTPUT_LOCK.lock();
    crate::serial::init();
    crate::vga::clear();
}

pub fn _print(args: fmt::Arguments) {
    let _lock = OUTPUT_LOCK.lock();
    crate::vga::write_fmt(args);
    crate::serial::write_fmt(args);
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => ($crate::console::_print(core::format_args!($($arg)*)));
}

#[macro_export]
macro_rules! kprintln {
    () => ($crate::kprint!("\n"));
    ($fmt:expr) => ($crate::kprint!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::kprint!(concat!($fmt, "\n"), $($arg)*));
}

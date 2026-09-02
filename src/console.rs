//! Kernel output mirrored to the screen and QEMU's COM1 serial device.

use core::{fmt, sync::atomic::{AtomicBool, Ordering}};
use spin::Mutex;
use x86_64::instructions::interrupts;

static OUTPUT_LOCK: Mutex<()> = Mutex::new(());
static INITIALIZED: AtomicBool = AtomicBool::new(false);

pub fn init() {
    let was_initialized = INITIALIZED.swap(true, Ordering::AcqRel);
    if was_initialized {
        return;
    }

    let interrupts_were_enabled = interrupts::are_enabled();
    interrupts::disable();
    {
        let _lock = OUTPUT_LOCK.lock();
        crate::serial::init();
        crate::vga::clear();
    }
    if interrupts_were_enabled {
        interrupts::enable();
    }
}

/// Print atomically with respect to timer IRQs. Interrupts are disabled while the
/// spinlock is held so a preempting IRQ can never deadlock trying to acquire it.
pub fn _print(args: fmt::Arguments) {
    let interrupts_were_enabled = interrupts::are_enabled();
    interrupts::disable();
    {
        let _lock = OUTPUT_LOCK.lock();
        crate::vga::write_fmt(args);
        crate::serial::write_fmt(args);
    }
    if interrupts_were_enabled {
        interrupts::enable();
    }
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

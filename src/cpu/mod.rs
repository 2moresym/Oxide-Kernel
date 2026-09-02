//! CPU descriptor-table, exception, hardware-interrupt, and scheduler initialization.

pub mod context;
pub mod gdt;
pub mod interrupts;
pub mod pic;
pub mod scheduler;
pub mod timer;

pub fn init() {
    gdt::init();
    interrupts::init();
    pic::init();
    timer::init();
}

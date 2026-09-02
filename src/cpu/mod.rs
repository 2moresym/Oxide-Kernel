//! CPU descriptor-table, exception, and hardware-interrupt initialization.

pub mod gdt;
pub mod interrupts;
pub mod pic;
pub mod timer;

pub fn init() {
    gdt::init();
    interrupts::init();
    pic::init();
    timer::init();
}

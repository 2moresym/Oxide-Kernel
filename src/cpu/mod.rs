//! CPU descriptor-table and exception initialization.

pub mod gdt;
pub mod interrupts;

pub fn init() {
    gdt::init();
    interrupts::init();
}

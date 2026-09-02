#![no_std]
#![no_main]

mod vga;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

entry_point!(kernel_main);

/// The first Rust code reached after the bootloader has prepared the machine.
fn kernel_main(_boot_info: &'static BootInfo) -> ! {
    vga::clear();
    vga::write_line("Oxide OS", vga::Color::LightGreen);
    vga::write_line("x86_64 kernel booted successfully.", vga::Color::LightGray);

    halt_loop()
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    vga::write_line("Kernel panic", vga::Color::LightRed);
    halt_loop()
}

fn halt_loop() -> ! {
    loop {
        // `hlt` keeps an idle kernel from consuming a full CPU core in QEMU.
        unsafe { core::arch::asm!("hlt", options(nomem, nostack, preserves_flags)) };
    }
}

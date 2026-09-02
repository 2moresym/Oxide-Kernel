#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod console;
mod cpu;
mod memory;
mod serial;
mod vga;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use x86_64::{
    structures::paging::FrameAllocator,
    VirtAddr,
};

entry_point!(kernel_main);

/// The first Rust code reached after the bootloader has prepared the machine.
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    console::init();
    kprintln!("Oxide OS");
    kprintln!("x86_64 kernel booted successfully.");

    cpu::init();
    kprintln!("CPU: GDT and exception IDT loaded (hardware interrupts remain disabled).");

    memory::log_memory_map(&boot_info.memory_map);
    let physical_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let level_4_table = unsafe { memory::active_level_4_table(physical_memory_offset) };
    kprintln!("Paging: active level-4 table at {:p}", level_4_table as *mut _);

    let mut frame_allocator = unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };
    if let Some(frame) = frame_allocator.allocate_frame() {
        kprintln!("Memory: first usable 4 KiB frame: {:?}", frame);
    }

    // Proves that the breakpoint exception is handled without rebooting QEMU.
    x86_64::instructions::interrupts::int3();
    kprintln!("CPU: breakpoint exception handled; Oxide OS is idle.");

    halt_loop()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kprintln!("\nKERNEL PANIC: {info}");
    halt_loop()
}

pub fn halt_loop() -> ! {
    loop {
        unsafe { core::arch::asm!("hlt", options(nomem, nostack, preserves_flags)) };
    }
}

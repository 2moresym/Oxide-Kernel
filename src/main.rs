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
use x86_64::{structures::paging::FrameAllocator, VirtAddr};

entry_point!(kernel_main);

/// The first Rust code reached after the bootloader has prepared the machine.
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    console::init();
    kprintln!("Oxide OS");
    kprintln!("x86_64 kernel booted successfully.");

    cpu::init();
    kprintln!("CPU: GDT, exception IDT, PIC, and PIT initialized.");
    kprintln!("Timer: {} Hz kernel tick source ready.", cpu::timer::frequency_hz());

    memory::log_memory_map(&boot_info.memory_map);
    let physical_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let level_4_table = unsafe { memory::active_level_4_table(physical_memory_offset) };
    kprintln!("Paging: active level-4 table at {:p}", level_4_table as *mut _);

    // Keep explicit checkpoints here while the memory subsystem is still small. If boot
    // stops, the last line identifies the exact subsystem that failed.
    kprintln!("Memory: initializing frame allocator...");
    let mut frame_allocator = unsafe {
        memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
    };
    kprintln!("Memory: frame allocator initialized.");

    match frame_allocator.allocate_frame() {
        Some(frame) => kprintln!("Memory: first usable 4 KiB frame: {:?}", frame),
        None => kprintln!("Memory: no usable 4 KiB frame was reported."),
    }
    kprintln!("Memory: frame allocator checkpoint passed.");

    // Proves that the breakpoint exception is handled without rebooting QEMU.
    kprintln!("CPU: testing breakpoint exception...");
    x86_64::instructions::interrupts::int3();
    kprintln!("CPU: breakpoint exception handled.");

    // Build the first scheduler tasks before enabling IRQ0.
    kprintln!("Scheduler: initializing...");
    cpu::scheduler::init();
    kprintln!("Scheduler: ready for preemptive task switching.");

    // IRQ0 now drives both the system tick and scheduler.
    x86_64::instructions::interrupts::enable();
    kprintln!("CPU: hardware interrupts enabled; multitasking is live.");

    halt_loop()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kprintln!("\nKERNEL PANIC: {}", info);
    halt_loop()
}

pub fn halt_loop() -> ! {
    loop {
        unsafe { core::arch::asm!("hlt", options(nomem, nostack, preserves_flags)) };
    }
}

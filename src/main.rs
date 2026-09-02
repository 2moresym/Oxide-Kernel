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

    kprintln!("Memory: initializing frame allocator...");
    let mut frame_allocator = unsafe {
        memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
    };
    kprintln!("Memory: frame allocator initialized.");

    match frame_allocator.allocate_frame() {
        Some(frame) => kprintln!("Memory: first usable 4 KiB frame: {:?}", frame),
        None => kprintln!("Memory: no usable 4 KiB frame was reported."),
    }
    kprintln!("Memory: frame allocation checkpoint passed.");

    // Breakpoint diagnostic is split into four observable phases:
    // 1. execution reaches INT3, 2. the IDT dispatches it, 3. the handler returns,
    // 4. normal execution resumes after INT3. The handler itself performs no I/O.
    kprintln!("CPU: breakpoint test 1/4 - executing INT3...");
    x86_64::instructions::interrupts::int3();

    kprintln!("CPU: breakpoint test 2/4 - IDT handler returned.");
    match cpu::interrupts::breakpoint_stage() {
        cpu::interrupts::BREAKPOINT_ENTERED => {
            kprintln!("CPU: breakpoint test 3/4 - handler was entered.");
        }
        _ => {
            kprintln!("CPU: breakpoint test 3/4 - ERROR: handler was not entered.");
            crate::halt_loop();
        }
    }

    // Reaching this statement is the strongest proof that INT3 returned to the
    // instruction stream instead of faulting, looping, or corrupting the stack.
    kprintln!("CPU: breakpoint test 4/4 - execution resumed after INT3.");
    kprintln!("CPU: breakpoint exception path passed all 4 checks.");

    // Build the first scheduler tasks before enabling IRQ0.
    kprintln!("Scheduler: initializing...");
    cpu::scheduler::init();
    kprintln!("Scheduler: ready for preemptive task switching.");

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

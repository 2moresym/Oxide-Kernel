use core::sync::atomic::{AtomicU8, Ordering};
use spin::Once;
use x86_64::{
    registers::control::Cr2,
    structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode},
};

use super::{gdt, pic, scheduler, timer};

static IDT: Once<InterruptDescriptorTable> = Once::new();
static BREAKPOINT_STAGE: AtomicU8 = AtomicU8::new(0);

pub const BREAKPOINT_ENTERED: u8 = 1;

#[inline]
pub fn breakpoint_stage() -> u8 { BREAKPOINT_STAGE.load(Ordering::Acquire) }

pub fn init() {
    IDT.call_once(|| {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt[pic::PIC1_OFFSET].set_handler_fn(timer_handler);
        idt
    }).load();
}

extern "x86-interrupt" fn breakpoint_handler(_stack_frame: InterruptStackFrame) {
    BREAKPOINT_STAGE.store(BREAKPOINT_ENTERED, Ordering::Release);
}

extern "x86-interrupt" fn double_fault_handler(_stack_frame: InterruptStackFrame, error_code: u64) -> ! {
    crate::kprintln!("EXCEPTION: DOUBLE FAULT ({})", error_code);
    crate::halt_loop()
}

extern "x86-interrupt" fn page_fault_handler(_stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode) {
    crate::kprintln!("EXCEPTION: PAGE FAULT at {:?} ({:?})", Cr2::read(), error_code);
    crate::halt_loop()
}

extern "x86-interrupt" fn timer_handler(_stack_frame: InterruptStackFrame) {
    timer::tick();
    pic::end_of_interrupt(0);
    scheduler::tick();
}

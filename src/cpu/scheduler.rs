//! Simple preemptive round-robin kernel scheduler.

use core::sync::atomic::{AtomicU64, Ordering};

use super::context::oxide_context_switch;

const TASK_COUNT: usize = 3;
const STACK_SIZE: usize = 16 * 1024;
const QUANTUM_TICKS: u64 = 10;

#[repr(C)]
struct TaskContext {
    rsp: u64,
}

impl TaskContext {
    const fn empty() -> Self { Self { rsp: 0 } }
}

#[repr(align(16))]
struct TaskStack([u8; STACK_SIZE]);

static mut TASK_STACKS: [TaskStack; TASK_COUNT] = [
    TaskStack([0; STACK_SIZE]),
    TaskStack([0; STACK_SIZE]),
    TaskStack([0; STACK_SIZE]),
];

static mut CONTEXTS: [TaskContext; TASK_COUNT] = [
    TaskContext::empty(), TaskContext::empty(), TaskContext::empty(),
];

static CURRENT_TASK: AtomicU64 = AtomicU64::new(0);
static QUANTUM: AtomicU64 = AtomicU64::new(0);
static INITIALIZED: AtomicU64 = AtomicU64::new(0);

/// Initialize two runnable kernel tasks. Task 0 is the boot/kernel context.
pub fn init() {
    if INITIALIZED.swap(1, Ordering::AcqRel) != 0 { return; }
    unsafe {
        CONTEXTS[1].rsp = make_task_stack(&mut TASK_STACKS[1], task_one);
        CONTEXTS[2].rsp = make_task_stack(&mut TASK_STACKS[2], task_two);
    }
    crate::kprintln!("Scheduler: 2 kernel tasks created (round-robin).");
    crate::kprintln!("Scheduler: quantum = {} ticks ({} ms).", QUANTUM_TICKS, QUANTUM_TICKS * 10);
}

/// Called from IRQ0. Switch tasks every quantum.
pub fn tick() {
    if INITIALIZED.load(Ordering::Acquire) == 0 { return; }
    let quantum = QUANTUM.fetch_add(1, Ordering::Relaxed) + 1;
    if quantum < QUANTUM_TICKS { return; }
    QUANTUM.store(0, Ordering::Relaxed);

    let current = CURRENT_TASK.load(Ordering::Relaxed) as usize;
    let next = match current { 0 => 1, 1 => 2, _ => 1 };
    CURRENT_TASK.store(next as u64, Ordering::Relaxed);

    unsafe { oxide_context_switch(&mut CONTEXTS[current].rsp, CONTEXTS[next].rsp); }
}

fn make_task_stack(stack: &mut TaskStack, entry: extern "C" fn() -> !) -> u64 {
    let top = stack.0.as_mut_ptr() as usize + STACK_SIZE;
    let rsp = (top & !0xF) - 7 * core::mem::size_of::<u64>();
    let frame = rsp as *mut u64;
    unsafe {
        frame.add(0).write(0); // r15
        frame.add(1).write(0); // r14
        frame.add(2).write(0); // r13
        frame.add(3).write(0); // r12
        frame.add(4).write(0); // rbx
        frame.add(5).write(0); // rbp
        frame.add(6).write(entry as usize as u64); // return address
    }
    rsp as u64
}

extern "C" fn task_one() -> ! {
    loop {
        crate::kprintln!("[task 1] running");
        wait_ticks(50);
    }
}

extern "C" fn task_two() -> ! {
    loop {
        crate::kprintln!("[task 2] running");
        wait_ticks(50);
    }
}

fn wait_ticks(count: u64) {
    let start = super::timer::ticks();
    while super::timer::ticks().wrapping_sub(start) < count {
        unsafe { core::arch::asm!("hlt", options(nomem, nostack, preserves_flags)) };
    }
}

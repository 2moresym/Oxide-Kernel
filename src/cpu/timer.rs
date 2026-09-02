use core::{arch::asm, sync::atomic::{AtomicU64, Ordering}};

const PIT_COMMAND: u16 = 0x43;
const PIT_CHANNEL0: u16 = 0x40;
const PIT_BASE_HZ: u32 = 1_193_182;
const FREQUENCY_HZ: u32 = 100;

static TICKS: AtomicU64 = AtomicU64::new(0);

/// Program PIT channel 0 for 100 interrupts per second.
pub fn init() {
    let divisor = (PIT_BASE_HZ / FREQUENCY_HZ) as u16;
    unsafe {
        outb(PIT_COMMAND, 0x36);
        outb(PIT_CHANNEL0, (divisor & 0xff) as u8);
        outb(PIT_CHANNEL0, (divisor >> 8) as u8);
    }
}

#[inline]
pub fn tick() -> u64 {
    TICKS.fetch_add(1, Ordering::Relaxed) + 1
}

#[inline]
pub fn ticks() -> u64 {
    TICKS.load(Ordering::Relaxed)
}

pub const fn frequency_hz() -> u32 { FREQUENCY_HZ }

unsafe fn outb(port: u16, value: u8) {
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
    }
}

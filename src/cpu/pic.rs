//! Minimal 8259A Programmable Interrupt Controller support for early x86 bring-up.

use core::arch::asm;

const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_COMMAND: u16 = 0xa0;
const PIC2_DATA: u16 = 0xa1;
const PIC_EOI: u8 = 0x20;
const ICW1_INIT: u8 = 0x10;
const ICW1_ICW4: u8 = 0x01;
const ICW4_8086: u8 = 0x01;

pub const PIC1_OFFSET: u8 = 32;
pub const PIC2_OFFSET: u8 = 40;

pub fn init() {
    unsafe {
        outb(PIC1_COMMAND, ICW1_INIT | ICW1_ICW4);
        io_wait();
        outb(PIC2_COMMAND, ICW1_INIT | ICW1_ICW4);
        io_wait();
        outb(PIC1_DATA, PIC1_OFFSET);
        io_wait();
        outb(PIC2_DATA, PIC2_OFFSET);
        io_wait();
        outb(PIC1_DATA, 4);
        io_wait();
        outb(PIC2_DATA, 2);
        io_wait();
        outb(PIC1_DATA, ICW4_8086);
        io_wait();
        outb(PIC2_DATA, ICW4_8086);
        io_wait();
        // Keep IRQ0 (PIT) enabled and mask all other legacy IRQ lines for now.
        outb(PIC1_DATA, 0xfe);
        outb(PIC2_DATA, 0xff);
    }
}

pub fn end_of_interrupt(irq: u8) {
    unsafe {
        if irq >= 8 {
            outb(PIC2_COMMAND, PIC_EOI);
        }
        outb(PIC1_COMMAND, PIC_EOI);
    }
}

unsafe fn io_wait() {
    unsafe {
        asm!("out 0x80, al", in("al") 0u8, options(nomem, nostack, preserves_flags));
    }
}

unsafe fn outb(port: u16, value: u8) {
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
    }
}

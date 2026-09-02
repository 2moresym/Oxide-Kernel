//! Minimal 16550 UART driver for QEMU's first serial port (COM1).

use core::fmt::{self, Write};

const COM1: u16 = 0x3f8;

pub fn init() {
    unsafe {
        outb(COM1 + 1, 0x00); // Disable interrupts.
        outb(COM1 + 3, 0x80); // Enable DLAB.
        outb(COM1, 0x03); // Divisor low byte: 38,400 baud.
        outb(COM1 + 1, 0x00); // Divisor high byte.
        outb(COM1 + 3, 0x03); // 8 bits, no parity, one stop bit.
        outb(COM1 + 2, 0xc7); // Enable FIFO and clear queues.
        outb(COM1 + 4, 0x0b); // IRQs enabled, RTS/DSR set.
    }
}

pub fn write_fmt(args: fmt::Arguments) {
    Serial.write_fmt(args).unwrap();
}

struct Serial;

impl Serial {
    fn write_byte(&mut self, byte: u8) {
        unsafe {
            while inb(COM1 + 5) & 0x20 == 0 {}
            outb(COM1, byte);
        }
    }
}

impl Write for Serial {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        for byte in text.bytes() {
            if byte == b'\n' {
                self.write_byte(b'\r');
            }
            self.write_byte(byte);
        }
        Ok(())
    }
}

unsafe fn outb(port: u16, value: u8) {
    unsafe { core::arch::asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags)) };
}

unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    unsafe { core::arch::asm!("in al, dx", out("al") value, in("dx") port, options(nomem, nostack, preserves_flags)) };
    value
}

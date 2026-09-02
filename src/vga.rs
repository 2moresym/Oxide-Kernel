//! VGA text-mode output for early boot diagnostics.

use core::fmt::{self, Write};

const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;
const VGA_BUFFER: *mut ScreenChar = 0xb8000 as *mut ScreenChar;

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Color {
    LightGray = 7,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ScreenChar {
    ascii: u8,
    color: u8,
}

pub fn clear() {
    let mut writer = Writer::new(Color::LightGray);
    writer.clear();
}

pub fn write_fmt(args: fmt::Arguments) {
    Writer::new(Color::LightGray).write_fmt(args).unwrap();
}

struct Writer {
    column: usize,
    row: usize,
    color: Color,
}

impl Writer {
    const fn new(color: Color) -> Self {
        Self { column: 0, row: 0, color }
    }

    fn clear(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row);
        }
        self.column = 0;
        self.row = 0;
    }

    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            0x20..=0x7e => {
                if self.column == BUFFER_WIDTH {
                    self.new_line();
                }
                write_at(self.row, self.column, byte, self.color);
                self.column += 1;
            }
            _ => self.write_byte(0xfe),
        }
    }

    fn new_line(&mut self) {
        if self.row < BUFFER_HEIGHT - 1 {
            self.row += 1;
        } else {
            for row in 1..BUFFER_HEIGHT {
                for column in 0..BUFFER_WIDTH {
                    let character = unsafe {
                        core::ptr::read_volatile(VGA_BUFFER.add(row * BUFFER_WIDTH + column))
                    };
                    unsafe {
                        core::ptr::write_volatile(
                            VGA_BUFFER.add((row - 1) * BUFFER_WIDTH + column),
                            character,
                        )
                    };
                }
            }
            self.clear_row(BUFFER_HEIGHT - 1);
        }
        self.column = 0;
    }

    fn clear_row(&self, row: usize) {
        for column in 0..BUFFER_WIDTH {
            write_at(row, column, b' ', self.color);
        }
    }
}

impl Write for Writer {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        for byte in text.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}

fn write_at(row: usize, column: usize, ascii: u8, color: Color) {
    unsafe {
        core::ptr::write_volatile(
            VGA_BUFFER.add(row * BUFFER_WIDTH + column),
            ScreenChar { ascii, color: color as u8 },
        );
    }
}

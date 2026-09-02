//! Minimal VGA text-mode output for early boot diagnostics.

const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;
const VGA_BUFFER: *mut ScreenChar = 0xb8000 as *mut ScreenChar;

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Color {
    LightGray = 7,
    LightGreen = 10,
    LightRed = 12,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ScreenChar {
    ascii: u8,
    color: u8,
}

pub fn clear() {
    for row in 0..BUFFER_HEIGHT {
        for column in 0..BUFFER_WIDTH {
            write_at(row, column, b' ', Color::LightGray);
        }
    }
}

pub fn write_line(text: &str, color: Color) {
    let row = if text.starts_with("Oxide") { 0 } else { 1 };
    for (column, byte) in text.bytes().take(BUFFER_WIDTH).enumerate() {
        write_at(row, column, byte, color);
    }
}

fn write_at(row: usize, column: usize, ascii: u8, color: Color) {
    // VGA text memory is a memory-mapped hardware buffer, so volatile writes
    // are required to ensure the compiler does not optimize them away.
    unsafe {
        core::ptr::write_volatile(
            VGA_BUFFER.add(row * BUFFER_WIDTH + column),
            ScreenChar { ascii, color: color as u8 },
        );
    }
}

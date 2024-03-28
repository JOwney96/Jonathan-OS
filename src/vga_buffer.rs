// Mod for writing to the screen using the VGA port 0xb8000

use core::fmt;

use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

// Create print macro by using built-in code but changing it to call our print function
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;

    x86_64::instructions::interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

// VGA Colors
#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

// struct where how to structure it is transparent
// Its transparent because it has a single u8 field
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    // Function for creating new color codes
    // Color codes first four bits is background and last 4 is foreground
    // Or the two together to get the two bytes as one
    fn new(background: Color, foreground: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | foreground as u8)
    }
}

// Screen chars which as a byte and a color code
// Use "C" style of representing the structure
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C)]
struct ScreenChar {
    ascii_char: u8,
    color: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

// Buffer is a 2D array with static size
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

// Writer struct which has a buffer and items needed for that buffer
pub struct Writer {
    column_pos: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    // Check the char or byte and then write to buffer
    pub fn write_byte(&mut self, char: u8) {
        match char {
            b'\n' => self.new_line(),
            char => {
                // Handle column overflow
                if self.column_pos >= BUFFER_WIDTH {
                    self.new_line();
                }

                let col = self.column_pos;
                let row = BUFFER_HEIGHT - 1;

                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_char: char,
                    color: self.color_code,
                });

                self.column_pos += 1;
            }
        }
    }
}

impl Writer {
    // Move everything up one line deleting the top line
    // Delete the bottom line when done
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }

        self.clear_line(BUFFER_HEIGHT - 1);
        self.column_pos = 0;
    }
}

impl Writer {
    // Move across a line and replace each char with a blank or "null" char.
    fn clear_line(&mut self, row: usize) {
        let null_char = ScreenChar {
            ascii_char: b' ',
            color: ColorCode::new(Color::Black, Color::Black),
        };

        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(null_char);
        }
    }
}

impl Writer {
    // Turn the string into bytes and call write_byte foreach
    pub fn write_string(&mut self, str: &str) {
        for char in str.bytes() {
            match char {
                0x20..=0x7e | b'\n' => self.write_byte(char),
                _ => self.write_byte(0xfe),
            }
        }
    }
}

impl fmt::Write for Writer {
    // fmt's write_str function
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

// Raw pointers cannot be determined at compile time.
// Lazy statics don't initialize until the first use which is at runtime.
// Our OS doesn't have threads, but we need "thread safety".
// We are using a spin mutex which means a thread spins or loops and keeps asking to lock until it can lock.
lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_pos: 0,
        color_code: ColorCode::new(Color::Black, Color::Green),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

//  ---Tests---
#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output")
    }
}

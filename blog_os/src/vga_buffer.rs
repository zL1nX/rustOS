use volatile::Volatile;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;


#[cfg(test)]
use crate::{serial_print, serial_println};

// VGA字符缓冲区是一个25行 x 80列的二维数组, 每个元素一个16bit长
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[allow(dead_code)] // 每个未使用的变量发出警告
#[derive(Debug, Clone, Copy, PartialEq, Eq)] // derive these traits
#[repr(u8)] // 标注枚举的类型是u8
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
// Color type is an atomic type

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)] // make sure the layout of ColorCode is the same as u8 (i.e. a byte), 只适用于single field struct
struct ColorCode(u8);
// ColorCode, based on Color, is the functional type of displaying colors
impl ColorCode {
    // 结构体的关联函数
    fn new(foreground : Color, background : Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    } // 前4位是背景色, 后四位是前景色
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)] // 按照C语言约定的顺序布局结构体成员变量
struct ScreenChar{
    ascii_code : u8,
    color_code : ColorCode
}

#[repr(transparent)]
struct Buffer {
    chars : [[Volatile<ScreenChar> ; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/*
目前已经有了VGA缓冲区这一层中间结构了, 下面我们希望实现该如何"从缓冲区XXXXX
*/

pub struct Writer {
    column_pos : usize, // 光标在屏幕最后一行的位置 (当前输入下一个字符的位置)
    color_code : ColorCode,
    buffer : &'static mut Buffer // static全局生命周期
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer { // static ref ,只需引用即可, 否则会发生所有权转移
        column_pos : 0,
        color_code : ColorCode::new(Color::Yellow, Color::Black),
        buffer : unsafe { &mut *(0xb8000 as *mut Buffer) }, // 先将b8000转换成Buffer指针, 然后再转换成可变引用&mut
    });
}
// 用Mutex互斥锁来实现多线程Sync条件下的内部可变性, 从而能在全局项目中使用Writer写入数据


impl Writer {
    pub fn write_string(&mut self, s : &str) {
        for b in s.bytes() {
            match b {
                0x20..=0x7e | b'\n' => self.write_byte(b), // 只考虑可打印字符
                _ => self.write_byte(0xfe) // ■ VGA硬件编码中的一个字符来指示那些不可打印字符
            }
        }
    }

    pub fn write_byte(&mut self, byte : u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => { // 设置字符, 目前仅写, 但还没有print
                if self.column_pos >= BUFFER_WIDTH {
                    self.new_line(); // 通过newline来为最后一行腾出空来
                }

                let row: usize = BUFFER_HEIGHT - 1; // 从最后一行写, 最终输出是在终端的最下面显示
                let col: usize = self.column_pos;
                let wb = ScreenChar {
                    ascii_code: byte,
                    color_code: self.color_code
                };
                self.buffer.chars[row][col].write(wb);

                self.column_pos += 1;
            }
        }
    }

    fn new_line(&mut self) {
        /*
        通过n^2的循环把整个buffer内的字符向上平移一行
        */
        for r in 1..BUFFER_HEIGHT {
            for c in 0..BUFFER_WIDTH {
                let ch = self.buffer.chars[r][c].read(); // why .read() ?
                self.buffer.chars[r - 1][c].write(ch);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_pos = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_code : b' ',
            color_code : self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

// 实现这个Write的trait以支持格式化输出各种类型的变量
impl fmt::Write for Writer {
    fn write_str(&mut self, s : &str)-> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

// 实现该trait之后, 就可以使用write!与writeln!等格式化宏实现各种复杂格式的输出了

//定义全局WRITER后, 就无需再vga_buffer中定义一个测试函数供外部使用了

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

// 和标准的print!宏与println!宏实现基本相同, 但使用了$crate来让println可以单独调用print

#[doc(hidden)] // _print是私有自定义函数, 但迫于宏导出得设置为pub, 因此干脆只让文档隐藏算了
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    })
    // 用without_interrupts来确保当Mutex是锁着的时候, 是不会有interrupt发生的 (从而防止发生死锁)
    
}

// 自定义的_print函数调用write_fmt这个trait传入参数

#[test_case]
pub fn test_vga_simple() {
    serial_println!("test_println_simple... ");
    println!("test_println_simple output from VGA");
    serial_println!("[ok]");
}

// 如果没有panic, 那控制台可以直接看到 ok

#[test_case]
pub fn test_vga_multiple_line() {
    serial_println!("test_println_multiple_line... ");
    for _ in 0..200 {
        println!("test_println_simple output from VGA");
    }
    serial_println!("[ok]");
}

#[test_case]
pub fn test_vga_checkch() {
    serial_println!("test_println_check_char... ");

    let s :&str = "This is a test string!";
    println!("{}", s); // now print it to the vga buffer with a \n
    for (i, c) in s.chars().enumerate() {
        let output_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
        assert_eq!(char::from(output_char.ascii_code), c);
    }

    serial_println!("[ok]");
}
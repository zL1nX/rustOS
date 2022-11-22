use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;

lazy_static!{
    pub static ref SERIAL1 : Mutex<SerialPort> = {
        let mut serial_port = unsafe {
            SerialPort::new(0x3F8)
        };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

// 类似于VGA缓冲区中的全局WRITER变量, 声明一个全局的静态引用, 这样只需要被init一次即可各处使用

// 传递的端口地址为0x3F8 ，该地址是第一个串行接口的标准端口号


// 添加一些宏让mod更加易用

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        SERIAL1.lock().write_fmt(args).expect("Printing to serial failed");
    });
    // 与vga buffer里的同理
    
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}
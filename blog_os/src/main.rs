#![no_std] // a stand alone OS Core should not use any std lib
#![no_main] // 

use core::panic::PanicInfo;

static HELLO :&[u8] = b"Hello World!";

// 由于去掉了no_std环境, 导致我们也不能用默认的Rust Runtime crt0来让程序初始化
// 因此我们需要重新定义一个自己的入口点
#[no_mangle] // name mangling, 这一选项防止编译器对函数签名进行重整, 导致链接器无法识别这一自定义入口
pub extern "C" fn _start()-> !{
    let vga_buffer: *mut u8 = 0xb8000 as *mut u8; // 
    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            // 
            *vga_buffer.offset(i as isize * 2) = byte; // 
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
        }
    }
    loop {}
}


/*
在no std环境中, 标准库中的panic handler函数将无法被使用
因此我们需要一个自己的panic handler函数
此处因为还没有任何处理逻辑, 因此只写个loop来满足编译器检查
*/

#[panic_handler]
fn panic(_info: &PanicInfo)->! {
    // PanicInfo includes panic_file, panic_lineno, available_error_message
    loop{} // for now
}


// fn main() {
//     //println!("Hello, world!");
// }

#![no_std] // a stand alone OS Core should not use any std lib
#![no_main] // 
#![feature(custom_test_frameworks)] // 替换原有的内置依赖标准库的test框架
#![test_runner(blog_os::test_runner)] // 复用库中的runner函数
#![reexport_test_harness_main = "test_main"] // 指定自定义测试框架生成的入口函数名称 (如果不指定, 即为main)

use core::panic::PanicInfo;
use blog_os::{println, serial_println, serial_print};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop{}
}


#[panic_handler]
fn panic(info :&PanicInfo)->! {
    blog_os::test_panic_handler(info)
}

#[test_case]
fn test_println() {
    serial_print!("test_println...in basic_boot");
    println!("test_println output");
    serial_println!("[ok]");
}

// 集成测试都是单独的可执行文件, 需要重新来一遍最小可执行文件的配置

// 在basic boot环境中也可以测试println这些函数的功能是否正常，而且是在初始化start之后的，这保证了函数在boot之后功能依然正常
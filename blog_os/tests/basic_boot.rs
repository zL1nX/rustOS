#![no_std] // a stand alone OS Core should not use any std lib
#![no_main] // 
#![feature(custom_test_frameworks)] // 替换原有的内置依赖标准库的test框架
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"] // 指定自定义测试框架生成的入口函数名称 (如果不指定, 即为main)

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop{}
}


#[panic_handler]
fn panic(info :&PanicInfo)->! {
    loop{}
}

// 集成测试都是单独的可执行文件, 需要重新来一遍最小可执行文件的配置
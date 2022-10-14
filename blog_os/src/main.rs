#![no_std] // a stand alone OS Core should not use any std lib
#![no_main] // 
#![feature(custom_test_frameworks)] // 替换原有的内置依赖标准库的test框架
#![test_runner(blog_os::test_runner)] // 改成了这个包的test runner
#![reexport_test_harness_main = "test_main"] // 指定自定义测试框架生成的入口函数名称 (如果不指定, 即为main)


use core::panic::PanicInfo;
use blog_os::println;


// 由于去掉了no_std环境, 导致我们也不能用默认的Rust Runtime crt0来让程序初始化
// 因此我们需要重新定义一个自己的入口点
#[no_mangle] // name mangling, 这一选项防止编译器对函数签名进行重整, 导致链接器无法识别这一自定义入口
pub extern "C" fn _start()-> !{
    println!("Hello world from println {}", "!"); // 可正常使用println宏
    
    blog_os::init();

    x86_64::instructions::interrupts::int3();

    #[cfg(test)]
    test_main(); // 调用入口函数
    loop {}
}  


/*
在no std环境中, 标准库中的panic handler函数将无法被使用
因此我们需要一个自己的panic handler函数
此处因为还没有任何处理逻辑, 因此只写个loop来满足编译器检查
*/

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo)->! {
    // PanicInfo includes panic_file, panic_lineno, available_error_message
    println!("{}", _info); // we can print some info (no actual content)
    loop{} // for now
}

#[cfg(test)]
#[panic_handler]
fn panic(_info: &PanicInfo)->! {
    blog_os::test_panic_handler(_info)
}

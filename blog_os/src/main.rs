#![no_std] // a stand alone OS Core should not use any std lib
#![no_main] // 
#![feature(custom_test_frameworks)] // 替换原有的内置依赖标准库的test框架
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"] // 指定自定义测试框架生成的入口函数名称 (如果不指定, 即为main)


use core::panic::PanicInfo;

mod vga_buffer;
mod serial;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Fail = 0x11, // 自定义两个状态码 
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len()); // 通过串口打印到宿主机系统的终端
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    exit_qemu(QemuExitCode::Success);
}
// 该函数会被测试框架的入口函数调用, 而它会收集所有被标注了[test_case]的函数加到runner里调用执行
// tests是个slice引用, slice中的元素是Fn()这个trait的一个特征对象, 从而实现了通过test进行函数调用的技巧



pub fn exit_qemu(exit_code : QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4); // 声明一个新IO端口来和内核沟通
        port.write(exit_code as u32);
    }
}

// 由于去掉了no_std环境, 导致我们也不能用默认的Rust Runtime crt0来让程序初始化
// 因此我们需要重新定义一个自己的入口点
#[no_mangle] // name mangling, 这一选项防止编译器对函数签名进行重整, 导致链接器无法识别这一自定义入口
pub extern "C" fn _start()-> !{
    println!("Hello world from println {}", "!"); // 可正常使用println宏
    
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
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", _info);
    exit_qemu(QemuExitCode::Fail);
    loop {}
}

#[test_case]
fn trivial_test() {
    // print!("This is a trivial test");
    serial_print!("trivial assertion... ");
    assert_eq!(1, 0);
    //print!("ok!");
    serial_println!("[ok]");
}
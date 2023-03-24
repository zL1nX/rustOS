#![no_std] // a stand alone OS Core should not use any std lib
#![no_main] // 
#![feature(custom_test_frameworks)] // 替换原有的内置依赖标准库的test框架
#![test_runner(blog_os::test_runner)] // 改成了这个包的test runner
#![reexport_test_harness_main = "test_main"] // 指定自定义测试框架生成的入口函数名称 (如果不指定, 即为main)

extern crate alloc; // 在main中需要重新声明, 因为彼此都是独立的crate

use core::panic::PanicInfo;
use alloc::{boxed::Box, vec::Vec, vec, rc::Rc};
use blog_os::{println, memory::{self, BootInfoFrameAllocator}, allocator};
use bootloader::{BootInfo, entry_point};
use x86_64::VirtAddr;

entry_point!(kernel_main); // 重新用entry point来规范我们的入口点函数签名, 让其能正确的被编译器识别为入口点函数

// 所以也就无需原来的extern C, no mangle等宏了
fn kernel_main(boot_info : &'static BootInfo)-> !{
    println!("Hello world from println {}", "!"); // 可正常使用println宏
    blog_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");
    
    let x = Box::new(41);
    println!("Heap value at: {:p}", x);

    let mut v = Vec::new();
    for i in 0..500 {
        v.push(i);
    }
    println!("Vector on Heap: {:p}", &v);

    let ref_count = Rc::new(vec![1, 2, 3]);
    let cloned_ref = ref_count.clone();
    println!("Current count of vec ref: {}", Rc::strong_count(&ref_count));
    core::mem::drop(cloned_ref);
    println!("Current count of vec ref: {} (after drop)", Rc::strong_count(&ref_count));

    #[cfg(test)]
    test_main(); // 调用入口函数

    println!("It did not crash!");
    blog_os::hlt_loop(); // CPU不用一直无限循环了
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
    blog_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(_info: &PanicInfo)->! {
    blog_os::test_panic_handler(_info)
}

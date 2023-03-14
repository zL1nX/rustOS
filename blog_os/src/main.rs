#![no_std] // a stand alone OS Core should not use any std lib
#![no_main] // 
#![feature(custom_test_frameworks)] // 替换原有的内置依赖标准库的test框架
#![test_runner(blog_os::test_runner)] // 改成了这个包的test runner
#![reexport_test_harness_main = "test_main"] // 指定自定义测试框架生成的入口函数名称 (如果不指定, 即为main)


use core::panic::PanicInfo;
use blog_os::{println, memory::{self, BootInfoFrameAllocator}};
use bootloader::{BootInfo, entry_point};
use x86_64::{VirtAddr, structures::paging::Page};

entry_point!(kernel_main); // 重新用entry point来规范我们的入口点函数签名, 让其能正确的被编译器识别为入口点函数

// 所以也就无需原来的extern C, no mangle等宏了
fn kernel_main(boot_info : &'static BootInfo)-> !{
    println!("Hello world from println {}", "!"); // 可正常使用println宏
    blog_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_alloc = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    let page = Page::containing_address(VirtAddr::new(0));
    memory::create_example_mapping(page, &mut mapper, &mut frame_alloc);

    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e)}; // New! 对应的码值

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

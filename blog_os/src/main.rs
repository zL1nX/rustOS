#![no_std] // a stand alone OS Core should not use any std lib
#![no_main] // 
#![feature(custom_test_frameworks)] // 替换原有的内置依赖标准库的test框架
#![test_runner(blog_os::test_runner)] // 改成了这个包的test runner
#![reexport_test_harness_main = "test_main"] // 指定自定义测试框架生成的入口函数名称 (如果不指定, 即为main)


use core::panic::PanicInfo;
use blog_os::{println, memory::active_level4_page_table};
use bootloader::{BootInfo, entry_point};
use x86_64::{VirtAddr, structures::paging::PageTable};

entry_point!(kernel_main); // 重新用entry point来规范我们的入口点函数签名, 让其能正确的被编译器识别为入口点函数

// 所以也就无需原来的extern C, no mangle等宏了
fn kernel_main(boot_info : &'static BootInfo)-> !{
    println!("Hello world from println {}", "!"); // 可正常使用println宏
    blog_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let l4_table = unsafe { active_level4_page_table(phys_mem_offset) };
    for (i, entry) in l4_table.iter().enumerate() {
        if !entry.is_unused() {
            // println!("L4 Entry {}: {:?}", i, entry); // 直接读取L4页表的地址, 权限等信息

            // 也可以根据每个L4的地址信息, 找更下一层的L3的页表地址信息, 整体的寻址方式与直接找L4是相同的
            let phys = entry.frame().unwrap().start_address();
            let virt = phys.as_u64() + boot_info.physical_memory_offset;
            let ptr = VirtAddr::new(virt).as_mut_ptr();
            let l3_table: &PageTable = unsafe { &*ptr};

            for (j, l3_entry) in l3_table.iter().enumerate() {
                if !l3_entry.is_unused() {
                    println!("  L3 Entry {}: {:?}", j, entry);
                }
            }
        }
    }

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

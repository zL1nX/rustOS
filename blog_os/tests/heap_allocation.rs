#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use alloc::{boxed::Box, vec::Vec};
use blog_os::{memory::BootInfoFrameAllocator, allocator::HEAP_SIZE};
use bootloader::{entry_point, BootInfo};

extern crate alloc;

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    use x86_64::VirtAddr;
    use blog_os::{memory, allocator};

    blog_os::init();

    let phy_mem_addr = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe {memory::init(phy_mem_addr)};
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init( &boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    test_main();
    loop {}

}
// 与main类似, 但是是给本测试文件作主函数

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info);
}

#[test_case]
fn box_allocation() {
    let v1 = Box::new(100);
    let v2 = Box::new(-3);
    assert_eq!(*v1, 100);
    assert_eq!(*v2, -3);
}


#[test_case]
fn vec_allocation() {
    let mut v = Vec::new();
    let n = 1000;
    for i in 0..n {
        v.push(i);
    }
    assert_eq!(v.iter().sum::<u64>(), (n - 1) * n / 2); // 测试vec的元素和
}

#[test_case]
fn realloc_box() {
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}

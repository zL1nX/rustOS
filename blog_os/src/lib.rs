// 从main.rs中分割出一个库，库中的函数可以被其他crate复用

#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]

extern crate alloc;

use core::panic::PanicInfo;

#[cfg(test)]
use bootloader::{BootInfo, entry_point};

pub mod serial;
pub mod vga_buffer;
pub mod interrupt;
pub mod gdt;
pub mod memory;
pub mod allocator;
pub mod task;

// 设置为公有mod，这样一来我们在库的外部也一样能使用它们了，此时main中的这些函数就可以删掉了

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub trait Testable {
    fn run(&self) ->();
}

impl<T> Testable for T where T: Fn() { // 泛型
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}


pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

#[alloc_error_handler]
pub fn alloc_error_handler(layout: alloc::alloc::Layout)->! {
    panic!("allocation error: {:?}", layout) // 现阶段什么都干不了, 直接panic
} 

#[cfg(test)]
entry_point!(test_kernel_main);

/// Entry point for `cargo xtest`
#[cfg(test)]
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    init();
    test_main();
    hlt_loop();
}
// 像main一样先修改入口点函数

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

pub fn init() {
    gdt::init(); // 将gdt load进来
    interrupt::init_idt();
    unsafe {
        interrupt::PICS.lock().initialize();
    }
    x86_64::instructions::interrupts::enable(); // set external interrupts
}

// 再封装一层init到lib中, 这样一来这个函数就可以被复用了

pub fn hlt_loop()->! {
    loop {
        x86_64::instructions::hlt(); // 通过instruction提供的api封装来防止CPU进行无限循环
        //其实就是halt命令
    }
}
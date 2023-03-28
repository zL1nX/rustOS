#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use lazy_static::lazy_static;
use core::panic::PanicInfo;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use blog_os::{serial_println, exit_qemu, QemuExitCode};



lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = { // 这里要区分下name, 因为test是一个单独的可执行文件, 需要另外一个IDT
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault.set_handler_fn(test_double_fault_handler).set_stack_index(blog_os::gdt::DOUBLE_FAULT_IST_INDEX);
            // 设置double fault 的handler, 并且指定stack的index
        }
        idt
    };
}

#[no_mangle]
pub extern "C" fn _start() ->! {
    serial_println!("stack_overflow::stack_overflow...");

    blog_os::gdt::init(); // 没用lib里的封装好的init
    init_idt_test();

    stack_overflow();

    panic!("Execution continued after stack overflow");
}

#[panic_handler]
fn panic(info: &PanicInfo)->! {
    blog_os::test_panic_handler(info)
}

#[allow(unconditional_recursion)] // 让编译器放松
fn stack_overflow() {
    stack_overflow();
    volatile::Volatile::new(0).read(); // dummy code 防止编译器把infinite recurse给优化掉, 这样触发不了异常了
}

fn init_idt_test() {
    TEST_IDT.load(); // 这个单独的IDT被load了进来
}

extern "x86-interrupt" fn test_double_fault_handler(_stack_frame: InterruptStackFrame, _error_code : u64) -> !{
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}

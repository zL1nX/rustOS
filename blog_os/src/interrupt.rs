
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::println;
use lazy_static::lazy_static;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt
    };
}

// IDT需要具有全局生命周期, 因此使用lazy static和静态引用

pub fn init_idt() {
    IDT.load(); // 载入IDT 从而能让CPU在遇到异常时读取到这个表
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}
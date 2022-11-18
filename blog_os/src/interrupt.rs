
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::println;
use crate::gdt;
use pic8259::ChainedPics;
use spin;
use lazy_static::lazy_static;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8; // 给第一层那8个interrupt要用

pub static PICS: spin::Mutex<ChainedPics> = spin::Mutex::new(unsafe {ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET)});
// 自己重新定义个controller的offset, 用内置的chainedpics结构就能实现

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler).set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
            // 设置double fault 的handler, 并且指定stack的index
        }
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

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _error_code : u64) -> !{
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::print;
use crate::println;
use crate::gdt;
use pic8259::ChainedPics;
use spin;
use lazy_static::lazy_static;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8; // 给第一层那8个interrupt要用

pub static PICS: spin::Mutex<ChainedPics> = spin::Mutex::new(unsafe {ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET)});
// 自己重新定义个controller的offset, 用内置的chainedpics结构就能实现

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET, // 第一个中断, 所以没offset
    Keyboard, // 自然会PIC_1_OFFSET + 1, 故省去
}

impl InterruptIndex {
    fn as_u8(self)->u8 {
        self as u8
    }

    fn as_usize(self)->usize {
        usize::from(self.as_u8())
    }
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler).set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
            // 设置double fault 的handler, 并且指定stack的index
        }
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler); // 给timer加上handler
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
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

extern "x86-interrupt" fn timer_interrupt_handler(stack_frame: InterruptStackFrame) {
    print!(".");
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8()); // 显式地告知PIC我们的handler function 已经结束了
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(stack_frame: InterruptStackFrame) {
    
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1}; // 现成的crate, 太赖了
    use spin::Mutex;
    use x86_64::instructions::port::Port;

    lazy_static!{
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = 
        Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore));
    }

    let mut p = Port::new(0x60); // PS/2键盘controller的data port
    let mut keyboard = KEYBOARD.lock();

    let scancode: u8 = unsafe {
        p.read()
    };

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(raw_key) => print!("{:?}", raw_key)
            }
        }
    }

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}
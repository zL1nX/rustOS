

use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use lazy_static::lazy_static;
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector}; // 用来切换segment, 从而让CPU能正确识别

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0; // 将double fault对应的栈作为IST的第一个

lazy_static!{
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK : [u8; STACK_SIZE] = [0; STACK_SIZE]; // 还没有动态内存分配, 只能先用static mut了
            let stack_start = VirtAddr::from_ptr(unsafe{&STACK});
            let stack_end = stack_start + STACK_SIZE;
            stack_end // 开辟出整个栈
        };
        tss
    };
}


struct Selectors {
    code_seg_sel : SegmentSelector,
    tss_seg_sel : SegmentSelector
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();

        let code_seg_sel = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_seg_sel = gdt.add_entry(Descriptor::tss_segment(&TSS)); // 添加之前声明好的TSS
        // 将接口暴露出来, 以便外部指定
        (gdt, Selectors{code_seg_sel, tss_seg_sel})
    };
}

pub fn init() {
    use x86_64::instructions::segmentation::{CS, Segment};
    use x86_64::instructions::tables::load_tss;

    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.code_seg_sel); // 指定code segment, 以及tss的地址, 
        load_tss(GDT.1.tss_seg_sel); // 只有这样CPU才能知道TSS在哪, 才能读出IST
    }
}

use x86_64::{structures::paging::PageTable, VirtAddr, registers::control::Cr3};

pub unsafe fn active_level4_page_table(physical_memory_offset: VirtAddr)->&'static mut PageTable
{
    let (level_4_table_frames, _) = Cr3::read();
    let phys = level_4_table_frames.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
    return &mut *page_table_ptr; // 先解引用成PageTable, 再返回可变mut引用
}
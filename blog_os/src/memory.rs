use x86_64::{structures::paging::{PageTable, frame, page_table::FrameError}, VirtAddr, registers::control::Cr3, PhysAddr};

pub unsafe fn active_level4_page_table(physical_memory_offset: VirtAddr)->&'static mut PageTable
{
    let (level_4_table_frames, _) = Cr3::read();
    let phys = level_4_table_frames.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
    return &mut *page_table_ptr; // 先解引用成PageTable, 再返回可变mut引用
}

pub unsafe fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr)-> Option<PhysAddr>
{
    translate_addr_inner(addr, physical_memory_offset) // 限制unsafe的区域
}

fn translate_addr_inner(addr: VirtAddr, physical_memory_offset: VirtAddr)-> Option<PhysAddr> // 给定一个虚拟地址addr, 翻译成对应的物理地址
{
    let (level4_table_frame, _) = Cr3::read();

    let table_indexes = [addr.p4_index(), addr.p3_index(), addr.p2_index(), addr.p1_index()]; // 这里的方法就是根据addr这个给定的地址, 通过位运算等方法获取特定比特范围的地址, 即为这个addr地址对应的哪一级页表的地址
    let mut frame = level4_table_frame;

    // 与之前的方法一样, 同样是先获取到物理地址偏移与l4的索引地址
    
    // 然后依次遍历, index为每一级页表的索引, 长度不同
    for &index in &table_indexes {
        let virt = physical_memory_offset + frame.start_address().as_u64();
        let table_ptr: *const PageTable = virt.as_ptr();
        let table = unsafe { &*table_ptr };

        let entry = &table[index]; // 根据索引得到对应页表的地址内容
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("huge pages not supported"),

        };
    }
    // 最终遍历完成后到达最底层的地址, 也就是真实的物理页地址, 然后加上虚拟地址的页内偏移来获得有效的数据位置
    Some(frame.start_address() + u64::from(addr.page_offset()))

}


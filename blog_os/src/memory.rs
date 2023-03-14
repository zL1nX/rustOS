use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{structures::paging::{PageTable, page_table::FrameError, OffsetPageTable, PhysFrame, FrameAllocator, Size4KiB, Page, Mapper}, VirtAddr, registers::control::Cr3, PhysAddr};
use x86_64::structures::paging::PageTableFlags as Flags;

pub struct EmptyFrameAllocator;
pub struct  BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map: memory_map,
            next: 0,
        }
    }

    fn usable_frames(&self)->impl Iterator<Item = PhysFrame> { // 使用impl让函数实现起来更容易
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        // 找到可用的内存地址并将其地址提取出来
        let regions_addr = usable_regions.map(|f| f.range.start_addr()..f.range.end_addr());
        let frame_addr = regions_addr.flat_map(|f| f.step_by(4096));

        frame_addr.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        None // placeholder
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.usable_frames().nth(self.next + 1); // 直接取页出来
        self.next += 1;
        frame
    }
}

pub unsafe fn init(physical_memory_offset: VirtAddr)->OffsetPageTable<'static>
{
    let level_4_table_frames = active_level4_page_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table_frames, physical_memory_offset)
}

unsafe fn active_level4_page_table(physical_memory_offset: VirtAddr)->&'static mut PageTable
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

pub fn create_example_mapping(page: Page, mapper: &mut OffsetPageTable, frame_allocator: &mut impl FrameAllocator<Size4KiB>)
{
    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe {
        mapper.map_to(page, frame, flags, frame_allocator) // 通过map to 函数来将一个虚拟页trivial地映射到给定的frame上
    };
    map_to_result.expect("map_to failed").flush(); //flush方法来将创建的页从TLB中flush出来确保其在#[must_used]属性下一定会被用到
}

// 目前的FrameAllocator只是一个dummy allocator
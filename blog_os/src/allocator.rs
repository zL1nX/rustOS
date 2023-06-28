use core::{alloc::{GlobalAlloc, Layout}, ptr::null_mut};
use x86_64::{structures::paging::{Mapper, Size4KiB, FrameAllocator, mapper::MapToError, Page, PageTableFlags}, VirtAddr};

//use self::bump::BumpAllocator;
use self::linked_list::LinkedListAllocator;

pub mod bump;
pub mod linked_list;

pub const HEAP_START : usize = 0x_4444_4444_0000;
pub const HEAP_SIZE : usize = 100 * 1024; // 4KB

pub struct Dummy;

unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut() // 返回一个空指针来骗过编译器
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("dealloc should be never called") // 正因为什么内存都不会被alloc, 所以dealloc应该从不被调用
    }
}
// 一个最基本的DummyAllocator, 单纯的把接口给实现以下

pub struct Locked<A> {
    inner: spin::Mutex<A>
}

impl<A> Locked<A> {
    pub const fn new(inner: A) ->Self {
        Locked {
            inner: spin::Mutex::new(inner)
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}


#[global_allocator]
static ALLOCATOR: Locked<LinkedListAllocator> = Locked::new(LinkedListAllocator::new()); // 直接替换原来的Dummy为自己实现的BumpAllocator

pub fn init_heap(mapper: &mut impl Mapper<Size4KiB>, allocator: &mut impl FrameAllocator<Size4KiB>)
->Result<(), MapToError<Size4KiB>>
{
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page: Page<Size4KiB> = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = allocator.allocate_frame().ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper.map_to(page, frame, flags, allocator)?.flush();
        }
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE); // 用lock来进行堆Allocator的初始化
    }
    
    Ok(())
    // 通过Rust中的问号机制来提前将错误返回
}
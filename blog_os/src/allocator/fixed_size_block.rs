use alloc::alloc::Layout;
use core::ptr;

const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];


struct ListNode {
    next : Option<&'static mut ListNode> // 因为是固定大小, 所以不需要size字段
}

pub struct FixedSizeBlockAllocator {
    lists_heads: [Option<&'static mut ListNode>; BLOCK_SIZES.len()],
    fallback_allocator: linked_list_allocator::Heap // 用人家默认的, 对效率比较友好
}

// new and init

impl FixedSizeBlockAllocator {
    pub const fn new() -> Self {
        const EMPTY: Option<&'static mut ListNode> = None;
        FixedSizeBlockAllocator { 
            lists_heads: [EMPTY; BLOCK_SIZES.len()], 
            fallback_allocator: linked_list_allocator::Heap::empty() 
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size:usize) {
        self.fallback_allocator.init(heap_start, heap_size);
    }
}


// 自己的alloc,直接用别人的实现

impl FixedSizeBlockAllocator {
    fn fallback_alloc(&mut self, layout: Layout) -> *mut u8 {
        match self.fallback_allocator.allocate_first_fit(layout) {
            Ok(ptr) => ptr.as_ptr(),
            Err(_) => ptr::null_mut()
        }
    }

    fn list_index(layout: &Layout) -> Option<usize> { // 根据给定的layout大小找到应该用哪个尺寸的block
        let requested_size = layout.size().max(layout.align());
        BLOCK_SIZES.iter().position(|&s| s >= requested_size)
    }
}
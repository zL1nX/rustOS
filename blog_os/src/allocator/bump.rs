use core::alloc::{GlobalAlloc, Layout};


pub struct BumpAllocator {
    heap_start : usize, // 上下界
    heap_end: usize,
    next: usize,
    allocations: usize,  // 还有多少空间能用
}

impl BumpAllocator {
    pub const fn new()-> Self {
        BumpAllocator { 
            heap_start: 0, 
            heap_end: 0, 
            next: 0, 
            allocations: 0 
        }
    }

    pub unsafe fn init(&mut self, heap_start : usize, heap_end: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_end;
        self.next = heap_start;  // 指向第一个没被用的空间位置
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 { // 这里的self不能是 mut 的, 因为alloc不支持这样做, 需要魔改
        let alloc_start = self.next;
        self.next = alloc_start + layout.size();
        self.allocations += 1;
        alloc_start as *mut u8
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout){
        todo!()
    }
}
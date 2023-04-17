use core::{alloc::{GlobalAlloc, Layout}, ptr};

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

// 用泛型来实现自己的spin Mutex方法, 从而用于当前这个crate中, 相当于在spin外面包一层

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

unsafe impl GlobalAlloc for Locked<BumpAllocator> { // 用自己定义的方法
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 { // 这里的self不能是 mut 的, 因为alloc不支持这样做, 需要魔改
        let mut bump = self.lock();

        let alloc_start = align_up(bump.next, layout.align());

        let alloc_end = match alloc_start.checked_add(layout.size()) { // 防止溢出
            Some(end) => end,
            None => return ptr::null_mut()
        };

        if alloc_end > bump.heap_end {
            return ptr::null_mut()
        }else {
            bump.next = alloc_end;
            bump.allocations += 1;
            alloc_start as *mut u8
        }
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout){
        let mut bump = self.lock();

        bump.allocations -= 1;
        if bump.allocations == 0 {
            bump.next = bump.heap_start;
        }
    }
}

fn align_up(addr: usize, align: usize) -> usize {
    let remainder = addr % align;

    if remainder == 0 {
        addr
    }else {
        addr - remainder + align // + align 保证新的地址不比原地址小
    }
}

// 追求高效实现可以这么写

// fn align_up(addr: usize, align: usize) -> usize {
//     (addr + align - 1) & !(align - 1)
// }

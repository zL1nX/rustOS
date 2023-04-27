use alloc::boxed::Box;

struct ListNode {
    size: usize,
    next : Option<&'static mut ListNode> ,
}

pub struct LinkedListAllocator {
    head: ListNode,
}

impl ListNode {
    const fn new(size: usize)->Self {
        ListNode { size , next: None }
    }

    fn start_addr(&self)->usize {
        self as *const Self as usize
    }

    fn end_addr(&self)->usize{
        self.start_addr() + self.size
    }
}

impl LinkedListAllocator {
    const fn new() -> Self {
        Self { head: ListNode::new(0) }
    }

    // 将空闲的内存记录下来, 会是dealloc的主要函数, 因为把用了的free之后, 就得赶紧push到这个list维护
    unsafe fn add_free_region(&mut self, addr: usize, size : usize) {
        todo!()
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.add_free_region(heap_start, heap_size)
    }
    // 只会被调用一次, 因为外部需要保证heap的内存是valid的, 所以这个函数是unsafe 的
}
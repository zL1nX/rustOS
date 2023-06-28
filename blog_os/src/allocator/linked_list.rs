use core::{mem, alloc::{GlobalAlloc, Layout}, ptr};
use super::{align_up, Locked};

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
    fn alloc_from_region(region: &ListNode, size: usize, align: usize) -> Result<usize, ()> 
    {
        let alloc_start = align_up(region.start_addr(), align);
        let alloc_end = alloc_start.checked_add(size).ok_or(())?;

        // 当前内存node太小, 无法满足align的需求
        if alloc_end > region.end_addr() {
            return Err(());
        }
        let execess_size = region.end_addr() - alloc_end;

        // 当前node虽然够了, 但是剩下的空间太小, 导致后面没法变回listnode存储起来
        if execess_size > 0 && execess_size < mem::size_of::<ListNode>() {
            return Err(());
        }

        // 所以要么当前region恰好满足alloc的需求, 要么这块region分配完之后, 剩下的空间还能用来承载它自己的listnode
        Ok(alloc_start)
    }

    // 保证每个分配出去的block都能容纳一个listnode, 这样后续被free掉之后, 就能加到链表里
    fn size_align(layout : Layout)-> (usize, usize) {
        let layout = layout.align_to(mem::size_of::<ListNode>()).expect("alignment failed").pad_to_align(); // 保证至少是整数倍的地址开始
        let size = layout.size().max(mem::size_of::<ListNode>()); // 保证大小至少为一个listnode的大小
        (size, layout.align())
    }
}

impl LinkedListAllocator {
    pub const fn new() -> Self {
        Self { head: ListNode::new(0) }
    }

    // 将空闲的内存记录下来, 会是dealloc的主要函数, 因为把用了的free之后, 就得赶紧push到这个list维护
    unsafe fn add_free_region(&mut self, addr: usize, size : usize) {
        assert_eq!(align_up(addr, mem::align_of::<ListNode>()),addr); // 刚好是node的整数倍
        assert!(size >= mem::size_of::<ListNode>()); // ensure the memory space holding a node

        let mut node = ListNode::new(size);
        node.next = self.head.next.take(); // 将新node的next接管
        let node_ptr = addr as *mut ListNode; // node的地址
        node_ptr.write(node);
        self.head.next = Some(&mut *node_ptr);
        // 新加入的node (node_ptr) 是加到list的最前面, 即head指向这个新的node
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.add_free_region(heap_start, heap_size)
    }
    // 只会被调用一次, 因为外部需要保证heap的内存是valid的, 所以这个函数是unsafe 的

    // 本质是在遍历链表寻找合适的内存node, 然后进行节点的删除
    fn find_region(&mut self, size: usize, align: usize)-> Option<(&'static mut ListNode, usize)> {
        let mut current = &mut self.head;
        while let Some(ref mut region) = current.next {
            if let Ok(alloc_start) = Self::alloc_from_region(&region, size, align) {
                let next = region.next.take();
                let ret = Some((current.next.take().unwrap(), alloc_start));
                current.next = next;
                return ret;

            }else {
                current = current.next.as_mut().unwrap();
            }
        }
        None
    }
}

unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let (size, align) = LinkedListAllocator::size_align(layout);
        let mut allocator = self.lock();

        if let Some((region, alloc_start)) = allocator.find_region(size, align) {
            let alloc_end = alloc_start.checked_add(size).expect("overflow");
            let execess_size = region.end_addr() - alloc_end;
            if execess_size > 0 {
                allocator.add_free_region(alloc_end, execess_size); // 如果这块内容还足够多, 那也先加进来维护, 这样不浪费内存
            }
            alloc_start as *mut u8
        }
        else {
            ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let (size, _) = LinkedListAllocator::size_align(layout);

        self.lock().add_free_region(ptr as usize, size); // 把free掉的内存加进来维护进去
    }
}
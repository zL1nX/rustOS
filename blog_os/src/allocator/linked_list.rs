use core::mem;
use super::align_up;

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
}

impl LinkedListAllocator {
    const fn new() -> Self {
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
    fn find_free_region(&mut self, size: usize, align: usize)-> Option<(&'static mut ListNode, usize)> {
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
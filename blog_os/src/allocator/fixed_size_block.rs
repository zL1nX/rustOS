use alloc::alloc::{Layout, GlobalAlloc};
use core::{mem, ptr::{self, NonNull}};

use super::Locked;

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
}

fn list_index(layout: &Layout) -> Option<usize> { // 根据给定的layout大小找到应该用哪个尺寸的block
    let requested_size = layout.size().max(layout.align());
    BLOCK_SIZES.iter().position(|&s| s >= requested_size)
}

unsafe impl GlobalAlloc for Locked<FixedSizeBlockAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock();
        match list_index(&layout) {
            Some(index) => {
                match allocator.lists_heads[index].take() {
                    Some(node) => { // 有合适的node list, 就返回这个list的头节点
                        allocator.lists_heads[index] = node.next.take(); // 更新头节点
                        node as *mut ListNode as *mut u8
                    }
                    None => { // 有合适的node list, 但是当前的block节点都被分配完了
                        let block_size = BLOCK_SIZES[index];
                        let block_align = block_size;
                        let new_layout = Layout::from_size_align(block_size, block_align).unwrap();
                        allocator.fallback_alloc(new_layout) // 那就没办法用fall back来alloc (根据block size重新来一个layout)
                    }
                }
            }
            None => allocator.fallback_alloc(layout), // 当前没有合适的block尺寸适合layout, 所以干脆就直接分配layout
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.lock();
        match list_index(&layout) {
            Some(index) => {
                let new_node = ListNode {
                    next : allocator.lists_heads[index].take()
                }; //新节点将直接在原来的block之前

                // 确保大小正确
                assert!(mem::size_of::<ListNode>() <= BLOCK_SIZES[index]);
                assert!(mem::align_of::<ListNode>() <= BLOCK_SIZES[index]);

                let new_node_ptr = ptr as *mut ListNode;
                new_node_ptr.write(new_node);
                allocator.lists_heads[index] = Some(&mut *new_node_ptr); // 解引用更新头节点
            }
            None => {
                let node_ptr = NonNull::new(ptr).unwrap(); // 适应下面函数的需求
                allocator.fallback_allocator.deallocate(node_ptr, layout);
            }
        }
    }
}
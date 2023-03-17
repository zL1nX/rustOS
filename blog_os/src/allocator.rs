use core::{alloc::{GlobalAlloc, Layout}, ptr::null_mut};

#[global_allocator]
static ALLOCATOR: Dummy = Dummy; // 一个无field的struct, 直接声明

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
use core::{future::Future, pin::Pin, task::{Context, Poll}, sync::atomic::{AtomicU64, Ordering}};
use alloc::boxed::Box;

pub mod simple_executor;
pub mod keyboard;
pub mod executor; // 真正的executor

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TaskId(u64);

impl TaskId {
    fn new()-> Self {
        static NEXTID : AtomicU64 = AtomicU64::new(0);
        TaskId(NEXTID.fetch_add(1, Ordering::Relaxed))
    }
    // 由于new函数也可能会被同时调用, 所以需要用原子类型来保证ID的唯一性
}

pub struct Task {
    id : TaskId,
    future : Pin<Box<dyn Future<Output=()>>>, // 类型是 (), 也就是这个Future不返回特定类型, 目的只是为了并行地执行任务
}

impl Task {
    pub fn new(future : impl Future<Output=()> + 'static)->Task { // 因为Task需要能存活任意长时间, 所以需要有static的生命周期
        Task {
            id: TaskId::new(),
            future: Box::pin(future),
        }
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context) // Poll方法需要调用Pin<&mut T>, 所以需要先as_mut先转一下
    }
}
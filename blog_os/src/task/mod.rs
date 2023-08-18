use core::{future::Future, pin::Pin, task::{Context, Poll}};
use alloc::boxed::Box;

pub struct Task {
    future : Pin<Box<dyn Future<Output=()>>>, // 类型是 (), 也就是这个Future不返回特定类型, 目的只是为了并行地执行任务
}

impl Task {
    pub fn new(future : impl Future<Output=()> + 'static)->Task { // 因为Task需要能存活任意长时间, 所以需要有static的生命周期
        Task {
            future: Box::pin(future),
        }
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context) // Poll方法需要调用Pin<&mut T>, 所以需要先as_mut先转一下
    }
}
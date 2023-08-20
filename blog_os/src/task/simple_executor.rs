use core::task::{RawWaker, Waker, RawWakerVTable, Context, Poll};

use alloc::collections::VecDeque;

use super::Task;

pub struct SimpleExecutor {
    task_queue: VecDeque<Task>
}

impl SimpleExecutor {
    pub fn new()->SimpleExecutor {
        SimpleExecutor { task_queue: VecDeque::new() }
    }

    pub fn spawn(&mut self, task: Task) {
        self.task_queue.push_back(task)
    }

    pub fn run(&mut self) {
        while let Some(mut task) = self.task_queue.pop_back() {
            let waker = dummy_waker();
            let mut context = Context::from_waker(&waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {}, // do nothing
                Poll::Pending => {self.task_queue.push_back(task)}
            }
        }
    }
}

fn dummy_raw_waker() -> RawWaker {
    fn no_op(_: *const()) {}
    fn clone(_: *const()) -> RawWaker {
        dummy_raw_waker()
    }
    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op); // 目前其他操作都不需要
    RawWaker::new(0 as *const(), vtable) // 此处的vtable和C++中的意义类似, 都是为了让RawWaker能泛化出支持不同类型实例的Waker
    // 由于目前只是为了一个最小的可运行实例, 所以data指针就先传一个0指针
}

fn dummy_waker() -> Waker {
    unsafe {
        Waker::from_raw(dummy_raw_waker())
    }
}
use core::task::{Waker, Context, Poll};
use alloc::{collections::BTreeMap, sync::Arc, task::Wake};
use crossbeam_queue::ArrayQueue;
use x86_64::instructions::interrupts::{self, enable_and_hlt};
use super::{TaskId, Task};

struct TaskWaker {
    id : TaskId,
    task_queue : Arc<ArrayQueue<TaskId>> // 注意这里的queue和Executor维护的queue是一样的, 只是不同的引用而已
}

pub struct Executor {
    tasks : BTreeMap<TaskId, Task>,
    task_queue : Arc<ArrayQueue<TaskId>>,
    waker_cache : BTreeMap<TaskId, Waker>
}

impl TaskWaker {
    fn wake_task(&self) {
        self.task_queue.push(self.id).expect("task_queue full");
    }
    // 将一个task叫醒本质上就是把其id推到队列里, 这样Executor就可以后续去轮询查询了

    fn new(id: TaskId, queue: Arc<ArrayQueue<TaskId>>)-> Waker {
        Waker::from(Arc::new(
            TaskWaker { id: id, task_queue: queue }
        ))
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) { // 参数类型是引用
        self.wake_task()
    }
}

impl Executor {
    pub fn new()-> Self {
        Executor { 
            tasks: BTreeMap::new(), 
            task_queue: Arc::new(ArrayQueue::new(100)), 
            waker_cache: BTreeMap::new() 
        }
    }

    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;
        if self.tasks.insert(task_id, task).is_some() {
            panic!("task with same ID already in tasks");
        }
        self.task_queue.push(task_id).expect("queue full"); // 新的task加进去
    }

    fn run_ready_task(&mut self) {
        let Self {
            tasks,
            task_queue,
            waker_cache
        } = self;
        // 将self中的成员解包出来, 防止borrow check的问题

        // 下面将对task队列中的每个任务进行操作
        while let Ok(task_id) = task_queue.pop() {
            
            // 首先获取task, 防止ID不存在
            let task = match tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue
            };
            
            // 为每个task生成对应的Waker, 而非在每次poll时才会生成waker
            // 并且通过waker cache能复用waker
            let waker = waker_cache.entry(task_id).or_insert_with(|| TaskWaker::new(task_id, task_queue.clone())); // 会clone整个waker队列, 不过只是在引用计数上增加
            let mut context = Context::from_waker(waker);

            //轮询task的状态
            match task.poll(&mut context) {
                Poll::Ready(()) => { // task如果结束了, 那就把该移出的移出
                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                }
                Poll::Pending => {}
            }
        }
    }

    pub fn run(&mut self) {
        loop {
            self.run_ready_task();
            self.sleep_if_idle();
        }
    }

    fn sleep_if_idle(&self) {
        interrupts::disable(); //手动disable, 来保证不会在语句执行间隙出现中断, 保证原子性
        if self.task_queue.is_empty() {
            enable_and_hlt(); //如果暂时没有任务了, 那就让CPU先sleep, 节省资源
        } else {
            interrupts::enable();
        }
    }


}
use core::{pin::Pin, task::{Context, Poll}};

use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use futures_util::{stream::{Stream, StreamExt}, task::AtomicWaker};
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};

use crate::{println, print};

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

pub struct ScancodeStream {
    _private: () // 防止ScancodeStream在其他地方被初始化
}


pub(crate) fn add_scancode(code: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(code) {
            println!("WARNING: scancode queue full; dropping keyboard input");
        } else {
            WAKER.wake(); // 说明IDT此时成功往里add了scancode, 即队列非空, 那么是时候把Waker叫醒, 去把scancodes里的字符pop出来进行操作了
        }
    }else {
        println!("WARNING: scancode queue uninitialized");
    }
}
// 注意, 这个函数是会被interrupt handler调用的, 因此这个函数之内不能包含任何堆上的内存分配操作 (比如QUEUE的初始化等)


impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE.try_init_once(|| ArrayQueue::new(100))
        .expect("ScancodeStream::new should only be called once"); // 提前分配好100的静态空间
        ScancodeStream { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8; // 此处的关联类型是 u8

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<u8>> {
        let queue = SCANCODE_QUEUE.try_get().expect("not initialized");

        if let Ok(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode)); // 如果当前queue不为空的话, 可以提前判断并返回
        }
        //此时IDT应该是在异步地填入这个queue
        //为了避免潜在的竞争, 这里先把Waker给注册进来
        WAKER.register(&cx.waker());

        match queue.pop() { // 再试着pop一次, 此时可能IDT已经把队列给填好了
            Ok(scancode) => {
                WAKER.take(); // 移除registered WAKER, 因为后续不再需要notification了
                Poll::Ready(Some(scancode))
            },
            Err(crossbeam_queue::PopError) => Poll::Pending, // 此时返回时还带了一个Waker
        }
    }
}


pub async fn press_keyboard() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore);

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(ch) => print!("{}", ch),
                    DecodedKey::RawKey(raw) => print!("{:?}", raw)
                }
            }
        }
    }

}
use crate::libra_channel;
use crate::libra_channel::MessageQueue;
use futures::{executor::block_on, FutureExt, StreamExt};
use std::collections::VecDeque;
use std::thread;
use std::time::Duration;

struct TestMessageQueue {
    queue: VecDeque<u8>,
}

impl MessageQueue for TestMessageQueue {
    type Key = u8;
    type Message = u8;

    fn push(&mut self, _key: Self::Key, message: Self::Message) {
        self.queue.push_back(message);
    }

    fn pop(&mut self) -> Option<Self::Message> {
        self.queue.pop_front()
    }
}

#[test]
fn test_send_recv_order() {
    let mq = TestMessageQueue {
        queue: VecDeque::new(),
    };
    let (mut sender, mut receiver) = libra_channel::new(mq);
    sender.put(0, 0);
    sender.put(0, 1);
    sender.put(0, 2);
    sender.put(0, 3);
    let task = async move {
        // Ensure that messages are received in order
        assert_eq!(receiver.select_next_some().await, 0);
        assert_eq!(receiver.select_next_some().await, 1);
        assert_eq!(receiver.select_next_some().await, 2);
        assert_eq!(receiver.select_next_some().await, 3);
        // Ensures that there is no other value which is ready
        assert_eq!(receiver.select_next_some().now_or_never(), None);
    };
    block_on(task);
}

#[test]
fn test_empty() {
    let mq = TestMessageQueue {
        queue: VecDeque::new(),
    };
    let (_, mut receiver) = libra_channel::new(mq);
    // Ensures that there is no other value which is ready
    assert_eq!(receiver.select_next_some().now_or_never(), None);
}

#[test]
fn test_waker() {
    let mq = TestMessageQueue {
        queue: VecDeque::new(),
    };
    let (mut sender, mut receiver) = libra_channel::new(mq);
    // Ensures that there is no other value which is ready
    assert_eq!(receiver.select_next_some().now_or_never(), None);
    let join_handle = thread::spawn(move || {
        block_on(async {
            assert_eq!(receiver.select_next_some().await, 0);
        });
        block_on(async {
            assert_eq!(receiver.select_next_some().await, 1);
        });
        block_on(async {
            assert_eq!(receiver.select_next_some().await, 2);
        });
    });
    thread::sleep(Duration::from_millis(100));
    sender.put(0, 0);
    thread::sleep(Duration::from_millis(100));
    sender.put(0, 1);
    thread::sleep(Duration::from_millis(100));
    sender.put(0, 2);
    join_handle.join().unwrap();
}

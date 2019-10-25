use crate::{
    libra_channel::{self, MessageQueue},
    message_queues::{PerValidatorQueue, QueueStyle},
};
use futures::{executor::block_on, FutureExt, StreamExt};
use libra_types::account_address::AccountAddress;
use libra_types::account_address::ADDRESS_LENGTH;
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

fn test_multiple_validators_helper(
    queue_style: QueueStyle,
    num_messages_per_validator: usize,
    expected_last_message: usize,
) {
    let (mut sender, mut receiver) = libra_channel::new(PerValidatorQueue::new(queue_style, 1));
    let num_validators = 128;
    for message in 0..num_messages_per_validator {
        for validator in 0..num_validators {
            sender.put(
                AccountAddress::new([validator as u8; ADDRESS_LENGTH]),
                (validator, message),
            );
        }
    }
    block_on(async {
        for i in 0..num_validators {
            assert_eq!(
                receiver.select_next_some().await,
                (i, expected_last_message)
            );
        }
    });
    assert_eq!(receiver.select_next_some().now_or_never(), None);
}

#[test]
fn test_multiple_validators_fifo() {
    test_multiple_validators_helper(QueueStyle::FIFO, 1024, 0);
}

#[test]
fn test_multiple_validators_lifo() {
    test_multiple_validators_helper(QueueStyle::LIFO, 1024, 1023);
}

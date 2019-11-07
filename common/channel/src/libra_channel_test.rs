// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{libra_channel, message_queues::QueueStyle};
use futures::{executor::block_on, future::join, future::FutureExt, stream::StreamExt};
use libra_types::account_address::{AccountAddress, ADDRESS_LENGTH};
use std::time::Duration;
use tokio::{runtime::Runtime, time::delay_for};

#[test]
fn test_send_recv_order() {
    let (mut sender, mut receiver) = libra_channel::new(QueueStyle::FIFO, 10, None);
    sender.push(0, 0).unwrap();
    sender.push(0, 1).unwrap();
    sender.push(0, 2).unwrap();
    sender.push(0, 3).unwrap();
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
    let (_, mut receiver) = libra_channel::new::<u8, u8>(QueueStyle::FIFO, 10, None);
    // Ensures that there is no other value which is ready
    assert_eq!(receiver.select_next_some().now_or_never(), None);
}

#[test]
fn test_waker() {
    let (mut sender, mut receiver) = libra_channel::new(QueueStyle::FIFO, 10, None);
    // Ensures that there is no other value which is ready
    assert_eq!(receiver.select_next_some().now_or_never(), None);
    let f1 = async move {
        assert_eq!(receiver.select_next_some().await, 0);
        assert_eq!(receiver.select_next_some().await, 1);
        assert_eq!(receiver.select_next_some().await, 2);
    };
    let f2 = async {
        delay_for(Duration::from_millis(100)).await;
        sender.push(0, 0).unwrap();
        delay_for(Duration::from_millis(100)).await;
        sender.push(0, 1).unwrap();
        delay_for(Duration::from_millis(100)).await;
        sender.push(0, 2).unwrap();
    };
    let mut rt = Runtime::new().unwrap();
    rt.block_on(join(f1, f2));
}

fn test_multiple_validators_helper(
    queue_style: QueueStyle,
    num_messages_per_validator: usize,
    expected_last_message: usize,
) {
    let (mut sender, mut receiver) = libra_channel::new(queue_style, 1, None);
    let num_validators = 128;
    for message in 0..num_messages_per_validator {
        for validator in 0..num_validators {
            sender
                .push(
                    AccountAddress::new([validator as u8; ADDRESS_LENGTH]),
                    (validator, message),
                )
                .unwrap();
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

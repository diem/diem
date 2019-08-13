// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{new_test, new_test_with_timeout, TEST_COUNTER};
use futures::{
    executor::block_on,
    task::{noop_waker, Context, Poll},
    FutureExt, SinkExt, StreamExt,
};
use rusty_fork::{rusty_fork_id, rusty_fork_test, rusty_fork_test_name};
use std::{thread, time::Duration};

#[test]
fn test_send() {
    let (mut tx, mut rx) = new_test(8);
    assert_eq!(TEST_COUNTER.get(), 0);
    let item = 42;
    block_on(tx.send(item)).unwrap();
    assert_eq!(TEST_COUNTER.get(), 1);
    let received_item = block_on(rx.next()).unwrap();
    assert_eq!(received_item, item);
    assert_eq!(TEST_COUNTER.get(), 0);
}

// Fork the unit tests into separate processes to avoid the conflict that these tests executed in
// multiple threads may manipulate TEST_COUNTER at the same time.
rusty_fork_test! {
#[test]
fn test_send_backpressure() {
    let waker = noop_waker();
    let mut cx = Context::from_waker(&waker);

    let (mut tx, mut rx) = new_test(1);
    assert_eq!(TEST_COUNTER.get(), 0);
    block_on(tx.send(1)).unwrap();
    assert_eq!(TEST_COUNTER.get(), 1);

    let mut task = tx.send(2);
    assert_eq!(task.poll_unpin(&mut cx), Poll::Pending);
    let item = block_on(rx.next()).unwrap();
    assert_eq!(item, 1);
    assert_eq!(TEST_COUNTER.get(), 1);
    assert_eq!(task.poll_unpin(&mut cx), Poll::Ready(Ok(())));
}
}

// Fork the unit tests into separate processes to avoid the conflict that these tests executed in
// multiple threads may manipulate TEST_COUNTER at the same time.
rusty_fork_test! {
#[test]
fn test_send_backpressure_multi_senders() {
    let waker = noop_waker();
    let mut cx = Context::from_waker(&waker);

    let (mut tx1, mut rx) = new_test(1);
    assert_eq!(TEST_COUNTER.get(), 0);
    block_on(tx1.send(1)).unwrap();
    assert_eq!(TEST_COUNTER.get(), 1);

    let mut tx2 = tx1.clone();
    let mut task = tx2.send(2);
    assert_eq!(task.poll_unpin(&mut cx), Poll::Pending);
    let item = block_on(rx.next()).unwrap();
    assert_eq!(item, 1);
    assert_eq!(TEST_COUNTER.get(), 1);
    assert_eq!(task.poll_unpin(&mut cx), Poll::Ready(Ok(())));
}
}

// Fork the unit tests into separate processes to avoid the conflict that these tests executed in
// multiple threads may manipulate TEST_COUNTER at the same time.
rusty_fork_test! {
#[test]
fn test_try_send() {
    let (mut tx, mut rx) = new_test(1);
    assert_eq!(TEST_COUNTER.get(), 0);
    let item = 42;
    tx.try_send(item).unwrap();
    assert_eq!(TEST_COUNTER.get(), 1);
    let received_item = block_on(rx.next()).unwrap();
    assert_eq!(received_item, item);
    assert_eq!(TEST_COUNTER.get(), 0);
}
}

// Fork the unit tests into separate processes to avoid the conflict that these tests executed in
// multiple threads may manipulate TEST_COUNTER at the same time.
rusty_fork_test! {
#[test]
fn test_try_send_full() {
    let (mut tx, mut rx) = new_test(1);
    assert_eq!(TEST_COUNTER.get(), 0);
    let item = 42;
    tx.try_send(item).unwrap();
    assert_eq!(TEST_COUNTER.get(), 1);
    tx.try_send(item).unwrap();
    assert_eq!(TEST_COUNTER.get(), 2);
    if let Err(e) = tx.try_send(item) {
        assert!(e.is_full());
    } else {
        panic!("Expect try_send return channel being full error");
    }

    let received_item = block_on(rx.next()).unwrap();
    assert_eq!(received_item, item);
    assert_eq!(TEST_COUNTER.get(), 1);
    let received_item = block_on(rx.next()).unwrap();
    assert_eq!(received_item, item);
    assert_eq!(TEST_COUNTER.get(), 0);
}
}

// Fork the unit tests into separate processes to avoid the conflict that these tests executed in
// multiple threads may manipulate TEST_COUNTER at the same time.
rusty_fork_test! {
#[test]
fn test_timeout() {
    let (mut tx, mut rx) = new_test_with_timeout(2, Duration::from_secs(1));
    assert_eq!(TEST_COUNTER.get(), 0);
    let item_1 = 1;
    block_on(tx.send(item_1)).unwrap();
    assert_eq!(TEST_COUNTER.get(), 1);
    thread::sleep(Duration::from_secs(1));
    let item_2 = 2;
    block_on(tx.send(item_2)).unwrap();
    let received_item = block_on(rx.next()).unwrap();
    assert_eq!(received_item, item_2);
    assert_eq!(TEST_COUNTER.get(), 0);
}
}

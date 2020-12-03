// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate as channel;
use diem_metrics::IntGauge;
use futures::{
    executor::block_on,
    task::{noop_waker, Context, Poll},
    FutureExt, SinkExt, StreamExt,
};

#[test]
fn test_send() {
    let counter = IntGauge::new("TEST_COUNTER", "test").unwrap();
    let (mut tx, mut rx) = channel::new(8, &counter);

    assert_eq!(counter.get(), 0);
    let item = 42;
    block_on(tx.send(item)).unwrap();
    assert_eq!(counter.get(), 1);
    let received_item = block_on(rx.next()).unwrap();
    assert_eq!(received_item, item);
    assert_eq!(counter.get(), 0);
}

#[test]
fn test_send_backpressure() {
    let waker = noop_waker();
    let mut cx = Context::from_waker(&waker);
    let counter = IntGauge::new("TEST_COUNTER", "test").unwrap();
    let (mut tx, mut rx) = channel::new(1, &counter);

    assert_eq!(counter.get(), 0);
    block_on(tx.send(1)).unwrap();
    assert_eq!(counter.get(), 1);

    let mut task = tx.send(2);
    assert_eq!(task.poll_unpin(&mut cx), Poll::Pending);
    let item = block_on(rx.next()).unwrap();
    assert_eq!(item, 1);
    assert_eq!(counter.get(), 1);
    assert_eq!(task.poll_unpin(&mut cx), Poll::Ready(Ok(())));
}

#[test]
fn test_send_backpressure_multi_senders() {
    let waker = noop_waker();
    let mut cx = Context::from_waker(&waker);
    let counter = IntGauge::new("TEST_COUNTER", "test").unwrap();
    let (mut tx1, mut rx) = channel::new(1, &counter);

    assert_eq!(counter.get(), 0);
    block_on(tx1.send(1)).unwrap();
    assert_eq!(counter.get(), 1);

    let mut tx2 = tx1;
    let mut task = tx2.send(2);
    assert_eq!(task.poll_unpin(&mut cx), Poll::Pending);
    let item = block_on(rx.next()).unwrap();
    assert_eq!(item, 1);
    assert_eq!(counter.get(), 1);
    assert_eq!(task.poll_unpin(&mut cx), Poll::Ready(Ok(())));
}

#[test]
fn test_try_send() {
    let counter = IntGauge::new("TEST_COUNTER", "test").unwrap();
    let (mut tx, mut rx) = channel::new(1, &counter);

    assert_eq!(counter.get(), 0);
    let item = 42;
    tx.try_send(item).unwrap();
    assert_eq!(counter.get(), 1);
    let received_item = block_on(rx.next()).unwrap();
    assert_eq!(received_item, item);
    assert_eq!(counter.get(), 0);
}

#[test]
fn test_try_send_full() {
    let counter = IntGauge::new("TEST_COUNTER", "test").unwrap();
    let (mut tx, mut rx) = channel::new(1, &counter);

    assert_eq!(counter.get(), 0);
    let item = 42;
    tx.try_send(item).unwrap();
    assert_eq!(counter.get(), 1);
    tx.try_send(item).unwrap();
    assert_eq!(counter.get(), 2);
    if let Err(e) = tx.try_send(item) {
        assert!(e.is_full());
    } else {
        panic!("Expect try_send return channel being full error");
    }

    let received_item = block_on(rx.next()).unwrap();
    assert_eq!(received_item, item);
    assert_eq!(counter.get(), 1);
    let received_item = block_on(rx.next()).unwrap();
    assert_eq!(received_item, item);
    assert_eq!(counter.get(), 0);
}

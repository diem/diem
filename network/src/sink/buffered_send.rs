// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use futures::{
    future::Future,
    ready,
    sink::Sink,
    task::{Context, Poll},
};
use std::pin::Pin;

/// Future for the [`buffered_send`](super::NetworkSinkExt::buffered_send) method.
#[derive(Debug)]
#[must_use = "futures do nothing unless polled"]
// TODO(philiphayes): remove
#[allow(dead_code)]
pub struct BufferedSend<'a, S: Sink<Item> + Unpin + ?Sized, Item> {
    sink: &'a mut S,
    item: Option<Item>,
}

// Pinning is never projected to children.
impl<S: Sink<Item> + Unpin + ?Sized, Item> Unpin for BufferedSend<'_, S, Item> {}

impl<'a, S: Sink<Item> + Unpin + ?Sized, Item> BufferedSend<'a, S, Item> {
    // TODO(philiphayes): remove
    #[allow(dead_code)]
    pub fn new(sink: &'a mut S, item: Item) -> Self {
        Self {
            sink,
            item: Some(item),
        }
    }
}

impl<S: Sink<Item> + Unpin + ?Sized, Item> Future for BufferedSend<'_, S, Item> {
    type Output = Result<(), S::Error>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        // Get a &mut Self from the Pin<&mut Self>.
        let this = &mut *self;

        // If someone polls us after we've already sent the item, successfully or
        // not, then we just return Ok(()).
        if this.item.is_none() {
            return Poll::Ready(Ok(()));
        }

        // Poll the underlying sink until it's ready to send an item (or errors).
        let mut sink = Pin::new(&mut this.sink);
        ready!(sink.as_mut().poll_ready(context))?;

        // We need ownership of the pending item to send it on the sink. We take
        // it _after_ the sink.poll_ready to avoid awkward control flow with
        // placing the item back in self if the underlying sink isn't ready.
        let item = this
            .item
            .take()
            .expect("We have already checked that item.is_none(), so this will never panic");

        // Actually send the item
        Poll::Ready(sink.as_mut().start_send(item))
    }
}

#[cfg(test)]
mod test {
    use crate::sink::NetworkSinkExt;
    use futures::{
        channel::mpsc, executor::block_on, future::join, sink::SinkExt, stream::StreamExt,
    };

    // It should work.
    #[test]
    fn buffered_send() {
        let (mut tx, mut rx) = mpsc::channel::<u32>(1);

        block_on(tx.send(123)).unwrap();
        assert_eq!(Some(123), block_on(rx.next()));
    }

    // It should not flush where `.send` otherwise would.
    #[test]
    fn doesnt_flush() {
        ::libra_logger::try_init_for_testing();

        // A 0-capacity channel + one sender gives the channel 1 available buffer
        // slot.
        let (tx, mut rx) = mpsc::channel::<u32>(1);
        let mut tx = tx.buffer(2);

        // Initial state
        //
        // +-----------------+
        // | _ | _ |    _    |
        // +-----------------+
        //  .buffer \ channel

        // `.buffer` only buffers items if the underlying sink is busy. So the
        // first send should write-through to the channel, since the channel has
        // 1 available buffer slot.
        block_on(tx.buffered_send(1)).unwrap();

        // +-----------------+
        // | _ | _ |    1    |
        // +-----------------+
        //  .buffer \ channel

        // If we used `tx.send(2)` here, it would block since `tx.send` requires
        // a flush after enqueueing. However, the channel is already full, so the
        // flush would never complete. Instead, we can use our new `.buffered_send`
        // which doesn't mandate a flush.

        // Next two should buffer in `.buffer` since the underlying channel is full.
        block_on(tx.buffered_send(2)).unwrap();
        block_on(tx.buffered_send(3)).unwrap();

        // +-----------------+
        // | 3 | 2 |    1    |
        // +-----------------+
        //  .buffer \ channel

        // If we used `tx.buffered_send(4)` here, it would block since both the
        // channel and `.buffer` are full.

        // This call should succeed and return the item buffered in the channel.
        assert_eq!(Some(1), block_on(rx.next()));

        // +-----------------+
        // | 3 | 2 |    _    | => 1
        // +-----------------+
        //  .buffer \ channel

        // The following calls would block, since 2 & 3 are stuck in `.buffer`
        // even though the channel buffer is empty.
        // assert_eq!(Some(2), block_on(rx.next()));
        // assert_eq!(Some(3), block_on(rx.next()));

        // Instead, we have to manually flush `tx` while dequeueing the remaining
        // items from the channel.

        // `f_flush` will complete when all items in `.buffer` are flushed down
        // to the underlying channel
        let f_flush = async move {
            tx.flush().await.unwrap();
            tx
        };

        let f_recv = async move {
            assert_eq!(Some(2), rx.next().await);
            assert_eq!(Some(3), rx.next().await);
            rx
        };

        // flush 2
        //
        // +-----------------+
        // | 3 | _ |    2    |
        // +-----------------+
        //  .buffer \ channel

        // dequeue 2
        //
        // +-----------------+
        // | 3 | _ |    _    | => 2
        // +-----------------+
        //  .buffer \ channel

        // flush 3 + f_flush done
        //
        // +-----------------+
        // | _ | _ |    3    |
        // +-----------------+
        //  .buffer \ channel

        // dequeue 3 + f_recv done
        //
        // +-----------------+
        // | _ | _ |    _    | => 3
        // +-----------------+
        //  .buffer \ channel

        let (_tx, _rx) = block_on(join(f_flush, f_recv));
    }

    // Polling after the future has completed should not panic.
    #[test]
    fn poll_after_ready() {
        let (mut tx, mut rx) = mpsc::channel::<u32>(1);

        let mut f_send = tx.send(123);

        // Poll the first time like normal.
        block_on(&mut f_send).unwrap();
        // Polling after it's already complete should just resolve immediately.
        block_on(&mut f_send).unwrap();

        block_on(tx.close()).unwrap();

        // There should only be one item in the channel.
        assert_eq!(Some(123), block_on(rx.next()));
        assert_eq!(None, block_on(rx.next()));
    }
}

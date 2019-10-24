//! libra_channel provides an mpsc channel which has two ends `libra_channel::Receiver`
//! and `libra_channel::Sender` similar to existing mpsc data structures.
//! What makes it different from existing mpsc channels is that we have full control
//! over how the internal queueing in the channel happens and how we schedule messages
//! to be sent out from this channel.
//! Note that libra_channel does not provide an implementation of this internal queueing
//! mechanism, but provides a trait (`MessageQueue`) which the consumers of this channel
//! can use to create libra_channels' according to their needs.

use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::Waker;

use futures::async_await::FusedStream;
use futures::stream::Stream;
use futures::task::Context;
use futures::Poll;
use std::hash::Hash;

/// MessageQueue is a trait which provides a very simple set of methods for implementing
/// a queue which will be used as the internal queue libra_channel.
pub trait MessageQueue {
    type Key: Eq + Hash;
    /// The actual type of the messages stored in this MessageQueue
    type Message;

    /// Push a message with the given key to this queue
    fn push(&mut self, key: Self::Key, message: Self::Message);

    /// Pop a message from this queue
    fn pop(&mut self) -> Option<Self::Message>;
}

/// SharedState is a data structure private to this module which is
/// shared by the sender and receiver.
struct SharedState<T> {
    /// The internal queue of messages in this Channel
    internal_queue: T,

    /// Waker is needed so that the Sender can notify the task executor/scheduler
    /// that something has been pushed to the internal_queue and it ready for
    /// consumption by the Receiver and then the executor/scheduler will wake up
    /// the Receiver task to process the next item.
    waker: Option<Waker>,
}

/// The sending end of the libra_channel.
#[derive(Clone)]
pub struct Sender<T> {
    shared_state: Arc<Mutex<SharedState<T>>>,
}

impl<T: MessageQueue> Sender<T> {
    /// This adds the message into the internal queue data structure. This is a non-blocking
    /// synchronous call.
    /// TODO: We can have this return a boolean if the queue of a validator is capacity
    pub fn put(&mut self, key: <T as MessageQueue>::Key, message: <T as MessageQueue>::Message) {
        let mut shared_state = self.shared_state.lock().unwrap();
        shared_state.internal_queue.push(key, message);
        if let Some(w) = shared_state.waker.take() {
            w.wake();
        }
    }
}

/// The receiving end of the libra_channel.
pub struct Receiver<T> {
    shared_state: Arc<Mutex<SharedState<T>>>,
}

impl<T: MessageQueue> Stream for Receiver<T> {
    type Item = <T as MessageQueue>::Message;

    /// poll_next checks whether there is something ready for consumption from the internal
    /// queue. If there is, then it returns immediately. If the internal_queue is empty,
    /// it sets the waker passed to it by the scheduler/executor and returns Pending
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut shared_state = self.shared_state.lock().unwrap();
        if let Some(val) = shared_state.internal_queue.pop() {
            Poll::Ready(Some(val))
        } else {
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl<T: MessageQueue> FusedStream for Receiver<T> {
    fn is_terminated(&self) -> bool {
        false
    }
}

/// Create a new Libra Channel and returns the two ends of the channel. The sender end can
/// be cloned making it an mpsc.
pub fn new<T>(queue: T) -> (Sender<T>, Receiver<T>)
where
    T: MessageQueue,
{
    let shared_state = Arc::new(Mutex::new(SharedState {
        internal_queue: queue,
        waker: None,
    }));
    let shared_state_clone = Arc::clone(&shared_state);
    (
        Sender { shared_state },
        Receiver {
            shared_state: shared_state_clone,
        },
    )
}

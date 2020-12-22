// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! diem_channel provides an mpsc channel which has two ends `diem_channel::Receiver`
//! and `diem_channel::Sender` similar to existing mpsc data structures.
//! What makes it different from existing mpsc channels is that we have full control
//! over how the internal queueing in the channel happens and how we schedule messages
//! to be sent out from this channel.
//! Internally, it uses the `PerKeyQueue` to store messages
use crate::message_queues::{PerKeyQueue, QueueStyle};
use anyhow::{ensure, Result};
use diem_infallible::{Mutex, NonZeroUsize};
use diem_metrics::IntCounterVec;
use futures::{
    channel::oneshot,
    stream::{FusedStream, Stream},
};
use std::{
    fmt::{Debug, Formatter},
    hash::Hash,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Waker},
};

/// SharedState is a data structure private to this module which is
/// shared by the `Receiver` and any `Sender`s.
#[derive(Debug)]
struct SharedState<K: Eq + Hash + Clone, M> {
    /// The internal queue of messages in this channel.
    internal_queue: PerKeyQueue<K, (M, Option<oneshot::Sender<ElementStatus<M>>>)>,
    /// The `Receiver` registers its `Waker` in this slot when the queue is empty.
    /// `Sender`s will try to wake the `Receiver` (if any) when they push a new
    /// item onto the queue. The last live `Sender` will also wake the `Receiver`
    /// as it's tearing down so the `Receiver` can gracefully drain and shutdown
    /// the channel.
    waker: Option<Waker>,
    /// The number of active senders. When this value reaches 0, all senders have
    /// been dropped.
    num_senders: usize,
    /// A boolean which tracks whether the receiver has dropped.
    receiver_dropped: bool,
    /// A boolean which tracks whether the stream has terminated. A stream is
    /// considered terminated when sender has dropped and we have drained everything
    /// inside our internal queue.
    stream_terminated: bool,
}

/// The sending end of the diem_channel.
#[derive(Debug)]
pub struct Sender<K: Eq + Hash + Clone, M> {
    shared_state: Arc<Mutex<SharedState<K, M>>>,
}

/// The status of an element inserted into a diem_channel. If the element is successfully
/// dequeued, ElementStatus::Dequeued is sent to the sender. If it is dropped
/// ElementStatus::Dropped is sent to the sender along with the dropped element.
pub enum ElementStatus<M> {
    Dequeued,
    Dropped(M),
}

impl<M: PartialEq> PartialEq for ElementStatus<M> {
    fn eq(&self, other: &ElementStatus<M>) -> bool {
        match (self, other) {
            (ElementStatus::Dequeued, ElementStatus::Dequeued) => true,
            (ElementStatus::Dropped(a), ElementStatus::Dropped(b)) => a.eq(b),
            _ => false,
        }
    }
}

impl<M: Debug> Debug for ElementStatus<M> {
    fn fmt(&self, f: &mut Formatter) -> std::result::Result<(), std::fmt::Error> {
        match self {
            ElementStatus::Dequeued => write!(f, "Dequeued"),
            ElementStatus::Dropped(v) => write!(f, "Dropped({:?})", v),
        }
    }
}

impl<K: Eq + Hash + Clone, M> Sender<K, M> {
    /// This adds the message into the internal queue data structure. This is a
    /// synchronous call.
    pub fn push(&mut self, key: K, message: M) -> Result<()> {
        self.push_with_feedback(key, message, None)
    }

    /// Same as `push`, but this function also accepts a oneshot::Sender over which the sender can
    /// be notified when the message eventually gets delivered or dropped.
    pub fn push_with_feedback(
        &mut self,
        key: K,
        message: M,
        status_ch: Option<oneshot::Sender<ElementStatus<M>>>,
    ) -> Result<()> {
        let mut shared_state = self.shared_state.lock();
        ensure!(!shared_state.receiver_dropped, "Channel is closed");
        debug_assert!(shared_state.num_senders > 0);

        let dropped = shared_state.internal_queue.push(key, (message, status_ch));
        // If this or an existing message had to be dropped because of the queue being full, we
        // notify the corresponding status channel if it was registered.
        if let Some((dropped_val, Some(dropped_status_ch))) = dropped {
            // Ignore errors.
            let _err = dropped_status_ch.send(ElementStatus::Dropped(dropped_val));
        }
        if let Some(w) = shared_state.waker.take() {
            w.wake();
        }
        Ok(())
    }
}

impl<K: Eq + Hash + Clone, M> Clone for Sender<K, M> {
    fn clone(&self) -> Self {
        let shared_state = self.shared_state.clone();
        {
            let mut shared_state_lock = shared_state.lock();
            debug_assert!(shared_state_lock.num_senders > 0);
            shared_state_lock.num_senders += 1;
        }
        Sender { shared_state }
    }
}

impl<K: Eq + Hash + Clone, M> Drop for Sender<K, M> {
    fn drop(&mut self) {
        let mut shared_state = self.shared_state.lock();

        debug_assert!(shared_state.num_senders > 0);
        shared_state.num_senders -= 1;

        if shared_state.num_senders == 0 {
            if let Some(waker) = shared_state.waker.take() {
                waker.wake();
            }
        }
    }
}

/// The receiving end of the diem_channel.
pub struct Receiver<K: Eq + Hash + Clone, M> {
    shared_state: Arc<Mutex<SharedState<K, M>>>,
}

impl<K: Eq + Hash + Clone, M> Receiver<K, M> {
    /// Removes all the previously sent transactions that have not been consumed yet and cleans up
    /// the internal queue structure (GC of the previous keys).
    pub fn clear(&mut self) {
        let mut shared_state = self.shared_state.lock();
        shared_state.internal_queue.clear();
    }
}

impl<K: Eq + Hash + Clone, M> Drop for Receiver<K, M> {
    fn drop(&mut self) {
        let mut shared_state = self.shared_state.lock();
        debug_assert!(!shared_state.receiver_dropped);
        shared_state.receiver_dropped = true;
    }
}

impl<K: Eq + Hash + Clone, M> Stream for Receiver<K, M> {
    type Item = M;
    /// poll_next checks whether there is something ready for consumption from the internal
    /// queue. If there is, then it returns immediately. If the internal_queue is empty,
    /// it sets the waker passed to it by the scheduler/executor and returns Pending
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut shared_state = self.shared_state.lock();
        if let Some((val, status_ch)) = shared_state.internal_queue.pop() {
            if let Some(status_ch) = status_ch {
                let _err = status_ch.send(ElementStatus::Dequeued);
            }
            Poll::Ready(Some(val))
        // all senders have been dropped (and so the stream is terminated)
        } else if shared_state.num_senders == 0 {
            shared_state.stream_terminated = true;
            Poll::Ready(None)
        } else {
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl<K: Eq + Hash + Clone, M> FusedStream for Receiver<K, M> {
    fn is_terminated(&self) -> bool {
        self.shared_state.lock().stream_terminated
    }
}

/// Create a new Diem Channel and returns the two ends of the channel.
pub fn new<K: Eq + Hash + Clone, M>(
    queue_style: QueueStyle,
    max_queue_size_per_key: usize,
    counters: Option<&'static IntCounterVec>,
) -> (Sender<K, M>, Receiver<K, M>) {
    let max_queue_size_per_key =
        NonZeroUsize!(max_queue_size_per_key, "diem_channel cannot be of size 0");
    let shared_state = Arc::new(Mutex::new(SharedState {
        internal_queue: PerKeyQueue::new(queue_style, max_queue_size_per_key, counters),
        waker: None,
        num_senders: 1,
        receiver_dropped: false,
        stream_terminated: false,
    }));
    let shared_state_clone = Arc::clone(&shared_state);
    (
        Sender { shared_state },
        Receiver {
            shared_state: shared_state_clone,
        },
    )
}

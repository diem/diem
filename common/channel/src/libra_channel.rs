// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! libra_channel provides an mpsc channel which has two ends `libra_channel::Receiver`
//! and `libra_channel::Sender` similar to existing mpsc data structures.
//! What makes it different from existing mpsc channels is that we have full control
//! over how the internal queueing in the channel happens and how we schedule messages
//! to be sent out from this channel.
//! Internally, it uses the `PerKeyQueue` to store messages
use crate::message_queues::{PerKeyQueue, QueueStyle};
use anyhow::{ensure, Result};
use futures::{
    channel::oneshot,
    stream::{FusedStream, Stream},
};
use libra_metrics::IntCounterVec;
use std::{
    fmt::{Debug, Formatter},
    hash::Hash,
    num::NonZeroUsize,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};

/// SharedState is a data structure private to this module which is
/// shared by the sender and receiver.
#[derive(Debug)]
struct SharedState<K: Eq + Hash + Clone, M> {
    /// The internal queue of messages in this Channel
    internal_queue: PerKeyQueue<K, (M, Option<oneshot::Sender<ElementStatus<M>>>)>,
    /// Waker is needed so that the Sender can notify the task executor/scheduler
    /// that something has been pushed to the internal_queue and it ready for
    /// consumption by the Receiver and then the executor/scheduler will wake up
    /// the Receiver task to process the next item.
    waker: Option<Waker>,

    /// A boolean which tracks whether the receiver has dropped
    receiver_dropped: bool,
    /// A boolean which tracks whether the stream has terminated
    /// A stream is considered terminated when sender has dropped
    /// and we have drained everything inside our internal queue
    stream_terminated: bool,
}

/// The sending end of the libra_channel.
#[derive(Debug)]
pub struct Sender<K: Eq + Hash + Clone, M> {
    shared_state: Arc<Mutex<SharedState<K, M>>>,
}

/// The status of an element inserted into a libra_channel. If the element is successfully
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
        let mut shared_state = self.shared_state.lock().unwrap();
        ensure!(!shared_state.receiver_dropped, "Channel is closed");
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
        Sender {
            shared_state: self.shared_state.clone(),
        }
    }
}

impl<K: Eq + Hash + Clone, M> Drop for Sender<K, M> {
    fn drop(&mut self) {
        let mut shared_state = self.shared_state.lock().unwrap();
        if let Some(w) = shared_state.waker.take() {
            w.wake();
        }
    }
}

/// The receiving end of the libra_channel.
pub struct Receiver<K: Eq + Hash + Clone, M> {
    shared_state: Arc<Mutex<SharedState<K, M>>>,
}

impl<K: Eq + Hash + Clone, M> Receiver<K, M> {
    /// Removes all the previously sent transactions that have not been consumed yet and cleans up
    /// the internal queue structure (GC of the previous keys).
    pub fn clear(&mut self) {
        let mut shared_state = self.shared_state.lock().unwrap();
        shared_state.internal_queue.clear();
    }
}

impl<K: Eq + Hash + Clone, M> Drop for Receiver<K, M> {
    fn drop(&mut self) {
        let mut shared_state = self.shared_state.lock().unwrap();
        shared_state.receiver_dropped = true;
    }
}

impl<K: Eq + Hash + Clone, M> Stream for Receiver<K, M> {
    type Item = M;
    /// poll_next checks whether there is something ready for consumption from the internal
    /// queue. If there is, then it returns immediately. If the internal_queue is empty,
    /// it sets the waker passed to it by the scheduler/executor and returns Pending
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut shared_state = self.shared_state.lock().unwrap();
        if let Some((val, status_ch)) = shared_state.internal_queue.pop() {
            if let Some(status_ch) = status_ch {
                let _err = status_ch.send(ElementStatus::Dequeued);
            }
            Poll::Ready(Some(val))
        // if the only Arc reference to `shared_state` is 1, which is this receiver,
        // this must mean all senders have been dropped (and so the stream is terminated)
        } else if Arc::strong_count(&self.shared_state) == 1 {
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
        self.shared_state.lock().unwrap().stream_terminated
    }
}

/// Create a new Libra Channel and returns the two ends of the channel.
pub fn new<K: Eq + Hash + Clone, M>(
    queue_style: QueueStyle,
    max_queue_size_per_key: NonZeroUsize,
    counters: Option<&'static IntCounterVec>,
) -> (Sender<K, M>, Receiver<K, M>) {
    let shared_state = Arc::new(Mutex::new(SharedState {
        internal_queue: PerKeyQueue::new(queue_style, max_queue_size_per_key, counters),
        waker: None,
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

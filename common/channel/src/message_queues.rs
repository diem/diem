// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_metrics::IntCounterVec;
use std::{
    collections::{HashMap, VecDeque},
    fmt::{Debug, Formatter, Result},
    hash::Hash,
    num::NonZeroUsize,
};

/// Remove empty per-key-queues every `POPS_PER_GC` dequeue operations.
const POPS_PER_GC: u32 = 50;

/// QueueStyle is an enum which can be used as a configuration option for
/// PerValidatorQueue. Since the queue per key is going to be bounded,
/// QueueStyle also determines the policy for dropping and retrieving messages.
///
/// With LIFO, oldest messages are dropped.
/// With FIFO, newest messages are dropped.
/// With KLAST, oldest messages are dropped, but remaining are retrieved in FIFO order
#[derive(Clone, Copy, Debug)]
pub enum QueueStyle {
    LIFO,
    FIFO,
    KLAST,
}

/// PerKeyQueue maintains a queue of messages per key. It
/// is a bounded queue of messages per Key and the style (FIFO, LIFO) is
/// configurable. When a new message is added using `push`, it is added to
/// the key's queue.
///
/// When `pop` is called, the next message is picked from one
/// of the key's queue and returned. This happens in a round-robin
/// fashion among keys.
///
/// If there are no messages, in any of the queues, `None` is returned.
pub(crate) struct PerKeyQueue<K: Eq + Hash + Clone, T> {
    /// QueueStyle for the messages stored per key
    queue_style: QueueStyle,
    /// per_key_queue maintains a map from a Key to a queue
    /// of all the messages from that Key. A Key is usually
    /// represented by AccountAddress
    per_key_queue: HashMap<K, VecDeque<T>>,
    /// This is a (round-robin)queue of Keys which have pending messages
    /// This queue will be used for performing round robin among
    /// Keys for choosing the next message
    round_robin_queue: VecDeque<K>,
    /// Maximum number of messages to store per key
    max_queue_size: NonZeroUsize,
    /// Number of messages dequeued since last GC
    num_popped_since_gc: u32,
    /// Optional counters for recording # enqueued, # dequeued, and # dropped
    /// messages
    counters: Option<&'static IntCounterVec>,
}

impl<K: Eq + Hash + Clone, T> Debug for PerKeyQueue<K, T> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.debug_struct("PerKeyQueue")
            .field("queue_style", &self.queue_style)
            .field("max_queue_size", &self.max_queue_size)
            .field("num_popped_since_gc", &self.num_popped_since_gc)
            .finish()
    }
}

impl<K: Eq + Hash + Clone, T> PerKeyQueue<K, T> {
    /// Create a new PerKeyQueue with the provided QueueStyle and
    /// max_queue_size_per_key
    pub(crate) fn new(
        queue_style: QueueStyle,
        max_queue_size_per_key: NonZeroUsize,
        counters: Option<&'static IntCounterVec>,
    ) -> Self {
        Self {
            queue_style,
            max_queue_size: max_queue_size_per_key,
            per_key_queue: HashMap::new(),
            round_robin_queue: VecDeque::new(),
            num_popped_since_gc: 0,
            counters,
        }
    }

    /// Given a key, pops the message from its queue and returns the message
    /// It also returns a boolean indicating whether the keys queue is empty
    /// after popping the message
    fn pop_from_key_queue(&mut self, key: &K) -> (Option<T>, bool) {
        if let Some(q) = self.per_key_queue.get_mut(key) {
            // Extract message from the key's queue
            let retval = match self.queue_style {
                QueueStyle::FIFO | QueueStyle::KLAST => q.pop_front(),
                QueueStyle::LIFO => q.pop_back(),
            };
            (retval, q.is_empty())
        } else {
            (None, true)
        }
    }

    /// push a message to the appropriate queue in per_key_queue
    /// add the key to round_robin_queue if it didnt already exist.
    /// Returns Some(T) if the new or an existing element was dropped. Returns None otherwise.
    pub(crate) fn push(&mut self, key: K, message: T) -> Option<T> {
        if let Some(c) = self.counters.as_ref() {
            c.with_label_values(&["enqueued"]).inc();
        }

        let key_message_queue = self
            .per_key_queue
            .entry(key.clone())
            // Only allocate a small initial queue for a new key. Previously, we
            // allocated a queue with all `max_queue_size_per_key` entries;
            // however, this breaks down when we have lots of transient peers.
            // For example, many of our queues have a max capacity of 1024. To
            // handle a single rpc from a transient peer, we would end up
            // allocating ~ 96 b * 1024 ~ 64 Kib per queue.
            .or_insert_with(|| VecDeque::with_capacity(1));

        // Add the key to our round-robin queue if it's not already there
        if key_message_queue.is_empty() {
            self.round_robin_queue.push_back(key);
        }

        // Push the message to the actual key message queue
        if key_message_queue.len() >= self.max_queue_size.get() {
            if let Some(c) = self.counters.as_ref() {
                c.with_label_values(&["dropped"]).inc();
            }
            match self.queue_style {
                // Drop the newest message for FIFO
                QueueStyle::FIFO => Some(message),
                // Drop the oldest message for LIFO
                QueueStyle::LIFO | QueueStyle::KLAST => {
                    let oldest = key_message_queue.pop_front();
                    key_message_queue.push_back(message);
                    oldest
                }
            }
        } else {
            key_message_queue.push_back(message);
            None
        }
    }

    /// pop a message from the appropriate queue in per_key_queue
    /// remove the key from the round_robin_queue if it has no more messages
    pub(crate) fn pop(&mut self) -> Option<T> {
        let key = match self.round_robin_queue.pop_front() {
            Some(v) => v,
            _ => {
                return None;
            }
        };

        let (message, is_q_empty) = self.pop_from_key_queue(&key);
        if !is_q_empty {
            self.round_robin_queue.push_back(key);
        }

        if message.is_some() {
            if let Some(c) = self.counters.as_ref() {
                c.with_label_values(&["dequeued"]).inc();
            }

            // Remove empty per-key-queues every `POPS_PER_GC` successful dequeue
            // operations.
            //
            // diem-channel never removes keys from its PerKeyQueue (without
            // this logic). This works fine for the validator network, where we
            // have a bounded set of peers that almost never changes; however,
            // this does not work for servicing public clients, where we can have
            // large and frequent connection churn.
            //
            // Periodically removing these empty queues prevents us from causing
            // an effective memory leak when we have lots of transient peers in
            // e.g. the public-facing vfn use-case.
            //
            // This GC strategy could probably be more sophisticated, though it
            // seems to work well in some basic stress tests / micro benches.
            //
            // See: common/channel/src/bin/many_keys_stress_test.rs
            //
            // For more context, see: https://github.com/diem/diem/issues/5543
            self.num_popped_since_gc += 1;
            if self.num_popped_since_gc >= POPS_PER_GC {
                self.num_popped_since_gc = 0;
                self.remove_empty_queues();
            }
        }

        message
    }

    /// Garbage collect any empty per-key-queues.
    fn remove_empty_queues(&mut self) {
        self.per_key_queue.retain(|_key, queue| !queue.is_empty());
    }

    /// Clears all the pending messages and cleans up the queue from the previous metadata.
    pub(crate) fn clear(&mut self) {
        self.per_key_queue.clear();
        self.round_robin_queue.clear();
    }
}

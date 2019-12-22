// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_metrics::IntCounterVec;
use std::{
    collections::{HashMap, VecDeque},
    fmt::{Debug, Formatter, Result},
    hash::Hash,
    num::NonZeroUsize,
};

/// QueueStyle is an enum which can be used as a configuration option for
/// PerValidatorQueue. Since the queue per key is going to be bounded,
/// QueueStyle also determines the policy for dropping messages.
/// With LIFO, oldest messages are dropped.
/// With FIFO, newest messages are dropped.
#[derive(Clone, Copy, Debug)]
pub enum QueueStyle {
    LIFO,
    FIFO,
}

/// PerKeyQueue maintains a queue of messages per key. It
/// is a bounded queue of messages per Key and the style (FIFO, LIFO) is
/// configurable. When a new message is added using `push`, it is added to
/// the key's queue.
/// When `pop` is called, the next message is picked from one
/// of the key's queue and returned. This happens in a round-robin
/// fashion among keys.
/// If there are no messages, in any of the queues, `None` is returned.
pub(crate) struct PerKeyQueue<K: Eq + Hash + Clone, T> {
    /// QueueStyle for the messages stored per key
    queue_style: QueueStyle,
    /// per_key_queue maintains a map from a Key to a queue
    /// of all the messages from that Key. A Key is
    /// represented by AccountAddress
    per_key_queue: HashMap<K, VecDeque<T>>,
    /// This is a (round-robin)queue of Keys which have pending messages
    /// This queue will be used for performing round robin among
    /// Keys for choosing the next message
    round_robin_queue: VecDeque<K>,
    /// Maximum number of messages to store per key
    max_queue_size: NonZeroUsize,
    counters: Option<&'static IntCounterVec>,
}

// TODO potentially add `per_key_queue` and `round_robin_queue`
impl<K: Eq + Hash + Clone, T> Debug for PerKeyQueue<K, T> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "PerKeyQueue {{\n\
             queue_style: {:?},\n\
             max_queue_size: {},\n\
             }}",
            self.queue_style, self.max_queue_size
        )
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
                QueueStyle::FIFO => q.pop_front(),
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
        let max_queue_size = self.max_queue_size.get();
        let key_message_queue = self
            .per_key_queue
            .entry(key.clone())
            .or_insert_with(|| VecDeque::with_capacity(max_queue_size));
        // Add the key to our round-robin queue if it's not already there
        if key_message_queue.is_empty() {
            self.round_robin_queue.push_back(key);
        }
        // Push the message to the actual key message queue
        if key_message_queue.len() == max_queue_size {
            if let Some(c) = self.counters.as_ref() {
                c.with_label_values(&["dropped"]).inc();
            }
            match self.queue_style {
                // Drop the newest message for FIFO
                QueueStyle::FIFO => Some(message),
                // Drop the oldest message for LIFO
                QueueStyle::LIFO => {
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
        }
        message
    }

    /// Clears all the pending messages and cleans up the queue from the previous metadata.
    pub(crate) fn clear(&mut self) {
        self.per_key_queue.clear();
        self.round_robin_queue.clear();
    }
}

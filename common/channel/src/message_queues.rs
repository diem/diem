use crate::libra_channel::MessageQueue;
use libra_metrics::IntCounter;
use libra_types::account_address::AccountAddress;
use std::collections::{HashMap, VecDeque};

/// QueueStyle is an enum which can be used as a configuration option for
/// PerValidatorQueue. Since the queue per validator is going to be bounded,
/// QueueStyle also determines the policy for dropping messages.
/// With LIFO, oldest messages are dropped.
/// With FIFO, newest messages are dropped.
#[derive(Clone, Copy)]
pub enum QueueStyle {
    LIFO,
    FIFO,
}

/// PerValidatorQueue maintains a queue of messages per validator. It
/// is a bounded queue per validator and the style (FIFO, LIFO) is
/// configurable. When a new message is added using `push`, it is added to
/// the validator's queue.
/// When `pop` is called, the next message is picked from one
/// of the validator's queue and returned. This happens in a round-robin
/// fashion among validators.
/// If there are no messages, in any of the queues, `None` is returned.
pub struct PerValidatorQueue<T> {
    /// QueueStyle for the messages stored per validator
    queue_style: QueueStyle,
    /// per_validator_queue maintains a map from a Validator to a queue
    /// of all the messages from that Validator. A Validator is
    /// represented by AccountAddress
    per_validator_queue: HashMap<AccountAddress, VecDeque<T>>,
    /// This is a (round-robin)queue of Validators which have pending messages
    /// This queue will be used for performing round robin among
    /// Validators for choosing the next message
    round_robin_queue: VecDeque<AccountAddress>,
    /// Maximum number of messages to store per validator
    max_queue_size: usize,
    dropped_msgs_counter: Option<&'static IntCounter>,
}

impl<T> PerValidatorQueue<T> {
    /// Create a new PerValidatorQueue with the provided QueueStyle and
    /// max_queue_size_per_validator
    pub fn new(
        queue_style: QueueStyle,
        max_queue_size_per_validator: usize,
        dropped_msgs_counter: Option<&'static IntCounter>,
    ) -> Self {
        assert!(
            max_queue_size_per_validator > 0,
            "max_queue_size_per_validator should be > 0"
        );
        Self {
            queue_style,
            max_queue_size: max_queue_size_per_validator,
            per_validator_queue: HashMap::new(),
            round_robin_queue: VecDeque::new(),
            dropped_msgs_counter,
        }
    }

    /// Given a validator, pops the message from its queue and returns the message
    /// It also returns a boolean indicating whether the validators queue is empty
    /// after popping the message
    fn pop_from_validator_queue(&mut self, validator: AccountAddress) -> (Option<T>, bool) {
        if let Some(q) = self.per_validator_queue.get_mut(&validator) {
            // Extract message from the validator's queue
            let retval = match self.queue_style {
                QueueStyle::FIFO => q.pop_front(),
                QueueStyle::LIFO => q.pop_back(),
            };
            (retval, q.is_empty())
        } else {
            (None, true)
        }
    }
}

impl<T> MessageQueue for PerValidatorQueue<T> {
    type Key = AccountAddress;
    type Message = T;

    /// push a message to the appropriate queue in per_validator_queue
    /// add the validator to round_robin_queue if it didnt already exist
    fn push(&mut self, key: Self::Key, message: Self::Message) {
        let max_queue_size = self.max_queue_size;
        let validator_message_queue = self
            .per_validator_queue
            .entry(key)
            .or_insert_with(|| VecDeque::with_capacity(max_queue_size));
        // Add the validator to our round-robin queue if it's not already there
        if validator_message_queue.is_empty() {
            self.round_robin_queue.push_back(key);
        }
        // Push the message to the actual validator message queue
        if validator_message_queue.len() == max_queue_size {
            if let Some(c) = self.dropped_msgs_counter {
                c.inc()
            }
            match self.queue_style {
                // Drop the newest message for FIFO
                QueueStyle::FIFO => (),
                // Drop the oldest message for LIFO
                QueueStyle::LIFO => {
                    validator_message_queue.pop_front();
                    validator_message_queue.push_back(message);
                }
            };
        } else {
            validator_message_queue.push_back(message);
        }
    }

    /// pop a message from the appropriate queue in per_validator_queue
    /// remove the validator from the round_robin_queue if it has no more messages
    fn pop(&mut self) -> Option<Self::Message> {
        let validator = match self.round_robin_queue.pop_front() {
            Some(v) => v,
            _ => {
                return None;
            }
        };
        let (message, is_q_empty) = self.pop_from_validator_queue(validator);
        if !is_q_empty {
            self.round_robin_queue.push_back(validator);
        }
        message
    }
}

use crate::libra_channel::MessageQueue;
use libra_logger::{self, prelude::*};
use libra_types::account_address::AccountAddress;
use std::collections::{HashMap, HashSet, VecDeque};

/// ValidatorMessage trait needs to be implemented by every message which gets pushed into the Libra
/// channel so that we can ensure fairness among validators
pub trait ValidatorMessage {
    /// Extract the Validator from which this message arrived. This
    /// will be used for ensuring fairness among validators.
    fn get_validator(&self) -> AccountAddress;
}

/// QueueStyle is an enum which can be used as a configuration option for
/// PerValidatorQueue. Since the queue per validator is going to be bounded,
/// QueueStyle also determines the policy for dropping messages.
/// With LIFO, oldest messages are dropped.
/// With FIFO, newest messages are dropped.
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
    queue_style: QueueStyle,
    /// per_validator_queue maintains a map from a Validator to a queue
    /// of all the messages from that Validator. A Validator is
    /// represented by AccountAddress
    per_validator_queue: HashMap<AccountAddress, VecDeque<T>>,
    /// This is a list of Validators which have pending messages
    /// This list will be used for performing round robin among
    /// Validators for choosing the next message
    round_robin_queue: VecDeque<AccountAddress>,
    /// Maximum number of messages to store per validator
    max_queue_size: usize,
}

impl<T> PerValidatorQueue<T> {
    /// Create a new PerValidatorQueue with the provided QueueStyle and
    /// max_queue_size_per_validator
    pub fn new(queue_style: QueueStyle, max_queue_size_per_validator: usize) -> Self {
        assert!(
            max_queue_size_per_validator > 0,
            "max_queue_size_per_validator should be > 0"
        );
        Self {
            queue_style,
            max_queue_size: max_queue_size_per_validator,
            per_validator_queue: HashMap::new(),
            round_robin_queue: Vec::new(),
        }
    }
}

impl<T: ValidatorMessage> MessageQueue for PerValidatorQueue<T> {
    type Message = T;

    fn push(&mut self, message: Self::Message) {
        let validator = message.get_validator();
        let validator_message_queue = self.per_validator_queue
            .entry(validator)
            .or_insert_with(|| VecDeque::with_capacity(self.max_queue_size));
        if validator_message_queue.is_empty() {
            self.round_robin_queue.push_back(validator);
        }
        if validator_message_queue.len() == self.max_queue_size {
            match self.queue_style {
                // Drop the newest message for FIFO
                QueueStyle::FIFO => (),
                // Drop the oldest message for LIFO
                QueueStyle::LIFO => {
                    q.pop_front();
                    q.push_back(message);
                }
            };
        } else {
            q.push_back(message);
        }
    }

    fn pop(&mut self) -> Option<Self::Message> {
        let validator;
        if let Some(v) = self.round_robin_queue.pop_front() {
            validator = v;
        } else {
            return None;
        }
        let retval;
        if let Some(q) = self.per_validator_queue.get_mut(&validator) {
            if q.is_empty() {
                crit!("validator: {:?} is present in round_robin_queue, its queue has zero messages", validator);
                return None;
            }
            // Extract message from the validator's queue
            retval = match self.queue_style {
                QueueStyle::FIFO => q.pop_front(),
                QueueStyle::LIFO => q.pop_back(),
            };
            // If the validator has no more messages, remove it from the round robin list:
            // self.round_robin_queue
            if !q.is_empty() {
                self.round_robin_queue.push_back(validator);
            }
        } else {
            crit!("validator: {:?} is present in round_robin_queue, its missing from self.per_validator_queue", validator);
            return None;
        }
        retval
    }
}

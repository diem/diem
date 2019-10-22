use std::collections::{HashMap, HashSet, VecDeque};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::Waker;

use futures::async_await::FusedStream;
use futures::stream::Stream;
use futures::task::Context;
use futures::Poll;
use libra_logger::{self, prelude::*};
use libra_types::account_address::AccountAddress;

/// ValidatorMessage trait needs to be implemented by every message which gets pushed into the Libra
/// channel so that we can ensure fairness among validators
pub trait ValidatorMessage {
    /// Extract the Validator from which this message arrived. This
    /// will be used for ensuring fairness among validators.
    fn get_validator(&self) -> AccountAddress;
}

/// Type is an enum which can be used as a configuration option for
/// libra_channel. Since the queue is going to be bounded, Type also
/// determines the policy for dropping messages.
/// With LIFO, oldest messages are dropped. With FIFO,
/// newest messages are dropped.
pub enum Type {
    LIFO,
    FIFO,
}

/// PerValidatorQueue maintains a queue of messages per validator. It
/// is a bounded queue per validator and the style (FIFO, LIFO) is
/// configurable. When a new message is added using, it is added to
/// the validator's queue.
/// When `get_message` is called, the next message is picked from one
/// of the validator's queue and returned. If there are no messages,
/// in any of the queues None is returned.
struct PerValidatorQueue<T> {
    queue_type: Type,
    per_validator_queue: HashMap<AccountAddress, VecDeque<T>>,
    validators_with_pending_msgs: Vec<AccountAddress>,
    validators_with_pending_msgs_set: HashSet<AccountAddress>,
    max_queue_size: usize,
    next_validator_index: usize,
}

impl<T: ValidatorMessage> PerValidatorQueue<T> {
    /// TODO: Think of how to implement a GC mechanism.
    fn new(queue_type: Type, max_queue_size: usize) -> Self {
        assert!(
            max_queue_size > 0,
            "cannot have a zero-capacity libra_channel"
        );
        Self {
            queue_type,
            per_validator_queue: HashMap::new(),
            validators_with_pending_msgs: Vec::new(),
            validators_with_pending_msgs_set: HashSet::new(),
            max_queue_size,
            next_validator_index: 0,
        }
    }

    fn add_message(&mut self, message: T) {
        let validator = message.get_validator();
        if !self.validators_with_pending_msgs_set.contains(&validator) {
            self.validators_with_pending_msgs_set.insert(validator);
            self.validators_with_pending_msgs.push(validator);
        }
        let max_queue_size = self.max_queue_size;
        self.per_validator_queue
            .entry(validator)
            .or_insert_with(|| VecDeque::with_capacity(max_queue_size));
        if let Some(q) = self.per_validator_queue.get_mut(&validator) {
            if q.len() == self.max_queue_size {
                match self.queue_type {
                    // Drop the newest message for FIFO
                    Type::FIFO => (),
                    // Drop the oldest message for LIFO
                    Type::LIFO => {
                        q.pop_front();
                        q.push_back(message);
                    }
                };
            } else {
                q.push_back(message);
            }
        }
    }

    // TODO: Come up with a better data structure to efficiently
    // do a round robin among validators
    fn get_message(&mut self) -> Option<T> {
        if self.validators_with_pending_msgs_set.is_empty() {
            return None;
        }
        if self.next_validator_index >= self.validators_with_pending_msgs.len() {
            crit!("This should never happen");
            return None;
        }
        let validator = self.validators_with_pending_msgs[self.next_validator_index];
        let retval;
        if let Some(q) = self.per_validator_queue.get_mut(&validator) {
            if q.is_empty() {
                crit!("This should never happen");
                return None;
            }
            retval = match self.queue_type {
                Type::FIFO => q.pop_front(),
                Type::LIFO => q.pop_back(),
            };
            if q.is_empty() {
                self.validators_with_pending_msgs_set.remove(&validator);
                if let Some(last_element) = self.validators_with_pending_msgs.pop() {
                    if last_element != validator {
                        self.validators_with_pending_msgs[self.next_validator_index] = last_element;
                    // next_validator_index remains same here
                    } else {
                        self.next_validator_index = 0;
                    }
                } else {
                    crit!("This should never happen");
                    return None;
                }
            } else {
                self.next_validator_index =
                    (self.next_validator_index + 1) % self.validators_with_pending_msgs.len();
            }
        } else {
            crit!("This should never happen");
            return None;
        }
        retval
    }
}

struct SharedState<T> {
    /// The data stored by this Channel
    /// TODO This needs to be updated to a data structure which can do round robin for validators
    data: PerValidatorQueue<T>,

    /// The thread can use this after calling `put` to tell
    /// `Receiver`'s task to wake up, and process the next item
    waker: Option<Waker>,
}

/// The sending end of the Channel
#[derive(Clone)]
pub struct Sender<T: ValidatorMessage> {
    shared_state: Arc<Mutex<SharedState<T>>>,
}

/// The receiving end of the Channel
pub struct Receiver<T: ValidatorMessage> {
    shared_state: Arc<Mutex<SharedState<T>>>,
}

impl<T: ValidatorMessage> Sender<T> {
    /// TODO: We can have this return a boolean if the queue of a validator is capacity
    /// This puts the data into the internal data structure. This is a non-blocking synchronous call
    pub fn put(&mut self, data: T) {
        let mut shared_state = self.shared_state.lock().unwrap();
        shared_state.data.add_message(data);
        if let Some(w) = shared_state.waker.take() {
            w.wake();
        }
    }
}

impl<T: ValidatorMessage> Stream for Receiver<T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut shared_state = self.shared_state.lock().unwrap();
        if let Some(val) = shared_state.data.get_message() {
            Poll::Ready(Some(val))
        } else {
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl<T: ValidatorMessage> FusedStream for Receiver<T> {
    fn is_terminated(&self) -> bool {
        false
    }
}

/// Create a new Libra Channel and returns the two ends of the channel. The sender end can be cloned making it an mpsc.
pub fn new_channel<T: ValidatorMessage>(
    per_validator_queue_size: usize,
    channel_type: Type,
) -> (Sender<T>, Receiver<T>) {
    let shared_state = Arc::new(Mutex::new(SharedState::<T> {
        data: PerValidatorQueue::new(channel_type, per_validator_queue_size),
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

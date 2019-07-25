mod log_tail;

pub use log_tail::AwsLogTail;

#[derive(Clone, Debug)]
pub struct Commit {
    commit: String,
    round: u64,
    parent: String,
}

#[derive(Debug)]
pub enum Event {
    Commit(Commit),
}

#[derive(Debug)]
pub struct ValidatorEvent {
    validator: String,
    event: Event,
}

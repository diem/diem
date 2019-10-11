pub mod aws;
pub mod cluster;
pub mod deployment;
pub mod effects;
pub mod experiments;
pub mod health;
pub mod instance;
pub mod log_prune;
pub mod prometheus;
pub mod slack;
pub mod suite;
pub mod tx_emitter;

pub mod util {
    use std::time::{Duration, SystemTime};

    pub fn unix_timestamp_now() -> Duration {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("now < UNIX_EPOCH")
    }
}

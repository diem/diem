// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod aws;
pub mod cluster;
pub mod deployment;
pub mod effects;
pub mod experiments;
pub mod github;
pub mod health;
pub mod instance;
pub mod prometheus;
pub mod report;
pub mod slack;
pub mod stats;
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

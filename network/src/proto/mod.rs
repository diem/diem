// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(bare_trait_objects)]
#![allow(clippy::large_enum_variant)]

//! Protobuf definitions for data structures sent over the network
mod consensus {
    include!(concat!(env!("OUT_DIR"), "/consensus.rs"));
}
mod network {
    include!(concat!(env!("OUT_DIR"), "/network.rs"));
}
mod mempool {
    include!(concat!(env!("OUT_DIR"), "/mempool.rs"));
}
mod state_synchronizer {
    include!(concat!(env!("OUT_DIR"), "/state_synchronizer.rs"));
}
mod health_checker {
    include!(concat!(env!("OUT_DIR"), "/health_checker.rs"));
}

pub use self::{
    consensus::ConsensusMsg, health_checker::HealthCheckerMsg, mempool::MempoolSyncMsg,
    network::DiscoveryMsg, state_synchronizer::StateSynchronizerMsg,
};

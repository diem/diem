// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod common;
mod consensus_types;
mod consensusdb;
mod safety;

mod block_storage;
pub mod chained_bft_consensus_provider;
pub use consensus_types::quorum_cert::QuorumCert;
mod chained_bft_smr;
mod network;

pub mod epoch_manager;
pub mod persistent_storage;
mod sync_manager;

#[cfg(test)]
mod chained_bft_smr_test;
#[cfg(test)]
mod network_tests;
#[cfg(test)]
mod proto_test;

#[cfg(any(test, feature = "fuzzing"))]
pub mod test_utils;

#[cfg(not(any(test, feature = "fuzzing")))]
mod liveness;
#[cfg(any(test, feature = "fuzzing"))]
pub mod liveness;

#[cfg(not(feature = "fuzzing"))]
mod event_processor;
#[cfg(feature = "fuzzing")]
pub mod event_processor;

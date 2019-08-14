// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod consensusdb;

mod block_storage;
pub mod chained_bft_consensus_provider;
pub use consensus_types::quorum_cert::QuorumCert;
mod chained_bft_smr;
mod network;

pub mod persistent_storage;
mod sync_manager;

#[cfg(any(test, fuzzing))]
pub mod chained_bft_smr_test;
#[cfg(any(test, fuzzing))]
pub mod event_processor_test;
#[cfg(any(test, fuzzing))]
pub mod network_tests;
#[cfg(test)]
mod proto_test;
#[cfg(any(test, fuzzing))]
pub mod test_utils;

#[cfg(fuzzing)]
pub mod common;
#[cfg(fuzzing)]
pub mod consensus_types;
#[cfg(fuzzing)]
pub mod event_processor;
#[cfg(fuzzing)]
pub mod liveness;
#[cfg(fuzzing)]
pub mod safety;

#[cfg(not(fuzzing))]
mod common;
#[cfg(not(fuzzing))]
mod consensus_types;
#[cfg(not(fuzzing))]
mod event_processor;
#[cfg(not(fuzzing))]
mod liveness;
#[cfg(not(fuzzing))]
mod safety;

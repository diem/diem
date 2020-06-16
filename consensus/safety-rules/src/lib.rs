// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

mod consensus_state;
mod counters;
mod error;
mod persistent_safety_storage;
mod process;
mod remote_service;
mod safety_rules;
mod serializer;
mod t_safety_rules;
mod thread;

pub use crate::{
    consensus_state::ConsensusState,
    counters::COUNTERS,
    error::Error,
    persistent_safety_storage::PersistentSafetyStorage,
    process::{Process, ProcessService},
    remote_service::RemoteService,
    safety_rules::SafetyRules,
    serializer::{SafetyRulesInput, SerializerService},
    t_safety_rules::TSafetyRules,
    thread::ThreadService,
};
use consensus_types::common::Author;
use libra_config::config::NodeConfig;
use libra_secure_storage::{config, Storage};
use std::convert::TryInto;

pub fn extract_service_inputs(config: &mut NodeConfig) -> (Author, PersistentSafetyStorage) {
    let author = config::peer_id(
        config
            .validator_network
            .as_ref()
            .expect("Missing validator network"),
    );

    let backend = &config.consensus.safety_rules.backend;
    let internal_storage: Storage = backend.try_into().expect("Unable to initialize storage");

    let storage = if let Some(test_config) = config.test.as_mut() {
        let private_key = test_config
            .consensus_keypair
            .as_mut()
            .expect("Missing consensus keypair in test config")
            .take_private()
            .expect("Failed to take Consensus private key, key absent or already read");
        let waypoint = config::waypoint(&config.base.waypoint);

        PersistentSafetyStorage::initialize(internal_storage, private_key, waypoint)
    } else {
        PersistentSafetyStorage::new(internal_storage)
    };

    (author, storage)
}

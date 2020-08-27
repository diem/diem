// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

mod consensus_state;
mod counters;
mod error;
mod local_client;
mod logging;
mod persistent_safety_storage;
mod process;
mod remote_service;
mod safety_rules;
mod safety_rules_manager;
mod serializer;
mod t_safety_rules;
mod thread;

pub use crate::{
    consensus_state::ConsensusState, error::Error,
    persistent_safety_storage::PersistentSafetyStorage, process::Process,
    safety_rules::SafetyRules, safety_rules_manager::SafetyRulesManager,
    t_safety_rules::TSafetyRules,
};

#[cfg(any(test, feature = "fuzzing"))]
pub mod fuzzing_utils;

#[cfg(any(test, feature = "fuzzing"))]
pub use crate::{
    safety_rules::fuzzing::{
        fuzz_construct_and_sign_vote, fuzz_initialize, fuzz_sign_proposal, fuzz_sign_timeout,
    },
    serializer::fuzzing::fuzz_handle_message,
};

#[cfg(any(test, feature = "testing"))]
pub mod test_utils;

#[cfg(test)]
mod tests;

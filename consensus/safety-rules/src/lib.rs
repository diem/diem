// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

mod configurable_validator_signer;
mod consensus_state;
mod counters;
mod error;
mod local_client;
mod logging;
mod persistent_safety_storage;
mod process;
mod remote_service;
mod safety_rules;
mod safety_rules_2chain;
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
pub use crate::fuzzing_utils::fuzzing;

#[cfg(any(test, feature = "testing"))]
pub mod test_utils;

#[cfg(test)]
mod tests;

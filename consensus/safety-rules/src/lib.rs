// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

mod consensus_state;
mod counters;
mod error;
mod local_client;
mod persistent_safety_storage;
mod process;
mod remote_service;
mod safety_rules;
mod safety_rules_manager;
mod serializer;
mod spawned_process;
mod t_safety_rules;
mod thread;
mod safety_rules_sgx;
mod safety_rules_sgx_runner;

pub use crate::{
    consensus_state::ConsensusState, counters::COUNTERS, error::Error,
    persistent_safety_storage::PersistentSafetyStorage, process::Process,
    safety_rules::SafetyRules, safety_rules_manager::SafetyRulesManager,
    t_safety_rules::TSafetyRules,
    safety_rules_sgx::SafetyRulesSGX,
};

#[cfg(any(test, feature = "testing"))]
#[path = "process_client_wrapper.rs"]
pub mod process_client_wrapper;

#[cfg(any(test, feature = "testing"))]
#[path = "test_utils.rs"]
pub mod test_utils;

#[cfg(test)]
mod tests;

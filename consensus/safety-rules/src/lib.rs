// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

mod consensus_state;
mod error;
mod local_client;
mod network;
mod persistent_storage;
mod remote;
mod safety_rules;
mod safety_rules_manager;
mod t_safety_rules;
mod thread;

pub use crate::{
    consensus_state::ConsensusState,
    error::Error,
    persistent_storage::{InMemoryStorage, OnDiskStorage},
    safety_rules::SafetyRules,
    safety_rules_manager::{SafetyRulesManager, SafetyRulesManagerConfig},
    t_safety_rules::TSafetyRules,
};

#[cfg(any(test, feature = "testing"))]
#[path = "test_utils.rs"]
pub mod test_utils;

#[cfg(test)]
mod tests;

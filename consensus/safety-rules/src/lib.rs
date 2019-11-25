// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

mod persistent_storage;
mod safety_rules;

pub use crate::{
    persistent_storage::{InMemoryStorage, OnDiskStorage},
    safety_rules::{ConsensusState, Error, SafetyRules},
};

#[cfg(test)]
#[path = "safety_rules_test.rs"]
mod safety_rules_test;

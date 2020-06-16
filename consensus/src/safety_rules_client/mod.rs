// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

mod local_client;
#[cfg(test)]
pub mod process_client_wrapper;
mod remote_client;
pub mod safety_rules_manager;
mod spawned_process;
#[cfg(any(test, feature = "fuzzing"))]
pub mod test_utils;
#[cfg(test)]
mod tests;

pub use safety_rules_manager::SafetyRulesManager;

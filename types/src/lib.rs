// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

pub mod access_path;
pub mod account_address;
pub mod account_config;
pub mod account_state;
pub mod account_state_blob;
pub mod block_info;
pub mod block_metadata;
pub mod byte_array;
pub mod contract_event;
pub mod crypto_proxies;
pub mod discovery_info;
pub mod discovery_set;
pub mod event;
pub mod get_with_proof;
pub mod identifier;
pub mod language_storage;
pub mod ledger_info;
pub mod mempool_status;
pub mod proof;
#[cfg(any(test, feature = "fuzzing"))]
pub mod proptest_types;
pub mod proto;
#[cfg(any(test, feature = "fuzzing"))]
pub mod test_helpers;
pub mod transaction;
pub mod validator_change;
pub mod validator_public_keys;
pub mod validator_set;
pub mod validator_signer;
pub mod validator_verifier;
pub mod vm_error;
pub mod waypoint;
pub mod write_set;

pub use account_address::AccountAddress as PeerId;

#[cfg(test)]
mod unit_tests;

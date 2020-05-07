// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! The purpose of this crate is to offer a single source of truth for the definitions of shared
//! constants within the Libra codebase. This is useful because many different components within
//! Libra often require access to global constant definitions (e.g., Libra Safety Rules,
//! the Key Manager, and Secure Storage). To avoid duplicating these definitions across crates
//! (and better allow these constants to be updated in a single location), we define them here.
#![forbid(unsafe_code)]

/// Definitions of global cryptographic keys (e.g., as held in secure storage)
pub const ASSOCIATION_KEY: &str = "association";
pub const CONSENSUS_KEY: &str = "consensus";
pub const FULLNODE_NETWORK_KEY: &str = "fullnode_network";
pub const OPERATOR_KEY: &str = "operator";
pub const OWNER_KEY: &str = "owner";
pub const VALIDATOR_NETWORK_KEY: &str = "validator_network";

/// Definitions of global data items (e.g., as held in secure storage)
pub const EPOCH: &str = "epoch";
pub const LAST_VOTED_ROUND: &str = "last_voted_round";
pub const PREFERRED_ROUND: &str = "preferred_round";
pub const WAYPOINT: &str = "waypoint";

// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

// <Black magic>
// Increase recursion limit to allow for use of select! macro.
#![recursion_limit = "1024"]
// </Black magic>

// Public exports
#[macro_use]
extern crate prometheus;

pub use common::NetworkPublicKeys;
pub use interface::NetworkProvider;

pub mod error;
pub mod interface;
pub mod peer_manager;
pub mod proto;
pub mod protocols;
pub mod validator_network;

mod common;
mod connectivity_manager;
mod counters;
mod peer;
mod sink;
mod transport;
mod utils;

/// Type for unique identifier associated with each network protocol
pub type ProtocolId = bytes::Bytes;
pub type DisconnectReason = peer::DisconnectReason;
pub type ConnectivityRequest = connectivity_manager::ConnectivityRequest;

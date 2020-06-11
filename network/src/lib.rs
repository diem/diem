// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

// <Black magic>
// Increase recursion limit to allow for use of select! macro.
#![recursion_limit = "1024"]
// </Black magic>

pub use interface::NetworkProvider;

pub mod common;
pub mod connectivity_manager;
pub mod error;
pub mod interface;
pub mod peer_manager;
pub mod protocols;
pub mod validator_network;

pub mod counters;
mod peer;
mod sink;
mod transport;

#[cfg(not(any(feature = "testing", feature = "fuzzing")))]
mod noise;
#[cfg(any(feature = "testing", feature = "fuzzing"))]
pub mod noise;

pub type DisconnectReason = peer::DisconnectReason;
pub type ConnectivityRequest = connectivity_manager::ConnectivityRequest;
pub type ProtocolId = protocols::wire::handshake::v1::ProtocolId;

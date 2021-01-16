// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// <Black magic>
// Increase recursion limit to allow for use of select! macro.
#![recursion_limit = "1024"]
// </Black magic>

// TODO(philiphayes): uncomment when feature stabilizes (est. 1.50.0)
// tracking issue: https://github.com/rust-lang/rust/issues/78835
// #![doc = include_str!("../README.md")]

pub mod connectivity_manager;
pub mod constants;
pub mod counters;
pub mod error;
pub mod logging;
pub mod noise;
pub mod peer;
pub mod peer_manager;
pub mod protocols;
pub mod transport;

#[cfg(feature = "fuzzing")]
pub mod fuzzing;
#[cfg(any(test, feature = "testing", feature = "fuzzing"))]
pub mod testutils;

pub type DisconnectReason = peer::DisconnectReason;
pub type ConnectivityRequest = connectivity_manager::ConnectivityRequest;
pub type ProtocolId = protocols::wire::handshake::v1::ProtocolId;

// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Core types and traits for building Peer to Peer networks.
//!
//! The `netcore` crate contains all of the core functionality needed to build a Peer to Peer
//! network from building `Transport`s and `StreamMultiplexer`s to negotiating protocols on a
//! socket.

pub mod compat;
pub mod framing;
pub mod negotiate;
pub mod transport;

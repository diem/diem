// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Protocols used by network module for external APIs and internal functionality
//!
//! Each protocol corresponds to a certain order of messages
pub mod direct_send;
pub mod rpc;

pub(crate) mod discovery;
pub(crate) mod health_checker;
pub(crate) mod identity;

// Keep this module for now to be used in PeerManager tests
#[cfg(test)]
pub(crate) mod peer_id_exchange;

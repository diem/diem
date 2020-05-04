// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Protocols used by network module for external APIs and internal functionality
//!
//! Each protocol corresponds to a certain order of messages
pub mod direct_send;
pub mod network;
pub mod rpc;

pub mod discovery;
pub mod health_checker;
pub mod identity;
pub mod wire;

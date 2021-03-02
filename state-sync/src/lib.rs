// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! Used to perform catching up between nodes for committed states.
//! Used for node restarts, network partitions, full node syncs
#![recursion_limit = "1024"]

pub mod bootstrapper;
pub mod chunk_request;
pub mod chunk_response;
pub mod client;
pub mod coordinator;
mod counters;
pub mod error;
pub mod executor_proxy;
mod logging;
pub mod network;
mod request_manager;
pub mod shared_components;

#[cfg(any(feature = "fuzzing", test))]
pub mod fuzzing;

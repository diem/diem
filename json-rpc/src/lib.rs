// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! JSON RPC endpoint
//!
//! Used as public API interface for interacting with Full Nodes
//! It serves HTTP API requests from various external clients (such as wallets)
//!
//! Protocol specification: https://www.jsonrpc.org/specification
//!
//! Module organization:
//! ├── methods.rs        # contains all available JSON RPC method handlers
//! ├── views.rs          # custom JSON serializers for Libra data types
//! ├── runtime.rs        # implementation of JSON RPC protocol over HTTP
//! ├── tests.rs          # tests

#[macro_use]
mod util;

#[macro_use]
extern crate prometheus;

mod client;
mod counters;
pub mod errors;
mod methods;
mod runtime;
pub mod views;

pub use client::{JsonRpcAsyncClient, JsonRpcBatch};
pub use runtime::bootstrap_from_config;

#[cfg(any(feature = "fuzzing", test))]
/// Fuzzer for JSON RPC service
pub mod fuzzing;
#[cfg(test)]
mod tests;

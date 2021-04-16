// Copyright (c) The Diem Core Contributors
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
//! ├── runtime.rs        # implementation of JSON RPC protocol over HTTP
//! ├── tests.rs          # tests

#[macro_use]
pub mod util;

mod counters;
pub mod data;
mod methods;
pub mod runtime;

pub use diem_json_rpc_types::{errors, response, views};

pub mod stream_rpc;

pub use runtime::{bootstrap, bootstrap_from_config};

#[cfg(any(feature = "fuzzing", test))]
/// Fuzzer for JSON RPC service
pub mod fuzzing;
#[cfg(any(test, feature = "fuzzing"))]
pub(crate) mod tests;
#[cfg(any(test, feature = "fuzzing"))]
pub use tests::utils::test_bootstrap;

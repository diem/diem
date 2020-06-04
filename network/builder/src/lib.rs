// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

/// The network::builder crate is a single entry point for constructing a network.
/// There should be **NO** dependencies from other network (sub-)crates on network_builder.
/// network_build should be used by broader contexts
pub use network::protocols::rpc::error::RpcError;
pub mod network_builder;

#[cfg(any(feature = "testing", test))]
pub mod dummy;
#[cfg(test)]
mod test;

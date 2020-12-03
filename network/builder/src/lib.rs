// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub use network::protocols::rpc::error::RpcError;
pub mod builder;

// TODO:  This module should be test-only, e.g., #[cfg(any(feature = "testing", test))]
// At present it cannot be because network_builder must be a separate crate and the current
// factoring of DummyNetwork requires use of network_builder.  A holistic review of the
// network directory is needed to break internal circular dependencies.
pub mod dummy;
#[cfg(test)]
mod test;

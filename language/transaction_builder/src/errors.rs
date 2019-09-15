// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub use failure::Error;

/// The common result type used in this crate.
pub type Result<T> = std::result::Result<T, Error>;

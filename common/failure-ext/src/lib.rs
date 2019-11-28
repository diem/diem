// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! A common error handling library for the Libra project.
//!
//! ## Usage
//!
//! // This crate must be imported as 'failure' in order to ensure the
//! // procedural derive macro for the `Fail` trait can function properly.
//! failure = { path = "../common/failure_ext", package = "failure_ext" }
//! // Most of the types and macros you'll need can be found in the prelude.
//! use failure::prelude::*;

pub use anyhow::{anyhow, bail, ensure, format_err, Chain, Context, Error, Result};

// Custom error handling macros are placed in the failure-macros crate. Due to
// the way intra-crate macro exports currently work, macros can't be exported
// from anywhere but the top level when they are defined in the same crate.
pub use libra_failure_macros::{bail_err, unrecoverable};

/// Prelude module containing most commonly used types/macros this crate exports.
pub mod prelude {
    pub use crate::Result;
    pub use anyhow::{anyhow, bail, ensure, format_err, Context, Error};
    pub use libra_failure_macros::{bail_err, unrecoverable};
}

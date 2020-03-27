// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! A library supplying various cryptographic primitives

pub mod bls12381;
pub mod ed25519;
pub mod error;
pub mod hash;
pub mod hkdf;
pub mod multi_ed25519;
pub mod slip0010;
pub mod traits;
pub mod vrf;
pub mod x25519;

#[cfg(test)]
mod unit_tests;

pub mod test_utils;

pub use self::traits::*;
pub use hash::HashValue;

// Reexport once_cell for use in CryptoHasher Derive implementation
#[doc(hidden)]
pub use once_cell as _once_cell;

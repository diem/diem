// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! A library supplying various cryptographic primitives that will be used in the next version.

#![deny(missing_docs)]
#![feature(test)]
#![feature(trait_alias)]

pub mod bls12381;
pub mod ed25519;
pub mod slip0010;
pub mod traits;
pub mod vrf;

#[cfg(test)]
mod unit_tests;

mod test_utils;

pub use crypto::{
    hash::HashValue,
    signing::{PrivateKey, PublicKey, Signature},
};

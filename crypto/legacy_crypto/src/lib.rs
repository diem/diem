// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! A library supplying various cryptographic primitives used in Libra.

#![deny(missing_docs)]
#![feature(test)]

pub mod hash;
pub mod hkdf;
pub mod signing;
pub mod utils;
pub mod x25519;

#[cfg(test)]
extern crate test;

#[cfg(test)]
mod unit_tests;

pub use crate::{
    hash::HashValue,
    signing::{PrivateKey, PublicKey, Signature},
};

// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! A library supplying various cryptographic primitives used in Libra.

#![deny(missing_docs)]

pub mod hash;
pub mod hkdf;

#[cfg(test)]
mod unit_tests;

pub use crate::hash::HashValue;

// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! This feature gets turned on only if libra-crypto is compiled via MIRAI in a nightly build.
#![cfg_attr(mirai, allow(incomplete_features), feature(const_generics))]

//! A library supplying various cryptographic primitives
pub mod compat;
pub mod ed25519;
pub mod error;
pub mod hash;
pub mod hkdf;
pub mod multi_ed25519;
pub mod noise;
pub mod test_utils;
pub mod traits;
pub mod x25519;

#[cfg(test)]
mod unit_tests;

#[cfg(mirai)]
mod tags;

pub use self::traits::*;
pub use hash::HashValue;

// Reexport once_cell and serde_name for use in CryptoHasher Derive implementation.
#[doc(hidden)]
pub use once_cell as _once_cell;
#[doc(hidden)]
pub use serde_name as _serde_name;

// We use [formally verified arithmetic](https://crates.io/crates/fiat-crypto)
// in maintained forks of the dalek suite of libraries ({curve, ed,
// x}25519-dalek). This is controlled by a feature in the forked crates
// ('fiat_u64_backend'), which we turn on by default.  In some contexts
// (e.g. vendored dependencies), where it is difficult to load several versions
// of the same package, we would like to not only not use this code, but not
// even download the forked packages, and rather use the underlying vanilla
// projects from the dalek suite of libraries.  This PR offers this opportunity
// by putting a set of features (fiat / vanilla) in control of the choice of
// dependency.
#[cfg(not(any(feature = "fiat", feature = "vanilla",)))]
compile_error!(
    "no dalek arithmetic backend cargo feature enabled! \
     please enable one of: fiat, vanilla"
);

#[cfg(all(feature = "fiat", feature = "vanilla"))]
compile_error!(
    "at most one dalek arithmetic backend cargo feature should be enabled! \
     please enable one of: fiat, vanilla"
);

// MIRAI's tag analysis makes use of the incomplete const_generics feature, so the module
// containing the definitions of MIRAI tag types should not get compiled in a release build.
// The code below fails a build of the crate if mirai is on but debug_assertions is not.
#[cfg(all(mirai, not(debug_assertions)))]
compile_error!("MIRAI can only be used to compile the crate in a debug build!");

// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! `datatest-stable` is a very simple test harness intended to meet some of the needs provided by
//! the `datatest` crate when using a stable rust compiler without using the `RUSTC_BOOTSTRAP` hack
//! to use nightly features on the stable track.
//!
//! In order to setup data-driven tests for a particular test target you must do the following:
//! 1. Configure the test target by setting the following in the `Cargo.toml`
//! ```lang=toml
//! [[test]]
//! name = "<test target name>"
//! harness = false
//! ```
//!
//! 2. Call the `datatest_stable::harness!(testfn, root, pattern)` macro with the following
//! parameters:
//! * `testfn` - The test function to be executed on each matching input. This function must have
//!   the type `fn(&Path) -> datatest_stable::Result<()>`
//! * `root` - The path to the root directory where the input files live. This path is relative to
//!   the root of the crate.
//! * `pattern` - the regex used to match against and select each file to be tested.
//!
//! The three parameters can be repeated if you have multiple sets of data-driven tests to be run:
//! `datatest_stable::harness!(testfn1, root1, pattern1, testfn2, root2, pattern2)`

mod macros;
mod runner;
pub mod utils;

pub type Result<T> = ::std::result::Result<T, Box<dyn ::std::error::Error>>;

pub use self::runner::{runner, Requirements};

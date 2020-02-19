// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! The core Move VM logic.
//!
//! It is a design goal for the Move VM to be independent of the Libra blockchain, so that
//! other blockchains can use it as well. The VM isn't there yet, but hopefully will be there
//! soon.

#[macro_use]
extern crate mirai_annotations;
#[macro_use]
extern crate rental;

// TODO: Currently we expose a bunch of extra types for cost synthesis via "pub mod". Figure out a
// more principled way to do this.
mod code_cache;
#[macro_use]
mod gas_meter;
pub mod interpreter;
mod interpreter_context;
pub mod loaded_data;
mod move_vm;
mod runtime;
mod special_names;
#[cfg(test)]
mod unit_tests;

pub use move_vm::*;

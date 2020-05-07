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

mod gas_meter;
mod interpreter;
mod loader;
mod move_vm;
mod native_functions;
mod runtime;
#[macro_use]
mod tracing;

pub use move_vm::*;

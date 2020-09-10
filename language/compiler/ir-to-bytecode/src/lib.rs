// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

extern crate log;

pub mod compiler;
mod context;
pub mod errors;
pub mod parser;

// Unit tests for this crate are in the parent "compiler" crate.

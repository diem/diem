// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! Move prover modules

pub mod boogie_helpers;
pub mod bytecode_translator;
pub mod cli;
pub mod driver;
pub mod env;
pub mod spec_translator;

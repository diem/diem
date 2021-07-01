// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

pub mod checker;
pub mod common;
pub mod compiler;
pub mod config;
pub mod errors;
pub mod evaluator;
mod genesis_accounts;
pub mod harness;
pub mod preprocessor;
pub mod testsuite;

#[cfg(test)]
pub mod tests;

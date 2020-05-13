// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Core types for Move.

pub mod account_address;
pub mod gas_schedule;
pub mod identifier;
pub mod language_storage;
pub mod move_resource;
pub mod parser;
#[cfg(any(test, feature = "fuzzing"))]
pub mod proptest_types;
pub mod transaction_argument;
#[cfg(test)]
mod unit_tests;

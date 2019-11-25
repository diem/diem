// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! Test infrastructure for the Libra VM.
//!
//! This crate contains helpers for executing tests against the Libra VM.

use libra_types::{transaction::TransactionStatus, vm_error::VMStatus};

#[cfg(test)]
mod tests;

pub mod account;
pub mod account_universe;
pub mod common_transactions;
pub mod compile;
pub mod data_store;
pub mod executor;
pub mod gas_costs;
mod proptest_types;

pub fn assert_status_eq(s1: &VMStatus, s2: &VMStatus) -> bool {
    assert_eq!(s1.major_status, s2.major_status);
    assert_eq!(s1.sub_status, s2.sub_status);
    true
}

pub fn transaction_status_eq(t1: &TransactionStatus, t2: &TransactionStatus) -> bool {
    match (t1, t2) {
        (TransactionStatus::Discard(s1), TransactionStatus::Discard(s2))
        | (TransactionStatus::Keep(s1), TransactionStatus::Keep(s2)) => assert_status_eq(s1, s2),
        _ => false,
    }
}

#[macro_export]
macro_rules! assert_prologue_parity {
    ($e1:expr, $e2:expr, $e3:expr) => {
        assert_status_eq(&$e1.unwrap(), &$e3);
        assert!(transaction_status_eq($e2, &TransactionStatus::Discard($e3)));
    };
}

#[macro_export]
macro_rules! assert_prologue_disparity {
    ($e1:expr => $e2:expr, $e3:expr => $e4:expr) => {
        assert_eq!($e1, $e2);
        assert!(transaction_status_eq($e3, &$e4));
    };
}

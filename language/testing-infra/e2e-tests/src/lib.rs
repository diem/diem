// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! Test infrastructure for the Diem VM.
//!
//! This crate contains helpers for executing tests against the Diem VM.

use diem_types::{transaction::TransactionStatus, vm_status::KeptVMStatus};

pub mod account;
pub mod account_universe;
pub mod common_transactions;
pub mod compile;
pub mod currencies;
pub mod data_store;
pub mod execution_strategies;
pub mod executor;
pub mod gas_costs;
mod golden_outputs;
pub mod keygen;
mod proptest_types;

pub fn assert_status_eq(s1: &KeptVMStatus, s2: &KeptVMStatus) -> bool {
    assert_eq!(s1, s2);
    true
}

pub fn transaction_status_eq(t1: &TransactionStatus, t2: &TransactionStatus) -> bool {
    match (t1, t2) {
        (TransactionStatus::Discard(s1), TransactionStatus::Discard(s2)) => {
            assert_eq!(s1, s2);
            true
        }
        (TransactionStatus::Keep(s1), TransactionStatus::Keep(s2)) => {
            assert_eq!(s1, s2);
            true
        }
        _ => false,
    }
}

#[macro_export]
macro_rules! assert_prologue_parity {
    ($e1:expr, $e2:expr, $e3:expr) => {
        assert_eq!($e1.unwrap(), $e3);
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

/// Returns the name of the current function. This macro is used to derive the name for the golden
/// file of each test case.
#[macro_export]
macro_rules! current_function_name {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }};
}

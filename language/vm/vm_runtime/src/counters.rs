// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use lazy_static;
use metrics::OpMetrics;
use prometheus::{IntCounter, IntGauge};
use std::convert::TryFrom;
use types::{
    transaction::TransactionStatus,
    vm_error::{VMStatus, VMValidationStatus},
};

// constants used to create counters
const TXN_EXECUTION_KEEP: &str = "txn.execution.keep";
const TXN_EXECUTION_DISCARD: &str = "txn.execution.discard";
const TXN_VERIFICATION_SUCCESS: &str = "txn.verification.success";
const TXN_VERIFICATION_FAIL: &str = "txn.verification.fail";
const TXN_BLOCK_COUNT: &str = "txn.block.count";

lazy_static::lazy_static! {
    // the main metric (move_vm)
    static ref VM_COUNTERS: OpMetrics = OpMetrics::new_and_registered("move_vm");

    static ref VERIFIED_TRANSACTION: IntCounter = VM_COUNTERS.counter(TXN_VERIFICATION_SUCCESS);
    static ref BLOCK_TRANSACTION_COUNT: IntGauge = VM_COUNTERS.gauge(TXN_BLOCK_COUNT);
}

/// Reports the number of transactions in a block.
pub fn report_block_count(count: usize) {
    match i64::try_from(count) {
        Ok(val) => BLOCK_TRANSACTION_COUNT.set(val),
        Err(_) => BLOCK_TRANSACTION_COUNT.set(std::i64::MAX),
    }
}

/// Reports the result of a transaction execution.
///
/// Counters are prefixed with `TXN_EXECUTION_KEEP` or `TXN_EXECUTION_DISCARD`.
/// The prefix can be used with regex to combine different counters in a dashboard.
pub fn report_execution_status(status: &TransactionStatus) {
    match status {
        TransactionStatus::Keep(vm_status) => inc_counter(TXN_EXECUTION_KEEP, vm_status),
        TransactionStatus::Discard(vm_status) => inc_counter(TXN_EXECUTION_DISCARD, vm_status),
    }
}

/// Reports the result of a transaction verification.
///
/// Counters are prefixed with `TXN_VERIFICATION_SUCCESS` or `TXN_VERIFICATION_FAIL`.
/// The prefix can be used with regex to combine different counters in a dashboard.
pub fn report_verification_status(result: &Option<VMStatus>) {
    match result {
        None => VERIFIED_TRANSACTION.inc(),
        Some(status) => inc_counter(TXN_VERIFICATION_FAIL, status),
    }
}

/// Increments one of the counter for verification or execution.
fn inc_counter(prefix: &str, status: &VMStatus) {
    match status {
        VMStatus::Deserialization(_) => {
            // all serialization error are lumped into one bucket
            VM_COUNTERS.inc(&format!("{}.deserialization", prefix));
        }
        VMStatus::Execution(status) => {
            // counters for ExecutionStatus are as granular as the enum
            VM_COUNTERS.inc(&format!("{}.{:?}", prefix, status));
        }
        VMStatus::InvariantViolation(violation) => {
            // counters for VMInvariantViolationError are as granular as the enum
            VM_COUNTERS.inc(&format!("{}.invariant_violation.{:?}", prefix, violation));
        }
        VMStatus::Validation(validation_status) => {
            // counters for validation errors are grouped according to get_validation_status()
            VM_COUNTERS.inc(&format!(
                "{}.validation.{:?}",
                prefix,
                get_validation_status(validation_status)
            ));
        }
        VMStatus::Verification(_) => {
            // all verifier errors are lumped into one bucket
            VM_COUNTERS.inc(&format!("{}.verifier_error", prefix));
        }
    }
}

/// Translate a `VMValidationStatus` enum to a set of strings that are appended to a 'base' counter
/// name.
fn get_validation_status(validation_status: &VMValidationStatus) -> &str {
    match validation_status {
        VMValidationStatus::InvalidSignature => "InvalidSignature",
        VMValidationStatus::InvalidAuthKey => "InvalidAuthKey",
        VMValidationStatus::SequenceNumberTooOld => "SequenceNumberTooOld",
        VMValidationStatus::SequenceNumberTooNew => "SequenceNumberTooNew",
        VMValidationStatus::InsufficientBalanceForTransactionFee => {
            "InsufficientBalanceForTransactionFee"
        }
        VMValidationStatus::TransactionExpired => "TransactionExpired",
        VMValidationStatus::SendingAccountDoesNotExist(_) => "SendingAccountDoesNotExist",
        VMValidationStatus::ExceededMaxTransactionSize(_) => "ExceededMaxTransactionSize",
        VMValidationStatus::UnknownScript => "UnknownScript",
        VMValidationStatus::UnknownModule => "UnknownModule",
        VMValidationStatus::MaxGasUnitsExceedsMaxGasUnitsBound(_)
        | VMValidationStatus::MaxGasUnitsBelowMinTransactionGasUnits(_)
        | VMValidationStatus::GasUnitPriceBelowMinBound(_)
        | VMValidationStatus::GasUnitPriceAboveMaxBound(_) => "GasError",
        VMValidationStatus::RejectedWriteSet | VMValidationStatus::InvalidWriteSet => {
            "WriteSetError"
        }
    }
}

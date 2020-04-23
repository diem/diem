// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_metrics::{IntCounter, IntGauge, OpMetrics};
use libra_types::{
    transaction::TransactionStatus,
    vm_error::{StatusCode, StatusType, VMStatus},
};
use once_cell::sync::Lazy;
use std::{convert::TryFrom, time::Instant};

// constants used to create counters
const TXN_EXECUTION_KEEP: &str = "txn.execution.keep";
const TXN_EXECUTION_DISCARD: &str = "txn.execution.discard";
const TXN_VERIFICATION_SUCCESS: &str = "txn.verification.success";
const TXN_VERIFICATION_FAIL: &str = "txn.verification.fail";
const TXN_BLOCK_COUNT: &str = "txn.block.count";
pub const TXN_TOTAL_TIME_TAKEN: &str = "txn_gas_total_time_taken";
pub const TXN_VERIFICATION_TIME_TAKEN: &str = "txn_gas_verification_time_taken";
pub const TXN_VALIDATION_TIME_TAKEN: &str = "txn_gas_validation_time_taken";
pub const TXN_EXECUTION_TIME_TAKEN: &str = "txn_gas_execution_time_taken";
pub const TXN_PROLOGUE_TIME_TAKEN: &str = "txn_gas_prologue_time_taken";
pub const TXN_EPILOGUE_TIME_TAKEN: &str = "txn_gas_epilogue_time_taken";
pub const TXN_EXECUTION_GAS_USAGE: &str = "txn_gas_execution_gas_usage";
pub const TXN_TOTAL_GAS_USAGE: &str = "txn_gas_total_gas_usage";

// the main metric (move_vm)
pub static VM_COUNTERS: Lazy<OpMetrics> = Lazy::new(|| OpMetrics::new_and_registered("move_vm"));

static VERIFIED_TRANSACTION: Lazy<IntCounter> =
    Lazy::new(|| VM_COUNTERS.counter(TXN_VERIFICATION_SUCCESS));
static BLOCK_TRANSACTION_COUNT: Lazy<IntGauge> = Lazy::new(|| VM_COUNTERS.gauge(TXN_BLOCK_COUNT));

/// Wrapper around time::Instant.
pub fn start_profile() -> Instant {
    Instant::now()
}

/// Reports the number of transactions in a block.
pub fn report_block_count(count: usize) {
    match i64::try_from(count) {
        Ok(val) => BLOCK_TRANSACTION_COUNT.set(val),
        Err(_) => BLOCK_TRANSACTION_COUNT.set(std::i64::MAX),
    }
}

// All statistics gather operations for the time taken/gas usage should go through this macro. This
// gives us the ability to turn these metrics on and off easily from one place.
#[macro_export]
macro_rules! record_stats {
    // Gather some information that is only needed in relation to recording statistics
    (info | $($stmt:stmt);+;) => {
        $($stmt);+;
    };
    // Set the $ident gauge to $amount
    (gauge set | $ident:ident | $amount:expr) => {
        VM_COUNTERS.set($ident, $amount as f64)
    };
    // Increment the $ident gauge by $amount
    (gauge inc | $ident:ident | $amount:expr) => {
        VM_COUNTERS.add($ident, $amount as f64)
    };
    // Decrement the $ident gauge by $amount
    (gauge dec | $ident:ident | $amount:expr) => {
        VM_COUNTERS.sub($ident, $amount as f64)
    };
    // Set the $ident gauge to $amount
    (counter set | $ident:ident | $amount:expr) => {
        VM_COUNTERS.set($ident, $amount as f64)
    };
    // Increment the $ident gauge by $amount
    (counter inc | $ident:ident | $amount:expr) => {
        VM_COUNTERS.add($ident, $amount as f64)
    };
    // Decrement the $ident gauge by $amount
    (counter dec | $ident:ident | $amount:expr) => {
        VM_COUNTERS.sub($ident, $amount as f64)
    };
    // Set the gas histogram for $ident to be $amount.
    (observe | $ident:ident | $amount:expr) => {
        VM_COUNTERS.observe($ident, $amount as f64)
    };
    // Per-block info: time and record the amount of time it took to execute $block under the
    // $ident histogram. NB that this does not provide per-transaction level information, but will
    // only per-block information.
    (time_hist | $ident:ident | $block:block) => {{
        let timer = start_profile();
        let tmp = $block;
        let duration = timer.elapsed();
        VM_COUNTERS.observe_duration($ident, duration);
        tmp
    }};
}

/// Reports the result of a transaction execution.
///
/// Counters are prefixed with `TXN_EXECUTION_KEEP` or `TXN_EXECUTION_DISCARD`.
/// The prefix can be used with regex to combine different counters in a dashboard.
pub fn report_execution_status(status: &TransactionStatus) {
    match status {
        TransactionStatus::Keep(vm_status) => inc_counter(TXN_EXECUTION_KEEP, vm_status),
        TransactionStatus::Discard(vm_status) => inc_counter(TXN_EXECUTION_DISCARD, vm_status),
        TransactionStatus::Retry => (),
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
    match status.status_type() {
        StatusType::Deserialization => {
            // all serialization error are lumped into one bucket
            VM_COUNTERS.inc(&format!("{}.deserialization", prefix));
        }
        StatusType::Execution => {
            // counters for ExecutionStatus are as granular as the enum
            VM_COUNTERS.inc(&format!("{}.{}", prefix, status));
        }
        StatusType::InvariantViolation => {
            // counters for VMInvariantViolationError are as granular as the enum
            VM_COUNTERS.inc(&format!("{}.invariant_violation.{}", prefix, status));
        }
        StatusType::Validation => {
            // counters for validation errors are grouped according to get_validation_status()
            VM_COUNTERS.inc(&format!(
                "{}.validation.{}",
                prefix,
                get_validation_status(status.major_status)
            ));
        }
        StatusType::Verification => {
            // all verifier errors are lumped into one bucket
            VM_COUNTERS.inc(&format!("{}.verifier_error", prefix));
        }
        StatusType::Unknown => {
            VM_COUNTERS.inc(&format!("{}.Unknown", prefix));
        }
    }
}

/// Translate a `VMValidationStatus` enum to a set of strings that are appended to a 'base' counter
/// name.
fn get_validation_status(validation_status: StatusCode) -> &'static str {
    match validation_status {
        StatusCode::INVALID_SIGNATURE => "InvalidSignature",
        StatusCode::INVALID_AUTH_KEY => "InvalidAuthKey",
        StatusCode::SEQUENCE_NUMBER_TOO_OLD => "SequenceNumberTooOld",
        StatusCode::SEQUENCE_NUMBER_TOO_NEW => "SequenceNumberTooNew",
        StatusCode::INSUFFICIENT_BALANCE_FOR_TRANSACTION_FEE => {
            "InsufficientBalanceForTransactionFee"
        }
        StatusCode::TRANSACTION_EXPIRED => "TransactionExpired",
        StatusCode::SENDING_ACCOUNT_DOES_NOT_EXIST => "SendingAccountDoesNotExist",
        StatusCode::EXCEEDED_MAX_TRANSACTION_SIZE => "ExceededMaxTransactionSize",
        StatusCode::UNKNOWN_SCRIPT => "UnknownScript",
        StatusCode::UNKNOWN_MODULE => "UnknownModule",
        StatusCode::MAX_GAS_UNITS_EXCEEDS_MAX_GAS_UNITS_BOUND
        | StatusCode::MAX_GAS_UNITS_BELOW_MIN_TRANSACTION_GAS_UNITS
        | StatusCode::GAS_UNIT_PRICE_BELOW_MIN_BOUND
        | StatusCode::GAS_UNIT_PRICE_ABOVE_MAX_BOUND => "GasError",
        StatusCode::REJECTED_WRITE_SET | StatusCode::INVALID_WRITE_SET => "WriteSetError",
        _ => "UnknownValidationStatus",
    }
}

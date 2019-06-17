// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use lazy_static;
use metrics::OpMetrics;
use prometheus::IntCounter;

lazy_static::lazy_static! {
    pub static ref VM_COUNTERS: OpMetrics = OpMetrics::new_and_registered("move_vm");
}

lazy_static::lazy_static! {
/// Counter of the successfully executed transactions.
pub static ref SUCCESSFUL_TRANSACTION: IntCounter = VM_COUNTERS.counter("txn.execution.success");

/// Counter of the transactions that failed to execute.
pub static ref FAILED_TRANSACTION: IntCounter = VM_COUNTERS.counter("txn.execution.fail");

/// Counter of the successfully verified transactions.
pub static ref VERIFIED_TRANSACTION: IntCounter = VM_COUNTERS.counter("txn.verification.success");

/// Counter of the transactions that failed to verify.
pub static ref UNVERIFIED_TRANSACTION: IntCounter = VM_COUNTERS.counter("txn.verification.fail");
}

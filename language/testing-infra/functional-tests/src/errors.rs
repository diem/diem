// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub use anyhow::{anyhow, bail, format_err, Error, Result};
use diem_types::{transaction::TransactionOutput, vm_status::VMStatus};
use thiserror::Error;

/// Defines all errors in this crate.
#[derive(Clone, Debug, Error)]
pub enum ErrorKind {
    #[error(
        "an error occurred when executing the transaction, vm status {:?}, txn status {:?}",
        .0,
        .1.status(),
    )]
    VMExecutionFailure(VMStatus, TransactionOutput),
    #[error("the transaction was discarded: {0:?}")]
    DiscardedTransaction(TransactionOutput),
    #[error("the checker has failed to match the directives against the output")]
    CheckerFailure,
    // TODO replace VMStatus with VMError
    #[error("VerificationError({0:?})")]
    VerificationError(VMStatus),
    #[error("other error: {0}")]
    #[allow(dead_code)]
    Other(String),
}

// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub use failure::{Error, Result};
use libra_types::{transaction::TransactionOutput, vm_error::VMStatus};
use thiserror::Error;

/// Defines all errors in this crate.
#[derive(Clone, Debug, Error)]
pub enum ErrorKind {
    #[error("an error occurred when executing the transaction")]
    VMExecutionFailure(TransactionOutput),
    #[error("the transaction was discarded")]
    DiscardedTransaction(TransactionOutput),
    #[error("the checker has failed to match the directives against the output")]
    CheckerFailure,
    #[error("verification error {0:?}")]
    VerificationFailure(Vec<VMStatus>),
    #[error("other error: {0}")]
    #[allow(dead_code)]
    Other(String),
}

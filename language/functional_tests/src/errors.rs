// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use failure::{Error, Fail};
use types::transaction::TransactionOutput;
use vm::errors::VerificationError as VMVerificationError;

/// Defines all errors in this crate.
#[derive(Clone, Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "an error occurred when executing the program")]
    VMExecutionFailure(TransactionOutput),
    #[fail(display = "the transaction was discarded")]
    DiscardedTransaction(TransactionOutput),
    #[fail(display = "the checker has failed to match the directives against the output")]
    CheckerFailure,
    #[fail(display = "verification error {:?}", _0)]
    VerificationFailure(Vec<VMVerificationError>),
    #[fail(display = "other error: {}", _0)]
    #[allow(dead_code)]
    Other(String),
}

/// The common result type used in this crate.
pub type Result<T> = std::result::Result<T, Error>;

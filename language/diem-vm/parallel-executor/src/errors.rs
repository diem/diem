// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug)]
pub enum Error<E> {
    /// Invariant violation that happens internally inside of scheduler, usually an indication of
    /// implementation error.
    InvariantViolation,
    /// The inference can't get the read/write set of a transaction, abort the entire execution pipeline.
    InferencerError,
    /// A transaction write to a key that wasn't estimated by the inferencer, abort the execution
    /// because we don't have a good way of handling read-after-write dependency. Will relax this limitation later.
    UnestimatedWrite,
    /// Execution of a thread yields a non-recoverable error, such error will be propagated back to
    /// the caller.
    UserError(E),
}

pub type Result<T, E> = ::std::result::Result<T, Error<E>>;

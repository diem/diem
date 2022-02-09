// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::executor::MVHashMapView;
use anyhow::Result;
use std::{fmt::Debug, hash::Hash};

/// The execution result of a transaction
#[derive(Debug)]
pub enum ExecutionStatus<T, E> {
    /// Transaction was executed successfully.
    Success(T),
    /// Transaction hit a none recoverable error during execution, halt the execution and propagate
    /// the error back to the caller.
    Abort(E),
    /// Transaction was executed successfully, but will skip the execution of the trailing
    /// transactions in the list
    SkipRest(T),
}

/// Trait that defines a transaction that could be parallel executed by the scheduler. Each
/// transaction will write to a key value storage as their side effect.
pub trait Transaction: Sync + Send + 'static {
    type Key: PartialOrd + Send + Sync + Clone + Hash + Eq;
    type Value: Send + Sync;
}

/// Inference result of a transaction.
pub struct Accesses<K> {
    pub keys_read: Vec<K>,
    pub keys_written: Vec<K>,
}

/// Trait for single threaded transaction executor.
// TODO: Sync should not be required. Sync is only introduced because this trait occurs as a phantom type of executor struct.
pub trait ExecutorTask: Sync {
    /// Type of transaction and its associated key and value.
    type T: Transaction;

    /// The output of a transaction. This should contain the side effect of this transaction.
    type Output: TransactionOutput<T = Self::T>;

    /// Type of error when the executor failed to process a transaction and needs to abort.
    type Error: Clone + Send + Sync;

    /// Type to intialize the single thread transaction executor. Copy and Sync are required because
    /// we will create an instance of executor on each individual thread.
    type Argument: Sync + Copy;

    /// Create an instance of the transaction executor.
    fn init(args: Self::Argument) -> Self;

    /// Execute one single transaction given the view of the current state.
    fn execute_transaction(
        &self,
        view: &MVHashMapView<<Self::T as Transaction>::Key, <Self::T as Transaction>::Value>,
        txn: &Self::T,
    ) -> ExecutionStatus<Self::Output, Self::Error>;
}

/// Trait for execution result of a transaction.
pub trait TransactionOutput: Send + Sync {
    /// Type of transaction and its associated key and value.
    type T: Transaction;

    /// Get the side effect of a transaction from its output.
    fn get_writes(
        &self,
    ) -> Vec<(
        <Self::T as Transaction>::Key,
        <Self::T as Transaction>::Value,
    )>;

    /// Execution output for transactions that comes after SkipRest signal.
    fn skip_output() -> Self;
}

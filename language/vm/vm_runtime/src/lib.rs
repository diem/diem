// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! # The VM runtime
//!
//! ## Transaction flow
//!
//! This is the path taken to process a single transaction.
//!
//! ```text
//!                   SignedTransaction
//!                            +
//!                            |
//! +--------------------------|-------------------+
//! | Validate  +--------------+--------------+    |
//! |           |                             |    |
//! |           |       check signature       |    |
//! |           |                             |    |
//! |           +--------------+--------------+    |
//! |                          |                   |
//! |                          |                   |
//! |                          v                   |
//! |           +--------------+--------------+    |
//! |           |                             |    |
//! |           |      check size and gas     |    |
//! |           |                             |    +---------------------------------+
//! |           +--------------+--------------+    |         validation error        |
//! |                          |                   |                                 |
//! |                          |                   |                                 |
//! |                          v                   |                                 |
//! |           +--------------+--------------+    |                                 |
//! |           |                             |    |                                 |
//! |           |         run prologue        |    |                                 |
//! |           |                             |    |                                 |
//! |           +--------------+--------------+    |                                 |
//! |                          |                   |                                 |
//! +--------------------------|-------------------+                                 |
//!                            |                                                     |
//! +--------------------------|-------------------+                                 |
//! |                          v                   |                                 |
//! |  Verify   +--------------+--------------+    |                                 |
//! |           |                             |    |                                 |
//! |           |     deserialize script,     |    |                                 |
//! |           |     verify arguments        |    |                                 |
//! |           |                             |    |                                 |
//! |           +--------------+--------------+    |                                 |
//! |                          |                   |                                 |
//! |                          |                   |                                 v
//! |                          v                   |                    +----------------+------+
//! |           +--------------+--------------+    |                    |                       |
//! |           |                             |    +------------------->+ discard, no write set |
//! |           |     deserialize modules     |    | verification error |                       |
//! |           |                             |    |                    +----------------+------+
//! |           +--------------+--------------+    |                                 ^
//! |                          |                   |                                 |
//! |                          |                   |                                 |
//! |                          v                   |                                 |
//! |           +--------------+--------------+    |                                 |
//! |           |                             |    |                                 |
//! |           | verify scripts and modules  |    |                                 |
//! |           |                             |    |                                 |
//! |           +--------------+--------------+    |                                 |
//! |                          |                   |                                 |
//! +--------------------------|-------------------+                                 |
//!                            |                                                     |
//! +--------------------------|-------------------+                                 |
//! |                          v                   |                                 |
//! | Execute   +--------------+--------------+    |                                 |
//! |           |                             |    |                                 |
//! |           |        execute main         |    |                                 |
//! |           |                             |    |                                 |
//! |           +--------------+--------------+    |                                 |
//! |                          |                   |                                 |
//! |      success or failure  |                   |                                 |
//! |                          v                   |                                 |
//! |           +--------------+--------------+    |                                 |
//! |           |                             |    +---------------------------------+
//! |           |        run epilogue         |    | invariant violation (internal panic)
//! |           |                             |    |
//! |           +--------------+--------------+    |
//! |                          |                   |
//! |                          |                   |
//! |                          v                   |
//! |           +--------------+--------------+    |                    +-----------------------+
//! |           |                             |    | execution failure  |                       |
//! |           |       make write set        +------------------------>+ keep, only charge gas |
//! |           |                             |    |                    |                       |
//! |           +--------------+--------------+    |                    +-----------------------+
//! |                          |                   |
//! +--------------------------|-------------------+
//!                            |
//!                            v
//!             +--------------+--------------+
//!             |                             |
//!             |  keep, transaction executed |
//!             |        + gas charged        |
//!             |                             |
//!             +-----------------------------+
//! ```

#[macro_use]
extern crate vm;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate rental;

mod block_processor;
mod counters;
mod frame;
mod gas_meter;
mod move_vm;
mod process_txn;
mod proptest_types;
mod runtime;
mod value_serializer;

pub mod code_cache;
pub mod data_cache;
pub mod identifier;
pub mod loaded_data;
pub mod txn_executor;
pub mod value;

#[cfg(feature = "instruction_synthesis")]
pub mod execution_stack;
#[cfg(not(feature = "instruction_synthesis"))]
mod execution_stack;

pub use move_vm::MoveVM;
pub use process_txn::verify::static_verify_program;
pub use txn_executor::execute_function;

use config::config::VMConfig;
use state_view::StateView;
use types::{
    transaction::{SignedTransaction, TransactionOutput},
    vm_error::VMStatus,
};
use vm::{errors::VMInvariantViolation, IndexKind};

pub(crate) fn bounded_fetch<T>(
    pool: &[T],
    idx: usize,
    bound_type: IndexKind,
) -> Result<&T, VMInvariantViolation> {
    pool.get(idx)
        .ok_or_else(|| VMInvariantViolation::IndexOutOfBounds(bound_type, pool.len(), idx))
}

/// This trait describes the VM's verification interfaces.
pub trait VMVerifier {
    /// Executes the prologue of the Libra Account and verifies that the transaction is valid.
    /// only. Returns `None` if the transaction was validated, or Some(VMStatus) if the transaction
    /// was unable to be validated with status `VMStatus`.
    fn validate_transaction(
        &self,
        transaction: SignedTransaction,
        state_view: &dyn StateView,
    ) -> Option<VMStatus>;
}

/// This trait describes the VM's execution interface.
pub trait VMExecutor {
    // NOTE: At the moment there are no persistent caches that live past the end of a block (that's
    // why execute_block doesn't take &self.)
    // There are some cache invalidation issues around transactions publishing code that need to be
    // sorted out before that's possible.

    /// Executes a block of transactions and returns output for each one of them.
    fn execute_block(
        transactions: Vec<SignedTransaction>,
        config: &VMConfig,
        state_view: &dyn StateView,
    ) -> Vec<TransactionOutput>;
}

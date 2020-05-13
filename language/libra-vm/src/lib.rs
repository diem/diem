// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

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
extern crate mirai_annotations;
#[macro_use]
mod counters;
pub mod data_cache;

#[cfg(feature = "mirai-contracts")]
pub mod foreign_contracts;

mod libra_vm;
#[cfg(test)]
mod unit_tests;

pub mod system_module_names;

pub use crate::libra_vm::LibraVM;

use libra_state_view::StateView;
use libra_types::{
    transaction::{SignedTransaction, Transaction, TransactionOutput, VMValidatorResult},
    vm_error::VMStatus,
};

/// This trait describes the VM's validation interfaces.
pub trait VMValidator {
    /// Executes the prologue of the Libra Account and verifies that the transaction is valid.
    /// only. Returns `None` if the transaction was validated, or Some(VMStatus) if the transaction
    /// was unable to be validated with status `VMStatus`.
    fn validate_transaction(
        &self,
        transaction: SignedTransaction,
        state_view: &dyn StateView,
    ) -> VMValidatorResult;
}

/// This trait describes the VM's execution interface.
pub trait VMExecutor: Send {
    // NOTE: At the moment there are no persistent caches that live past the end of a block (that's
    // why execute_block doesn't take &self.)
    // There are some cache invalidation issues around transactions publishing code that need to be
    // sorted out before that's possible.

    /// Executes a block of transactions and returns output for each one of them.
    fn execute_block(
        transactions: Vec<Transaction>,
        state_view: &dyn StateView,
    ) -> Result<Vec<TransactionOutput>, VMStatus>;
}

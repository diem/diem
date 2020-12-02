// Copyright (c) The Diem Core Contributors
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
mod access_path_cache;
#[macro_use]
mod counters;
pub mod data_cache;

#[cfg(feature = "mirai-contracts")]
pub mod foreign_contracts;

mod diem_vm;
mod errors;
pub mod transaction_metadata;

pub mod diem_transaction_executor;
pub mod diem_transaction_validator;
pub mod logging;
pub mod system_module_names;

pub use crate::{
    diem_transaction_executor::DiemVM, diem_transaction_validator::DiemVMValidator,
    diem_vm::txn_effects_to_writeset_and_events,
};

use diem_state_view::StateView;
use diem_types::{
    access_path::AccessPath,
    transaction::{SignedTransaction, Transaction, TransactionOutput, VMValidatorResult},
    vm_status::VMStatus,
};
use move_core_types::{
    account_address::AccountAddress,
    language_storage::{ResourceKey, StructTag},
};

/// This trait describes the VM's validation interfaces.
pub trait VMValidator {
    /// Executes the prologue of the Diem Account and verifies that the transaction is valid.
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

/// Get the AccessPath to a resource stored under `address` with type name `tag`
fn create_access_path(address: AccountAddress, tag: StructTag) -> AccessPath {
    let resource_tag = ResourceKey::new(address, tag);
    AccessPath::resource_access_path(resource_tag)
}

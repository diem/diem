// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::counters::CRITICAL_ERRORS;
use libra_crypto::HashValue;
use libra_logger::{prelude::*, Schema};
use libra_state_view::StateViewId;
use libra_types::transaction::Version;
use move_vm_types::logger::Logger;
use serde::Serialize;

#[derive(Schema, Clone)]
pub struct LogSchema {
    name: LogEntry,

    // only one of the next 3 `Option`s will be set. Unless it is in testing mode
    // in which case nothing will be set.
    // Those values are coming from `StateView::id()` and the info carried by
    // `StateViewId`

    // StateViewId::BlockExecution - typical transaction execution
    block_id: Option<HashValue>,
    // StateViewId::ChunkExecution - state sync
    first_version: Option<Version>,
    // StateViewId::TransactionValidation - validation
    base_version: Option<Version>,

    move_vm: u64,

    // transaction position in the list of transactions in the block,
    // 0 if the transaction is not part of a block (i.e. validation).
    txn_id: usize,
}

impl LogSchema {
    pub fn new(view_id: StateViewId, txn_id: usize) -> Self {
        match view_id {
            StateViewId::BlockExecution { block_id } => Self {
                name: LogEntry::Execution,
                block_id: Some(block_id),
                first_version: None,
                base_version: None,
                move_vm: 42,
                txn_id,
            },
            StateViewId::ChunkExecution { first_version } => Self {
                name: LogEntry::Execution,
                block_id: None,
                first_version: Some(first_version),
                base_version: None,
                move_vm: 42,
                txn_id,
            },
            StateViewId::TransactionValidation { base_version } => Self {
                name: LogEntry::Validation,
                block_id: None,
                first_version: None,
                base_version: Some(base_version),
                move_vm: 42,
                txn_id,
            },
            StateViewId::Miscellaneous => Self {
                name: LogEntry::Miscellaneous,
                block_id: None,
                first_version: None,
                base_version: None,
                move_vm: 42,
                txn_id,
            },
        }
    }

    // Restart of an adapter for validation may be done without a `StateView` and so
    // we create a `LogSchema` specific for that
    pub(crate) fn new_for_validation() -> Self {
        Self {
            name: LogEntry::Validation,
            block_id: None,
            first_version: None,
            base_version: None,
            move_vm: 42,
            txn_id: 0,
        }
    }
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LogEntry {
    Execution,
    Validation,
    Miscellaneous, // usually testing
}

#[derive(Clone)]
pub struct LibraLogger {
    schema: LogSchema,
}

impl LibraLogger {
    pub(crate) fn new(view_id: StateViewId, txn_id: usize) -> Self {
        Self {
            schema: LogSchema::new(view_id, txn_id),
        }
    }

    // Restart of an adapter for validation may be done without a `StateView` and so
    // we create a `LogSchema` specific for that
    pub(crate) fn new_for_validation() -> Self {
        Self {
            schema: LogSchema::new_for_validation(),
        }
    }

    pub(crate) fn txn_id(&mut self, idx: usize) {
        self.schema.txn_id = idx;
    }
}

impl Logger for LibraLogger {
    // A critical log for the adapter, increment the `CRITICAL_ERRORS` monitor event so we can
    // hook an alert when a critical occurs
    fn crit(&self, message: &str) {
        CRITICAL_ERRORS.inc();
        error!(self.schema.clone(), "{}", message);
    }

    // REVIEW: consider hooking up a monitor event that we can use with a threshold
    // over a period of time to raise an alert
    fn error(&self, message: &str) {
        error!(self.schema.clone(), "{}", message);
    }

    fn warn(&self, message: &str) {
        warn!(self.schema.clone(), "{}", message);
    }

    fn info(&self, message: &str) {
        info!(self.schema.clone(), "{}", message);
    }

    fn debug(&self, message: &str) {
        debug!(self.schema.clone(), "{}", message);
    }
}

// Helper `Logger` implementation that does nothing
#[derive(Clone)]
pub struct NoLogLogger;

impl Logger for NoLogLogger {
    fn crit(&self, _message: &str) {}
    fn error(&self, _message: &str) {}
    fn warn(&self, _message: &str) {}
    fn info(&self, _message: &str) {}
    fn debug(&self, _message: &str) {}
}

// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::counters::CRITICAL_ERRORS;
use diem_crypto::HashValue;
use diem_logger::Schema;
use diem_state_view::StateViewId;
use diem_types::transaction::Version;
use serde::Serialize;

#[derive(Schema, Clone)]
pub struct AdapterLogSchema {
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

    // transaction position in the list of transactions in the block,
    // 0 if the transaction is not part of a block (i.e. validation).
    txn_id: usize,
}

impl AdapterLogSchema {
    pub fn new(view_id: StateViewId, txn_id: usize) -> Self {
        match view_id {
            StateViewId::BlockExecution { block_id } => Self {
                name: LogEntry::Execution,
                block_id: Some(block_id),
                first_version: None,
                base_version: None,
                txn_id,
            },
            StateViewId::ChunkExecution { first_version } => Self {
                name: LogEntry::Execution,
                block_id: None,
                first_version: Some(first_version),
                base_version: None,
                txn_id,
            },
            StateViewId::TransactionValidation { base_version } => Self {
                name: LogEntry::Validation,
                block_id: None,
                first_version: None,
                base_version: Some(base_version),
                txn_id,
            },
            StateViewId::Miscellaneous => Self {
                name: LogEntry::Miscellaneous,
                block_id: None,
                first_version: None,
                base_version: None,
                txn_id,
            },
        }
    }

    // Increment the `CRITICAL_ERRORS` monitor event that will fire an alert
    pub fn alert(&self) {
        CRITICAL_ERRORS.inc();
    }
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LogEntry {
    Execution,
    Validation,
    Miscellaneous, // usually testing
}

// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines representation of Libra core data structures at physical level via schemas
//! that implement [`schemadb::schema::Schema`].
//!
//! All schemas are `pub(crate)` so not shown in rustdoc, refer to the source code to see details.

pub mod event;
pub mod event_accumulator;
pub mod event_by_key;
pub mod jellyfish_merkle_node;
pub mod ledger_counters;
pub mod ledger_info;
pub mod stale_node_index;
pub mod transaction;
pub mod transaction_accumulator;
pub mod transaction_by_account;
pub mod transaction_info;
pub mod validator;

use failure::prelude::*;
use schemadb::ColumnFamilyName;

pub const EVENT_ACCUMULATOR_CF_NAME: ColumnFamilyName = "event_accumulator";
pub const EVENT_BY_KEY_CF_NAME: ColumnFamilyName = "event_by_key";
pub const EVENT_CF_NAME: ColumnFamilyName = "event";
pub const JELLYFISH_MERKLE_NODE_CF_NAME: ColumnFamilyName = "jellyfish_merkle_node";
pub const LEDGER_COUNTERS_CF_NAME: ColumnFamilyName = "ledger_counters";
pub const STALE_NODE_INDEX_CF_NAME: ColumnFamilyName = "stale_node_index";
pub const TRANSACTION_CF_NAME: ColumnFamilyName = "transaction";
pub const TRANSACTION_ACCUMULATOR_CF_NAME: ColumnFamilyName = "transaction_accumulator";
pub const TRANSACTION_BY_ACCOUNT_CF_NAME: ColumnFamilyName = "transaction_by_account";
pub const TRANSACTION_INFO_CF_NAME: ColumnFamilyName = "transaction_info";
pub const VALIDATOR_CF_NAME: ColumnFamilyName = "validator";

pub fn ensure_slice_len_eq(data: &[u8], len: usize) -> Result<()> {
    ensure!(
        data.len() == len,
        "Unexpected data len {}, expected {}.",
        data.len(),
        len,
    );
    Ok(())
}

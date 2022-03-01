// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines representation of Diem core data structures at physical level via schemas
//! that implement [`schemadb::schema::Schema`].
//!
//! All schemas are `pub(crate)` so not shown in rustdoc, refer to the source code to see details.

pub(crate) mod account_state_blob;
pub(crate) mod bsmt_update_log;
pub(crate) mod epoch_by_version;
pub(crate) mod event;
pub(crate) mod event_accumulator;
pub(crate) mod event_by_key;
pub(crate) mod event_by_version;
pub(crate) mod jellyfish_merkle_node;
pub(crate) mod ledger_counters;
pub(crate) mod ledger_info;
pub(crate) mod stale_node_index;
pub(crate) mod state_tree;
pub(crate) mod transaction;
pub(crate) mod transaction_accumulator;
pub(crate) mod transaction_by_account;
pub(crate) mod transaction_by_hash;
pub(crate) mod transaction_info;
pub(crate) mod write_set;

use anyhow::{ensure, Result};
use schemadb::ColumnFamilyName;

pub const EPOCH_BY_VERSION_CF_NAME: ColumnFamilyName = "epoch_by_version";
pub const EVENT_ACCUMULATOR_CF_NAME: ColumnFamilyName = "event_accumulator";
pub const EVENT_BY_KEY_CF_NAME: ColumnFamilyName = "event_by_key";
pub const EVENT_BY_VERSION_CF_NAME: ColumnFamilyName = "event_by_version";
pub const EVENT_CF_NAME: ColumnFamilyName = "event";
pub const JELLYFISH_MERKLE_NODE_CF_NAME: ColumnFamilyName = "jellyfish_merkle_node";
pub const LEDGER_COUNTERS_CF_NAME: ColumnFamilyName = "ledger_counters";
pub const STALE_NODE_INDEX_CF_NAME: ColumnFamilyName = "stale_node_index";
pub const TRANSACTION_CF_NAME: ColumnFamilyName = "transaction";
pub const TRANSACTION_ACCUMULATOR_CF_NAME: ColumnFamilyName = "transaction_accumulator";
pub const TRANSACTION_BY_ACCOUNT_CF_NAME: ColumnFamilyName = "transaction_by_account";
pub const TRANSACTION_BY_HASH_CF_NAME: ColumnFamilyName = "transaction_by_hash";
pub const TRANSACTION_INFO_CF_NAME: ColumnFamilyName = "transaction_info";
pub const WRITE_SET_CF_NAME: ColumnFamilyName = "write_set";

// FIXME(aldenhu): sort
pub const STATE_TREE_CF_NAME: ColumnFamilyName = "state_tree";
pub const ACCOUNT_STATE_BLOB_CF_NAME: ColumnFamilyName = "account_state_blob";
pub const BSMT_UPDATE_LOG_CF_NAME: ColumnFamilyName = "bsmt_update_log";

fn ensure_slice_len_eq(data: &[u8], len: usize) -> Result<()> {
    ensure!(
        data.len() == len,
        "Unexpected data len {}, expected {}.",
        data.len(),
        len,
    );
    Ok(())
}

fn ensure_slice_len_gt(data: &[u8], len: usize) -> Result<()> {
    ensure!(
        data.len() > len,
        "Unexpected data len {}, expected to be greater than {}.",
        data.len(),
        len,
    );
    Ok(())
}

#[cfg(feature = "fuzzing")]
pub mod fuzzing {
    use schemadb::schema::fuzzing::assert_no_panic_decoding;

    pub fn fuzz_decode(data: &[u8]) {
        #[allow(unused_must_use)]
        {
            assert_no_panic_decoding::<super::epoch_by_version::EpochByVersionSchema>(data);
            assert_no_panic_decoding::<super::event::EventSchema>(data);
            assert_no_panic_decoding::<super::event_accumulator::EventAccumulatorSchema>(data);
            assert_no_panic_decoding::<super::event_by_key::EventByKeySchema>(data);
            assert_no_panic_decoding::<super::event_by_version::EventByVersionSchema>(data);
            assert_no_panic_decoding::<super::jellyfish_merkle_node::JellyfishMerkleNodeSchema>(
                data,
            );
            assert_no_panic_decoding::<super::ledger_counters::LedgerCountersSchema>(data);
            assert_no_panic_decoding::<super::ledger_info::LedgerInfoSchema>(data);
            assert_no_panic_decoding::<super::stale_node_index::StaleNodeIndexSchema>(data);
            assert_no_panic_decoding::<super::transaction::TransactionSchema>(data);
            assert_no_panic_decoding::<super::transaction_accumulator::TransactionAccumulatorSchema>(
                data,
            );
            assert_no_panic_decoding::<super::transaction_by_account::TransactionByAccountSchema>(
                data,
            );
            assert_no_panic_decoding::<super::transaction_by_hash::TransactionByHashSchema>(data);
            assert_no_panic_decoding::<super::transaction_info::TransactionInfoSchema>(data);
            assert_no_panic_decoding::<super::write_set::WriteSetSchema>(data);
        }
    }
}

// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{BlockExecutor, Executor};
use anyhow::{format_err, Result};
use libra_crypto::{hash::PRE_GENESIS_BLOCK_ID, HashValue};
use libra_logger::prelude::*;
use libra_types::{
    block_info::{BlockInfo, GENESIS_EPOCH, GENESIS_ROUND, GENESIS_TIMESTAMP_USECS},
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    transaction::Transaction,
    waypoint::Waypoint,
};
use libra_vm::VMExecutor;
use std::collections::btree_map::BTreeMap;
use storage_interface::{DbReaderWriter, TreeState};

pub fn bootstrap_db_if_empty<V: VMExecutor>(
    db: &DbReaderWriter,
    genesis_txn: &Transaction,
) -> Result<Option<Waypoint>> {
    let tree_state = db.reader.get_latest_tree_state()?;
    if !tree_state.is_empty() {
        return Ok(None);
    }

    Ok(Some(bootstrap_db::<V>(db, tree_state, genesis_txn)?))
}

pub fn bootstrap_db<V: VMExecutor>(
    db: &DbReaderWriter,
    tree_state: TreeState,
    genesis_txn: &Transaction,
) -> Result<Waypoint> {
    let new_genesis_version = tree_state.num_transactions;
    let mut executor = Executor::<V>::new_on_unbootstrapped_db(db.clone(), tree_state);

    let block_id = HashValue::zero();
    let epoch = if new_genesis_version == 0 {
        GENESIS_EPOCH
    } else {
        db.reader.get_latest_ledger_info()?.ledger_info().epoch() + 1
        db.reader.get_account_state_with_proof()
    };

    // Create a block with genesis_txn being the only txn. Execute it then commit it immediately.
    let result =
        executor.execute_block((block_id, vec![genesis_txn.clone()]), *PRE_GENESIS_BLOCK_ID)?;

    let root_hash = result.root_hash();
    let next_validator_set = result
        .validators()
        .as_ref()
        .ok_or_else(|| format_err!("Genesis transaction must emit a validator set."))?;

    let ledger_info_with_sigs = LedgerInfoWithSignatures::new(
        LedgerInfo::new(
            BlockInfo::new(
                epoch,
                GENESIS_ROUND,
                block_id,
                root_hash,
                new_genesis_version,
                GENESIS_TIMESTAMP_USECS,
                Some(next_validator_set.clone()),
            ),
            HashValue::zero(), /* consensus_data_hash */
        ),
        BTreeMap::default(),
    );
    let waypoint = Waypoint::new(ledger_info_with_sigs.ledger_info())?;

    executor.commit_blocks(vec![block_id], ledger_info_with_sigs)?;
    info!(
        "GENESIS transaction is committed with state_id {} and ValidatorSet {}.",
        root_hash, next_validator_set
    );
    // DB bootstrapped, avoid anything that could fail after this.

    Ok(waypoint)
}

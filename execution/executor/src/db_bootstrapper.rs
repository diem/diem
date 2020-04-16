// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{BlockExecutor, Executor};
use anyhow::Result;
use libra_crypto::{hash::PRE_GENESIS_BLOCK_ID, HashValue};
use libra_logger::prelude::*;
use libra_types::{
    ledger_info::LedgerInfoWithSignatures, transaction::Transaction, waypoint::Waypoint,
};
use libra_vm::VMExecutor;
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
    let mut executor = Executor::<V>::new_on_unbootstrapped_db(db.clone(), tree_state);

    let block_id = HashValue::zero(); // match with the id in BlockInfo::genesis(...)

    // Create a block with genesis_txn being the only txn. Execute it then commit it immediately.
    let result =
        executor.execute_block((block_id, vec![genesis_txn.clone()]), *PRE_GENESIS_BLOCK_ID)?;

    let root_hash = result.root_hash();
    let validator_set = result
        .validators()
        .clone()
        .expect("Genesis transaction must emit a validator set.");

    let ledger_info_with_sigs = LedgerInfoWithSignatures::genesis(root_hash, validator_set.clone());
    let waypoint = Waypoint::new(ledger_info_with_sigs.ledger_info())?;

    executor.commit_blocks(vec![HashValue::zero()], ledger_info_with_sigs)?;
    info!(
        "GENESIS transaction is committed with state_id {} and ValidatorSet {}.",
        root_hash, validator_set
    );
    // DB bootstrapped, avoid anything that could fail after this.

    Ok(waypoint)
}

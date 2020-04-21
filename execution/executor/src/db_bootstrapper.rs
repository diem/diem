// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{BlockExecutor, Executor};
use anyhow::{format_err, Result};
use libra_crypto::{hash::PRE_GENESIS_BLOCK_ID, HashValue};
use libra_logger::prelude::*;
use libra_state_view::StateView;
use libra_types::{
    access_path::AccessPath,
    account_config::association_address,
    block_info::{BlockInfo, GENESIS_ROUND},
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    libra_timestamp::LibraTimestampResource,
    move_resource::MoveResource,
    transaction::Transaction,
    waypoint::Waypoint,
};
use libra_vm::VMExecutor;
use std::collections::btree_map::BTreeMap;
use storage_interface::{state_view::VerifiedStateView, DbReaderWriter, TreeState};

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
    let executed_trees = executor.get_executed_trees(block_id)?;
    let state_view = executor.get_executed_state_view(&executed_trees);

    let epoch = 0;
    let timestamp_usecs = get_libra_timestamp(&state_view)?;
    let ledger_info_with_sigs = LedgerInfoWithSignatures::new(
        LedgerInfo::new(
            BlockInfo::new(
                epoch,
                GENESIS_ROUND,
                block_id,
                root_hash,
                0, /* version */
                timestamp_usecs,
                Some(validator_set.clone()),
            ),
            HashValue::zero(), /* consensus_data_hash */
        ),
        BTreeMap::default(), /* signatures */
    );
    let waypoint = Waypoint::new(ledger_info_with_sigs.ledger_info())?;

    executor.commit_blocks(vec![HashValue::zero()], ledger_info_with_sigs)?;
    info!(
        "GENESIS transaction is committed with state_id {} and ValidatorSet {}.",
        root_hash, validator_set
    );
    // DB bootstrapped, avoid anything that could fail after this.

    Ok(waypoint)
}

fn get_libra_timestamp(state_view: &VerifiedStateView) -> Result<u64> {
    let rsrc_bytes = &state_view
        .get(&AccessPath::new(
            association_address(),
            LibraTimestampResource::resource_path(),
        ))?
        .ok_or_else(|| format_err!("LibraTimestamp missing."))?;
    let rsrc = lcs::from_bytes::<LibraTimestampResource>(&rsrc_bytes)?;
    Ok(rsrc.libra_timestamp.microseconds)
}

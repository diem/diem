// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{BlockExecutor, Executor};
use anyhow::{format_err, Result};
use libra_config::config::NodeConfig;
use libra_crypto::{hash::PRE_GENESIS_BLOCK_ID, HashValue};
use libra_logger::prelude::*;
use libra_temppath::TempPath;
use libra_types::{
    ledger_info::LedgerInfoWithSignatures, transaction::Transaction, waypoint::Waypoint,
};
use libra_vm::VMExecutor;
use libradb::LibraDB;
use storage_interface::DbReaderWriter;
use storage_proto::TreeState;

pub fn bootstrap_db_if_empty<V: VMExecutor>(
    db: DbReaderWriter,
    config: &NodeConfig,
) -> Result<Option<Waypoint>> {
    let genesis = config
        .execution
        .genesis
        .as_ref()
        .expect("failed to load genesis txn!")
        .clone();

    bootstrap_db_if_empty_impl::<V>(db, genesis)
}

fn bootstrap_db_if_empty_impl<V: VMExecutor>(
    db: DbReaderWriter,
    genesis: Transaction,
) -> Result<Option<Waypoint>> {
    let tree_state = db.reader.get_latest_tree_state()?;
    if !tree_state.is_empty() {
        return Ok(None);
    }

    Ok(Some(bootstrap_db::<V>(db, tree_state, genesis)?))
}

pub fn bootstrap_db<V: VMExecutor>(
    db: DbReaderWriter,
    tree_state: TreeState,
    genesis: Transaction,
) -> Result<Waypoint> {
    let mut executor = Executor::<V>::new_on_unbootstrapped_db(db, tree_state);

    let block_id = HashValue::zero(); // match with the id in BlockInfo::genesis(...)
                                      // Create a block with genesis being the only txn. Execute it then commit it immediately.
    let result = executor.execute_block((block_id, vec![genesis]), *PRE_GENESIS_BLOCK_ID)?;

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

pub fn compute_genesis_waypoint<V: VMExecutor>(genesis_txn: Transaction) -> Result<Waypoint> {
    let path = TempPath::new();
    let (_, db_rw) = DbReaderWriter::wrap(LibraDB::new(&path.path()));
    bootstrap_db_if_empty_impl::<V>(db_rw, genesis_txn)?
        .ok_or_else(|| format_err!("Failed to bootstrap empty DB."))
}

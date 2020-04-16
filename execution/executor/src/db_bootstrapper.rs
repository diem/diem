// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{BlockExecutor, Executor};
use anyhow::Result;
use libra_config::config::NodeConfig;
use libra_crypto::{hash::PRE_GENESIS_BLOCK_ID, HashValue};
use libra_logger::prelude::*;
use libra_temppath::TempPath;
use libra_types::{
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    transaction::Transaction,
    waypoint::Waypoint,
};
use libra_vm::VMExecutor;
use libradb::LibraDB;
use storage_interface::DbReaderWriter;

pub fn maybe_bootstrap_db<V: VMExecutor>(db: DbReaderWriter, config: &NodeConfig) -> Result<()> {
    let startup_info_opt = db.reader.get_startup_info()?;
    if startup_info_opt.is_some() {
        return Ok(());
    }

    let genesis_txn = config
        .execution
        .genesis
        .as_ref()
        .expect("failed to load genesis txn!")
        .clone();

    execute_and_commit::<V>(db, genesis_txn).map(|_| ())
}

pub fn compute_genesis_waypoint<V: VMExecutor>(genesis: Transaction) -> Result<Waypoint> {
    let path = TempPath::new();
    path.create_as_dir().expect("Unable to create directory");
    let (_, db) = DbReaderWriter::wrap(LibraDB::new(&path.path()));
    let ledger_info = execute_and_commit::<V>(db, genesis)?;
    Waypoint::new(&ledger_info)
}

fn execute_and_commit<V: VMExecutor>(
    db: DbReaderWriter,
    genesis: Transaction,
) -> Result<LedgerInfo> {
    let mut executor = Executor::<V>::new_on_unbootstrapped_db(db);

    let block_id = HashValue::zero(); // match with the id in BlockInfo::genesis(...)
                                      // Create a block with genesis being the only txn. Execute it then commit it immediately.
    let result = executor.execute_block((block_id, vec![genesis]), *PRE_GENESIS_BLOCK_ID)?;

    let root_hash = result.root_hash();
    let validator_set = result
        .validators()
        .clone()
        .expect("Genesis transaction must emit a validator set.");

    let ledger_info_with_sigs = LedgerInfoWithSignatures::genesis(root_hash, validator_set.clone());
    let ledger_info = ledger_info_with_sigs.ledger_info().clone();
    executor.commit_blocks(vec![HashValue::zero()], ledger_info_with_sigs)?;
    info!(
        "GENESIS transaction is committed with state_id {} and ValidatorSet {}.",
        root_hash, validator_set
    );
    Ok(ledger_info)
}

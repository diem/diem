// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::Executor;
use anyhow::Result;
use libra_config::config::NodeConfig;
use libra_crypto::{hash::PRE_GENESIS_BLOCK_ID, HashValue};
use libra_logger::prelude::*;
use libra_types::ledger_info::LedgerInfoWithSignatures;
use libra_vm::VMExecutor;
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
        .expect("failed to load genesis transaction!")
        .clone();

    let mut executor = Executor::<V>::new_on_unbootstrapped_db(db);

    // Create a block with genesis_txn being the only transaction. Execute it then commit it
    // immediately.
    let result = executor.execute_block(
        (
            HashValue::zero(), // match with the id in BlockInfo::genesis(...)
            vec![genesis_txn],
        ),
        *PRE_GENESIS_BLOCK_ID,
    )?;

    let root_hash = result.root_hash();
    let validator_set = result
        .validators()
        .clone()
        .expect("Genesis transaction must emit a validator set.");

    let ledger_info_with_sigs = LedgerInfoWithSignatures::genesis(root_hash, validator_set.clone());
    executor.commit_blocks(vec![HashValue::zero()], ledger_info_with_sigs)?;
    info!(
        "GENESIS transaction is committed with state_id {} and ValidatorSet {}.",
        root_hash, validator_set
    );
    Ok(())
}

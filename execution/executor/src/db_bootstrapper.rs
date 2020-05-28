// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::Executor;
use anyhow::{ensure, format_err, Result};
use executor_types::BlockExecutor;
use libra_crypto::{hash::PRE_GENESIS_BLOCK_ID, HashValue};
use libra_logger::prelude::*;
use libra_state_view::StateView;
use libra_types::{
    access_path::AccessPath,
    account_config::association_address,
    block_info::{BlockInfo, GENESIS_EPOCH, GENESIS_ROUND, GENESIS_TIMESTAMP_USECS},
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    libra_timestamp::LibraTimestampResource,
    on_chain_config::{config_address, ConfigurationResource},
    transaction::Transaction,
    waypoint::Waypoint,
};
use libra_vm::VMExecutor;
use move_core_types::move_resource::MoveResource;
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

    let committer = calculate_genesis::<V>(db, tree_state, genesis_txn)?;
    let waypoint = committer.waypoint;
    committer.commit()?;
    Ok(Some(waypoint))
}

pub struct GenesisCommitter<V: VMExecutor> {
    executor: Executor<V>,
    ledger_info_with_sigs: LedgerInfoWithSignatures,
    waypoint: Waypoint,
}

impl<V: VMExecutor> GenesisCommitter<V> {
    pub fn new(
        executor: Executor<V>,
        ledger_info_with_sigs: LedgerInfoWithSignatures,
    ) -> Result<Self> {
        let waypoint = Waypoint::new_epoch_boundary(ledger_info_with_sigs.ledger_info())?;

        Ok(Self {
            executor,
            ledger_info_with_sigs,
            waypoint,
        })
    }

    pub fn ledger_info_with_sigs(&self) -> &LedgerInfoWithSignatures {
        &self.ledger_info_with_sigs
    }

    pub fn waypoint(&self) -> Waypoint {
        self.waypoint
    }

    pub fn commit(mut self) -> Result<()> {
        self.executor
            .commit_blocks(vec![genesis_block_id()], self.ledger_info_with_sigs)?;
        info!("Genesis commited.");
        // DB bootstrapped, avoid anything that could fail after this.

        Ok(())
    }
}

pub fn calculate_genesis<V: VMExecutor>(
    db: &DbReaderWriter,
    tree_state: TreeState,
    genesis_txn: &Transaction,
) -> Result<GenesisCommitter<V>> {
    // DB bootstrapper works on either an empty transaction accumulator or an existing block chain.
    // In the very exetreme and sad situation of losing quorum among validators, we refer to the
    // second use case said above.
    let genesis_version = tree_state.num_transactions;
    let mut executor = Executor::<V>::new_on_unbootstrapped_db(db.clone(), tree_state);

    let block_id = HashValue::zero();
    let epoch = if genesis_version == 0 {
        GENESIS_EPOCH
    } else {
        let executor_trees = executor.get_executed_trees(*PRE_GENESIS_BLOCK_ID)?;
        let state_view = executor.get_executed_state_view(&executor_trees);
        get_state_epoch(&state_view)?
    };

    // Create a block with genesis_txn being the only txn. Execute it then commit it immediately.
    let result =
        executor.execute_block((block_id, vec![genesis_txn.clone()]), *PRE_GENESIS_BLOCK_ID)?;

    let root_hash = result.root_hash();
    let next_epoch_state = result
        .epoch_state()
        .as_ref()
        .ok_or_else(|| format_err!("Genesis transaction must emit a epoch change."))?;
    let executed_trees = executor.get_executed_trees(block_id)?;
    let state_view = executor.get_executed_state_view(&executed_trees);
    let timestamp_usecs = if genesis_version == 0 {
        // TODO(aldenhu): fix existing tests before using real timestamp and check on-chain epoch.
        GENESIS_TIMESTAMP_USECS
    } else {
        ensure!(
            epoch + 1 == get_state_epoch(&state_view)?,
            "Genesis txn didn't bump epoch."
        );
        get_state_timestamp(&state_view)?
    };

    let ledger_info_with_sigs = LedgerInfoWithSignatures::new(
        LedgerInfo::new(
            BlockInfo::new(
                epoch,
                GENESIS_ROUND,
                block_id,
                root_hash,
                genesis_version,
                timestamp_usecs,
                Some(next_epoch_state.clone()),
            ),
            HashValue::zero(), /* consensus_data_hash */
        ),
        BTreeMap::default(), /* signatures */
    );

    let committer = GenesisCommitter::new(executor, ledger_info_with_sigs)?;
    info!(
        "Genesis calculated: ledger_info_with_sigs {:?}, waypoint {:?}",
        committer.ledger_info_with_sigs, committer.waypoint,
    );
    Ok(committer)
}

fn get_state_timestamp(state_view: &VerifiedStateView) -> Result<u64> {
    let rsrc_bytes = &state_view
        .get(&AccessPath::new(
            association_address(),
            LibraTimestampResource::resource_path(),
        ))?
        .ok_or_else(|| format_err!("LibraTimestampResource missing."))?;
    let rsrc = lcs::from_bytes::<LibraTimestampResource>(&rsrc_bytes)?;
    Ok(rsrc.libra_timestamp.microseconds)
}

fn get_state_epoch(state_view: &VerifiedStateView) -> Result<u64> {
    let rsrc_bytes = &state_view
        .get(&AccessPath::new(
            config_address(),
            ConfigurationResource::resource_path(),
        ))?
        .ok_or_else(|| format_err!("ConfigurationResource missing."))?;
    let rsrc = lcs::from_bytes::<ConfigurationResource>(&rsrc_bytes)?;
    Ok(rsrc.epoch())
}

fn genesis_block_id() -> HashValue {
    HashValue::zero()
}

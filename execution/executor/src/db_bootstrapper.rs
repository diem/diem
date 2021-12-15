// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::components::{apply_chunk_output::IntoLedgerView, chunk_output::ChunkOutput};
use anyhow::{anyhow, ensure, format_err, Result};
use diem_crypto::HashValue;
use diem_logger::prelude::*;
use diem_state_view::{StateView, StateViewId};
use diem_types::{
    access_path::AccessPath,
    account_config::diem_root_address,
    block_info::{BlockInfo, GENESIS_EPOCH, GENESIS_ROUND, GENESIS_TIMESTAMP_USECS},
    diem_timestamp::DiemTimestampResource,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    on_chain_config::{config_address, ConfigurationResource},
    transaction::Transaction,
    waypoint::Waypoint,
};
use diem_vm::VMExecutor;
use executor_types::ExecutedChunk;
use move_core_types::move_resource::MoveResource;
use std::{collections::btree_map::BTreeMap, sync::Arc};
use storage_interface::{state_view::VerifiedStateView, DbReaderWriter, DbWriter, TreeState};

pub fn generate_waypoint<V: VMExecutor>(
    db: &DbReaderWriter,
    genesis_txn: &Transaction,
) -> Result<Waypoint> {
    let tree_state = db.reader.get_latest_tree_state()?;

    let committer = calculate_genesis::<V>(db, tree_state, genesis_txn)?;
    Ok(committer.waypoint)
}

/// If current version + 1 != waypoint.version(), return Ok(false) indicating skipping the txn.
/// otherwise apply the txn and commit it if the result matches the waypoint.
/// Returns Ok(true) if committed otherwise Err.
pub fn maybe_bootstrap<V: VMExecutor>(
    db: &DbReaderWriter,
    genesis_txn: &Transaction,
    waypoint: Waypoint,
) -> Result<bool> {
    let tree_state = db.reader.get_latest_tree_state()?;
    // if the waypoint is not targeted with the genesis txn, it may be either already bootstrapped, or
    // aiming for state sync to catch up.
    if tree_state.num_transactions != waypoint.version() {
        info!(waypoint = %waypoint, "Skip genesis txn.");
        return Ok(false);
    }

    let committer = calculate_genesis::<V>(db, tree_state, genesis_txn)?;
    ensure!(
        waypoint == committer.waypoint(),
        "Waypoint verification failed. Expected {:?}, got {:?}.",
        waypoint,
        committer.waypoint(),
    );
    committer.commit()?;
    Ok(true)
}

pub struct GenesisCommitter {
    db: Arc<dyn DbWriter>,
    output: ExecutedChunk,
    waypoint: Waypoint,
}

impl GenesisCommitter {
    pub fn new(db: Arc<dyn DbWriter>, output: ExecutedChunk) -> Result<Self> {
        let ledger_info = output
            .ledger_info
            .as_ref()
            .ok_or_else(|| anyhow!("LedgerInfo missing."))?
            .ledger_info();
        let waypoint = Waypoint::new_epoch_boundary(ledger_info)?;

        Ok(Self {
            db,
            output,
            waypoint,
        })
    }

    pub fn waypoint(&self) -> Waypoint {
        self.waypoint
    }

    pub fn commit(self) -> Result<()> {
        self.db.save_transactions(
            &self.output.transactions_to_commit()?,
            self.output.result_view.txn_accumulator().version(),
            self.output.ledger_info.as_ref(),
        )?;
        info!("Genesis commited.");
        // DB bootstrapped, avoid anything that could fail after this.

        Ok(())
    }
}

pub fn calculate_genesis<V: VMExecutor>(
    db: &DbReaderWriter,
    tree_state: TreeState,
    genesis_txn: &Transaction,
) -> Result<GenesisCommitter> {
    // DB bootstrapper works on either an empty transaction accumulator or an existing block chain.
    // In the very extreme and sad situation of losing quorum among validators, we refer to the
    // second use case said above.
    let genesis_version = tree_state.num_transactions;
    let base_view = tree_state.into_ledger_view(&db.reader)?;
    let base_state_view =
        base_view.state_view(&base_view, StateViewId::Miscellaneous, db.reader.clone());

    let epoch = if genesis_version == 0 {
        GENESIS_EPOCH
    } else {
        get_state_epoch(&base_state_view)?
    };

    let (mut output, _, _) =
        ChunkOutput::by_transaction_execution::<V>(vec![genesis_txn.clone()], base_state_view)?
            .apply_to_ledger(&base_view)?;
    ensure!(
        !output.to_commit.is_empty(),
        "Genesis txn execution failed."
    );

    let timestamp_usecs = if genesis_version == 0 {
        // TODO(aldenhu): fix existing tests before using real timestamp and check on-chain epoch.
        GENESIS_TIMESTAMP_USECS
    } else {
        let state_view = output.result_view.state_view(
            &base_view,
            StateViewId::Miscellaneous,
            db.reader.clone(),
        );
        let next_epoch = epoch
            .checked_add(1)
            .ok_or_else(|| format_err!("integer overflow occurred"))?;
        ensure!(
            next_epoch == get_state_epoch(&state_view)?,
            "Genesis txn didn't bump epoch."
        );
        get_state_timestamp(&state_view)?
    };
    ensure!(
        output.next_epoch_state.is_some(),
        "Genesis txn didn't output reconfig event."
    );

    let ledger_info_with_sigs = LedgerInfoWithSignatures::new(
        LedgerInfo::new(
            BlockInfo::new(
                epoch,
                GENESIS_ROUND,
                genesis_block_id(),
                output.result_view.txn_accumulator().root_hash(),
                genesis_version,
                timestamp_usecs,
                output.next_epoch_state.clone(),
            ),
            genesis_block_id(), /* consensus_data_hash */
        ),
        BTreeMap::default(), /* signatures */
    );
    output.ledger_info = Some(ledger_info_with_sigs);

    let committer = GenesisCommitter::new(db.writer.clone(), output)?;
    info!(
        "Genesis calculated: ledger_info_with_sigs {:?}, waypoint {:?}",
        &committer.output.ledger_info, committer.waypoint,
    );
    Ok(committer)
}

fn get_state_timestamp(state_view: &VerifiedStateView) -> Result<u64> {
    let rsrc_bytes = &state_view
        .get(&AccessPath::new(
            diem_root_address(),
            DiemTimestampResource::resource_path(),
        ))?
        .ok_or_else(|| format_err!("DiemTimestampResource missing."))?;
    let rsrc = bcs::from_bytes::<DiemTimestampResource>(rsrc_bytes)?;
    Ok(rsrc.diem_timestamp.microseconds)
}

fn get_state_epoch(state_view: &VerifiedStateView) -> Result<u64> {
    let rsrc_bytes = &state_view
        .get(&AccessPath::new(
            config_address(),
            ConfigurationResource::resource_path(),
        ))?
        .ok_or_else(|| format_err!("ConfigurationResource missing."))?;
    let rsrc = bcs::from_bytes::<ConfigurationResource>(rsrc_bytes)?;
    Ok(rsrc.epoch())
}

fn genesis_block_id() -> HashValue {
    HashValue::zero()
}

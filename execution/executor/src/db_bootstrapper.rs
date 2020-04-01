// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::Executor;
use anyhow::Result;
use futures::executor::block_on;
use libra_config::config::NodeConfig;
use libra_crypto::{hash::PRE_GENESIS_BLOCK_ID, HashValue};
use libra_logger::prelude::*;
use libra_types::ledger_info::LedgerInfoWithSignatures;
use libra_vm::VMExecutor;
use std::sync::Arc;
use storage_client::{StorageRead, StorageReadServiceClient, StorageWriteServiceClient};
use storage_proto::StartupInfo;

fn get_startup_info(storage_read_client: Arc<dyn StorageRead>) -> Result<Option<StartupInfo>> {
    let read_client_clone = Arc::clone(&storage_read_client);

    // TODO(aldenhu): remove once we switch to blocking Storage interface.
    let rt = tokio::runtime::Builder::new()
        .threaded_scheduler()
        .enable_all()
        .thread_name("tokio-executor")
        .build()
        .unwrap();
    Ok(block_on(rt.spawn(async move {
        read_client_clone
            .get_startup_info()
            .await
            .expect("Shouldn't fail")
    }))?)
}

pub fn maybe_bootstrap_db<V: VMExecutor>(config: &NodeConfig) -> Result<()> {
    let storage_read_client = Arc::new(StorageReadServiceClient::new(&config.storage.address));

    let startup_info_opt = get_startup_info(storage_read_client.clone())?;
    if startup_info_opt.is_some() {
        return Ok(());
    }

    let genesis_txn = config
        .execution
        .genesis
        .as_ref()
        .expect("failed to load genesis transaction!")
        .clone();
    let storage_write_client = Arc::new(StorageWriteServiceClient::new(&config.storage.address));
    let mut executor =
        Executor::<V>::new_on_unbootstrapped_db(storage_read_client, storage_write_client);

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

// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::SynchronizerState;
use anyhow::{format_err, Result};
use executor::{ExecutedTrees, Executor};
use futures::{Future, FutureExt};
use grpcio::EnvBuilder;
use libra_config::config::NodeConfig;
use libra_types::{
    crypto_proxies::{LedgerInfoWithSignatures, ValidatorChangeEventWithProof},
    transaction::TransactionListWithProof,
};
use std::{pin::Pin, sync::Arc};
use storage_client::{StorageRead, StorageReadServiceClient};
use vm_runtime::MoveVM;

/// Proxies interactions with execution and storage for state synchronization
pub trait ExecutorProxyTrait: Sync + Send {
    /// Sync the local state with the latest in storage.
    fn get_local_storage_state(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<SynchronizerState>> + Send>>;

    /// Execute and commit a batch of transactions
    fn execute_chunk(
        &self,
        txn_list_with_proof: TransactionListWithProof,
        ledger_info_with_sigs: LedgerInfoWithSignatures,
        synced_trees: &mut ExecutedTrees,
    ) -> Result<()>;

    /// Gets chunk of transactions given the known version, target version and the max limit.
    fn get_chunk(
        &self,
        known_version: u64,
        limit: u64,
        target_version: u64,
    ) -> Pin<Box<dyn Future<Output = Result<TransactionListWithProof>> + Send>>;

    fn get_epoch_proof(&self, start_epoch: u64) -> Result<ValidatorChangeEventWithProof>;
}

pub(crate) struct ExecutorProxy {
    storage_read_client: Arc<StorageReadServiceClient>,
    executor: Arc<Executor<MoveVM>>,
}

impl ExecutorProxy {
    pub(crate) fn new(executor: Arc<Executor<MoveVM>>, config: &NodeConfig) -> Self {
        let client_env = Arc::new(EnvBuilder::new().name_prefix("grpc-coord-").build());
        let storage_read_client = Arc::new(StorageReadServiceClient::new(
            client_env,
            &config.storage.address,
            config.storage.port,
        ));
        Self {
            storage_read_client,
            executor,
        }
    }
}

impl ExecutorProxyTrait for ExecutorProxy {
    fn get_local_storage_state(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<SynchronizerState>> + Send>> {
        let client = Arc::clone(&self.storage_read_client);
        async move {
            let storage_info = client
                .get_startup_info_async()
                .await?
                .ok_or_else(|| format_err!("[state sync] Failed to access storage info"))?;
            let synced_trees = if let Some(synced_tree_state) = storage_info.synced_tree_state {
                ExecutedTrees::new(
                    synced_tree_state.account_state_root_hash,
                    synced_tree_state.ledger_frozen_subtree_hashes,
                    synced_tree_state.version + 1,
                )
            } else {
                ExecutedTrees::new(
                    storage_info.committed_tree_state.account_state_root_hash,
                    storage_info
                        .committed_tree_state
                        .ledger_frozen_subtree_hashes,
                    storage_info.committed_tree_state.version + 1,
                )
            };
            let current_verifier = storage_info
                .ledger_info_with_validators
                .ledger_info()
                .next_validator_set()
                .expect("No ValidatorSet found for the start of the epoch")
                .into();
            Ok(SynchronizerState::new(
                storage_info.ledger_info,
                synced_trees,
                current_verifier,
            ))
        }
            .boxed()
    }

    fn execute_chunk(
        &self,
        txn_list_with_proof: TransactionListWithProof,
        ledger_info_with_sigs: LedgerInfoWithSignatures,
        synced_trees: &mut ExecutedTrees,
    ) -> Result<()> {
        self.executor.execute_and_commit_chunk(
            txn_list_with_proof,
            ledger_info_with_sigs,
            synced_trees,
        )
    }

    fn get_chunk(
        &self,
        known_version: u64,
        limit: u64,
        target_version: u64,
    ) -> Pin<Box<dyn Future<Output = Result<TransactionListWithProof>> + Send>> {
        let client = Arc::clone(&self.storage_read_client);
        async move {
            client
                .get_transactions_async(known_version + 1, limit, target_version, false)
                .await
        }
            .boxed()
    }

    fn get_epoch_proof(&self, start_epoch: u64) -> Result<ValidatorChangeEventWithProof> {
        let ledger_info_per_epoch = self
            .storage_read_client
            .get_epoch_change_ledger_infos(start_epoch)?;
        Ok(ValidatorChangeEventWithProof::new(ledger_info_per_epoch))
    }
}

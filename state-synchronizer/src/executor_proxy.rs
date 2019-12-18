// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::SynchronizerState;
use anyhow::{ensure, format_err, Result};
use executor::{ExecutedTrees, Executor};
use grpcio::EnvBuilder;
use libra_config::config::NodeConfig;
use libra_types::{
    crypto_proxies::{LedgerInfoWithSignatures, ValidatorChangeProof},
    transaction::TransactionListWithProof,
};
use std::sync::Arc;
use storage_client::{StorageRead, StorageReadServiceClient};
use vm_runtime::LibraVM;

/// Proxies interactions with execution and storage for state synchronization
#[async_trait::async_trait]
pub trait ExecutorProxyTrait: Sync + Send {
    /// Sync the local state with the latest in storage.
    async fn get_local_storage_state(&self) -> Result<SynchronizerState>;

    /// Execute and commit a batch of transactions
    async fn execute_chunk(
        &self,
        txn_list_with_proof: TransactionListWithProof,
        verified_target_li: LedgerInfoWithSignatures,
        intermediate_end_of_epoch_li: Option<LedgerInfoWithSignatures>,
        synced_trees: &mut ExecutedTrees,
    ) -> Result<()>;

    /// Gets chunk of transactions given the known version, target version and the max limit.
    async fn get_chunk(
        &self,
        known_version: u64,
        limit: u64,
        target_version: u64,
    ) -> Result<TransactionListWithProof>;

    /// Get the epoch change ledger info for [start_epoch, end_epoch) so that we can move to end_epoch.
    async fn get_epoch_proof(
        &self,
        start_epoch: u64,
        end_epoch: u64,
    ) -> Result<ValidatorChangeProof>;

    /// Tries to find a LedgerInfo for a given version.
    async fn get_ledger_info(&self, version: u64) -> Result<LedgerInfoWithSignatures>;
}

pub(crate) struct ExecutorProxy {
    storage_read_client: Arc<StorageReadServiceClient>,
    executor: Arc<Executor<LibraVM>>,
}

impl ExecutorProxy {
    pub(crate) fn new(executor: Arc<Executor<LibraVM>>, config: &NodeConfig) -> Self {
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

#[async_trait::async_trait]
impl ExecutorProxyTrait for ExecutorProxy {
    async fn get_local_storage_state(&self) -> Result<SynchronizerState> {
        let storage_info = self
            .storage_read_client
            .get_startup_info_async()
            .await?
            .ok_or_else(|| format_err!("[state sync] Failed to access storage info"))?;

        let current_verifier = storage_info.get_validator_set().into();

        let synced_trees = if let Some(synced_tree_state) = storage_info.synced_tree_state {
            ExecutedTrees::from(synced_tree_state)
        } else {
            ExecutedTrees::from(storage_info.committed_tree_state)
        };

        Ok(SynchronizerState::new(
            storage_info.latest_ledger_info,
            synced_trees,
            current_verifier,
        ))
    }

    async fn execute_chunk(
        &self,
        txn_list_with_proof: TransactionListWithProof,
        verified_target_li: LedgerInfoWithSignatures,
        intermediate_end_of_epoch_li: Option<LedgerInfoWithSignatures>,
        synced_trees: &mut ExecutedTrees,
    ) -> Result<()> {
        self.executor.execute_and_commit_chunk(
            txn_list_with_proof,
            verified_target_li,
            intermediate_end_of_epoch_li,
            synced_trees,
        )
    }

    async fn get_chunk(
        &self,
        known_version: u64,
        limit: u64,
        target_version: u64,
    ) -> Result<TransactionListWithProof> {
        self.storage_read_client
            .get_transactions_async(known_version + 1, limit, target_version, false)
            .await
    }

    async fn get_epoch_proof(
        &self,
        start_epoch: u64,
        end_epoch: u64,
    ) -> Result<ValidatorChangeProof> {
        let validator_change_proof = self
            .storage_read_client
            .get_epoch_change_ledger_infos(start_epoch, end_epoch)?;
        Ok(validator_change_proof)
    }

    async fn get_ledger_info(&self, version: u64) -> Result<LedgerInfoWithSignatures> {
        let (_, _, li_chain, _) = self
            .storage_read_client
            .update_to_latest_ledger(version, vec![])?;
        let waypoint_li = li_chain
            .ledger_info_with_sigs
            .first()
            .ok_or_else(|| format_err!("No waypoint found for version {}", version))?;
        ensure!(
            waypoint_li.ledger_info().version() == version,
            "Version of Waypoint LI {} is different from requested waypoint version {}",
            waypoint_li.ledger_info().version(),
            version
        );
        Ok(waypoint_li.clone())
    }
}

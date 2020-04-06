// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::SynchronizerState;
use anyhow::{ensure, format_err, Result};
use executor::Executor;
use executor_types::ExecutedTrees;
use itertools::Itertools;
use libra_config::config::NodeConfig;
use libra_types::{
    contract_event::ContractEvent,
    event_subscription::ReconfigSubscription,
    ledger_info::LedgerInfoWithSignatures,
    on_chain_config::{ConfigID, OnChainConfigPayload, ON_CHAIN_CONFIG_REGISTRY},
    transaction::TransactionListWithProof,
    validator_change::ValidatorChangeProof,
};
use libra_vm::LibraVM;
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};
use storage_client::{StorageRead, StorageReadServiceClient};

/// Proxies interactions with execution and storage for state synchronization
#[async_trait::async_trait]
pub trait ExecutorProxyTrait: Sync + Send {
    /// Sync the local state with the latest in storage.
    async fn get_local_storage_state(&self) -> Result<SynchronizerState>;

    /// Execute and commit a batch of transactions
    async fn execute_chunk(
        &mut self,
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

    /// Load all on-chain configs from storage
    /// Note: this method is being exposed as executor proxy trait temporarily because storage read is currently
    /// using the tonic storage read client, which needs the tokio runtime to block on with no runtime/async issues
    /// Once we make storage reads sync (by replacing the storage read client with direct LibraDB),
    /// we can make this entirely internal to `ExecutorProxy`'s initialization procedure
    async fn load_on_chain_configs(&mut self) -> Result<()>;

    /// publishes on-chain config updates to subscribed components
    async fn publish_on_chain_config_updates(&mut self, events: Vec<ContractEvent>) -> Result<()>;
}

pub(crate) struct ExecutorProxy {
    storage_read_client: Arc<StorageReadServiceClient>,
    executor: Arc<Mutex<Executor<LibraVM>>>,
    reconfig_subscriptions: Vec<ReconfigSubscription>,
    on_chain_configs: Arc<HashMap<ConfigID, Vec<u8>>>,
}

impl ExecutorProxy {
    pub(crate) fn new(
        executor: Arc<Mutex<Executor<LibraVM>>>,
        config: &NodeConfig,
        reconfig_subscriptions: Vec<ReconfigSubscription>,
    ) -> Self {
        let storage_read_client = Arc::new(StorageReadServiceClient::new(&config.storage.address));

        let on_chain_configs = Arc::new(HashMap::new());
        Self {
            storage_read_client,
            executor,
            reconfig_subscriptions,
            on_chain_configs,
        }
    }

    // TODO make this into more general trait method in `on_chain_config.rs`
    // once `StorageRead` trait is replaced with `DbReader` and `batch_fetch_config` method is no longer async
    async fn fetch_all_configs(&self) -> Result<HashMap<ConfigID, Vec<u8>>> {
        let access_paths = ON_CHAIN_CONFIG_REGISTRY
            .iter()
            .map(|config_id| config_id.access_path())
            .collect();
        let configs = self
            .storage_read_client
            .batch_fetch_config(access_paths)
            .await?;

        Ok(ON_CHAIN_CONFIG_REGISTRY
            .iter()
            .cloned()
            .zip_eq(configs)
            .collect())
    }
}

#[async_trait::async_trait]
impl ExecutorProxyTrait for ExecutorProxy {
    async fn get_local_storage_state(&self) -> Result<SynchronizerState> {
        let storage_info = self
            .storage_read_client
            .get_startup_info()
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
        &mut self,
        txn_list_with_proof: TransactionListWithProof,
        verified_target_li: LedgerInfoWithSignatures,
        intermediate_end_of_epoch_li: Option<LedgerInfoWithSignatures>,
        _synced_trees: &mut ExecutedTrees,
    ) -> Result<()> {
        let reconfig_events = self.executor.lock().unwrap().execute_and_commit_chunk(
            txn_list_with_proof,
            verified_target_li,
            intermediate_end_of_epoch_li,
        )?;
        self.publish_on_chain_config_updates(reconfig_events).await
    }

    async fn get_chunk(
        &self,
        known_version: u64,
        limit: u64,
        target_version: u64,
    ) -> Result<TransactionListWithProof> {
        self.storage_read_client
            .get_transactions(known_version + 1, limit, target_version, false)
            .await
    }

    async fn get_epoch_proof(
        &self,
        start_epoch: u64,
        end_epoch: u64,
    ) -> Result<ValidatorChangeProof> {
        let validator_change_proof = self
            .storage_read_client
            .get_epoch_change_ledger_infos(start_epoch, end_epoch)
            .await?;
        Ok(validator_change_proof)
    }

    async fn get_ledger_info(&self, version: u64) -> Result<LedgerInfoWithSignatures> {
        let (_, _, li_chain, _) = self
            .storage_read_client
            .update_to_latest_ledger(version, vec![])
            .await?;
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

    async fn load_on_chain_configs(&mut self) -> Result<()> {
        self.on_chain_configs = Arc::new(self.fetch_all_configs().await?);
        Ok(())
    }

    async fn publish_on_chain_config_updates(&mut self, events: Vec<ContractEvent>) -> Result<()> {
        if events.is_empty() {
            return Ok(());
        }

        // calculate deltas
        let new_configs = self.fetch_all_configs().await?;
        let changed_configs = new_configs
            .iter()
            .filter(|(id, cfg)| {
                &self
                    .on_chain_configs
                    .get(id)
                    .expect("missing on-chain config value in local copy")
                    != cfg
            })
            .map(|(id, _)| *id)
            .collect::<HashSet<_>>();

        // notify subscribers
        let new_configs = Arc::new(new_configs);
        let payload = OnChainConfigPayload::new(new_configs.clone());
        for subscription in self.reconfig_subscriptions.iter_mut() {
            // publish updates if *any* of the subscribed configs changed
            if !changed_configs.is_disjoint(&subscription.subscribed_configs()) {
                subscription.publish(payload.clone())?;
            }
        }

        self.on_chain_configs = new_configs;
        Ok(())
    }
}

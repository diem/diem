// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::SynchronizerState;
use anyhow::{ensure, format_err, Result};
use executor::ChunkExecutor;
use executor_types::ExecutedTrees;
use itertools::Itertools;
use libra_types::{
    account_config::association_address,
    account_state::AccountState,
    contract_event::ContractEvent,
    epoch_change::EpochChangeProof,
    ledger_info::LedgerInfoWithSignatures,
    move_resource::MoveStorage,
    on_chain_config::{OnChainConfigPayload, ON_CHAIN_CONFIG_REGISTRY},
    transaction::TransactionListWithProof,
};
use std::{collections::HashSet, convert::TryFrom, sync::Arc};
use storage_interface::DbReader;
use subscription_service::ReconfigSubscription;

/// Proxies interactions with execution and storage for state synchronization
pub trait ExecutorProxyTrait: Send {
    /// Sync the local state with the latest in storage.
    fn get_local_storage_state(&self) -> Result<SynchronizerState>;

    /// Execute and commit a batch of transactions
    fn execute_chunk(
        &mut self,
        txn_list_with_proof: TransactionListWithProof,
        verified_target_li: LedgerInfoWithSignatures,
        intermediate_end_of_epoch_li: Option<LedgerInfoWithSignatures>,
        synced_trees: &mut ExecutedTrees,
    ) -> Result<()>;

    /// Gets chunk of transactions given the known version, target version and the max limit.
    fn get_chunk(
        &self,
        known_version: u64,
        limit: u64,
        target_version: u64,
    ) -> Result<TransactionListWithProof>;

    /// Get the epoch change ledger info for [start_epoch, end_epoch) so that we can move to end_epoch.
    fn get_epoch_proof(&self, start_epoch: u64, end_epoch: u64) -> Result<EpochChangeProof>;

    /// Tries to find a LedgerInfo for a given version.
    fn get_ledger_info(&self, version: u64) -> Result<LedgerInfoWithSignatures>;

    /// Load all on-chain configs from storage
    /// Note: this method is being exposed as executor proxy trait temporarily because storage read is currently
    /// using the tonic storage read client, which needs the tokio runtime to block on with no runtime/async issues
    /// Once we make storage reads sync (by replacing the storage read client with direct LibraDB),
    /// we can make this entirely internal to `ExecutorProxy`'s initialization procedure
    fn load_on_chain_configs(&mut self) -> Result<()>;

    /// publishes on-chain config updates to subscribed components
    fn publish_on_chain_config_updates(&mut self, events: Vec<ContractEvent>) -> Result<()>;
}

pub(crate) struct ExecutorProxy {
    storage: Arc<dyn DbReader>,
    executor: Box<dyn ChunkExecutor>,
    reconfig_subscriptions: Vec<ReconfigSubscription>,
    on_chain_configs: OnChainConfigPayload,
}

impl ExecutorProxy {
    pub(crate) fn new(
        storage: Arc<dyn DbReader>,
        executor: Box<dyn ChunkExecutor>,
        mut reconfig_subscriptions: Vec<ReconfigSubscription>,
    ) -> Self {
        let on_chain_configs = Self::fetch_all_configs(&*storage)
            .expect("[state sync] Failed initial read of on-chain configs");
        for subscription in reconfig_subscriptions.iter_mut() {
            subscription
                .publish(on_chain_configs.clone())
                .expect("[state sync] Failed to publish initial on-chain config");
        }
        Self {
            storage,
            executor,
            reconfig_subscriptions,
            on_chain_configs,
        }
    }

    // TODO make this into more general trait method in `on_chain_config`
    // once `StorageRead` trait is replaced with `DbReader` and `batch_fetch_config` method is no longer async
    fn fetch_all_configs(storage: &dyn DbReader) -> Result<OnChainConfigPayload> {
        let access_paths = ON_CHAIN_CONFIG_REGISTRY
            .iter()
            .map(|config_id| config_id.access_path())
            .collect();
        let configs = storage.batch_fetch_resources(access_paths)?;
        let epoch = storage
            .get_latest_account_state(association_address())?
            .map(|blob| {
                AccountState::try_from(&blob).and_then(|state| {
                    Ok(state
                        .get_configuration_resource()?
                        .ok_or_else(|| format_err!("ConfigurationResource does not exist"))?
                        .epoch())
                })
            })
            .ok_or_else(|| format_err!("Failed to fetch ConfigurationResource"))??;

        Ok(OnChainConfigPayload::new(
            epoch,
            Arc::new(
                ON_CHAIN_CONFIG_REGISTRY
                    .iter()
                    .cloned()
                    .zip_eq(configs)
                    .collect(),
            ),
        ))
    }
}

impl ExecutorProxyTrait for ExecutorProxy {
    fn get_local_storage_state(&self) -> Result<SynchronizerState> {
        let storage_info = self
            .storage
            .get_startup_info()?
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

    fn execute_chunk(
        &mut self,
        txn_list_with_proof: TransactionListWithProof,
        verified_target_li: LedgerInfoWithSignatures,
        intermediate_end_of_epoch_li: Option<LedgerInfoWithSignatures>,
        _synced_trees: &mut ExecutedTrees,
    ) -> Result<()> {
        let reconfig_events = self.executor.execute_and_commit_chunk(
            txn_list_with_proof,
            verified_target_li,
            intermediate_end_of_epoch_li,
        )?;
        self.publish_on_chain_config_updates(reconfig_events)
    }

    fn get_chunk(
        &self,
        known_version: u64,
        limit: u64,
        target_version: u64,
    ) -> Result<TransactionListWithProof> {
        self.storage
            .get_transactions(known_version + 1, limit, target_version, false)
    }

    fn get_epoch_proof(&self, start_epoch: u64, end_epoch: u64) -> Result<EpochChangeProof> {
        let epoch_change_proof = self
            .storage
            .get_epoch_change_ledger_infos(start_epoch, end_epoch)?;
        Ok(epoch_change_proof)
    }

    fn get_ledger_info(&self, version: u64) -> Result<LedgerInfoWithSignatures> {
        let waypoint_li = self.storage.get_ledger_info(version)?;
        ensure!(
            waypoint_li.ledger_info().version() == version,
            "Version of Waypoint LI {} is different from requested waypoint version {}",
            waypoint_li.ledger_info().version(),
            version
        );
        Ok(waypoint_li)
    }

    fn load_on_chain_configs(&mut self) -> Result<()> {
        self.on_chain_configs = Self::fetch_all_configs(&*self.storage)?;
        Ok(())
    }

    fn publish_on_chain_config_updates(&mut self, events: Vec<ContractEvent>) -> Result<()> {
        if events.is_empty() {
            return Ok(());
        }

        // calculate deltas
        let new_configs = Self::fetch_all_configs(&*self.storage)?;
        let changed_configs = new_configs
            .configs()
            .iter()
            .filter(|(id, cfg)| {
                &self
                    .on_chain_configs
                    .configs()
                    .get(id)
                    .expect("missing on-chain config value in local copy")
                    != cfg
            })
            .map(|(id, _)| *id)
            .collect::<HashSet<_>>();

        // notify subscribers
        for subscription in self.reconfig_subscriptions.iter_mut() {
            // publish updates if *any* of the subscribed configs changed
            if !changed_configs.is_disjoint(&subscription.subscribed_configs()) {
                subscription.publish(new_configs.clone())?;
            }
        }

        self.on_chain_configs = new_configs;
        Ok(())
    }
}

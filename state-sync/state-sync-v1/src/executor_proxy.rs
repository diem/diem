// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    counters,
    error::Error,
    logging::{LogEntry, LogEvent, LogSchema},
    shared_components::SyncState,
};
use diem_logger::prelude::*;
use diem_types::{
    account_state::AccountState,
    contract_event::ContractEvent,
    ledger_info::LedgerInfoWithSignatures,
    move_resource::MoveStorage,
    on_chain_config,
    on_chain_config::{config_address, ConfigID, OnChainConfigPayload, ON_CHAIN_CONFIG_REGISTRY},
    transaction::TransactionListWithProof,
};
use executor_types::{ChunkExecutor, ExecutedTrees};
use itertools::Itertools;
use std::{collections::HashSet, convert::TryFrom, sync::Arc};
use storage_interface::DbReader;
use subscription_service::ReconfigSubscription;

/// Proxies interactions with execution and storage for state synchronization
pub trait ExecutorProxyTrait: Send {
    /// Sync the local state with the latest in storage.
    fn get_local_storage_state(&self) -> Result<SyncState, Error>;

    /// Execute and commit a batch of transactions
    fn execute_chunk(
        &mut self,
        txn_list_with_proof: TransactionListWithProof,
        verified_target_li: LedgerInfoWithSignatures,
        intermediate_end_of_epoch_li: Option<LedgerInfoWithSignatures>,
    ) -> Result<(), Error>;

    /// Gets chunk of transactions given the known version, target version and the max limit.
    fn get_chunk(
        &self,
        known_version: u64,
        limit: u64,
        target_version: u64,
    ) -> Result<TransactionListWithProof, Error>;

    /// Get the epoch changing ledger info for the given epoch so that we can move to next epoch.
    fn get_epoch_change_ledger_info(&self, epoch: u64) -> Result<LedgerInfoWithSignatures, Error>;

    /// Get ledger info at an epoch boundary version.
    fn get_epoch_ending_ledger_info(&self, version: u64)
        -> Result<LedgerInfoWithSignatures, Error>;

    /// Returns the ledger's timestamp for the given version in microseconds
    fn get_version_timestamp(&self, version: u64) -> Result<u64, Error>;

    /// publishes on-chain config updates to subscribed components
    fn publish_on_chain_config_updates(&mut self, events: Vec<ContractEvent>) -> Result<(), Error>;
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
        let on_chain_configs = Self::publish_initial_on_chain_configs(
            ON_CHAIN_CONFIG_REGISTRY,
            &*storage,
            &mut reconfig_subscriptions,
        );
        Self {
            storage,
            executor,
            reconfig_subscriptions,
            on_chain_configs,
        }
    }

    #[cfg(test)]
    pub(crate) fn new_for_test(
        storage: Arc<dyn DbReader>,
        executor: Box<dyn ChunkExecutor>,
        mut reconfig_subscriptions: Vec<ReconfigSubscription>,
        config_registry: &[ConfigID],
    ) -> Self {
        let on_chain_configs = Self::publish_initial_on_chain_configs(
            config_registry,
            &*storage,
            &mut reconfig_subscriptions,
        );
        Self {
            storage,
            executor,
            reconfig_subscriptions,
            on_chain_configs,
        }
    }

    fn publish_initial_on_chain_configs(
        config_registry: &[ConfigID],
        storage: &dyn DbReader,
        reconfig_subscriptions: &mut Vec<ReconfigSubscription>,
    ) -> OnChainConfigPayload {
        let on_chain_configs = Self::fetch_all_configs(config_registry, storage)
            .expect("[state sync] Failed initial read of on-chain configs");

        for subscription in reconfig_subscriptions.iter_mut() {
            subscription
                .publish(on_chain_configs.clone())
                .expect("[state sync] Failed to publish initial on-chain config");
        }

        on_chain_configs
    }

    fn fetch_all_configs(
        config_registry: &[ConfigID],
        storage: &dyn DbReader,
    ) -> Result<OnChainConfigPayload, Error> {
        let access_paths = config_registry
            .iter()
            .map(|config_id| config_id.access_path())
            .collect();
        let configs = storage
            .batch_fetch_resources(access_paths)
            .map_err(|error| {
                Error::UnexpectedError(format!("Failed batch fetch of resources: {}", error))
            })?;
        let synced_version = storage.fetch_synced_version().map_err(|error| {
            Error::UnexpectedError(format!("Failed to fetch storage synced version: {}", error))
        })?;

        let account_state_blob = storage
            .get_account_state_with_proof_by_version(config_address(), synced_version)
            .map_err(|error| {
                Error::UnexpectedError(format!(
                    "Failed to fetch account state with proof {}",
                    error
                ))
            })?
            .0;
        let epoch = account_state_blob
            .map(|blob| {
                AccountState::try_from(&blob).and_then(|state| {
                    Ok(state
                        .get_configuration_resource()?
                        .ok_or_else(|| {
                            Error::UnexpectedError("Configuration resource does not exist".into())
                        })?
                        .epoch())
                })
            })
            .ok_or_else(|| Error::UnexpectedError("Missing account state blob".into()))?
            .map_err(|error| {
                Error::UnexpectedError(format!("Failed to fetch configuration resource: {}", error))
            })?;

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
    fn get_local_storage_state(&self) -> Result<SyncState, Error> {
        let storage_info = self.storage.get_startup_info().map_err(|error| {
            Error::UnexpectedError(format!(
                "Failed to get startup info from storage: {}",
                error
            ))
        })?;
        let storage_info = storage_info
            .ok_or_else(|| Error::UnexpectedError("Missing startup info from storage".into()))?;
        let current_epoch_state = storage_info.get_epoch_state().clone();

        let synced_trees = if let Some(synced_tree_state) = storage_info.synced_tree_state {
            ExecutedTrees::from(synced_tree_state)
        } else {
            ExecutedTrees::from(storage_info.committed_tree_state)
        };

        Ok(SyncState::new(
            storage_info.latest_ledger_info,
            synced_trees,
            current_epoch_state,
        ))
    }

    fn execute_chunk(
        &mut self,
        txn_list_with_proof: TransactionListWithProof,
        verified_target_li: LedgerInfoWithSignatures,
        intermediate_end_of_epoch_li: Option<LedgerInfoWithSignatures>,
    ) -> Result<(), Error> {
        // track chunk execution time
        let timer = counters::EXECUTE_CHUNK_DURATION.start_timer();
        let events = self
            .executor
            .execute_and_commit_chunk(
                txn_list_with_proof,
                verified_target_li,
                intermediate_end_of_epoch_li,
            )
            .map_err(|error| {
                Error::UnexpectedError(format!("Execute and commit chunk failed: {}", error))
            })?;
        timer.stop_and_record();
        let reconfig_events = extract_reconfig_events(events);
        if let Err(e) = self.publish_on_chain_config_updates(reconfig_events) {
            error!(
                LogSchema::event_log(LogEntry::Reconfig, LogEvent::Fail).error(&e),
                "Failed to publish reconfig updates in execute_chunk"
            );
            counters::RECONFIG_PUBLISH_COUNT
                .with_label_values(&[counters::FAIL_LABEL])
                .inc();
        }
        Ok(())
    }

    fn get_chunk(
        &self,
        known_version: u64,
        limit: u64,
        target_version: u64,
    ) -> Result<TransactionListWithProof, Error> {
        let starting_version = known_version
            .checked_add(1)
            .ok_or_else(|| Error::IntegerOverflow("Starting version has overflown!".into()))?;
        self.storage
            .get_transactions(starting_version, limit, target_version, false)
            .map_err(|error| {
                Error::UnexpectedError(format!("Failed to get transactions from storage {}", error))
            })
    }

    fn get_epoch_change_ledger_info(&self, epoch: u64) -> Result<LedgerInfoWithSignatures, Error> {
        let next_epoch = epoch
            .checked_add(1)
            .ok_or_else(|| Error::IntegerOverflow("Next epoch has overflown!".into()))?;
        let mut epoch_ending_ledger_infos = self
            .storage
            .get_epoch_ending_ledger_infos(epoch, next_epoch)
            .map_err(|error| Error::UnexpectedError(error.to_string()))?;

        epoch_ending_ledger_infos
            .ledger_info_with_sigs
            .pop()
            .ok_or_else(|| {
                Error::UnexpectedError(format!(
                    "Missing epoch change ledger info for epoch: {:?}",
                    epoch
                ))
            })
    }

    fn get_epoch_ending_ledger_info(
        &self,
        version: u64,
    ) -> Result<LedgerInfoWithSignatures, Error> {
        self.storage
            .get_epoch_ending_ledger_info(version)
            .map_err(|error| Error::UnexpectedError(error.to_string()))
    }

    fn get_version_timestamp(&self, version: u64) -> Result<u64, Error> {
        self.storage
            .get_block_timestamp(version)
            .map_err(|error| Error::UnexpectedError(error.to_string()))
    }

    fn publish_on_chain_config_updates(&mut self, events: Vec<ContractEvent>) -> Result<(), Error> {
        if events.is_empty() {
            return Ok(());
        }
        info!(LogSchema::new(LogEntry::Reconfig)
            .count(events.len())
            .reconfig_events(events.clone()));

        let event_keys = events
            .iter()
            .map(|event| *event.key())
            .collect::<HashSet<_>>();

        // calculate deltas
        let new_configs = Self::fetch_all_configs(ON_CHAIN_CONFIG_REGISTRY, &*self.storage)?;
        let changed_configs = new_configs
            .configs()
            .iter()
            .filter(|(id, cfg)| {
                &self.on_chain_configs.configs().get(id).unwrap_or_else(|| {
                    panic!("Missing on-chain config value in local copy: {}", id)
                }) != cfg
            })
            .map(|(id, _)| *id)
            .collect::<HashSet<_>>();

        // notify subscribers
        let mut publish_success = true;
        for subscription in self.reconfig_subscriptions.iter_mut() {
            // publish updates if *any* of the subscribed configs changed
            // or any of the subscribed events were emitted
            let subscribed_items = subscription.subscribed_items();
            if !changed_configs.is_disjoint(&subscribed_items.configs)
                || !event_keys.is_disjoint(&subscribed_items.events)
            {
                if let Err(e) = subscription.publish(new_configs.clone()) {
                    publish_success = false;
                    error!(
                        LogSchema::event_log(LogEntry::Reconfig, LogEvent::PublishError)
                            .subscription_name(subscription.name.clone())
                            .error(&Error::UnexpectedError(e.to_string())),
                        "Failed to publish reconfig notification to subscription {}",
                        subscription.name
                    );
                } else {
                    info!(
                        LogSchema::event_log(LogEntry::Reconfig, LogEvent::Success)
                            .subscription_name(subscription.name.clone()),
                        "Successfully published reconfig notification to subscription {}",
                        subscription.name
                    );
                }
            }
        }

        self.on_chain_configs = new_configs;
        if publish_success {
            counters::RECONFIG_PUBLISH_COUNT
                .with_label_values(&[counters::SUCCESS_LABEL])
                .inc();
            Ok(())
        } else {
            Err(Error::UnexpectedError(
                "Failed to publish at least one subscription!".into(),
            ))
        }
    }
}

fn extract_reconfig_events(events: Vec<ContractEvent>) -> Vec<ContractEvent> {
    let new_epoch_event_key = on_chain_config::new_epoch_event_key();
    events
        .into_iter()
        .filter(|event| *event.key() == new_epoch_event_key)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use channel::diem_channel::Receiver;
    use diem_crypto::{ed25519::*, PrivateKey, Uniform};
    use diem_transaction_builder::stdlib::{
        encode_peer_to_peer_with_metadata_script,
        encode_set_validator_config_and_reconfigure_script,
        encode_update_diem_consensus_config_script_function, encode_update_diem_version_script,
    };
    use diem_types::{
        account_address::AccountAddress,
        account_config::{diem_root_address, xus_tag},
        block_metadata::BlockMetadata,
        contract_event::ContractEvent,
        ledger_info::LedgerInfoWithSignatures,
        on_chain_config::{
            ConsensusConfigV1, DiemVersion, OnChainConfig, OnChainConfigPayload,
            OnChainConsensusConfig, VMConfig, ValidatorSet,
        },
        transaction::{Transaction, TransactionPayload, WriteSetPayload},
    };
    use diem_vm::DiemVM;
    use diemdb::DiemDB;
    use executor::Executor;
    use executor_test_helpers::{
        bootstrap_genesis, gen_block_id, gen_ledger_info_with_sigs, get_test_signed_transaction,
    };
    use executor_types::BlockExecutor;
    use futures::{future::FutureExt, stream::StreamExt};
    use serde::{Deserialize, Serialize};
    use storage_interface::DbReaderWriter;
    use subscription_service::ReconfigSubscription;
    use vm_genesis::TestValidator;

    // TODO(joshlind): add unit tests for general executor proxy behaviour!
    // TODO(joshlind): add unit tests for subscription events.. seems like these are missing?

    #[test]
    fn test_pub_sub_different_subscription() {
        let (subscription, mut reconfig_receiver) =
            ReconfigSubscription::subscribe_all("", vec![VMConfig::CONFIG_ID], vec![]);
        let (validators, mut block_executor, mut executor_proxy) =
            bootstrap_genesis_and_set_subscription(subscription, &mut reconfig_receiver);

        // Create a dummy prologue transaction that will bump the timer, and update the validator set
        let validator_account = validators[0].data.address;
        let dummy_txn = create_dummy_transaction(1, validator_account);
        let reconfig_txn = create_new_update_diem_version_transaction(1);

        // Execute and commit the block
        let block = vec![dummy_txn, reconfig_txn];
        let (reconfig_events, _) = execute_and_commit_block(&mut block_executor, block, 1);

        // Publish the on chain config updates
        executor_proxy
            .publish_on_chain_config_updates(reconfig_events)
            .unwrap();

        // Verify no reconfig notification is sent (we only subscribed to VMConfig)
        assert!(reconfig_receiver
            .select_next_some()
            .now_or_never()
            .is_none());
    }

    #[test]
    fn test_pub_sub_drop_receiver() {
        let (subscription, mut reconfig_receiver) =
            ReconfigSubscription::subscribe_all("", vec![DiemVersion::CONFIG_ID], vec![]);
        let (validators, mut block_executor, mut executor_proxy) =
            bootstrap_genesis_and_set_subscription(subscription, &mut reconfig_receiver);

        // Create a dummy prologue transaction that will bump the timer, and update the Diem version
        let validator_account = validators[0].data.address;
        let dummy_txn = create_dummy_transaction(1, validator_account);
        let reconfig_txn = create_new_update_diem_version_transaction(1);

        // Execute and commit the reconfig block
        let block = vec![dummy_txn, reconfig_txn];
        let (reconfig_events, _) = execute_and_commit_block(&mut block_executor, block, 1);

        // Drop the reconfig receiver
        drop(reconfig_receiver);

        // Verify publishing on-chain config updates fails due to dropped receiver
        assert!(executor_proxy
            .publish_on_chain_config_updates(reconfig_events)
            .is_err());
    }

    #[test]
    fn test_pub_sub_multiple_subscriptions() {
        let (subscription, mut reconfig_receiver) = ReconfigSubscription::subscribe_all(
            "",
            vec![ValidatorSet::CONFIG_ID, DiemVersion::CONFIG_ID],
            vec![],
        );
        let (validators, mut block_executor, mut executor_proxy) =
            bootstrap_genesis_and_set_subscription(subscription, &mut reconfig_receiver);

        // Create a dummy prologue transaction that will bump the timer, and update the Diem version
        let validator_account = validators[0].data.address;
        let dummy_txn = create_dummy_transaction(1, validator_account);
        let reconfig_txn = create_new_update_diem_version_transaction(1);

        // Give the validator some money so it can send a rotation tx and rotate the validator's consensus key.
        let money_txn = create_transfer_to_validator_transaction(validator_account, 2);
        let rotation_txn = create_consensus_key_rotation_transaction(&validators[0], 0);

        // Execute and commit the reconfig block
        let block = vec![dummy_txn, reconfig_txn, money_txn, rotation_txn];
        let (reconfig_events, _) = execute_and_commit_block(&mut block_executor, block, 1);

        // Publish the on chain config updates
        executor_proxy
            .publish_on_chain_config_updates(reconfig_events)
            .unwrap();

        // Verify reconfig notification is sent
        assert!(reconfig_receiver
            .select_next_some()
            .now_or_never()
            .is_some());
    }

    #[test]
    fn test_pub_sub_no_reconfig_events() {
        let (subscription, mut reconfig_receiver) =
            ReconfigSubscription::subscribe_all("", vec![DiemVersion::CONFIG_ID], vec![]);
        let (_, _, mut executor_proxy) =
            bootstrap_genesis_and_set_subscription(subscription, &mut reconfig_receiver);

        // Publish no on chain config updates
        executor_proxy
            .publish_on_chain_config_updates(vec![])
            .unwrap();

        // Verify no reconfig notification is sent
        assert!(reconfig_receiver
            .select_next_some()
            .now_or_never()
            .is_none());
    }

    #[test]
    fn test_pub_sub_no_subscriptions() {
        let (subscription, mut reconfig_receiver) =
            ReconfigSubscription::subscribe_all("", vec![], vec![]);
        let (validators, mut block_executor, mut executor_proxy) =
            bootstrap_genesis_and_set_subscription(subscription, &mut reconfig_receiver);

        // Create a dummy prologue transaction that will bump the timer, and update the Diem version
        let validator_account = validators[0].data.address;
        let dummy_txn = create_dummy_transaction(1, validator_account);
        let reconfig_txn = create_new_update_diem_version_transaction(1);

        // Execute and commit the reconfig block
        let block = vec![dummy_txn, reconfig_txn];
        let (reconfig_events, _) = execute_and_commit_block(&mut block_executor, block, 1);

        // Publish the on chain config updates
        executor_proxy
            .publish_on_chain_config_updates(reconfig_events)
            .unwrap();

        // Verify no reconfig notification is sent
        assert!(reconfig_receiver
            .select_next_some()
            .now_or_never()
            .is_none());
    }

    #[test]
    fn test_pub_sub_diem_version() {
        let (subscription, mut reconfig_receiver) =
            ReconfigSubscription::subscribe_all("", vec![DiemVersion::CONFIG_ID], vec![]);
        let (validators, mut block_executor, mut executor_proxy) =
            bootstrap_genesis_and_set_subscription(subscription, &mut reconfig_receiver);

        // Create a dummy prologue transaction that will bump the timer, and update the Diem version
        let validator_account = validators[0].data.address;
        let dummy_txn = create_dummy_transaction(1, validator_account);
        let allowlist_txn = create_new_update_diem_version_transaction(1);

        // Execute and commit the reconfig block
        let block = vec![dummy_txn, allowlist_txn];
        let (reconfig_events, _) = execute_and_commit_block(&mut block_executor, block, 1);

        // Publish the on chain config updates
        executor_proxy
            .publish_on_chain_config_updates(reconfig_events)
            .unwrap();

        // Verify the correct reconfig notification is sent
        let payload = reconfig_receiver.select_next_some().now_or_never().unwrap();
        let received_config = payload.get::<DiemVersion>().unwrap();
        assert_eq!(received_config, DiemVersion { major: 7 });
    }

    #[test]
    fn test_pub_sub_with_executor_proxy() {
        let (subscription, mut reconfig_receiver) = ReconfigSubscription::subscribe_all(
            "",
            vec![ValidatorSet::CONFIG_ID, DiemVersion::CONFIG_ID],
            vec![],
        );
        let (validators, mut block_executor, mut executor_proxy) =
            bootstrap_genesis_and_set_subscription(subscription, &mut reconfig_receiver);

        // Create a dummy prologue transaction that will bump the timer and update the Diem version
        let validator_account = validators[0].data.address;
        let dummy_txn_1 = create_dummy_transaction(1, validator_account);
        let reconfig_txn = create_new_update_diem_version_transaction(1);

        // Execute and commit the reconfig block
        let block = vec![dummy_txn_1.clone(), reconfig_txn.clone()];
        let (_, ledger_info_epoch_1) = execute_and_commit_block(&mut block_executor, block, 1);

        // Give the validator some money so it can send a rotation tx, create another dummy prologue
        // to bump the timer and rotate the validator's consensus key.
        let money_txn = create_transfer_to_validator_transaction(validator_account, 2);
        let dummy_txn_2 = create_dummy_transaction(2, validator_account);
        let rotation_txn = create_consensus_key_rotation_transaction(&validators[0], 0);

        // Execute and commit the reconfig block
        let block = vec![money_txn.clone(), dummy_txn_2.clone(), rotation_txn.clone()];
        let (_, ledger_info_epoch_2) = execute_and_commit_block(&mut block_executor, block, 2);

        // Grab the first two executed transactions and verify responses
        let txns = executor_proxy.get_chunk(0, 2, 2).unwrap();
        assert_eq!(txns.transactions, vec![dummy_txn_1, reconfig_txn]);
        executor_proxy
            .execute_chunk(txns, ledger_info_epoch_1.clone(), None)
            .unwrap();
        assert_eq!(
            ledger_info_epoch_1,
            executor_proxy.get_epoch_change_ledger_info(1).unwrap()
        );
        assert_eq!(
            ledger_info_epoch_1,
            executor_proxy.get_epoch_ending_ledger_info(2).unwrap()
        );

        // Grab the next two executed transactions (forced by limit) and verify responses
        let txns = executor_proxy.get_chunk(2, 2, 5).unwrap();
        assert_eq!(txns.transactions, vec![money_txn, dummy_txn_2]);
        executor_proxy.get_epoch_ending_ledger_info(4).unwrap_err();

        // Grab the last transaction and verify responses
        let txns = executor_proxy.get_chunk(4, 1, 5).unwrap();
        assert_eq!(txns.transactions, vec![rotation_txn]);
        executor_proxy
            .execute_chunk(txns, ledger_info_epoch_2.clone(), None)
            .unwrap();
        assert_eq!(
            ledger_info_epoch_2,
            executor_proxy.get_epoch_change_ledger_info(2).unwrap()
        );
        assert_eq!(
            ledger_info_epoch_2,
            executor_proxy.get_epoch_ending_ledger_info(5).unwrap()
        );
    }

    #[test]
    fn test_pub_sub_with_executor_sync_state() {
        let (subscription, mut reconfig_receiver) = ReconfigSubscription::subscribe_all(
            "",
            vec![ValidatorSet::CONFIG_ID, DiemVersion::CONFIG_ID],
            vec![],
        );
        let (validators, mut block_executor, executor_proxy) =
            bootstrap_genesis_and_set_subscription(subscription, &mut reconfig_receiver);

        // Create a dummy prologue transaction that will bump the timer and update the Diem version
        let validator_account = validators[0].data.address;
        let dummy_txn = create_dummy_transaction(1, validator_account);
        let reconfig_txn = create_new_update_diem_version_transaction(1);

        // Execute and commit the reconfig block
        let block = vec![dummy_txn, reconfig_txn];
        let _ = execute_and_commit_block(&mut block_executor, block, 1);

        // Verify executor proxy sync state
        let sync_state = executor_proxy.get_local_storage_state().unwrap();
        assert_eq!(sync_state.trusted_epoch(), 2); // 1 reconfiguration has occurred, trusted = next
        assert_eq!(sync_state.committed_version(), 2); // 2 transactions have committed
        assert_eq!(sync_state.synced_version(), 2); // 2 transactions have synced

        // Give the validator some money so it can send a rotation tx, create another dummy prologue
        // to bump the timer and rotate the validator's consensus key.
        let money_txn = create_transfer_to_validator_transaction(validator_account, 2);
        let dummy_txn = create_dummy_transaction(2, validator_account);
        let rotation_txn = create_consensus_key_rotation_transaction(&validators[0], 0);

        // Execute and commit the reconfig block
        let block = vec![money_txn, dummy_txn, rotation_txn];
        let _ = execute_and_commit_block(&mut block_executor, block, 2);

        // Verify executor proxy sync state
        let sync_state = executor_proxy.get_local_storage_state().unwrap();
        assert_eq!(sync_state.trusted_epoch(), 3); // 2 reconfigurations have occurred, trusted = next
        assert_eq!(sync_state.committed_version(), 5); // 5 transactions have committed
        assert_eq!(sync_state.synced_version(), 5); // 5 transactions have synced
    }

    #[test]
    fn test_pub_sub_consensus_config() {
        let (subscription, mut reconfig_receiver) = ReconfigSubscription::subscribe_all(
            "",
            vec![OnChainConsensusConfig::CONFIG_ID],
            vec![],
        );
        let (validators, mut block_executor, mut executor_proxy) =
            bootstrap_genesis_and_set_subscription(subscription, &mut reconfig_receiver);
        // it's initialized to empty vec so can't be deserialized
        assert!(executor_proxy
            .on_chain_configs
            .get::<OnChainConsensusConfig>()
            .is_err());

        // Create a dummy prologue transaction that will bump the timer, and update the Diem version
        let validator_account = validators[0].data.address;
        let dummy_txn = create_dummy_transaction(1, validator_account);
        let allowlist_txn = create_new_update_consensus_config_transaction(1);

        // Execute and commit the reconfig block
        let block = vec![dummy_txn, allowlist_txn];
        let (reconfig_events, _) = execute_and_commit_block(&mut block_executor, block, 1);

        // Publish the on chain config updates
        executor_proxy
            .publish_on_chain_config_updates(reconfig_events)
            .unwrap();

        // Verify the correct reconfig notification is sent
        let payload = reconfig_receiver.select_next_some().now_or_never().unwrap();
        let received_config = payload.get::<OnChainConsensusConfig>().unwrap();
        assert_eq!(
            received_config,
            OnChainConsensusConfig::V1(ConsensusConfigV1 { two_chain: true })
        );
    }

    #[test]
    fn test_missing_on_chain_config() {
        // Create a new subscriber for on-chain configs (including the new on-chain test config!)
        let (subscription, mut reconfig_receiver) = ReconfigSubscription::subscribe_all(
            "",
            vec![
                DiemVersion::CONFIG_ID,
                OnChainConsensusConfig::CONFIG_ID,
                TestOnChainConfig::CONFIG_ID,
            ],
            vec![],
        );

        // Create a test diem database
        let db_path = diem_temppath::TempPath::new();
        db_path.create_as_dir().unwrap();
        let (db, db_rw) = DbReaderWriter::wrap(DiemDB::new_for_test(db_path.path()));

        // Bootstrap the database with regular genesis
        let (genesis, validators) = vm_genesis::test_genesis_change_set_and_validators(Some(1));
        let genesis_txn = Transaction::GenesisTransaction(WriteSetPayload::Direct(genesis));
        bootstrap_genesis::<DiemVM>(&db_rw, &genesis_txn).unwrap();

        // Initialize the executor proxy and verify that the node doesn't panic
        // (even though it can't find the TestOnChainConfig on the blockchain!).
        let chunk_executor = Box::new(Executor::<DiemVM>::new(db_rw.clone()));
        let mut config_registry = ON_CHAIN_CONFIG_REGISTRY.to_owned();
        config_registry.push(TestOnChainConfig::CONFIG_ID);
        let mut executor_proxy =
            ExecutorProxy::new_for_test(db, chunk_executor, vec![subscription], &config_registry);

        // Verify that the initial configs returned to the subscriber don't contain the unknown on-chain config
        let payload = reconfig_receiver.select_next_some().now_or_never().unwrap();
        payload.get::<DiemVersion>().unwrap();
        assert!(payload.get::<OnChainConsensusConfig>().is_err());
        assert!(payload.get::<TestOnChainConfig>().is_err());

        // Create a dummy prologue transaction that will bump the timer, and update the Diem version
        let validator_account = validators[0].data.address;
        let dummy_txn = create_dummy_transaction(1, validator_account);
        let allowlist_txn = create_new_update_consensus_config_transaction(1);

        // Execute and commit the reconfig block
        let mut block_executor = Box::new(Executor::<DiemVM>::new(db_rw));
        let block = vec![dummy_txn, allowlist_txn];
        let (reconfig_events, _) = execute_and_commit_block(&mut block_executor, block, 1);

        // Publish the on chain config updates
        executor_proxy
            .publish_on_chain_config_updates(reconfig_events)
            .unwrap();

        // Verify the reconfig notification still doesn't contain the unknown config
        let payload = reconfig_receiver.select_next_some().now_or_never().unwrap();
        payload.get::<DiemVersion>().unwrap();
        payload.get::<OnChainConsensusConfig>().unwrap();
        assert!(payload.get::<TestOnChainConfig>().is_err());
    }

    /// Executes a genesis transaction, creates the executor proxy and sets the given reconfig
    /// subscription.
    fn bootstrap_genesis_and_set_subscription(
        subscription: ReconfigSubscription,
        reconfig_receiver: &mut Receiver<(), OnChainConfigPayload>,
    ) -> (Vec<TestValidator>, Box<Executor<DiemVM>>, ExecutorProxy) {
        // Generate a genesis change set
        let (genesis, validators) = vm_genesis::test_genesis_change_set_and_validators(Some(1));

        // Create test diem database
        let db_path = diem_temppath::TempPath::new();
        db_path.create_as_dir().unwrap();
        let (db, db_rw) = DbReaderWriter::wrap(DiemDB::new_for_test(db_path.path()));

        // Boostrap the genesis transaction
        let genesis_txn = Transaction::GenesisTransaction(WriteSetPayload::Direct(genesis));
        bootstrap_genesis::<DiemVM>(&db_rw, &genesis_txn).unwrap();

        // Create executor proxy with given subscription
        let block_executor = Box::new(Executor::<DiemVM>::new(db_rw.clone()));
        let chunk_executor = Box::new(Executor::<DiemVM>::new(db_rw));
        let executor_proxy = ExecutorProxy::new(db, chunk_executor, vec![subscription]);

        // Verify initial reconfiguration notification is sent
        assert!(
            reconfig_receiver
                .select_next_some()
                .now_or_never()
                .is_some(),
            "Expected an initial reconfig notification on executor proxy creation!",
        );

        (validators, block_executor, executor_proxy)
    }

    /// Creates a transaction that rotates the consensus key of the given validator account.
    fn create_consensus_key_rotation_transaction(
        validator: &TestValidator,
        sequence_number: u64,
    ) -> Transaction {
        let operator_key = validator.key.clone();
        let operator_public_key = operator_key.public_key();
        let operator_account = validator.data.operator_address;
        let new_consensus_key = Ed25519PrivateKey::generate_for_testing().public_key();

        get_test_signed_transaction(
            operator_account,
            sequence_number,
            operator_key,
            operator_public_key,
            Some(TransactionPayload::Script(
                encode_set_validator_config_and_reconfigure_script(
                    validator.data.address,
                    new_consensus_key.to_bytes().to_vec(),
                    Vec::new(),
                    Vec::new(),
                ),
            )),
        )
    }

    /// Creates a dummy transaction (useful for bumping the timer).
    fn create_dummy_transaction(index: u8, validator_account: AccountAddress) -> Transaction {
        Transaction::BlockMetadata(BlockMetadata::new(
            gen_block_id(index),
            index as u64,
            (index as u64 + 1) * 100000010,
            vec![],
            validator_account,
        ))
    }

    /// Creates a transaction that creates a reconfiguration event by changing the Diem version
    fn create_new_update_diem_version_transaction(sequence_number: u64) -> Transaction {
        let genesis_key = vm_genesis::GENESIS_KEYPAIR.0.clone();
        get_test_signed_transaction(
            diem_root_address(),
            sequence_number,
            genesis_key.clone(),
            genesis_key.public_key(),
            Some(TransactionPayload::Script(
                encode_update_diem_version_script(
                    0, 7, // version
                ),
            )),
        )
    }

    fn create_new_update_consensus_config_transaction(sequence_number: u64) -> Transaction {
        let genesis_key = vm_genesis::GENESIS_KEYPAIR.0.clone();
        get_test_signed_transaction(
            diem_root_address(),
            sequence_number,
            genesis_key.clone(),
            genesis_key.public_key(),
            Some(encode_update_diem_consensus_config_script_function(
                0,
                bcs::to_bytes(&OnChainConsensusConfig::V1(ConsensusConfigV1 {
                    two_chain: true,
                }))
                .unwrap(),
            )),
        )
    }

    /// Creates a transaction that sends funds to the specified validator account.
    fn create_transfer_to_validator_transaction(
        validator_account: AccountAddress,
        sequence_number: u64,
    ) -> Transaction {
        let genesis_key = vm_genesis::GENESIS_KEYPAIR.0.clone();
        get_test_signed_transaction(
            diem_root_address(),
            sequence_number,
            genesis_key.clone(),
            genesis_key.public_key(),
            Some(TransactionPayload::Script(
                encode_peer_to_peer_with_metadata_script(
                    xus_tag(),
                    validator_account,
                    1_000_000,
                    vec![],
                    vec![],
                ),
            )),
        )
    }

    /// Executes and commits a given block that will cause a reconfiguration event.
    fn execute_and_commit_block(
        block_executor: &mut Box<Executor<DiemVM>>,
        block: Vec<Transaction>,
        block_id: u8,
    ) -> (Vec<ContractEvent>, LedgerInfoWithSignatures) {
        let block_hash = gen_block_id(block_id);

        // Execute block
        let output = block_executor
            .execute_block((block_hash, block), block_executor.committed_block_id())
            .expect("Failed to execute block!");
        assert!(
            output.has_reconfiguration(),
            "Block execution is missing a reconfiguration!"
        );

        // Commit block
        let ledger_info_with_sigs =
            gen_ledger_info_with_sigs(block_id.into(), &output, block_hash, vec![]);
        block_executor
            .commit_blocks(vec![block_hash], ledger_info_with_sigs.clone())
            .unwrap();

        (output.reconfig_events().to_vec(), ledger_info_with_sigs)
    }

    /// Defines a new on-chain config for test purposes.
    #[derive(Clone, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord, Serialize)]
    pub struct TestOnChainConfig {
        pub some_value: u64,
    }

    impl OnChainConfig for TestOnChainConfig {
        const IDENTIFIER: &'static str = "TestOnChainConfig";
    }
}

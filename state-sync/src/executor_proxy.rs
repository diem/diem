// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    counters,
    logging::{LogEntry, LogEvent, LogSchema},
    shared_components::SyncState,
};
use anyhow::{format_err, Result};
use diem_logger::prelude::*;
use diem_types::{
    account_state::AccountState,
    contract_event::ContractEvent,
    ledger_info::LedgerInfoWithSignatures,
    move_resource::MoveStorage,
    on_chain_config::{config_address, OnChainConfigPayload, ON_CHAIN_CONFIG_REGISTRY},
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
    fn get_local_storage_state(&self) -> Result<SyncState>;

    /// Execute and commit a batch of transactions
    fn execute_chunk(
        &mut self,
        txn_list_with_proof: TransactionListWithProof,
        verified_target_li: LedgerInfoWithSignatures,
        intermediate_end_of_epoch_li: Option<LedgerInfoWithSignatures>,
    ) -> Result<()>;

    /// Gets chunk of transactions given the known version, target version and the max limit.
    fn get_chunk(
        &self,
        known_version: u64,
        limit: u64,
        target_version: u64,
    ) -> Result<TransactionListWithProof>;

    /// Get the epoch change ledger info for epoch so that we can move to next epoch.
    fn get_epoch_proof(&self, epoch: u64) -> Result<LedgerInfoWithSignatures>;

    /// Get ledger info at an epoch boundary version.
    fn get_epoch_ending_ledger_info(&self, version: u64) -> Result<LedgerInfoWithSignatures>;

    /// Returns the ledger's timestamp for the given version in microseconds
    fn get_version_timestamp(&self, version: u64) -> Result<u64>;

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
            .get_account_state_with_proof_by_version(
                config_address(),
                storage.fetch_synced_version()?,
            )?
            .0
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
    fn get_local_storage_state(&self) -> Result<SyncState> {
        let storage_info = self
            .storage
            .get_startup_info()?
            .ok_or_else(|| format_err!("[state sync] Missing storage info"))?;

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
    ) -> Result<()> {
        // track chunk execution time
        let timer = counters::EXECUTE_CHUNK_DURATION.start_timer();
        let reconfig_events = self.executor.execute_and_commit_chunk(
            txn_list_with_proof,
            verified_target_li,
            intermediate_end_of_epoch_li,
        )?;
        timer.stop_and_record();
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
    ) -> Result<TransactionListWithProof> {
        let starting_version = known_version
            .checked_add(1)
            .ok_or_else(|| format_err!("Starting version has overflown!"))?;
        self.storage
            .get_transactions(starting_version, limit, target_version, false)
    }

    fn get_epoch_proof(&self, epoch: u64) -> Result<LedgerInfoWithSignatures> {
        let next_epoch = epoch
            .checked_add(1)
            .ok_or_else(|| format_err!("Next epoch has overflown!"))?;
        self.storage
            .get_epoch_ending_ledger_infos(epoch, next_epoch)?
            .ledger_info_with_sigs
            .pop()
            .ok_or_else(|| format_err!("Empty EpochChangeProof"))
    }

    fn get_epoch_ending_ledger_info(&self, version: u64) -> Result<LedgerInfoWithSignatures> {
        self.storage.get_epoch_ending_ledger_info(version)
    }

    fn get_version_timestamp(&self, version: u64) -> Result<u64> {
        self.storage.get_block_timestamp(version)
    }

    fn publish_on_chain_config_updates(&mut self, events: Vec<ContractEvent>) -> Result<()> {
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
                            .error(&e),
                        "Failed to publish reconfig notification to subscription {}",
                        subscription.name
                    );
                } else {
                    debug!(
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
            Err(format_err!("failed to publish at least one subscription"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use channel::diem_channel::Receiver;
    use compiled_stdlib::transaction_scripts::StdlibScript;
    use diem_crypto::{ed25519::*, HashValue, PrivateKey, Uniform};
    use diem_types::{
        account_address::AccountAddress,
        account_config::{diem_root_address, xus_tag},
        block_metadata::BlockMetadata,
        contract_event::ContractEvent,
        ledger_info::LedgerInfoWithSignatures,
        on_chain_config::{
            OnChainConfig, OnChainConfigPayload, VMConfig, VMPublishingOption, ValidatorSet,
        },
        transaction::{Transaction, WriteSetPayload},
    };
    use diem_vm::DiemVM;
    use diemdb::DiemDB;
    use executor::Executor;
    use executor_test_helpers::{
        bootstrap_genesis, gen_block_id, gen_ledger_info_with_sigs, get_test_signed_transaction,
    };
    use executor_types::BlockExecutor;
    use futures::{future::FutureExt, stream::StreamExt};
    use storage_interface::DbReaderWriter;
    use subscription_service::ReconfigSubscription;
    use transaction_builder::{
        encode_add_to_script_allow_list_script, encode_block_prologue_script,
        encode_peer_to_peer_with_metadata_script,
        encode_set_validator_config_and_reconfigure_script,
    };
    use vm_genesis::Validator;

    // TODO(joshlind): add unit tests for general executor proxy behaviour!
    // TODO(joshlind): add unit tests for subscription events.. seems like these are missing?

    #[test]
    fn test_pub_sub_different_subscription() {
        let (subscription, mut reconfig_receiver) =
            ReconfigSubscription::subscribe_all("", vec![VMConfig::CONFIG_ID], vec![]);
        let (validators, mut block_executor, mut executor_proxy) =
            bootstrap_genesis_and_set_subscription(subscription, &mut reconfig_receiver);

        // Create a dummy prologue transaction that will bump the timer, and update the VMPublishingOption
        let validator_account = validators[0].owner_address;
        let dummy_txn = create_dummy_transaction(1, validator_account);
        let (allowlist_txn, _) = create_new_allowlist_transaction(1);

        // Execute and commit the block
        let block = vec![dummy_txn, allowlist_txn];
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
            ReconfigSubscription::subscribe_all("", vec![VMPublishingOption::CONFIG_ID], vec![]);
        let (validators, mut block_executor, mut executor_proxy) =
            bootstrap_genesis_and_set_subscription(subscription, &mut reconfig_receiver);

        // Create a dummy prologue transaction that will bump the timer, and update the VMPublishingOption
        let validator_account = validators[0].owner_address;
        let dummy_txn = create_dummy_transaction(1, validator_account);
        let (allowlist_txn, _) = create_new_allowlist_transaction(1);

        // Execute and commit the reconfig block
        let block = vec![dummy_txn, allowlist_txn];
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
            vec![ValidatorSet::CONFIG_ID, VMPublishingOption::CONFIG_ID],
            vec![],
        );
        let (validators, mut block_executor, mut executor_proxy) =
            bootstrap_genesis_and_set_subscription(subscription, &mut reconfig_receiver);

        // Create a dummy prologue transaction that will bump the timer, and update the VMPublishingOption
        let validator_account = validators[0].owner_address;
        let dummy_txn = create_dummy_transaction(1, validator_account);
        let (allowlist_txn, _) = create_new_allowlist_transaction(1);

        // Give the validator some money so it can send a rotation tx and rotate the validator's consensus key.
        let money_txn = create_transfer_to_validator_transaction(validator_account, 2);
        let rotation_txn = create_consensus_key_rotation_transaction(&validators[0], 0);

        // Execute and commit the reconfig block
        let block = vec![dummy_txn, allowlist_txn, money_txn, rotation_txn];
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
            ReconfigSubscription::subscribe_all("", vec![VMPublishingOption::CONFIG_ID], vec![]);
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

        // Create a dummy prologue transaction that will bump the timer, and update the VMPublishingOption
        let validator_account = validators[0].owner_address;
        let dummy_txn = create_dummy_transaction(1, validator_account);
        let (allowlist_txn, _) = create_new_allowlist_transaction(1);

        // Execute and commit the reconfig block
        let block = vec![dummy_txn, allowlist_txn];
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
    fn test_pub_sub_vm_publishing_option() {
        let (subscription, mut reconfig_receiver) =
            ReconfigSubscription::subscribe_all("", vec![VMPublishingOption::CONFIG_ID], vec![]);
        let (validators, mut block_executor, mut executor_proxy) =
            bootstrap_genesis_and_set_subscription(subscription, &mut reconfig_receiver);

        // Create a dummy prologue transaction that will bump the timer, and update the VMPublishingOption
        let validator_account = validators[0].owner_address;
        let dummy_txn = create_dummy_transaction(1, validator_account);
        let (allowlist_txn, vm_publishing_option) = create_new_allowlist_transaction(1);

        // Execute and commit the reconfig block
        let block = vec![dummy_txn, allowlist_txn];
        let (reconfig_events, _) = execute_and_commit_block(&mut block_executor, block, 1);

        // Publish the on chain config updates
        executor_proxy
            .publish_on_chain_config_updates(reconfig_events)
            .unwrap();

        // Verify the correct reconfig notification is sent
        let payload = reconfig_receiver.select_next_some().now_or_never().unwrap();
        let received_config = payload.get::<VMPublishingOption>().unwrap();
        assert_eq!(received_config, vm_publishing_option);
    }

    #[test]
    fn test_pub_sub_with_executor_proxy() {
        let (subscription, mut reconfig_receiver) = ReconfigSubscription::subscribe_all(
            "",
            vec![ValidatorSet::CONFIG_ID, VMPublishingOption::CONFIG_ID],
            vec![],
        );
        let (validators, mut block_executor, mut executor_proxy) =
            bootstrap_genesis_and_set_subscription(subscription, &mut reconfig_receiver);

        // Create a dummy prologue transaction that will bump the timer and update the VMPublishingOption
        let validator_account = validators[0].owner_address;
        let dummy_txn_1 = create_dummy_transaction(1, validator_account);
        let (allowlist_txn, _) = create_new_allowlist_transaction(1);

        // Execute and commit the reconfig block
        let block = vec![dummy_txn_1.clone(), allowlist_txn.clone()];
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
        assert_eq!(txns.transactions, vec![dummy_txn_1, allowlist_txn]);
        assert!(executor_proxy
            .execute_chunk(txns, ledger_info_epoch_1.clone(), None)
            .is_ok());
        assert_eq!(
            ledger_info_epoch_1,
            executor_proxy.get_epoch_proof(1).unwrap()
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
        assert!(executor_proxy
            .execute_chunk(txns, ledger_info_epoch_2.clone(), None)
            .is_ok());
        assert_eq!(
            ledger_info_epoch_2,
            executor_proxy.get_epoch_proof(2).unwrap()
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
            vec![ValidatorSet::CONFIG_ID, VMPublishingOption::CONFIG_ID],
            vec![],
        );
        let (validators, mut block_executor, executor_proxy) =
            bootstrap_genesis_and_set_subscription(subscription, &mut reconfig_receiver);

        // Create a dummy prologue transaction that will bump the timer and update the VMPublishingOption
        let validator_account = validators[0].owner_address;
        let dummy_txn = create_dummy_transaction(1, validator_account);
        let (allowlist_txn, _) = create_new_allowlist_transaction(1);

        // Execute and commit the reconfig block
        let block = vec![dummy_txn, allowlist_txn];
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

    /// Executes a genesis transaction, creates the executor proxy and sets the given reconfig
    /// subscription.
    fn bootstrap_genesis_and_set_subscription(
        subscription: ReconfigSubscription,
        reconfig_receiver: &mut Receiver<(), OnChainConfigPayload>,
    ) -> (Vec<Validator>, Box<Executor<DiemVM>>, ExecutorProxy) {
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
        validator: &Validator,
        sequence_number: u64,
    ) -> Transaction {
        let operator_key = validator.key.clone();
        let operator_public_key = operator_key.public_key();
        let operator_account = validator.operator_address;
        let new_consensus_key = Ed25519PrivateKey::generate_for_testing().public_key();

        get_test_signed_transaction(
            operator_account,
            sequence_number,
            operator_key,
            operator_public_key,
            Some(encode_set_validator_config_and_reconfigure_script(
                validator.owner_address,
                new_consensus_key.to_bytes().to_vec(),
                Vec::new(),
                Vec::new(),
            )),
        )
    }

    /// Creates a dummy transaction (useful for bumping the timer).
    fn create_dummy_transaction(index: u8, validator_account: AccountAddress) -> Transaction {
        encode_block_prologue_script(BlockMetadata::new(
            gen_block_id(index),
            index as u64,
            (index as u64 + 1) * 100000010,
            vec![],
            validator_account,
        ))
    }

    /// Creates a transaction that updates the on chain allowlist.
    fn create_new_allowlist_transaction(
        sequencer_number: u64,
    ) -> (Transaction, VMPublishingOption) {
        // Add a new script to the allowlist
        let new_allowlist = {
            let mut existing_list = StdlibScript::allowlist();
            existing_list.push(*HashValue::sha3_256_of(&[]).as_ref());
            existing_list
        };
        let vm_publishing_option = VMPublishingOption::locked(new_allowlist);

        // Create a transaction for the new allowlist
        let genesis_key = vm_genesis::GENESIS_KEYPAIR.0.clone();
        let txn = get_test_signed_transaction(
            diem_root_address(),
            /* sequence_number = */ sequencer_number,
            genesis_key.clone(),
            genesis_key.public_key(),
            Some(encode_add_to_script_allow_list_script(
                HashValue::sha3_256_of(&[]).to_vec(),
                0,
            )),
        );

        (txn, vm_publishing_option)
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
            Some(encode_peer_to_peer_with_metadata_script(
                xus_tag(),
                validator_account,
                1_000_000,
                vec![],
                vec![],
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
            gen_ledger_info_with_sigs(block_id.into(), output, block_hash, vec![]);
        let (_, reconfig_events) = block_executor
            .commit_blocks(vec![block_hash], ledger_info_with_sigs.clone())
            .unwrap();
        assert!(
            !reconfig_events.is_empty(),
            "Expected reconfig events from block commit!"
        );

        (reconfig_events, ledger_info_with_sigs)
    }
}

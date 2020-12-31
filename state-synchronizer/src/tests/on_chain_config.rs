// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::executor_proxy::{ExecutorProxy, ExecutorProxyTrait};
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
    encode_peer_to_peer_with_metadata_script, encode_set_validator_config_and_reconfigure_script,
};
use vm_genesis::Validator;

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
fn create_new_allowlist_transaction(sequencer_number: u64) -> (Transaction, VMPublishingOption) {
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

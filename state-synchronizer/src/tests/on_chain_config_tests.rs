// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::executor_proxy::{ExecutorProxy, ExecutorProxyTrait};
use compiled_stdlib::transaction_scripts::StdlibScript;
use diem_crypto::{ed25519::*, HashValue, PrivateKey, Uniform};
use diem_types::{
    account_config::{diem_root_address, xus_tag},
    block_metadata::BlockMetadata,
    on_chain_config::{OnChainConfig, VMPublishingOption},
    transaction::{Transaction, WriteSetPayload},
};
use diem_vm::DiemVM;
use diemdb::DiemDB;
use executor::Executor;
use executor_test_helpers::{
    bootstrap_genesis, gen_block_id, gen_block_metadata, gen_ledger_info_with_sigs,
    get_test_signed_transaction,
};
use executor_types::BlockExecutor;
use futures::{future::FutureExt, stream::StreamExt};
use storage_interface::DbReaderWriter;
use subscription_service::ReconfigSubscription;
use transaction_builder::{
    encode_add_to_script_allow_list_script, encode_block_prologue_script,
    encode_peer_to_peer_with_metadata_script, encode_set_validator_config_and_reconfigure_script,
};

// TODO test for subscription with multiple subscribed configs once there are >1 on-chain configs
#[test]
fn test_on_chain_config_pub_sub() {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    // set up reconfig subscription
    let (subscription, mut reconfig_receiver) =
        ReconfigSubscription::subscribe_all("test", vec![VMPublishingOption::CONFIG_ID], vec![]);

    let (genesis, validators) = vm_genesis::test_genesis_change_set_and_validators(Some(1));
    let genesis_key = vm_genesis::GENESIS_KEYPAIR.0.clone();
    let genesis_txn = Transaction::GenesisTransaction(WriteSetPayload::Direct(genesis));
    let db_path = diem_temppath::TempPath::new();
    db_path.create_as_dir().unwrap();
    let (db, db_rw) = DbReaderWriter::wrap(DiemDB::new_for_test(db_path.path()));
    bootstrap_genesis::<DiemVM>(&db_rw, &genesis_txn).unwrap();

    let mut block_executor = Box::new(Executor::<DiemVM>::new(db_rw.clone()));
    let chunk_executor = Box::new(Executor::<DiemVM>::new(db_rw));
    let mut executor_proxy = ExecutorProxy::new(db, chunk_executor, vec![subscription]);

    assert!(
        reconfig_receiver
            .select_next_some()
            .now_or_never()
            .is_some(),
        "expect initial config notification",
    );

    // start state sync with initial loading of on-chain configs
    executor_proxy
        .load_on_chain_configs()
        .expect("failed to load on-chain configs");

    ////////////////////////////////////////////////////////
    // Case 1: don't publish for no reconfiguration event //
    ////////////////////////////////////////////////////////
    executor_proxy
        .publish_on_chain_config_updates(vec![])
        .expect("failed to publish on-chain configs");

    assert_eq!(
        reconfig_receiver.select_next_some().now_or_never(),
        None,
        "did not expect reconfig update"
    );

    //////////////////////////////////////////////////
    // Case 2: publish if subscribed config changed //
    //////////////////////////////////////////////////
    let genesis_account = diem_root_address();
    let validator_account = validators[0].owner_address;
    let operator_key = validators[0].key.clone();
    let operator_public_key = operator_key.public_key();
    let operator_account = validators[0].operator_address;

    // Create a dummy block prologue transaction that will bump the timer.
    let txn1 = encode_block_prologue_script(gen_block_metadata(1, validator_account));

    // Add a script to allowlist.
    let new_allowlist = {
        let mut existing_list = StdlibScript::allowlist();
        existing_list.push(*HashValue::sha3_256_of(&[]).as_ref());
        existing_list
    };
    let vm_publishing_option = VMPublishingOption::locked(new_allowlist);

    let txn2 = get_test_signed_transaction(
        genesis_account,
        /* sequence_number = */ 1,
        genesis_key.clone(),
        genesis_key.public_key(),
        Some(encode_add_to_script_allow_list_script(
            HashValue::sha3_256_of(&[]).to_vec(),
            0,
        )),
    );

    let block1 = vec![txn1.clone(), txn2.clone()];
    let block1_id = gen_block_id(1);
    let parent_block_id = block_executor.committed_block_id();

    let output = block_executor
        .execute_block((block1_id, block1), parent_block_id)
        .expect("failed to execute block");
    assert!(
        output.has_reconfiguration(),
        "execution missing reconfiguration"
    );

    let ledger_info_with_sigs_epoch_1 = gen_ledger_info_with_sigs(1, output, block1_id, vec![]);
    let (_, reconfig_events) = block_executor
        .commit_blocks(vec![block1_id], ledger_info_with_sigs_epoch_1.clone())
        .unwrap();
    assert!(
        !reconfig_events.is_empty(),
        "expected reconfig events from executor commit"
    );
    executor_proxy
        .publish_on_chain_config_updates(reconfig_events)
        .expect("failed to publish on-chain configs");

    let receive_reconfig = async {
        let payload = reconfig_receiver.select_next_some().await;
        let received_config = payload.get::<VMPublishingOption>().unwrap();
        assert_eq!(received_config, vm_publishing_option);
    };

    rt.block_on(receive_reconfig);

    //////////////////////////////////////////////////////////////////////////////////////
    // Case 3: don't publish for reconfiguration that doesn't change subscribed configs //
    //////////////////////////////////////////////////////////////////////////////////////
    // give the validator some money so they can send a tx
    let txn3 = get_test_signed_transaction(
        genesis_account,
        /* sequence_number = */ 2,
        genesis_key.clone(),
        genesis_key.public_key(),
        Some(encode_peer_to_peer_with_metadata_script(
            xus_tag(),
            validator_account,
            1_000_000,
            vec![],
            vec![],
        )),
    );

    // Create a dummy block prologue transaction that will bump the timer.
    let txn4 = encode_block_prologue_script(BlockMetadata::new(
        gen_block_id(2),
        2,
        300000010,
        vec![],
        validator_account,
    ));

    // rotate the validator's consensus pubkey to trigger a reconfiguration
    let new_pubkey = Ed25519PrivateKey::generate_for_testing().public_key();
    let txn5 = get_test_signed_transaction(
        operator_account,
        /* sequence_number = */ 0,
        operator_key,
        operator_public_key,
        Some(encode_set_validator_config_and_reconfigure_script(
            validator_account,
            new_pubkey.to_bytes().to_vec(),
            Vec::new(),
            Vec::new(),
        )),
    );

    let block2 = vec![txn3, txn4, txn5];
    let block2_id = gen_block_id(2);

    let output = block_executor
        .execute_block((block2_id, block2), block_executor.committed_block_id())
        .expect("failed to execute block");
    assert!(
        output.has_reconfiguration(),
        "execution missing reconfiguration"
    );

    let ledger_info_with_sigs_epoch_2 = gen_ledger_info_with_sigs(2, output, block2_id, vec![]);
    let (_, reconfig_events) = block_executor
        .commit_blocks(vec![block2_id], ledger_info_with_sigs_epoch_2.clone())
        .unwrap();
    assert!(
        !reconfig_events.is_empty(),
        "expected reconfig events from executor commit"
    );

    executor_proxy
        .publish_on_chain_config_updates(reconfig_events)
        .expect("failed to publish on-chain configs");

    assert_eq!(
        reconfig_receiver.select_next_some().now_or_never(),
        None,
        "did not expect reconfig update"
    );

    ////////////////////////////////////////////////////////////////
    // Case 3: test failed on-chain reconfig publish throws error //
    ////////////////////////////////////////////////////////////////
    // test error throwing on failed reconfig publish
    drop(reconfig_receiver);

    // Create a dummy block prologue transaction that will bump the timer.
    let txn6 = encode_block_prologue_script(BlockMetadata::new(
        gen_block_id(2),
        3,
        300000020,
        vec![],
        validator_account,
    ));
    let txn7 = get_test_signed_transaction(
        genesis_account,
        /* sequence_number = */ 3,
        genesis_key.clone(),
        genesis_key.public_key(),
        Some(encode_add_to_script_allow_list_script(
            HashValue::sha3_256_of(&[1]).to_vec(),
            0,
        )),
    );

    let block3 = vec![txn6, txn7];
    let block3_id = gen_block_id(3);
    let parent_block_id = block_executor.committed_block_id();

    let output = block_executor
        .execute_block((block3_id, block3), parent_block_id)
        .expect("failed to execute block");
    assert!(
        output.has_reconfiguration(),
        "execution missing reconfiguration"
    );

    let ledger_info_with_sigs = gen_ledger_info_with_sigs(3, output, block3_id, vec![]);
    let (_, reconfig_events) = block_executor
        .commit_blocks(vec![block3_id], ledger_info_with_sigs)
        .unwrap();
    assert!(
        !reconfig_events.is_empty(),
        "expected reconfig events from executor commit"
    );

    // assert publishing on-chain config updates fails for dropped receiver
    assert!(executor_proxy
        .publish_on_chain_config_updates(reconfig_events)
        .is_err());

    // test state of DB using non-mocked executor proxy interface
    // Note: we mock out the executor proxy in the state sync integration tests because we also mock
    // out the storage/execution layer. We piggy-back on this expensive DB/runtime setup here in testing
    // on-chain reconfig to test the executor proxy interface against.
    let state = executor_proxy.get_local_storage_state().unwrap();
    assert_eq!(state.epoch(), 4);
    assert_eq!(state.highest_version_in_local_storage(), 7);

    let txns = executor_proxy.get_chunk(0, 2, 2).unwrap();
    assert_eq!(txns.transactions, vec![txn1, txn2]);

    assert!(executor_proxy
        .execute_chunk(txns, ledger_info_with_sigs_epoch_1.clone(), None)
        .is_ok());

    let epoch_li = executor_proxy.get_epoch_proof(1).unwrap();
    assert_eq!(epoch_li, ledger_info_with_sigs_epoch_1);
    let epoch_li = executor_proxy.get_epoch_proof(2).unwrap();
    assert_eq!(epoch_li, ledger_info_with_sigs_epoch_2);

    // epoch 1 ended at version 2
    let epoch_li = executor_proxy.get_epoch_ending_ledger_info(2).unwrap();
    assert_eq!(epoch_li, ledger_info_with_sigs_epoch_1);
    // epoch 2 ended at version 5
    let epoch_li = executor_proxy.get_epoch_ending_ledger_info(5).unwrap();
    assert_eq!(epoch_li, ledger_info_with_sigs_epoch_2);

    assert!(executor_proxy.get_epoch_ending_ledger_info(3).is_err());
}

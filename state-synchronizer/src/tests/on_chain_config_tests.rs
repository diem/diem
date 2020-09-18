// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::executor_proxy::{ExecutorProxy, ExecutorProxyTrait};
use compiled_stdlib::transaction_scripts::StdlibScript;
use executor::Executor;
use executor_test_helpers::{
    bootstrap_genesis, gen_block_id, gen_block_metadata, gen_ledger_info_with_sigs,
    get_test_signed_transaction,
};
use executor_types::BlockExecutor;
use futures::{future::FutureExt, stream::StreamExt};
use libra_crypto::{ed25519::*, HashValue, PrivateKey, Uniform};
use libra_types::{
    account_config::{lbr_type_tag, libra_root_address},
    on_chain_config::{OnChainConfig, VMPublishingOption},
    transaction::{Transaction, WriteSetPayload},
};
use libra_vm::LibraVM;
use libradb::LibraDB;
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
    let db_path = libra_temppath::TempPath::new();
    db_path.create_as_dir().unwrap();
    let (db, db_rw) = DbReaderWriter::wrap(LibraDB::new_for_test(db_path.path()));
    bootstrap_genesis::<LibraVM>(&db_rw, &genesis_txn).unwrap();

    let mut block_executor = Box::new(Executor::<LibraVM>::new(db_rw.clone()));
    let chunk_executor = Box::new(Executor::<LibraVM>::new(db_rw));
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
    let genesis_account = libra_root_address();
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

    let block1 = vec![txn1, txn2];
    let block1_id = gen_block_id(1);
    let parent_block_id = block_executor.committed_block_id();

    let output = block_executor
        .execute_block((block1_id, block1), parent_block_id)
        .expect("failed to execute block");
    assert!(
        output.has_reconfiguration(),
        "execution missing reconfiguration"
    );

    let ledger_info_with_sigs = gen_ledger_info_with_sigs(1, output, block1_id, vec![]);
    let (_, reconfig_events) = block_executor
        .commit_blocks(vec![block1_id], ledger_info_with_sigs)
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
            lbr_type_tag(),
            validator_account,
            1_000_000,
            vec![],
            vec![],
        )),
    );

    // Create a dummy block prologue transaction that will bump the timer.
    let txn4 = encode_block_prologue_script(gen_block_metadata(2, validator_account));

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

    let ledger_info_with_sigs = gen_ledger_info_with_sigs(2, output, block2_id, vec![]);
    let (_, reconfig_events) = block_executor
        .commit_blocks(vec![block2_id], ledger_info_with_sigs)
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
}

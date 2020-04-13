// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::executor_proxy::{ExecutorProxy, ExecutorProxyTrait};
use executor_utils::{
    create_storage_service_and_executor,
    test_helpers::{
        gen_block_id, gen_block_metadata, gen_ledger_info_with_sigs, get_test_signed_transaction,
    },
};
use futures::{future::FutureExt, stream::StreamExt};
use libra_crypto::{
    ed25519,
    traits::{PrivateKeyExt, Uniform},
    HashValue,
};
use libra_types::{
    account_config::{association_address, lbr_type_tag},
    on_chain_config::{OnChainConfig, VMPublishingOption},
    transaction::authenticator::AuthenticationKey,
};
use std::sync::{Arc, Mutex};
use stdlib::transaction_scripts::StdlibScript;
use subscription_service::ReconfigSubscription;
use transaction_builder::{
    encode_block_prologue_script, encode_publishing_option_script,
    encode_rotate_consensus_pubkey_script, encode_transfer_script,
};

// TODO test for subscription with multiple subscribed configs once there are >1 on-chain configs
#[test]
fn test_on_chain_config_pub_sub() {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    // set up reconfig subscription
    let subscribed_configs = &[VMPublishingOption::CONFIG_ID];
    let (subscription, mut reconfig_receiver) = ReconfigSubscription::subscribe(subscribed_configs);

    let (mut config, genesis_key) = config_builder::test_config();
    let (_storage_server_handle, executor) = create_storage_service_and_executor(&config);
    let executor = Arc::new(Mutex::new(executor));
    let mut executor_proxy = ExecutorProxy::new(executor.clone(), &config, vec![subscription]);

    // start state sync with initial loading of on-chain configs
    rt.block_on(executor_proxy.load_on_chain_configs())
        .expect("failed to load on-chain configs");

    ////////////////////////////////////////////////////////
    // Case 1: don't publish for no reconfiguration event //
    ////////////////////////////////////////////////////////
    rt.block_on(executor_proxy.publish_on_chain_config_updates(vec![]))
        .expect("failed to publish on-chain configs");

    assert_eq!(
        reconfig_receiver.select_next_some().now_or_never(),
        None,
        "did not expect reconfig update"
    );

    //////////////////////////////////////////////////
    // Case 2: publish if subscribed config changed //
    //////////////////////////////////////////////////
    let genesis_account = association_address();
    let network_config = config.validator_network.as_ref().unwrap();
    let validator_account = network_config.peer_id;
    let keys = config
        .test
        .as_mut()
        .unwrap()
        .account_keypair
        .as_mut()
        .unwrap();

    let validator_privkey = keys.take_private().unwrap();
    let validator_pubkey = keys.public().clone();
    let auth_key = AuthenticationKey::ed25519(&validator_pubkey);
    let validator_auth_key_prefix = auth_key.prefix().to_vec();

    // Create a dummy block prologue transaction that will bump the timer.
    let txn1 = encode_block_prologue_script(gen_block_metadata(1, validator_account));

    // Add a script to whitelist.
    let new_whitelist = {
        let mut existing_list = StdlibScript::whitelist();
        existing_list.push(*HashValue::from_sha3_256(&[]).as_ref());
        existing_list
    };
    let vm_publishing_option = VMPublishingOption::Locked(new_whitelist);

    let txn2 = get_test_signed_transaction(
        genesis_account,
        /* sequence_number = */ 1,
        genesis_key.clone(),
        genesis_key.public_key(),
        Some(encode_publishing_option_script(
            vm_publishing_option.clone(),
        )),
    );

    let block1 = vec![txn1, txn2];
    let block1_id = gen_block_id(1);
    let parent_block_id = executor.lock().unwrap().committed_block_id();

    let output = executor
        .lock()
        .unwrap()
        .execute_block((block1_id, block1), parent_block_id)
        .expect("failed to execute block");
    assert!(
        output.has_reconfiguration(),
        "execution missing reconfiguration"
    );

    let ledger_info_with_sigs = gen_ledger_info_with_sigs(2, output.root_hash(), block1_id);
    let (_, reconfig_events) = executor
        .lock()
        .unwrap()
        .commit_blocks(vec![block1_id], ledger_info_with_sigs)
        .unwrap();
    assert!(
        !reconfig_events.is_empty(),
        "expected reconfig events from executor commit"
    );
    rt.block_on(executor_proxy.publish_on_chain_config_updates(reconfig_events))
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
        Some(encode_transfer_script(
            lbr_type_tag(),
            &validator_account,
            validator_auth_key_prefix,
            1_000_000,
        )),
    );

    // Create a dummy block prologue transaction that will bump the timer.
    let txn4 = encode_block_prologue_script(gen_block_metadata(2, validator_account));

    // rotate the validator's consensus pubkey to trigger a reconfiguration
    let new_pubkey = ed25519::PrivateKey::generate_for_testing().public_key();
    let txn5 = get_test_signed_transaction(
        validator_account,
        /* sequence_number = */ 0,
        validator_privkey,
        validator_pubkey,
        Some(encode_rotate_consensus_pubkey_script(
            new_pubkey.to_bytes().to_vec(),
        )),
    );

    let block2 = vec![txn3, txn4, txn5];
    let block2_id = gen_block_id(2);

    let output = executor
        .lock()
        .unwrap()
        .execute_block((block2_id, block2), block1_id)
        .expect("failed to execute block");
    assert!(
        output.has_reconfiguration(),
        "execution missing reconfiguration"
    );

    let ledger_info_with_sigs = gen_ledger_info_with_sigs(5, output.root_hash(), block2_id);
    let (_, reconfig_events) = executor
        .lock()
        .unwrap()
        .commit_blocks(vec![block2_id], ledger_info_with_sigs)
        .unwrap();
    assert!(
        !reconfig_events.is_empty(),
        "expected reconfig events from executor commit"
    );

    rt.block_on(executor_proxy.publish_on_chain_config_updates(reconfig_events))
        .expect("failed to publish on-chain configs");

    assert_eq!(
        reconfig_receiver.select_next_some().now_or_never(),
        None,
        "did not expect reconfig update"
    );
}

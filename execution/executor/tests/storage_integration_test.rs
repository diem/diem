// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use compiled_stdlib::transaction_scripts::StdlibScript;
use executor_test_helpers::{
    extract_signer, gen_block_id, gen_block_metadata, gen_ledger_info_with_sigs,
    get_test_signed_transaction,
    integration_test_impl::{
        create_db_and_executor, test_execution_with_storage_impl, verify_committed_txn_status,
    },
};
use executor_types::BlockExecutor;
use libra_crypto::{ed25519::*, test_utils::TEST_SEED, x25519, HashValue, PrivateKey, Uniform};
use libra_types::{
    account_address,
    account_config::{coin1_tag, libra_root_address, treasury_compliance_account_address},
    account_state::AccountState,
    on_chain_config::VMPublishingOption,
    transaction::{authenticator::AuthenticationKey, Script},
    trusted_state::{TrustedState, TrustedStateChange},
};
use rand::SeedableRng;
use std::convert::TryFrom;
use transaction_builder::{
    encode_block_prologue_script, encode_modify_publishing_option_script,
    encode_peer_to_peer_with_metadata_script, encode_set_validator_config_and_reconfigure_script,
};

#[test]
fn test_genesis() {
    let (config, _genesis_key) = config_builder::test_config();
    let (_, db, _executor) = create_db_and_executor(&config);

    let (li, epoch_change_proof, _accumulator_consistency_proof) =
        db.reader.get_state_proof(0).unwrap();

    let trusted_state = TrustedState::from(config.base.waypoint.waypoint_from_config().unwrap());
    trusted_state
        .verify_and_ratchet(&li, &epoch_change_proof)
        .unwrap();
    let li = li.ledger_info();
    assert_eq!(li.version(), 0);

    let libra_root_account = db
        .reader
        .get_account_state_with_proof(libra_root_address(), 0, 0)
        .unwrap();
    libra_root_account
        .verify(li, 0, libra_root_address())
        .unwrap();
}

#[test]
fn test_reconfiguration() {
    // When executing a transaction emits a validator set change,
    // storage should propagate the new validator set

    let (mut config, genesis_key) = config_builder::test_config();
    let (_, db, mut executor) = create_db_and_executor(&config);
    let parent_block_id = executor.committed_block_id();
    let signer = extract_signer(&mut config);

    let network_config = config.validator_network.as_ref().unwrap();
    let validator_account = network_config.peer_id();
    let validator_pubkey = config
        .test
        .as_ref()
        .unwrap()
        .owner_key
        .as_ref()
        .unwrap()
        .public_key();
    let auth_key = AuthenticationKey::ed25519(&validator_pubkey);
    assert!(
        auth_key.derived_address() == validator_account,
        "Address derived from validator auth key does not match validator account address"
    );

    // test the current keys in the validator's account equals to the key in the validator set
    let (li, _epoch_change_proof, _accumulator_consistency_proof) =
        db.reader.get_state_proof(0).unwrap();
    let current_version = li.ledger_info().version();
    let validator_account_state_with_proof = db
        .reader
        .get_account_state_with_proof(validator_account, current_version, current_version)
        .unwrap();
    let libra_root_account_state_with_proof = db
        .reader
        .get_account_state_with_proof(libra_root_address(), current_version, current_version)
        .unwrap();
    assert_eq!(
        AccountState::try_from(&libra_root_account_state_with_proof.blob.unwrap())
            .unwrap()
            .get_validator_set()
            .unwrap()
            .unwrap()
            .payload()[0]
            .consensus_public_key(),
        &AccountState::try_from(&validator_account_state_with_proof.blob.unwrap())
            .unwrap()
            .get_validator_config_resource()
            .unwrap()
            .unwrap()
            .validator_config
            .unwrap()
            .consensus_public_key
    );

    // txn1 = give the validator some money so they can send a tx
    let txn1 = get_test_signed_transaction(
        treasury_compliance_account_address(),
        /* sequence_number = */ 0,
        genesis_key.clone(),
        genesis_key.public_key(),
        Some(encode_peer_to_peer_with_metadata_script(
            coin1_tag(),
            validator_account,
            1_000_000,
            vec![],
            vec![],
        )),
    );
    // txn2 = a dummy block prologue to bump the timer.
    let txn2 = encode_block_prologue_script(gen_block_metadata(1, validator_account));

    // txn3 = rotate the validator's consensus pubkey
    let operator_key = config
        .test
        .as_ref()
        .unwrap()
        .operator_key
        .as_ref()
        .unwrap()
        .private_key();
    let operator_account = account_address::from_public_key(&operator_key.public_key());

    let new_pubkey = Ed25519PrivateKey::generate_for_testing().public_key();
    let mut rng = ::rand::rngs::StdRng::from_seed(TEST_SEED);
    let new_network_pubkey = x25519::PrivateKey::generate(&mut rng).public_key();
    let txn3 = get_test_signed_transaction(
        operator_account,
        /* sequence_number = */ 0,
        operator_key.clone(),
        operator_key.public_key(),
        Some(encode_set_validator_config_and_reconfigure_script(
            validator_account,
            new_pubkey.to_bytes().to_vec(),
            new_network_pubkey.as_slice().to_vec(),
            Vec::new(),
            new_network_pubkey.as_slice().to_vec(),
            Vec::new(),
        )),
    );

    let txn_block = vec![txn1, txn2, txn3];
    let block_id = gen_block_id(1);
    let vm_output = executor
        .execute_block((block_id, txn_block.clone()), parent_block_id)
        .unwrap();

    // Make sure the execution result sees the reconfiguration
    assert!(
        vm_output.has_reconfiguration(),
        "StateComputeResult does not see a reconfiguration"
    );
    let ledger_info_with_sigs = gen_ledger_info_with_sigs(1, vm_output, block_id, vec![&signer]);
    let (_, reconfig_events) = executor
        .commit_blocks(vec![block_id], ledger_info_with_sigs)
        .unwrap();
    assert!(
        !reconfig_events.is_empty(),
        "expected reconfiguration event"
    );

    let (li, _epoch_change_proof, _accumulator_consistency_proof) =
        db.reader.get_state_proof(0).unwrap();
    let current_version = li.ledger_info().version();

    let t3 = db
        .reader
        .get_txn_by_account(operator_account, 0, current_version, true)
        .unwrap();
    verify_committed_txn_status(t3.as_ref(), &txn_block[2]).unwrap();

    // test validator's key under validator_account is as expected
    let validator_account_state_with_proof = db
        .reader
        .get_account_state_with_proof(validator_account, current_version, current_version)
        .unwrap();
    assert_eq!(
        AccountState::try_from(&validator_account_state_with_proof.blob.unwrap())
            .unwrap()
            .get_validator_config_resource()
            .unwrap()
            .unwrap()
            .validator_config
            .unwrap()
            .consensus_public_key,
        new_pubkey
    );

    // test validator's key under validator's account is now equal to the key in the
    // validator set since the reconfiguration was invoked
    let validator_account_state_with_proof = db
        .reader
        .get_account_state_with_proof(validator_account, current_version, current_version)
        .unwrap();
    let libra_root_account_state_with_proof = db
        .reader
        .get_account_state_with_proof(libra_root_address(), current_version, current_version)
        .unwrap();
    assert_eq!(
        AccountState::try_from(&libra_root_account_state_with_proof.blob.unwrap())
            .unwrap()
            .get_validator_set()
            .unwrap()
            .unwrap()
            .payload()[0]
            .consensus_public_key(),
        &AccountState::try_from(&validator_account_state_with_proof.blob.unwrap())
            .unwrap()
            .get_validator_config_resource()
            .unwrap()
            .unwrap()
            .validator_config
            .unwrap()
            .consensus_public_key
    );

    // test validator's key in the validator set is as expected
    let libra_root_account_state_with_proof = db
        .reader
        .get_account_state_with_proof(libra_root_address(), current_version, current_version)
        .unwrap();
    let blob = &libra_root_account_state_with_proof.blob.unwrap();
    assert_eq!(
        AccountState::try_from(blob)
            .unwrap()
            .get_validator_set()
            .unwrap()
            .unwrap()
            .payload()[0]
            .consensus_public_key(),
        &new_pubkey
    );
}

#[test]
fn test_change_publishing_option_to_custom() {
    // Publishing Option is set to locked at genesis.
    let (mut config, genesis_key) = config_builder::test_config();

    let (_, db, mut executor) = create_db_and_executor(&config);
    let parent_block_id = executor.committed_block_id();

    let treasury_compliance_account = treasury_compliance_account_address();
    let genesis_account = libra_root_address();
    let network_config = config.validator_network.as_ref().unwrap();
    let validator_account = network_config.peer_id();
    let validator_privkey = config
        .test
        .as_ref()
        .unwrap()
        .owner_key
        .as_ref()
        .unwrap()
        .private_key();
    let validator_pubkey = validator_privkey.public_key();

    let signer = extract_signer(&mut config);

    // give the validator some money so they can send a tx
    let txn1 = get_test_signed_transaction(
        treasury_compliance_account,
        /* sequence_number = */ 0,
        genesis_key.clone(),
        genesis_key.public_key(),
        Some(encode_peer_to_peer_with_metadata_script(
            coin1_tag(),
            validator_account,
            1_000_000,
            vec![],
            vec![],
        )),
    );

    let script1 = Script::new(vec![], vec![], vec![]);
    let script2 = Script::new(vec![1], vec![], vec![]);

    // Create a transaction that is not allowed with default publishing option and make sure it is
    // rejected.
    let txn2 = get_test_signed_transaction(
        validator_account,
        /* sequence_number = */ 0,
        validator_privkey.clone(),
        validator_pubkey.clone(),
        Some(script1.clone()),
    );

    let txn3 = get_test_signed_transaction(
        validator_account,
        /* sequence_number = */ 0,
        validator_privkey.clone(),
        validator_pubkey.clone(),
        Some(script2.clone()),
    );

    // Create a dummy block prologue transaction that will bump the timer.
    let txn4 = encode_block_prologue_script(gen_block_metadata(1, validator_account));

    let txn5 = get_test_signed_transaction(
        genesis_account,
        /* sequence_number = */ 1,
        genesis_key.clone(),
        genesis_key.public_key(),
        Some(encode_modify_publishing_option_script(
            VMPublishingOption::custom_scripts(),
        )),
    );

    let block1_id = gen_block_id(1);
    let block1 = vec![txn1, txn2, txn3, txn4, txn5];
    let output1 = executor
        .execute_block((block1_id, block1.clone()), parent_block_id)
        .unwrap();

    assert!(
        output1.has_reconfiguration(),
        "StateComputeResult has a new validator set"
    );

    let ledger_info_with_sigs = gen_ledger_info_with_sigs(1, output1, block1_id, vec![&signer]);
    let (_, reconfig_events) = executor
        .commit_blocks(vec![block1_id], ledger_info_with_sigs)
        .unwrap();
    assert!(
        !reconfig_events.is_empty(),
        "executor commit should return reconfig events for reconfiguration"
    );

    let (li, epoch_change_proof, _accumulator_consistency_proof) =
        db.reader.get_state_proof(0).unwrap();
    let mut trusted_state =
        TrustedState::from(config.base.waypoint.waypoint_from_config().unwrap());
    match trusted_state.verify_and_ratchet(&li, &epoch_change_proof) {
        Ok(TrustedStateChange::Epoch { new_state, .. }) => trusted_state = new_state,
        _ => panic!("unexpected state change"),
    }
    let current_version = li.ledger_info().version();
    assert_eq!(current_version, 3);
    // Transaction 1 is committed as it's in the allowlist
    let txn1 = db
        .reader
        .get_txn_by_account(treasury_compliance_account, 0, current_version, false)
        .unwrap();
    verify_committed_txn_status(txn1.as_ref(), &block1[0]).unwrap();
    // Transaction 2, 3 are rejected
    assert!(db
        .reader
        .get_txn_by_account(validator_account, 0, current_version, false)
        .unwrap()
        .is_none());

    // Now that the PublishingOption is modified to CustomScript, we can resubmit the script again.
    let txn2 = get_test_signed_transaction(
        validator_account,
        /* sequence_number = */ 0,
        validator_privkey.clone(),
        validator_pubkey.clone(),
        Some(script1),
    );

    let txn3 = get_test_signed_transaction(
        validator_account,
        /* sequence_number = */ 1,
        validator_privkey,
        validator_pubkey,
        Some(script2),
    );

    let block2_id = gen_block_id(2);
    let block2 = vec![txn2, txn3];
    let output2 = executor
        .execute_block((block2_id, block2.clone()), executor.committed_block_id())
        .unwrap();

    let ledger_info_with_sigs = gen_ledger_info_with_sigs(2, output2, block2_id, vec![&signer]);
    let (_, reconfig_events) = executor
        .commit_blocks(vec![block2_id], ledger_info_with_sigs)
        .unwrap();
    assert!(
        reconfig_events.is_empty(),
        "expect executor to reutrn no reconfig events"
    );

    let (li, epoch_change_proof, _accumulator_consistency_proof) =
        db.reader.get_state_proof(current_version).unwrap();
    trusted_state
        .verify_and_ratchet(&li, &epoch_change_proof)
        .unwrap();
    let current_version = li.ledger_info().version();
    assert_eq!(current_version, 5);
    // Transaction 2 is committed.
    let txn2 = db
        .reader
        .get_txn_by_account(validator_account, 0, current_version, false)
        .unwrap();
    verify_committed_txn_status(txn2.as_ref(), &block2[0]).unwrap();
    // Transaction 3 is committed.
    let txn3 = db
        .reader
        .get_txn_by_account(validator_account, 1, current_version, false)
        .unwrap();
    verify_committed_txn_status(txn3.as_ref(), &block2[1]).unwrap();
}

#[test]
fn test_extend_allowlist() {
    // Publishing Option is set to locked at genesis.
    let (mut config, genesis_key) = config_builder::test_config();

    let (_, db, mut executor) = create_db_and_executor(&config);
    let parent_block_id = executor.committed_block_id();

    let treasury_compliance_account = treasury_compliance_account_address();
    let genesis_account = libra_root_address();
    let network_config = config.validator_network.as_ref().unwrap();
    let validator_account = network_config.peer_id();
    let validator_privkey = config
        .test
        .as_ref()
        .unwrap()
        .owner_key
        .as_ref()
        .unwrap()
        .private_key();
    let validator_pubkey = validator_privkey.public_key();
    let signer = extract_signer(&mut config);
    let auth_key = AuthenticationKey::ed25519(&validator_pubkey);
    assert!(
        auth_key.derived_address() == validator_account,
        "Address derived from validator auth key does not match validator account address"
    );

    // give the validator some money so they can send a tx
    let txn1 = get_test_signed_transaction(
        treasury_compliance_account,
        /* sequence_number = */ 0,
        genesis_key.clone(),
        genesis_key.public_key(),
        Some(encode_peer_to_peer_with_metadata_script(
            coin1_tag(),
            validator_account,
            1_000_000,
            vec![],
            vec![],
        )),
    );

    let script1 = Script::new(vec![], vec![], vec![]);
    let script2 = Script::new(vec![1], vec![], vec![]);

    // Create a transaction that is not allowed with default publishing option and make sure it is
    // rejected.
    let txn2 = get_test_signed_transaction(
        validator_account,
        /* sequence_number = */ 0,
        validator_privkey.clone(),
        validator_pubkey.clone(),
        Some(script1.clone()),
    );

    let txn3 = get_test_signed_transaction(
        validator_account,
        /* sequence_number = */ 0,
        validator_privkey.clone(),
        validator_pubkey.clone(),
        Some(script2.clone()),
    );

    // Create a dummy block prologue transaction that will bump the timer.
    let txn4 = encode_block_prologue_script(gen_block_metadata(1, validator_account));

    // Add script1 to allowlist.
    let new_allowlist = {
        let mut existing_list = StdlibScript::allowlist();
        existing_list.push(*HashValue::sha3_256_of(&[]).as_ref());
        existing_list
    };

    let txn5 = get_test_signed_transaction(
        genesis_account,
        /* sequence_number = */ 1,
        genesis_key.clone(),
        genesis_key.public_key(),
        Some(encode_modify_publishing_option_script(
            VMPublishingOption::locked(new_allowlist),
        )),
    );

    let block1 = vec![txn1, txn2, txn3, txn4, txn5];
    let block1_id = gen_block_id(1);
    let output1 = executor
        .execute_block((block1_id, block1.clone()), parent_block_id)
        .unwrap();

    assert!(
        output1.has_reconfiguration(),
        "StateComputeResult has a new validator set"
    );

    let ledger_info_with_sigs = gen_ledger_info_with_sigs(1, output1, block1_id, vec![&signer]);
    let (_, reconfig_events) = executor
        .commit_blocks(vec![block1_id], ledger_info_with_sigs)
        .unwrap();
    assert!(
        !reconfig_events.is_empty(),
        "executor commit should return reconfig events for reconfiguration"
    );

    let (li, epoch_change_proof, _accumulator_consistency_proof) =
        db.reader.get_state_proof(0).unwrap();
    let mut trusted_state =
        TrustedState::from(config.base.waypoint.waypoint_from_config().unwrap());
    match trusted_state.verify_and_ratchet(&li, &epoch_change_proof) {
        Ok(TrustedStateChange::Epoch { new_state, .. }) => trusted_state = new_state,
        _ => panic!("unexpected state change"),
    }
    let current_version = li.ledger_info().version();
    assert_eq!(current_version, 3);
    // Transaction 1 is committed as it's in the allowlist
    let t1 = db
        .reader
        .get_txn_by_account(treasury_compliance_account, 0, current_version, false)
        .unwrap();
    verify_committed_txn_status(t1.as_ref(), &block1[0]).unwrap();

    // Transaction 2, 3 are rejected
    let t2 = db
        .reader
        .get_txn_by_account(validator_account, 0, current_version, false)
        .unwrap();
    assert!(t2.is_none());

    // Now that the PublishingOption is modified to allowlist with script1 allowed, we can resubmit
    // the script again.

    let txn2 = get_test_signed_transaction(
        validator_account,
        /* sequence_number = */ 0,
        validator_privkey.clone(),
        validator_pubkey.clone(),
        Some(script1),
    );

    let txn3 = get_test_signed_transaction(
        validator_account,
        /* sequence_number = */ 1,
        validator_privkey,
        validator_pubkey,
        Some(script2),
    );

    let block2_id = gen_block_id(2);
    let block2 = vec![txn2, txn3];
    let output2 = executor
        .execute_block((block2_id, block2.clone()), executor.committed_block_id())
        .unwrap();

    let ledger_info_with_sigs = gen_ledger_info_with_sigs(2, output2, block2_id, vec![&signer]);
    let (_, reconfig_events) = executor
        .commit_blocks(vec![block2_id], ledger_info_with_sigs)
        .unwrap();
    assert!(
        reconfig_events.is_empty(),
        "expect executor to reutrn no reconfig events"
    );

    let (li, epoch_change_proof, _accumulator_consistency_proof) =
        db.reader.get_state_proof(current_version).unwrap();
    trusted_state
        .verify_and_ratchet(&li, &epoch_change_proof)
        .unwrap();
    let current_version = li.ledger_info().version();
    assert_eq!(current_version, 4);
    // Transaction 2 is committed.
    let t2 = db
        .reader
        .get_txn_by_account(validator_account, 0, current_version, false)
        .unwrap();
    verify_committed_txn_status(t2.as_ref(), &block2[0]).unwrap();

    // Transaction 3 is NOT committed.
    let t3 = db
        .reader
        .get_txn_by_account(validator_account, 1, current_version, false)
        .unwrap();
    assert!(t3.is_none());
}

#[test]
fn test_execution_with_storage() {
    test_execution_with_storage_impl();
}

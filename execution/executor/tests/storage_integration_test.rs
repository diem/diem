// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{ensure, format_err, Result};
use compiled_stdlib::transaction_scripts::StdlibScript;
use executor::{db_bootstrapper::bootstrap_db_if_empty, Executor};
use executor_test_helpers::{
    extract_signer, gen_block_id, gen_block_metadata, gen_ledger_info_with_sigs,
    get_test_signed_transaction,
};
use executor_types::BlockExecutor;
use libra_config::{config::NodeConfig, utils::get_genesis_txn};
use libra_crypto::{ed25519::*, test_utils::TEST_SEED, x25519, HashValue, PrivateKey, Uniform};
use libra_types::{
    account_config::{
        coin1_tag, from_currency_code_string, libra_root_address, testnet_dd_account_address,
        treasury_compliance_account_address, COIN1_NAME,
    },
    account_state::AccountState,
    account_state_blob::AccountStateWithProof,
    event::EventKey,
    on_chain_config::VMPublishingOption,
    transaction::{
        authenticator::AuthenticationKey, Script, Transaction, TransactionListWithProof,
        TransactionWithProof,
    },
    trusted_state::{TrustedState, TrustedStateChange},
};
use libra_vm::LibraVM;
use libradb::LibraDB;
use rand::SeedableRng;
use std::convert::TryFrom;
use storage_interface::DbReaderWriter;
use transaction_builder::{
    encode_block_prologue_script, encode_create_testing_account_script,
    encode_modify_publishing_option_script, encode_peer_to_peer_with_metadata_script,
    encode_reconfigure_script, encode_set_validator_config_script, encode_testnet_mint_script,
};

fn create_db_and_executor(config: &NodeConfig) -> (DbReaderWriter, Executor<LibraVM>) {
    let db = DbReaderWriter::new(LibraDB::new_for_test(config.storage.dir()));
    bootstrap_db_if_empty::<LibraVM>(&db, get_genesis_txn(config).unwrap()).unwrap();
    let executor = Executor::<LibraVM>::new(db.clone());

    (db, executor)
}

#[test]
fn test_genesis() {
    let (config, _genesis_key) = config_builder::test_config();
    let (db, _executor) = create_db_and_executor(&config);

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
    let (db, mut executor) = create_db_and_executor(&config);
    let parent_block_id = executor.committed_block_id();
    let signer = extract_signer(&mut config);

    let network_config = config.validator_network.as_ref().unwrap();
    let validator_account = network_config.peer_id();
    let keys = config
        .test
        .as_mut()
        .unwrap()
        .operator_keypair
        .as_mut()
        .unwrap();
    let validator_privkey = keys.take_private().unwrap();
    let validator_pubkey = keys.public_key();
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
    let new_pubkey = Ed25519PrivateKey::generate_for_testing().public_key();
    let mut rng = ::rand::rngs::StdRng::from_seed(TEST_SEED);
    let new_network_pubkey = x25519::PrivateKey::generate(&mut rng).public_key();
    let txn3 = get_test_signed_transaction(
        validator_account,
        /* sequence_number = */ 0,
        validator_privkey,
        validator_pubkey,
        Some(encode_set_validator_config_script(
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

    // Make sure there is no reconfiguration in the execution result
    assert!(
        !vm_output.has_reconfiguration(),
        "StateComputeResult has unexpected reconfiguration"
    );
    let ledger_info_with_sigs = gen_ledger_info_with_sigs(1, vm_output, block_id, vec![&signer]);
    let (_, reconfig_events) = executor
        .commit_blocks(vec![block_id], ledger_info_with_sigs)
        .unwrap();
    assert!(
        reconfig_events.is_empty(),
        "unexpected reconfiguration event"
    );

    let (li, _epoch_change_proof, _accumulator_consistency_proof) =
        db.reader.get_state_proof(0).unwrap();
    let current_version = li.ledger_info().version();

    let t3 = db
        .reader
        .get_txn_by_account(validator_account, 0, current_version, true)
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

    // test validator's key under validator's account is not equal to the key in the
    // validator set since the reconfiguration was not invoked by the association
    let validator_account_state_with_proof = db
        .reader
        .get_account_state_with_proof(validator_account, current_version, current_version)
        .unwrap();
    let libra_root_account_state_with_proof = db
        .reader
        .get_account_state_with_proof(libra_root_address(), current_version, current_version)
        .unwrap();
    assert_ne!(
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

    // txn4 = reconfigure the system with a new consensus key
    let txn4 = get_test_signed_transaction(
        libra_root_address(),
        /* sequence_number = */ 1,
        genesis_key.clone(),
        genesis_key.public_key(),
        Some(encode_reconfigure_script()),
    );

    let txn_block = vec![txn4];
    let block_id = gen_block_id(2);
    let vm_output = executor
        .execute_block((block_id, txn_block.clone()), executor.committed_block_id())
        .unwrap();

    // Make sure the execution result sees the reconfiguration
    assert!(
        vm_output.has_reconfiguration(),
        "StateComputeResult does not see reconfiguration"
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

    let t4 = db
        .reader
        .get_txn_by_account(libra_root_address(), 1, current_version, true)
        .unwrap();
    verify_committed_txn_status(t4.as_ref(), &txn_block[0]).unwrap();
    assert_eq!(t4.unwrap().events.unwrap().len(), 1);

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

    let (db, mut executor) = create_db_and_executor(&config);
    let parent_block_id = executor.committed_block_id();

    let treasury_compliance_account = treasury_compliance_account_address();
    let genesis_account = libra_root_address();
    let network_config = config.validator_network.as_ref().unwrap();
    let validator_account = network_config.peer_id();
    let keys = config
        .test
        .as_mut()
        .unwrap()
        .operator_keypair
        .as_mut()
        .unwrap();
    let validator_privkey = keys.take_private().unwrap();
    let validator_pubkey = keys.public_key();

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
            VMPublishingOption::CustomScripts,
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
    // Transaction 1 is committed as it's in the whitelist
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
fn test_extend_whitelist() {
    // Publishing Option is set to locked at genesis.
    let (mut config, genesis_key) = config_builder::test_config();

    let (db, mut executor) = create_db_and_executor(&config);
    let parent_block_id = executor.committed_block_id();

    let treasury_compliance_account = treasury_compliance_account_address();
    let genesis_account = libra_root_address();
    let network_config = config.validator_network.as_ref().unwrap();
    let validator_account = network_config.peer_id();
    let keys = config
        .test
        .as_mut()
        .unwrap()
        .operator_keypair
        .as_mut()
        .unwrap();
    let validator_privkey = keys.take_private().unwrap();
    let validator_pubkey = keys.public_key();
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

    // Add script1 to whitelist.
    let new_whitelist = {
        let mut existing_list = StdlibScript::whitelist();
        existing_list.push(*HashValue::sha3_256_of(&[]).as_ref());
        existing_list
    };

    let txn5 = get_test_signed_transaction(
        genesis_account,
        /* sequence_number = */ 1,
        genesis_key.clone(),
        genesis_key.public_key(),
        Some(encode_modify_publishing_option_script(
            VMPublishingOption::Locked(new_whitelist),
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
    // Transaction 1 is committed as it's in the whitelist
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

    // Now that the PublishingOption is modified to whitelist with script1 allowed, we can resubmit
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
    let (mut config, genesis_key) = config_builder::test_config();
    let (db, mut executor) = create_db_and_executor(&config);
    let parent_block_id = executor.committed_block_id();
    let signer = extract_signer(&mut config);

    let seed = [1u8; 32];
    // TEST_SEED is also used to generate a random validator set in get_test_config. Each account
    // in this random validator set gets created in genesis. If one of {account1, account2,
    // account3} already exists in genesis, the code below will fail.
    assert!(seed != TEST_SEED);
    let mut rng = ::rand::rngs::StdRng::from_seed(seed);

    let privkey1 = Ed25519PrivateKey::generate(&mut rng);
    let pubkey1 = privkey1.public_key();
    let account1_auth_key = AuthenticationKey::ed25519(&pubkey1);
    let account1 = account1_auth_key.derived_address();

    let privkey2 = Ed25519PrivateKey::generate(&mut rng);
    let pubkey2 = privkey2.public_key();
    let account2_auth_key = AuthenticationKey::ed25519(&pubkey2);
    let account2 = account2_auth_key.derived_address();

    let pubkey3 = Ed25519PrivateKey::generate(&mut rng).public_key();
    let account3_auth_key = AuthenticationKey::ed25519(&pubkey3);
    let account3 = account3_auth_key.derived_address();

    let pubkey4 = Ed25519PrivateKey::generate(&mut rng).public_key();
    let account4_auth_key = AuthenticationKey::ed25519(&pubkey4); // non-existent account
    let account4 = account4_auth_key.derived_address();
    let genesis_account = testnet_dd_account_address();
    let root_account = libra_root_address();

    let tx1 = get_test_signed_transaction(
        root_account,
        /* sequence_number = */ 1,
        genesis_key.clone(),
        genesis_key.public_key(),
        Some(encode_create_testing_account_script(
            coin1_tag(),
            account1,
            account1_auth_key.prefix().to_vec(),
            false, /* add all currencies */
        )),
    );

    let tx2 = get_test_signed_transaction(
        root_account,
        /* sequence_number = */ 2,
        genesis_key.clone(),
        genesis_key.public_key(),
        Some(encode_create_testing_account_script(
            coin1_tag(),
            account2,
            account2_auth_key.prefix().to_vec(),
            false, /* add all currencies */
        )),
    );

    let tx3 = get_test_signed_transaction(
        root_account,
        /* sequence_number = */ 3,
        genesis_key.clone(),
        genesis_key.public_key(),
        Some(encode_create_testing_account_script(
            coin1_tag(),
            account3,
            account3_auth_key.prefix().to_vec(),
            false, /* add all currencies */
        )),
    );

    // Create account1 with 2M coins.
    let txn1 = get_test_signed_transaction(
        genesis_account,
        /* sequence_number = */ 0,
        genesis_key.clone(),
        genesis_key.public_key(),
        Some(encode_testnet_mint_script(coin1_tag(), account1, 2_000_000)),
    );

    // Create account2 with 1.2M coins.
    let txn2 = get_test_signed_transaction(
        genesis_account,
        /* sequence_number = */ 1,
        genesis_key.clone(),
        genesis_key.public_key(),
        Some(encode_testnet_mint_script(coin1_tag(), account2, 1_200_000)),
    );

    // Create account3 with 1M coins.
    let txn3 = get_test_signed_transaction(
        genesis_account,
        /* sequence_number = */ 2,
        genesis_key.clone(),
        genesis_key.public_key(),
        Some(encode_testnet_mint_script(coin1_tag(), account3, 1_000_000)),
    );

    // Transfer 20k coins from account1 to account2.
    // balance: <1.98M, 1.22M, 1M
    let txn4 = get_test_signed_transaction(
        account1,
        /* sequence_number = */ 0,
        privkey1.clone(),
        pubkey1.clone(),
        Some(encode_peer_to_peer_with_metadata_script(
            coin1_tag(),
            account2,
            20_000,
            vec![],
            vec![],
        )),
    );

    // Transfer 10k coins from account2 to account3.
    // balance: <1.98M, <1.21M, 1.01M
    let txn5 = get_test_signed_transaction(
        account2,
        /* sequence_number = */ 0,
        privkey2,
        pubkey2,
        Some(encode_peer_to_peer_with_metadata_script(
            coin1_tag(),
            account3,
            10_000,
            vec![],
            vec![],
        )),
    );

    // Transfer 70k coins from account1 to account3.
    // balance: <1.91M, <1.21M, 1.08M
    let txn6 = get_test_signed_transaction(
        account1,
        /* sequence_number = */ 1,
        privkey1.clone(),
        pubkey1.clone(),
        Some(encode_peer_to_peer_with_metadata_script(
            coin1_tag(),
            account3,
            70_000,
            vec![],
            vec![],
        )),
    );

    let block1 = vec![tx1, tx2, tx3, txn1, txn2, txn3, txn4, txn5, txn6];
    let block1_id = gen_block_id(1);

    let mut block2 = vec![];
    let block2_id = gen_block_id(2);

    // Create 14 txns transferring 10k from account1 to account3 each.
    for i in 2..=15 {
        block2.push(get_test_signed_transaction(
            account1,
            /* sequence_number = */ i,
            privkey1.clone(),
            pubkey1.clone(),
            Some(encode_peer_to_peer_with_metadata_script(
                coin1_tag(),
                account3,
                10_000,
                vec![],
                vec![],
            )),
        ));
    }

    let output1 = executor
        .execute_block((block1_id, block1.clone()), parent_block_id)
        .unwrap();
    let ledger_info_with_sigs = gen_ledger_info_with_sigs(1, output1, block1_id, vec![&signer]);
    let (_, reconfig_events) = executor
        .commit_blocks(vec![block1_id], ledger_info_with_sigs)
        .unwrap();
    assert!(
        reconfig_events.is_empty(),
        "expected no reconfiguration event from executor commit"
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
    assert_eq!(trusted_state.latest_version(), 9);

    let t1 = db
        .reader
        .get_txn_by_account(genesis_account, 0, current_version, false)
        .unwrap();
    verify_committed_txn_status(t1.as_ref(), &block1[3]).unwrap();

    let t2 = db
        .reader
        .get_txn_by_account(genesis_account, 1, current_version, false)
        .unwrap();
    verify_committed_txn_status(t2.as_ref(), &block1[4]).unwrap();

    let t3 = db
        .reader
        .get_txn_by_account(genesis_account, 2, current_version, false)
        .unwrap();
    verify_committed_txn_status(t3.as_ref(), &block1[5]).unwrap();

    let tn = db
        .reader
        .get_txn_by_account(genesis_account, 3, current_version, false)
        .unwrap();
    assert!(tn.is_none());

    let t4 = db
        .reader
        .get_txn_by_account(account1, 0, current_version, true)
        .unwrap();
    verify_committed_txn_status(t4.as_ref(), &block1[6]).unwrap();
    // We requested the events to come back from this one, so verify that they did
    assert_eq!(t4.unwrap().events.unwrap().len(), 2);

    let t5 = db
        .reader
        .get_txn_by_account(account2, 0, current_version, false)
        .unwrap();
    verify_committed_txn_status(t5.as_ref(), &block1[7]).unwrap();

    let t6 = db
        .reader
        .get_txn_by_account(account1, 1, current_version, true)
        .unwrap();
    verify_committed_txn_status(t6.as_ref(), &block1[8]).unwrap();

    let account1_state_with_proof = db
        .reader
        .get_account_state_with_proof(account1, current_version, current_version)
        .unwrap();
    verify_account_balance(&account1_state_with_proof, |x| x == 1_910_000).unwrap();

    let account2_state_with_proof = db
        .reader
        .get_account_state_with_proof(account2, current_version, current_version)
        .unwrap();
    verify_account_balance(&account2_state_with_proof, |x| x == 1_210_000).unwrap();

    let account3_state_with_proof = db
        .reader
        .get_account_state_with_proof(account3, current_version, current_version)
        .unwrap();
    verify_account_balance(&account3_state_with_proof, |x| x == 1_080_000).unwrap();

    let transaction_list_with_proof = db
        .reader
        .get_transactions(3, 10, current_version, false)
        .unwrap();
    verify_transactions(&transaction_list_with_proof, &block1[2..]).unwrap();

    let account1_sent_events = db
        .reader
        .get_events(&EventKey::new_from_address(&account1, 1), 0, true, 10)
        .unwrap();
    assert_eq!(account1_sent_events.len(), 2);

    let account2_sent_events = db
        .reader
        .get_events(&EventKey::new_from_address(&account2, 1), 0, true, 10)
        .unwrap();
    assert_eq!(account2_sent_events.len(), 1);

    let account3_sent_events = db
        .reader
        .get_events(&EventKey::new_from_address(&account3, 1), 0, true, 10)
        .unwrap();
    assert_eq!(account3_sent_events.len(), 0);

    let account1_received_events = db
        .reader
        .get_events(&EventKey::new_from_address(&account1, 0), 0, true, 10)
        .unwrap();
    assert_eq!(account1_received_events.len(), 1);

    let account2_received_events = db
        .reader
        .get_events(&EventKey::new_from_address(&account2, 0), 0, true, 10)
        .unwrap();
    assert_eq!(account2_received_events.len(), 2);

    let account3_received_events = db
        .reader
        .get_events(&EventKey::new_from_address(&account3, 0), 0, true, 10)
        .unwrap();
    assert_eq!(account3_received_events.len(), 3);

    let account4_state = db
        .reader
        .get_account_state_with_proof(account4, current_version, current_version)
        .unwrap();
    assert!(account4_state.blob.is_none());

    let account4_transaction = db
        .reader
        .get_txn_by_account(account4, 0, current_version, true)
        .unwrap();
    assert!(account4_transaction.is_none());

    let account4_sent_events = db
        .reader
        .get_events(&EventKey::new_from_address(&account4, 1), 0, true, 10)
        .unwrap();
    assert!(account4_sent_events.is_empty());

    // Execute the 2nd block.
    let output2 = executor
        .execute_block((block2_id, block2.clone()), block1_id)
        .unwrap();
    let ledger_info_with_sigs = gen_ledger_info_with_sigs(1, output2, block2_id, vec![&signer]);
    executor
        .commit_blocks(vec![block2_id], ledger_info_with_sigs)
        .unwrap();

    let (li, epoch_change_proof, _accumulator_consistency_proof) = db
        .reader
        .get_state_proof(trusted_state.latest_version())
        .unwrap();
    trusted_state
        .verify_and_ratchet(&li, &epoch_change_proof)
        .unwrap();
    let current_version = li.ledger_info().version();
    assert_eq!(current_version, 23);

    let t7 = db
        .reader
        .get_txn_by_account(account1, 2, current_version, false)
        .unwrap();
    verify_committed_txn_status(t7.as_ref(), &block2[0]).unwrap();

    let t20 = db
        .reader
        .get_txn_by_account(account1, 15, current_version, false)
        .unwrap();
    verify_committed_txn_status(t20.as_ref(), &block2[13]).unwrap();

    let account1_state_with_proof = db
        .reader
        .get_account_state_with_proof(account1, current_version, current_version)
        .unwrap();
    verify_account_balance(&account1_state_with_proof, |x| x == 1_770_000).unwrap();

    let account3_state_with_proof = db
        .reader
        .get_account_state_with_proof(account3, current_version, current_version)
        .unwrap();
    verify_account_balance(&account3_state_with_proof, |x| x == 1_220_000).unwrap();

    let transaction_list_with_proof = db
        .reader
        .get_transactions(10, 17, current_version, false)
        .unwrap();
    verify_transactions(&transaction_list_with_proof, &block2[..]).unwrap();

    let account1_sent_events_batch1 = db
        .reader
        .get_events(&EventKey::new_from_address(&account1, 1), 0, true, 10)
        .unwrap();
    assert_eq!(account1_sent_events_batch1.len(), 10);

    let account1_sent_events_batch2 = db
        .reader
        .get_events(&EventKey::new_from_address(&account1, 1), 10, true, 10)
        .unwrap();
    assert_eq!(account1_sent_events_batch2.len(), 6);

    let account3_received_events_batch1 = db
        .reader
        .get_events(
            &EventKey::new_from_address(&account3, 0),
            u64::max_value(),
            false,
            10,
        )
        .unwrap();
    assert_eq!(account3_received_events_batch1.len(), 10);
    assert_eq!(account3_received_events_batch1[0].1.sequence_number(), 16);

    let account3_received_events_batch2 = db
        .reader
        .get_events(&EventKey::new_from_address(&account3, 0), 6, false, 10)
        .unwrap();
    assert_eq!(account3_received_events_batch2.len(), 7);
    assert_eq!(account3_received_events_batch2[0].1.sequence_number(), 6);
}

fn verify_account_balance<F>(account_state_with_proof: &AccountStateWithProof, f: F) -> Result<()>
where
    F: Fn(u64) -> bool,
{
    let balance = if let Some(blob) = &account_state_with_proof.blob {
        AccountState::try_from(blob)?
            .get_balance_resources(&[from_currency_code_string(COIN1_NAME).unwrap()])?
            .get(&from_currency_code_string(COIN1_NAME).unwrap())
            .map(|b| b.coin())
            .unwrap_or(0)
    } else {
        0
    };
    ensure!(
        f(balance),
        "balance {} doesn't satisfy the condition passed in",
        balance
    );
    Ok(())
}

fn verify_transactions(
    txn_list_with_proof: &TransactionListWithProof,
    expected_txns: &[Transaction],
) -> Result<()> {
    let txns = &txn_list_with_proof.transactions;
    ensure!(
        *txns == expected_txns,
        "expected txns {:?} doesn't equal to returned txns {:?}",
        expected_txns,
        txns
    );
    Ok(())
}

fn verify_committed_txn_status(
    txn_with_proof: Option<&TransactionWithProof>,
    expected_txn: &Transaction,
) -> Result<()> {
    let txn = &txn_with_proof
        .ok_or_else(|| format_err!("Transaction is not committed."))?
        .transaction;

    ensure!(
        expected_txn == txn,
        "The two transactions do not match. Expected txn: {:?}, returned txn: {:?}",
        expected_txn,
        txn,
    );

    Ok(())
}

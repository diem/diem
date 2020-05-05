// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{ensure, format_err, Result};
use executor::{db_bootstrapper::bootstrap_db_if_empty, Executor};
use executor_types::BlockExecutor;
use executor_utils::test_helpers::{
    extract_signer, gen_block_id, gen_block_metadata, gen_ledger_info_with_sigs,
    get_test_signed_transaction,
};
use libra_config::{config::NodeConfig, utils::get_genesis_txn};
use libra_crypto::{ed25519::*, test_utils::TEST_SEED, HashValue, PrivateKey, Uniform};
use libra_types::{
    account_config::{
        association_address, discovery_set_address, from_currency_code_string, lbr_type_tag,
        LBR_NAME,
    },
    account_state::AccountState,
    account_state_blob::AccountStateWithProof,
    discovery_set::DISCOVERY_SET_CHANGE_EVENT_PATH,
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
use stdlib::transaction_scripts::StdlibScript;
use storage_interface::DbReaderWriter;
use transaction_builder::{
    encode_block_prologue_script, encode_mint_script, encode_publishing_option_script,
    encode_rotate_consensus_pubkey_script, encode_transfer_with_metadata_script,
};

fn create_db_and_executor(config: &NodeConfig) -> (DbReaderWriter, Executor<LibraVM>) {
    let db = DbReaderWriter::new(LibraDB::new(config.storage.dir()));
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

    let trusted_state = TrustedState::from(config.base.waypoint.unwrap());
    trusted_state
        .verify_and_ratchet(&li, &epoch_change_proof)
        .unwrap();
    let li = li.ledger_info();
    assert_eq!(li.version(), 0);

    let discovery_set_account = db
        .reader
        .get_account_state_with_proof(discovery_set_address(), 0, 0)
        .unwrap();
    discovery_set_account
        .verify(li, 0, discovery_set_address())
        .unwrap();
    let (event_key, count) = discovery_set_account
        .get_event_key_and_count_by_query_path(&DISCOVERY_SET_CHANGE_EVENT_PATH)
        .unwrap();
    assert_eq!(count, 1);
    assert_eq!(
        db.reader
            .get_events(&event_key.unwrap(), 0, true, 100)
            .unwrap()
            .len(),
        1
    );

    let association_account = db
        .reader
        .get_account_state_with_proof(association_address(), 0, 0)
        .unwrap();
    association_account
        .verify(li, 0, association_address())
        .unwrap();
    assert_eq!(
        association_account
            .get_event_key_and_count_by_query_path(&DISCOVERY_SET_CHANGE_EVENT_PATH)
            .unwrap(),
        (None, 0),
    );
}

#[test]
fn test_reconfiguration() {
    // When executing a transaction emits a validator set change, storage should propagate the new
    // validator set

    let (mut config, genesis_key) = config_builder::test_config();
    if let Some(test_config) = config.test.as_mut() {
        test_config.publishing_option = Some(VMPublishingOption::CustomScripts);
    }

    let (_db, mut executor) = create_db_and_executor(&config);
    let parent_block_id = executor.committed_block_id();

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
    let validator_pubkey = keys.public_key();
    let auth_key = AuthenticationKey::ed25519(&validator_pubkey);
    let validator_auth_key_prefix = auth_key.prefix().to_vec();
    assert!(
        auth_key.derived_address() == validator_account,
        "Address derived from validator auth key does not match validator account address"
    );

    // give the validator some money so they can send a tx
    let txn1 = get_test_signed_transaction(
        genesis_account,
        /* sequence_number = */ 1,
        genesis_key.clone(),
        genesis_key.public_key(),
        Some(encode_transfer_with_metadata_script(
            lbr_type_tag(),
            &validator_account,
            validator_auth_key_prefix,
            1_000_000,
            vec![],
            vec![],
        )),
    );
    // Create a dummy block prologue transaction that will bump the timer.
    let txn2 = encode_block_prologue_script(gen_block_metadata(1, validator_account));

    // rotate the validator's connsensus pubkey to trigger a reconfiguration
    let new_pubkey = Ed25519PrivateKey::generate_for_testing().public_key();
    let txn3 = get_test_signed_transaction(
        validator_account,
        /* sequence_number = */ 0,
        validator_privkey.clone(),
        validator_pubkey.clone(),
        Some(encode_rotate_consensus_pubkey_script(
            new_pubkey.to_bytes().to_vec(),
        )),
    );

    let txn_block = vec![txn1, txn2, txn3];
    let vm_output = executor
        .execute_block((HashValue::random(), txn_block), parent_block_id)
        .unwrap();

    // Make sure the execution result sees the reconfiguration
    assert!(
        vm_output.has_reconfiguration(),
        "StateComputeResult is missing the new validator set"
    );

    // rotating to the same key should not trigger a reconfiguration
    let txn4 = encode_block_prologue_script(gen_block_metadata(2, validator_account));
    let txn5 = get_test_signed_transaction(
        validator_account,
        /* sequence_number = */ 1,
        validator_privkey,
        validator_pubkey,
        Some(encode_rotate_consensus_pubkey_script(
            new_pubkey.to_bytes().to_vec(),
        )),
    );
    let txn_block = vec![txn4, txn5];
    let output = executor
        .execute_block((HashValue::random(), txn_block), parent_block_id)
        .unwrap();

    assert!(
        !output.has_reconfiguration(),
        "StateComputeResult has a new validator set, but should not"
    );

    // TODO: test rotating to invalid key. Currently, this crashes the executor because the
    // validator set fails to parse
}

#[test]
fn test_change_publishing_option_to_custom() {
    // Publishing Option is set to locked at genesis.
    let (mut config, genesis_key) = config_builder::test_config();

    let (db, mut executor) = create_db_and_executor(&config);
    let parent_block_id = executor.committed_block_id();

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
    let validator_pubkey = keys.public_key();

    let signer = extract_signer(&mut config);

    let auth_key = AuthenticationKey::ed25519(&validator_pubkey);
    let validator_auth_key_prefix = auth_key.prefix().to_vec();
    assert_eq!(
        auth_key.derived_address(),
        validator_account,
        "Address derived from validator auth key does not match validator account address"
    );

    // give the validator some money so they can send a tx
    let txn1 = get_test_signed_transaction(
        genesis_account,
        /* sequence_number = */ 1,
        genesis_key.clone(),
        genesis_key.public_key(),
        Some(encode_transfer_with_metadata_script(
            lbr_type_tag(),
            &validator_account,
            validator_auth_key_prefix,
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
        /* sequence_number = */ 2,
        genesis_key.clone(),
        genesis_key.public_key(),
        Some(encode_publishing_option_script(
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
    let mut trusted_state = TrustedState::from(config.base.waypoint.unwrap());
    match trusted_state.verify_and_ratchet(&li, &epoch_change_proof) {
        Ok(TrustedStateChange::Epoch { new_state, .. }) => trusted_state = new_state,
        _ => panic!("unexpected state change"),
    }
    let current_version = li.ledger_info().version();
    assert_eq!(current_version, 3);
    // Transaction 1 is committed as it's in the whitelist
    let txn1 = db
        .reader
        .get_txn_by_account(genesis_account, 1, current_version, false)
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
    let validator_pubkey = keys.public_key();
    let signer = extract_signer(&mut config);
    let auth_key = AuthenticationKey::ed25519(&validator_pubkey);
    let validator_auth_key_prefix = auth_key.prefix().to_vec();
    assert!(
        auth_key.derived_address() == validator_account,
        "Address derived from validator auth key does not match validator account address"
    );

    // give the validator some money so they can send a tx
    let txn1 = get_test_signed_transaction(
        genesis_account,
        /* sequence_number = */ 1,
        genesis_key.clone(),
        genesis_key.public_key(),
        Some(encode_transfer_with_metadata_script(
            lbr_type_tag(),
            &validator_account,
            validator_auth_key_prefix,
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
        existing_list.push(*HashValue::from_sha3_256(&[]).as_ref());
        existing_list
    };

    let txn5 = get_test_signed_transaction(
        genesis_account,
        /* sequence_number = */ 2,
        genesis_key.clone(),
        genesis_key.public_key(),
        Some(encode_publishing_option_script(VMPublishingOption::Locked(
            new_whitelist,
        ))),
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
    let mut trusted_state = TrustedState::from(config.base.waypoint.unwrap());
    match trusted_state.verify_and_ratchet(&li, &epoch_change_proof) {
        Ok(TrustedStateChange::Epoch { new_state, .. }) => trusted_state = new_state,
        _ => panic!("unexpected state change"),
    }
    let current_version = li.ledger_info().version();
    assert_eq!(current_version, 3);
    // Transaction 1 is committed as it's in the whitelist
    let t1 = db
        .reader
        .get_txn_by_account(genesis_account, 1, current_version, false)
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
    let genesis_account = association_address();

    // Create account1 with 2M coins.
    let txn1 = get_test_signed_transaction(
        genesis_account,
        /* sequence_number = */ 1,
        genesis_key.clone(),
        genesis_key.public_key(),
        Some(encode_mint_script(
            lbr_type_tag(),
            &account1,
            account1_auth_key.prefix().to_vec(),
            2_000_000,
        )),
    );

    // Create account2 with 1.2M coins.
    let txn2 = get_test_signed_transaction(
        genesis_account,
        /* sequence_number = */ 2,
        genesis_key.clone(),
        genesis_key.public_key(),
        Some(encode_mint_script(
            lbr_type_tag(),
            &account2,
            account2_auth_key.prefix().to_vec(),
            1_200_000,
        )),
    );

    // Create account3 with 1M coins.
    let txn3 = get_test_signed_transaction(
        genesis_account,
        /* sequence_number = */ 3,
        genesis_key.clone(),
        genesis_key.public_key(),
        Some(encode_mint_script(
            lbr_type_tag(),
            &account3,
            account3_auth_key.prefix().to_vec(),
            1_000_000,
        )),
    );

    // Transfer 20k coins from account1 to account2.
    // balance: <1.98M, 1.22M, 1M
    let txn4 = get_test_signed_transaction(
        account1,
        /* sequence_number = */ 0,
        privkey1.clone(),
        pubkey1.clone(),
        Some(encode_transfer_with_metadata_script(
            lbr_type_tag(),
            &account2,
            account2_auth_key.prefix().to_vec(),
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
        Some(encode_transfer_with_metadata_script(
            lbr_type_tag(),
            &account3,
            account3_auth_key.prefix().to_vec(),
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
        Some(encode_transfer_with_metadata_script(
            lbr_type_tag(),
            &account3,
            account3_auth_key.prefix().to_vec(),
            70_000,
            vec![],
            vec![],
        )),
    );

    let block1 = vec![txn1, txn2, txn3, txn4, txn5, txn6];
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
            Some(encode_transfer_with_metadata_script(
                lbr_type_tag(),
                &account3,
                account3_auth_key.prefix().to_vec(),
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
    let mut trusted_state = TrustedState::from(config.base.waypoint.unwrap());
    match trusted_state.verify_and_ratchet(&li, &epoch_change_proof) {
        Ok(TrustedStateChange::Epoch { new_state, .. }) => trusted_state = new_state,
        _ => panic!("unexpected state change"),
    }
    let current_version = li.ledger_info().version();
    assert_eq!(trusted_state.latest_version(), 6);

    let t1 = db
        .reader
        .get_txn_by_account(genesis_account, 1, current_version, false)
        .unwrap();
    verify_committed_txn_status(t1.as_ref(), &block1[0]).unwrap();

    let t2 = db
        .reader
        .get_txn_by_account(genesis_account, 2, current_version, false)
        .unwrap();
    verify_committed_txn_status(t2.as_ref(), &block1[1]).unwrap();

    let t3 = db
        .reader
        .get_txn_by_account(genesis_account, 3, current_version, false)
        .unwrap();
    verify_committed_txn_status(t3.as_ref(), &block1[2]).unwrap();

    let tn = db
        .reader
        .get_txn_by_account(genesis_account, 4, current_version, false)
        .unwrap();
    assert!(tn.is_none());

    let t4 = db
        .reader
        .get_txn_by_account(account1, 0, current_version, true)
        .unwrap();
    verify_committed_txn_status(t4.as_ref(), &block1[3]).unwrap();
    // We requested the events to come back from this one, so verify that they did
    assert_eq!(t4.unwrap().events.unwrap().len(), 2);

    let t5 = db
        .reader
        .get_txn_by_account(account2, 0, current_version, false)
        .unwrap();
    verify_committed_txn_status(t5.as_ref(), &block1[4]).unwrap();

    let t6 = db
        .reader
        .get_txn_by_account(account1, 1, current_version, true)
        .unwrap();
    verify_committed_txn_status(t6.as_ref(), &block1[5]).unwrap();

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
    assert_eq!(current_version, 20);

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
        .get_transactions(7, 14, current_version, false)
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
            .get_balance_resources(&[from_currency_code_string(LBR_NAME).unwrap()])?
            .last()
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

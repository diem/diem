// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use anyhow::Result;
use config_builder::test_config;
use executor::{
    db_bootstrapper::{generate_waypoint, maybe_bootstrap},
    Executor,
};
use executor_test_helpers::{
    bootstrap_genesis, extract_signer, gen_ledger_info_with_sigs, get_test_signed_transaction,
};
use executor_types::BlockExecutor;
use libra_config::utils::get_genesis_txn;
use libra_crypto::{
    ed25519::Ed25519PrivateKey, test_utils::TEST_SEED, HashValue, PrivateKey, Uniform,
};
use libra_temppath::TempPath;
use libra_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config::{
        coin1_tag, from_currency_code_string, libra_root_address, testnet_dd_account_address,
        BalanceResource, COIN1_NAME,
    },
    account_state::AccountState,
    account_state_blob::AccountStateBlob,
    contract_event::ContractEvent,
    on_chain_config,
    on_chain_config::{config_address, ConfigurationResource, OnChainConfig, ValidatorSet},
    proof::SparseMerkleRangeProof,
    transaction::{
        authenticator::AuthenticationKey, ChangeSet, Transaction, Version, WriteSetPayload,
        PRE_GENESIS_VERSION,
    },
    trusted_state::TrustedState,
    validator_signer::ValidatorSigner,
    waypoint::Waypoint,
    write_set::{WriteOp, WriteSetMut},
};
use libra_vm::LibraVM;
use libradb::{GetRestoreHandler, LibraDB};
use move_core_types::move_resource::MoveResource;
use rand::SeedableRng;
use std::{convert::TryFrom, sync::Arc};
use storage_interface::{DbReader, DbReaderWriter};
use transaction_builder::{
    encode_create_testing_account_script, encode_peer_to_peer_with_metadata_script,
};

#[test]
fn test_empty_db() {
    let (config, _) = test_config();
    let tmp_dir = TempPath::new();
    let db_rw = DbReaderWriter::new(LibraDB::new_for_test(&tmp_dir));

    // Executor won't be able to boot on empty db due to lack of StartupInfo.
    assert!(db_rw.reader.get_startup_info().unwrap().is_none());

    // Bootstrap empty DB.
    let genesis_txn = get_genesis_txn(&config).unwrap();
    let waypoint = generate_waypoint::<LibraVM>(&db_rw, genesis_txn).expect("Should not fail.");
    maybe_bootstrap::<LibraVM>(&db_rw, genesis_txn, waypoint).unwrap();
    let startup_info = db_rw
        .reader
        .get_startup_info()
        .expect("Should not fail.")
        .expect("Should not be None.");
    assert_eq!(
        Waypoint::new_epoch_boundary(startup_info.latest_ledger_info.ledger_info()).unwrap(),
        waypoint
    );
    let (li, epoch_change_proof, _) = db_rw.reader.get_state_proof(waypoint.version()).unwrap();
    let trusted_state = TrustedState::from(waypoint);
    trusted_state
        .verify_and_ratchet(&li, &epoch_change_proof)
        .unwrap();

    // `maybe_bootstrap()` does nothing on non-empty DB.
    assert!(!maybe_bootstrap::<LibraVM>(&db_rw, genesis_txn, waypoint).unwrap());
}

fn execute_and_commit(txns: Vec<Transaction>, db: &DbReaderWriter, signer: &ValidatorSigner) {
    let block_id = HashValue::random();
    let li = db.reader.get_latest_ledger_info().unwrap();
    let version = li.ledger_info().version();
    let epoch = li.ledger_info().next_block_epoch();
    let target_version = version + txns.len() as u64;
    let mut executor = Executor::<LibraVM>::new(db.clone());
    let output = executor
        .execute_block((block_id, txns), executor.committed_block_id())
        .unwrap();
    assert_eq!(output.num_leaves(), target_version + 1);
    let ledger_info_with_sigs = gen_ledger_info_with_sigs(epoch, output, block_id, vec![&signer]);
    executor
        .commit_blocks(vec![block_id], ledger_info_with_sigs)
        .unwrap();
}

fn get_demo_accounts() -> (
    AccountAddress,
    Ed25519PrivateKey,
    AccountAddress,
    Ed25519PrivateKey,
) {
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

    (account1, privkey1, account2, privkey2)
}

fn get_mint_transaction(
    libra_root_key: &Ed25519PrivateKey,
    libra_root_seq_num: u64,
    account: &AccountAddress,
    amount: u64,
) -> Transaction {
    get_test_signed_transaction(
        testnet_dd_account_address(),
        /* sequence_number = */ libra_root_seq_num,
        libra_root_key.clone(),
        libra_root_key.public_key(),
        Some(encode_peer_to_peer_with_metadata_script(
            coin1_tag(),
            *account,
            amount,
            vec![],
            vec![],
        )),
    )
}

fn get_account_transaction(
    libra_root_key: &Ed25519PrivateKey,
    libra_root_seq_num: u64,
    account: &AccountAddress,
    account_key: &Ed25519PrivateKey,
) -> Transaction {
    let account_auth_key = AuthenticationKey::ed25519(&account_key.public_key());
    get_test_signed_transaction(
        libra_root_address(),
        /* sequence_number = */ libra_root_seq_num,
        libra_root_key.clone(),
        libra_root_key.public_key(),
        Some(encode_create_testing_account_script(
            coin1_tag(),
            *account,
            account_auth_key.prefix().to_vec(),
            false,
        )),
    )
}

fn get_transfer_transaction(
    sender: AccountAddress,
    sender_seq_number: u64,
    sender_key: &Ed25519PrivateKey,
    recipient: AccountAddress,
    amount: u64,
) -> Transaction {
    get_test_signed_transaction(
        sender,
        sender_seq_number,
        sender_key.clone(),
        sender_key.public_key(),
        Some(encode_peer_to_peer_with_metadata_script(
            coin1_tag(),
            recipient,
            amount,
            vec![],
            vec![],
        )),
    )
}

fn get_balance(account: &AccountAddress, db: &DbReaderWriter) -> u64 {
    let account_state_blob = db
        .reader
        .get_latest_account_state(*account)
        .unwrap()
        .unwrap();
    let account_state = AccountState::try_from(&account_state_blob).unwrap();
    account_state
        .get_balance_resources(&[from_currency_code_string(COIN1_NAME).unwrap()])
        .unwrap()
        .get(&from_currency_code_string(COIN1_NAME).unwrap())
        .unwrap()
        .coin()
}

fn get_configuration(db: &DbReaderWriter) -> ConfigurationResource {
    let config_blob = db
        .reader
        .get_latest_account_state(config_address())
        .unwrap()
        .unwrap();
    let config_state = AccountState::try_from(&config_blob).unwrap();
    config_state.get_configuration_resource().unwrap().unwrap()
}

fn get_state_backup(
    db: &LibraDB,
) -> (
    Vec<(HashValue, AccountStateBlob)>,
    SparseMerkleRangeProof,
    HashValue,
) {
    let backup_handler = db.get_backup_handler();
    let accounts = backup_handler
        .get_account_iter(4)
        .unwrap()
        .collect::<Result<Vec<_>>>()
        .unwrap();
    let proof = backup_handler
        .get_account_state_range_proof(accounts.last().unwrap().0, 1)
        .unwrap();
    let root_hash = db.get_latest_state_root().unwrap().1;

    (accounts, proof, root_hash)
}

fn restore_state_to_db(
    db: &Arc<LibraDB>,
    accounts: Vec<(HashValue, AccountStateBlob)>,
    proof: SparseMerkleRangeProof,
    root_hash: HashValue,
    version: Version,
) {
    let rh = db.get_restore_handler();
    let mut receiver = rh.get_state_restore_receiver(version, root_hash).unwrap();
    for (chunk, proof) in vec![(accounts, proof)].into_iter() {
        receiver.add_chunk(chunk, proof).unwrap();
    }
    receiver.finish().unwrap();
}

#[test]
fn test_pre_genesis() {
    let (mut config, genesis_key) = config_builder::test_config();

    // Create bootstrapped DB.
    let tmp_dir = TempPath::new();
    let (db, db_rw) = DbReaderWriter::wrap(LibraDB::new_for_test(&tmp_dir));
    let signer = extract_signer(&mut config);
    let genesis_txn = get_genesis_txn(&config).unwrap().clone();
    let waypoint = bootstrap_genesis::<LibraVM>(&db_rw, &genesis_txn).unwrap();

    // Mint for 2 demo accounts.
    let (account1, account1_key, account2, account2_key) = get_demo_accounts();
    let txn1 = get_account_transaction(&genesis_key, 1, &account1, &account1_key);
    let txn2 = get_account_transaction(&genesis_key, 2, &account2, &account2_key);
    let txn3 = get_mint_transaction(&genesis_key, 0, &account1, 2000);
    let txn4 = get_mint_transaction(&genesis_key, 1, &account2, 2000);
    execute_and_commit(vec![txn1, txn2, txn3, txn4], &db_rw, &signer);
    assert_eq!(get_balance(&account1, &db_rw), 2000);
    assert_eq!(get_balance(&account2, &db_rw), 2000);

    // Get state tree backup.
    let (accounts_backup, proof, root_hash) = get_state_backup(&db);
    // Restore into PRE-GENESIS state of a new empty DB.
    let tmp_dir = TempPath::new();
    let (db, db_rw) = DbReaderWriter::wrap(LibraDB::new_for_test(&tmp_dir));
    restore_state_to_db(&db, accounts_backup, proof, root_hash, PRE_GENESIS_VERSION);

    // DB is not empty, `maybe_bootstrap()` will try to apply and fail the waypoint check.
    assert!(maybe_bootstrap::<LibraVM>(&db_rw, &genesis_txn, waypoint).is_err());
    // Nor is it able to boot Executor.
    assert!(db_rw.reader.get_startup_info().unwrap().is_none());

    // New genesis transaction: set validator set and overwrite account1 balance
    let genesis_txn = Transaction::GenesisTransaction(WriteSetPayload::Direct(ChangeSet::new(
        WriteSetMut::new(vec![
            (
                ValidatorSet::CONFIG_ID.access_path(),
                WriteOp::Value(lcs::to_bytes(&ValidatorSet::new(vec![])).unwrap()),
            ),
            (
                AccessPath::new(account1, BalanceResource::access_path_for(coin1_tag())),
                WriteOp::Value(lcs::to_bytes(&BalanceResource::new(1000)).unwrap()),
            ),
        ])
        .freeze()
        .unwrap(),
        vec![ContractEvent::new(
            on_chain_config::new_epoch_event_key(),
            0,
            coin1_tag(),
            vec![],
        )],
    )));

    // Bootstrap DB on top of pre-genesis state.
    let waypoint = generate_waypoint::<LibraVM>(&db_rw, &genesis_txn).unwrap();
    assert!(maybe_bootstrap::<LibraVM>(&db_rw, &genesis_txn, waypoint).unwrap());
    let (li, epoch_change_proof, _) = db_rw.reader.get_state_proof(waypoint.version()).unwrap();
    let trusted_state = TrustedState::from(waypoint);
    trusted_state
        .verify_and_ratchet(&li, &epoch_change_proof)
        .unwrap();

    // Effect of bootstrapping reflected.
    assert_eq!(get_balance(&account1, &db_rw), 1000);
    // Pre-genesis state accessible.
    assert_eq!(get_balance(&account2, &db_rw), 2000);
}

#[test]
fn test_new_genesis() {
    let (mut config, genesis_key) = config_builder::test_config();
    // Create bootstrapped DB.
    let tmp_dir = TempPath::new();
    let db = DbReaderWriter::new(LibraDB::new_for_test(&tmp_dir));
    let waypoint = {
        let genesis_txn = get_genesis_txn(&config).unwrap();
        bootstrap_genesis::<LibraVM>(&db, genesis_txn).unwrap()
    };
    let signer = extract_signer(&mut config);

    // Mint for 2 demo accounts.
    let (account1, account1_key, account2, account2_key) = get_demo_accounts();
    let txn1 = get_account_transaction(&genesis_key, 1, &account1, &account1_key);
    let txn2 = get_account_transaction(&genesis_key, 2, &account2, &account2_key);
    let txn3 = get_mint_transaction(&genesis_key, 0, &account1, 2_000_000);
    let txn4 = get_mint_transaction(&genesis_key, 1, &account2, 2_000_000);
    execute_and_commit(vec![txn1, txn2, txn3, txn4], &db, &signer);
    assert_eq!(get_balance(&account1, &db), 2_000_000);
    assert_eq!(get_balance(&account2, &db), 2_000_000);
    let (li, epoch_change_proof, _) = db.reader.get_state_proof(waypoint.version()).unwrap();
    let trusted_state = TrustedState::from(waypoint);
    trusted_state
        .verify_and_ratchet(&li, &epoch_change_proof)
        .unwrap();

    // New genesis transaction: set validator set, bump epoch and overwrite account1 balance.
    let configuration = get_configuration(&db);
    let genesis_txn = Transaction::GenesisTransaction(WriteSetPayload::Direct(ChangeSet::new(
        WriteSetMut::new(vec![
            (
                ValidatorSet::CONFIG_ID.access_path(),
                WriteOp::Value(lcs::to_bytes(&ValidatorSet::new(vec![])).unwrap()),
            ),
            (
                AccessPath::new(config_address(), ConfigurationResource::resource_path()),
                WriteOp::Value(lcs::to_bytes(&configuration.bump_epoch_for_test()).unwrap()),
            ),
            (
                AccessPath::new(account1, BalanceResource::access_path_for(coin1_tag())),
                WriteOp::Value(lcs::to_bytes(&BalanceResource::new(1_000_000)).unwrap()),
            ),
        ])
        .freeze()
        .unwrap(),
        vec![ContractEvent::new(
            *configuration.events().key(),
            0,
            coin1_tag(),
            vec![],
        )],
    )));

    // Bootstrap DB into new genesis.
    let waypoint = generate_waypoint::<LibraVM>(&db, &genesis_txn).unwrap();
    assert!(maybe_bootstrap::<LibraVM>(&db, &genesis_txn, waypoint).unwrap());
    assert_eq!(waypoint.version(), 5);

    // Client bootable from waypoint.
    let trusted_state = TrustedState::from(waypoint);
    let (li, epoch_change_proof, accumulator_consistency_proof) = db
        .reader
        .get_state_proof(trusted_state.latest_version())
        .unwrap();
    assert_eq!(li.ledger_info().version(), 5);
    assert!(accumulator_consistency_proof.subtrees().is_empty());
    trusted_state
        .verify_and_ratchet(&li, &epoch_change_proof)
        .unwrap();

    // Effect of bootstrapping reflected.
    assert_eq!(get_balance(&account1, &db), 1_000_000);
    // State before new genesis accessible.
    assert_eq!(get_balance(&account2, &db), 2_000_000);

    // Transfer some money.
    let txn = get_transfer_transaction(account1, 0, &account1_key, account2, 500_000);
    execute_and_commit(vec![txn], &db, &signer);

    // And verify.
    assert_eq!(get_balance(&account2, &db), 2_500_000);
}

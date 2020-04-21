// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use anyhow::Result;
use config_builder::test_config;
use executor::{
    db_bootstrapper::{bootstrap_db, bootstrap_db_if_empty},
    BlockExecutor, Executor,
};
use executor_utils::test_helpers::{gen_ledger_info_with_sigs, get_test_signed_transaction};
use libra_config::utils::get_genesis_txn;
use libra_crypto::{
    ed25519::Ed25519PrivateKey, test_utils::TEST_SEED, HashValue, PrivateKey, Uniform,
};
use libra_temppath::TempPath;
use libra_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config::{association_address, lbr_type_tag, BalanceResource},
    account_state::AccountState,
    account_state_blob::AccountStateBlob,
    contract_event::ContractEvent,
    move_resource::MoveResource,
    on_chain_config,
    on_chain_config::{OnChainConfig, ValidatorSet},
    proof::SparseMerkleRangeProof,
    transaction::{
        authenticator::AuthenticationKey, ChangeSet, Transaction, Version, PRE_GENESIS_VERSION,
    },
    trusted_state::TrustedState,
    waypoint::Waypoint,
    write_set::{WriteOp, WriteSetMut},
};
use libra_vm::LibraVM;
use libradb::LibraDB;
use rand::SeedableRng;
use std::convert::TryFrom;
use storage_interface::{DbReader, DbReaderWriter};
use transaction_builder::encode_create_account_script;

#[test]
fn test_empty_db() {
    let (config, _) = test_config();
    let tmp_dir = TempPath::new();
    let db_rw = DbReaderWriter::new(LibraDB::new(&tmp_dir));

    // Executor won't be able to boot on empty db due to lack of StartupInfo.
    assert!(db_rw.reader.get_startup_info().unwrap().is_none());

    // Bootstrap empty DB.
    let genesis_txn = get_genesis_txn(&config).unwrap();
    let waypoint = bootstrap_db_if_empty::<LibraVM>(&db_rw, genesis_txn)
        .expect("Should not fail.")
        .expect("Should not be None.");
    let startup_info = db_rw
        .reader
        .get_startup_info()
        .expect("Should not fail.")
        .expect("Should not be None.");
    assert_eq!(
        Waypoint::new(startup_info.latest_ledger_info.ledger_info()).unwrap(),
        waypoint
    );
    let (li, validator_change_proof, _) = db_rw.reader.get_state_proof(waypoint.version()).unwrap();
    let trusted_state = TrustedState::from_waypoint(waypoint);
    trusted_state
        .verify_and_ratchet(&li, &validator_change_proof)
        .unwrap();

    // `bootstrap_db_if_empty()` does nothing on non-empty DB.
    assert!(bootstrap_db_if_empty::<LibraVM>(&db_rw, genesis_txn)
        .unwrap()
        .is_none())
}

fn execute_and_commit(txns: Vec<Transaction>, db: &DbReaderWriter) {
    let block_id = HashValue::random();
    let version = db
        .reader
        .get_latest_ledger_info()
        .unwrap()
        .ledger_info()
        .version();
    let target_version = version + txns.len() as u64;
    let mut executor = Executor::<LibraVM>::new(db.clone());
    let output = executor
        .execute_block((block_id, txns), executor.committed_block_id())
        .unwrap();
    let ledger_info_with_sigs =
        gen_ledger_info_with_sigs(target_version, output.root_hash(), block_id);
    executor
        .commit_blocks(vec![block_id], ledger_info_with_sigs)
        .unwrap();
}

fn get_demo_accounts() -> (
    AccountAddress,
    AuthenticationKey,
    AccountAddress,
    AuthenticationKey,
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

    (account1, account1_auth_key, account2, account2_auth_key)
}

fn get_mint_transaction(
    association_key: &Ed25519PrivateKey,
    association_seq_num: u64,
    account: &AccountAddress,
    account_auth_key: &AuthenticationKey,
    amount: u64,
) -> Transaction {
    get_test_signed_transaction(
        association_address(),
        /* sequence_number = */ association_seq_num,
        association_key.clone(),
        association_key.public_key(),
        Some(encode_create_account_script(
            lbr_type_tag(),
            &account,
            account_auth_key.prefix().to_vec(),
            amount,
        )),
    )
}

fn get_balance(account: &AccountAddress, db: &DbReaderWriter) -> u64 {
    let account_state_blob = db
        .reader
        .get_latest_account_state(account.clone())
        .unwrap()
        .unwrap();
    let account_state = AccountState::try_from(&account_state_blob).unwrap();
    account_state
        .get_balance_resource()
        .unwrap()
        .unwrap()
        .coin()
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
        .get_account_iter(2)
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
    db: &LibraDB,
    accounts: Vec<(HashValue, AccountStateBlob)>,
    proof: SparseMerkleRangeProof,
    root_hash: HashValue,
    version: Version,
) {
    db.restore_account_state(vec![(accounts, proof)].into_iter(), version, root_hash)
        .unwrap();
}

#[test]
fn test_pre_genesis() {
    let (config, genesis_key) = config_builder::test_config();

    // Create bootstrapped DB.
    let tmp_dir = TempPath::new();
    let (db, db_rw) = DbReaderWriter::wrap(LibraDB::new(&tmp_dir));
    let genesis_txn = get_genesis_txn(&config).unwrap();
    bootstrap_db_if_empty::<LibraVM>(&db_rw, genesis_txn).unwrap();

    // Mint for 2 demo accounts.
    let (account1, account1_auth_key, account2, account2_auth_key) = get_demo_accounts();
    let txn1 = get_mint_transaction(&genesis_key, 1, &account1, &account1_auth_key, 2000);
    let txn2 = get_mint_transaction(&genesis_key, 2, &account2, &account2_auth_key, 2000);
    execute_and_commit(vec![txn1, txn2], &db_rw);
    assert_eq!(get_balance(&account1, &db_rw), 2000);
    assert_eq!(get_balance(&account2, &db_rw), 2000);

    // Get state tree backup.
    let (accounts_backup, proof, root_hash) = get_state_backup(&db);
    // Restore into PRE-GENESIS state of a new empty DB.
    let tmp_dir = TempPath::new();
    let (db, db_rw) = DbReaderWriter::wrap(LibraDB::new(&tmp_dir));
    restore_state_to_db(&db, accounts_backup, proof, root_hash, PRE_GENESIS_VERSION);

    // DB is not empty, `bootstrap_db_if_empty()` won't apply default genesis txn.
    assert!(bootstrap_db_if_empty::<LibraVM>(&db_rw, genesis_txn)
        .unwrap()
        .is_none());
    // Nor is it able to boot Executor.
    assert!(db_rw.reader.get_startup_info().unwrap().is_none());

    // New genesis transaction: set validator set and overwrite account1 balance
    let genesis_txn = Transaction::WaypointWriteSet(ChangeSet::new(
        WriteSetMut::new(vec![
            (
                ValidatorSet::CONFIG_ID.access_path(),
                WriteOp::Value(lcs::to_bytes(&ValidatorSet::new(vec![])).unwrap()),
            ),
            (
                AccessPath::new(account1, BalanceResource::resource_path()),
                WriteOp::Value(lcs::to_bytes(&BalanceResource::new(1000)).unwrap()),
            ),
        ])
        .freeze()
        .unwrap(),
        vec![ContractEvent::new(
            on_chain_config::new_epoch_event_key(),
            0,
            lbr_type_tag(),
            vec![],
        )],
    ));

    // Bootstrap DB on top of pre-genesis state.
    let tree_state = db_rw.reader.get_latest_tree_state().unwrap();
    let waypoint = bootstrap_db::<LibraVM>(&db_rw, tree_state, &genesis_txn).unwrap();
    let (li, validator_change_proof, _) = db_rw.reader.get_state_proof(waypoint.version()).unwrap();
    let trusted_state = TrustedState::from_waypoint(waypoint);
    trusted_state
        .verify_and_ratchet(&li, &validator_change_proof)
        .unwrap();

    // Effect of bootstrapping reflected.
    assert_eq!(get_balance(&account1, &db_rw), 1000);
    // Pre-genesis state accessible.
    assert_eq!(get_balance(&account2, &db_rw), 2000);
}

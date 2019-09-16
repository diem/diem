use tools::tempdir::TempPath;
use crate::{LibraDB, mock_genesis::*, data_storage::*};
use types::{transaction::TransactionToCommit, ledger_info::LedgerInfoWithSignatures};
use logger::prelude::*;
use failure::prelude::*;
use crypto::{
    ed25519::*,
    hash::{CryptoHash, ACCUMULATOR_PLACEHOLDER_HASH, GENESIS_BLOCK_ID},
    HashValue,
};
use rand::{
    rngs::{OsRng, StdRng},
    Rng, SeedableRng,
};
use types::account_address::AccountAddress;
use types::transaction::{RawTransaction, Program};
use types::account_state_blob::AccountStateBlob;
use std::collections::HashMap;

fn data_storage_create() -> DataStorage {
    let tmp_path = TempPath::new();
    tmp_path.create_as_dir();
    let path: &str = tmp_path.path().to_str().unwrap();
    let (db, _) = DataStorage::new(LibraDB::new(path));
    db
}

fn data_storage_genesis(data_storage: &DataStorage) -> Result<()> {
    let signatures = GENESIS_INFO.1.clone();
    let genesis_txn = GENESIS_INFO.2.clone();
    data_storage.save_genesis_transactions(vec![genesis_txn],  &Some(signatures))
}

fn new_commit_tx() -> TransactionToCommit {
    let mut seed_rng = OsRng::new().expect("can't access OsRng");
    let seed_buf: [u8; 32] = seed_rng.gen();
    let mut rng = StdRng::from_seed(seed_buf);
    let (privkey, pubkey) = compat::generate_keypair(&mut rng);
    let some_addr = AccountAddress::from_public_key(&pubkey);
    let raw_txn = RawTransaction::new(
        some_addr,
        /* sequence_number = */ 0,
        Program::new(vec![], vec![], vec![]),
        /* max_gas_amount = */ 0,
        /* gas_unit_price = */ 0,
        /* expiration_time = */ std::time::Duration::new(0, 0),
    );
    let signed_txn = raw_txn
        .sign(&privkey, pubkey)
        .expect("Signing failed.")
        .into_inner();
    let signed_txn_hash = signed_txn.hash();

    let some_blob = AccountStateBlob::from(vec![1u8]);
    let account_states = vec![(some_addr, some_blob.clone())]
        .into_iter()
        .collect::<HashMap<_, _>>();

    TransactionToCommit::new(
        signed_txn,
        account_states.clone(),
        vec![], /* events */
        0, /* gas_used */
    )
}

#[test]
fn test_data_storage_create() {
    let data_storage = data_storage_create();
    assert_eq!(data_storage.genesis_state().unwrap(), Some(false));
}

#[test]
fn test_data_storage_save_genesis_tx() {
    let data_storage = data_storage_create();
    assert_eq!(data_storage.genesis_state().unwrap(), Some(false));
    let genesis_result = data_storage_genesis(&data_storage);
    match genesis_result {
        Ok(tmp) => {
            println!("{}", "save genesis tx succ.");
        }
        Err(e) => {
            println!("{:?}", e);
        }
    }

    assert_eq!(data_storage.genesis_state().unwrap(), Some(true));
}

#[test]
fn test_data_storage_save_tx() {
    let data_storage = data_storage_create();
    let genesis_result = data_storage_genesis(&data_storage);
    let commit_tx = new_commit_tx();
    let result = data_storage.save_transactions(vec![commit_tx], 1);
    match result {
        Ok(tmp) => {
            println!("{}", "save txs succ.");
            assert_eq!(true, true);
        }
        Err(e) => {
            println!("{:?}", e);
            assert_eq!(true, false);
        }
    }
}
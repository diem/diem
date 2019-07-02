// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::LibraDB;
use crypto::hash::{CryptoHash, SPARSE_MERKLE_PLACEHOLDER_HASH};
use tempfile::tempdir;
use types::{
    account_address::{AccountAddress, ADDRESS_LENGTH},
    account_state_blob::AccountStateBlob,
    proof::verify_sparse_merkle_element,
};

fn put_account_state_set(
    db: &LibraDB,
    account_state_set: Vec<(AccountAddress, AccountStateBlob)>,
    version: Version,
    root_hash: HashValue,
) -> HashValue {
    let mut batch = SchemaBatch::new();
    let root = db
        .state_store
        .put_account_state_sets(
            vec![account_state_set.into_iter().collect::<HashMap<_, _>>()],
            version,
            root_hash,
            &mut batch,
        )
        .unwrap()[0];
    db.commit(batch).unwrap();

    root
}

fn verify_state_in_store(
    store: &StateStore,
    address: AccountAddress,
    expected_value: Option<&AccountStateBlob>,
    root: HashValue,
) {
    let (value, proof) = store
        .get_account_state_with_proof_by_state_root(address, root)
        .unwrap();
    assert_eq!(value.as_ref(), expected_value);
    verify_sparse_merkle_element(root, address.hash(), &value, &proof).unwrap();
}

#[test]
fn test_empty_store() {
    let tmp_dir = tempdir().unwrap();
    let db = LibraDB::new(&tmp_dir);
    let store = &db.state_store;
    let address = AccountAddress::new([1u8; ADDRESS_LENGTH]);
    let root = *SPARSE_MERKLE_PLACEHOLDER_HASH;
    let (value, proof) = store
        .get_account_state_with_proof_by_state_root(address, root)
        .unwrap();
    assert!(value.is_none());
    assert!(verify_sparse_merkle_element(root, address.hash(), &None, &proof).is_ok());
}

#[test]
fn test_state_store_reader_writer() {
    let tmp_dir = tempdir().unwrap();
    let db = LibraDB::new(&tmp_dir);
    let store = &db.state_store;
    let address1 = AccountAddress::new([1u8; ADDRESS_LENGTH]);
    let address2 = AccountAddress::new([2u8; ADDRESS_LENGTH]);
    let address3 = AccountAddress::new([3u8; ADDRESS_LENGTH]);
    let value1 = AccountStateBlob::from(vec![0x01]);
    let value1_update = AccountStateBlob::from(vec![0x00]);
    let value2 = AccountStateBlob::from(vec![0x02]);
    let value3 = AccountStateBlob::from(vec![0x03]);
    let mut root = *SPARSE_MERKLE_PLACEHOLDER_HASH;

    // Verify initial states.
    verify_state_in_store(&store, address1, None, root);
    verify_state_in_store(&store, address2, None, root);
    verify_state_in_store(&store, address3, None, root);

    // Insert address1 with value 1 and verify new states.
    root = put_account_state_set(
        &db,
        vec![(address1, value1.clone())],
        0, /* version */
        root,
    );
    verify_state_in_store(&store, address1, Some(&value1), root);
    verify_state_in_store(&store, address2, None, root);
    verify_state_in_store(&store, address3, None, root);

    // Insert address 1 with updated value1, address2 with value 2 and address3 with value3 and
    // verify new states.
    root = put_account_state_set(
        &db,
        vec![
            (address1, value1_update.clone()),
            (address2, value2.clone()),
            (address3, value3.clone()),
        ],
        1, /* version */
        root,
    );
    verify_state_in_store(&store, address1, Some(&value1_update), root);
    verify_state_in_store(&store, address2, Some(&value2), root);
    verify_state_in_store(&store, address3, Some(&value3), root);
}

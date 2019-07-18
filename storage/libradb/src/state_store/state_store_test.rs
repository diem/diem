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
    store: &StateStore,
    account_state_set: Vec<(AccountAddress, AccountStateBlob)>,
    version: Version,
    root_hash: HashValue,
    expected_nodes_created: usize,
    expected_nodes_retired: usize,
    expected_blobs_retired: usize,
) -> HashValue {
    let mut cs = ChangeSet::new();
    let blobs_created = account_state_set.len();
    let root = store
        .put_account_state_sets(
            vec![account_state_set.into_iter().collect::<HashMap<_, _>>()],
            version,
            root_hash,
            &mut cs,
        )
        .unwrap()[0];
    store.db.write_schemas(cs.batch).unwrap();
    assert_eq!(
        cs.counters.get(LedgerCounter::StateNodesCreated),
        expected_nodes_created
    );
    assert_eq!(
        cs.counters.get(LedgerCounter::StateNodesRetired),
        expected_nodes_retired
    );
    assert_eq!(
        cs.counters.get(LedgerCounter::StateBlobsCreated),
        blobs_created
    );
    assert_eq!(
        cs.counters.get(LedgerCounter::StateBlobsRetired),
        expected_blobs_retired
    );

    root
}

fn purge_retired_records(
    store: &StateStore,
    least_readable_version: Version,
    limit: usize,
    expected_num_purged: usize,
) {
    let mut cs = ChangeSet::new();
    let num_purged = store
        .purge_retired_records(least_readable_version, limit, &mut cs)
        .unwrap();
    assert_eq!(num_purged, expected_num_purged);
    store.db.write_schemas(cs.batch).unwrap();
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
    verify_state_in_store(store, address1, None, root);
    verify_state_in_store(store, address2, None, root);
    verify_state_in_store(store, address3, None, root);

    // Insert address1 with value 1 and verify new states.
    root = put_account_state_set(
        store,
        vec![(address1, value1.clone())],
        0, /* version */
        root,
        1, /* expected_nodes_created */
        0, /* expected_nodes_retired */
        0, /* expected_blobs_retired */
    );
    verify_state_in_store(store, address1, Some(&value1), root);
    verify_state_in_store(store, address2, None, root);
    verify_state_in_store(store, address3, None, root);

    // Insert address 1 with updated value1, address2 with value 2 and address3 with value3 and
    // verify new states.
    root = put_account_state_set(
        store,
        vec![
            (address1, value1_update.clone()),
            (address2, value2.clone()),
            (address3, value3.clone()),
        ],
        1, /* version */
        root,
        4, /* expected_nodes_created */
        1, /* expected_nodes_retired */
        1, /* expected_blobs_retired */
    );
    verify_state_in_store(store, address1, Some(&value1_update), root);
    verify_state_in_store(store, address2, Some(&value2), root);
    verify_state_in_store(store, address3, Some(&value3), root);
}

#[test]
fn test_purge_retired_records() {
    let address1 = AccountAddress::new([1u8; ADDRESS_LENGTH]);
    let address2 = AccountAddress::new([2u8; ADDRESS_LENGTH]);
    let address3 = AccountAddress::new([3u8; ADDRESS_LENGTH]);
    let value1 = AccountStateBlob::from(vec![0x01]);
    let value2 = AccountStateBlob::from(vec![0x02]);
    let value2_update = AccountStateBlob::from(vec![0x12]);
    let value3 = AccountStateBlob::from(vec![0x03]);
    let value3_update = AccountStateBlob::from(vec![0x13]);
    let root_default = *SPARSE_MERKLE_PLACEHOLDER_HASH;

    let tmp_dir = tempdir().unwrap();
    let db = LibraDB::new(&tmp_dir);
    let store = &db.state_store;

    // Update.
    // ```text
    // | batch    | 0      | 1             | 2             |
    // | address1 | value1 |               |               |
    // | address2 | value2 | value2_update |               |
    // | address3 |        | value3        | value3_update |
    // ```
    let root0 = put_account_state_set(
        store,
        vec![(address1, value1.clone()), (address2, value2.clone())],
        0, /* version */
        root_default,
        3, /* expected_nodes_created */
        0, /* expected_nodes_retired */
        0, /* expected_blobs_retired */
    );
    let root1 = put_account_state_set(
        store,
        vec![
            (address2, value2_update.clone()),
            (address3, value3.clone()),
        ],
        1, /* version */
        root0,
        3, /* expected_nodes_created */
        2, /* expected_nodes_retired */
        1, /* expected_blobs_retired */
    );
    let root2 = put_account_state_set(
        store,
        vec![(address3, value3_update.clone())],
        2, /* version */
        root1,
        2, /* expected_nodes_created */
        2, /* expected_nodes_retired */
        1, /* expected_blobs_retired */
    );

    // Verify.
    // Purge with limit=0, nothing is gone.
    {
        purge_retired_records(
            store, 1, /* least_readable_version */
            0, /* limit */
            0, /* expected_num_purged */
        );
        verify_state_in_store(store, address1, Some(&value1), root0);
    }
    // Purge till version=1.
    {
        purge_retired_records(
            store, 1,   /* least_readable_version */
            100, /* limit */
            3,   /* expected_num_purged */
        );
        // root0 is gone.
        assert!(store
            .get_account_state_with_proof_by_state_root(address2, root0)
            .is_err());
        // root1 is still there.
        verify_state_in_store(store, address1, Some(&value1), root1);
        verify_state_in_store(store, address2, Some(&value2_update), root1);
        verify_state_in_store(store, address3, Some(&value3), root1);
    }
    // Purge till version=2.
    {
        purge_retired_records(
            store, 2,   /* least_readable_version */
            100, /* limit */
            3,   /* expected_num_purged */
        );
        // root1 is gone.
        assert!(store
            .get_account_state_with_proof_by_state_root(address2, root1)
            .is_err());
        // root2 is still there.
        verify_state_in_store(store, address1, Some(&value1), root2);
        verify_state_in_store(store, address2, Some(&value2_update), root2);
        verify_state_in_store(store, address3, Some(&value3_update), root2);
    }
}

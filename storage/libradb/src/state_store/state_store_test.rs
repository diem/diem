// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::{pruner, LibraDB};
use libra_crypto::hash::CryptoHash;
use libra_tools::tempdir::TempPath;
use libra_types::{
    account_address::{AccountAddress, ADDRESS_LENGTH},
    account_state_blob::AccountStateBlob,
};
use proptest::{collection::hash_map, prelude::*};

fn put_account_state_set(
    store: &StateStore,
    account_state_set: Vec<(AccountAddress, AccountStateBlob)>,
    version: Version,
    expected_new_nodes: usize,
    expected_stale_nodes: usize,
    expected_stale_leaves: usize,
) -> HashValue {
    let mut cs = ChangeSet::new();
    let expected_new_leaves = account_state_set.len();
    let root = store
        .put_account_state_sets(
            vec![account_state_set.into_iter().collect::<HashMap<_, _>>()],
            version,
            &mut cs,
        )
        .unwrap()[0];
    store.db.write_schemas(cs.batch).unwrap();
    assert_eq!(
        cs.counter_bumps.get(LedgerCounter::NewStateNodes),
        expected_new_nodes
    );
    assert_eq!(
        cs.counter_bumps.get(LedgerCounter::StaleStateNodes),
        expected_stale_nodes
    );
    assert_eq!(
        cs.counter_bumps.get(LedgerCounter::NewStateLeaves),
        expected_new_leaves
    );
    assert_eq!(
        cs.counter_bumps.get(LedgerCounter::StaleStateLeaves),
        expected_stale_leaves
    );

    root
}

fn prune_stale_indices(
    store: &StateStore,
    least_readable_version: Version,
    target_least_readable_version: Version,
    limit: usize,
) {
    pruner::prune_state(
        Arc::clone(&store.db),
        least_readable_version,
        target_least_readable_version,
        limit,
    )
    .unwrap();
}

fn verify_state_in_store(
    store: &StateStore,
    address: AccountAddress,
    expected_value: Option<&AccountStateBlob>,
    version: Version,
    root: HashValue,
) {
    let (value, proof) = store
        .get_account_state_with_proof_by_version(address, version)
        .unwrap();
    assert_eq!(value.as_ref(), expected_value);
    proof.verify(root, address.hash(), value.as_ref()).unwrap();
}

#[test]
fn test_empty_store() {
    let tmp_dir = TempPath::new();
    let db = LibraDB::new(&tmp_dir);
    let store = &db.state_store;
    let address = AccountAddress::new([1u8; ADDRESS_LENGTH]);
    assert!(store
        .get_account_state_with_proof_by_version(address, 0)
        .is_err());
}

#[test]
fn test_state_store_reader_writer() {
    let tmp_dir = TempPath::new();
    let db = LibraDB::new(&tmp_dir);
    let store = &db.state_store;
    let address1 = AccountAddress::new([1u8; ADDRESS_LENGTH]);
    let address2 = AccountAddress::new([2u8; ADDRESS_LENGTH]);
    let address3 = AccountAddress::new([3u8; ADDRESS_LENGTH]);
    let value1 = AccountStateBlob::from(vec![0x01]);
    let value1_update = AccountStateBlob::from(vec![0x00]);
    let value2 = AccountStateBlob::from(vec![0x02]);
    let value3 = AccountStateBlob::from(vec![0x03]);

    // Insert address1 with value 1 and verify new states.
    let mut root = put_account_state_set(
        store,
        vec![(address1, value1.clone())],
        0, /* version */
        1, /* expected_nodes_created */
        0, /* expected_nodes_retired */
        0, /* expected_blobs_retired */
    );
    verify_state_in_store(store, address1, Some(&value1), 0, root);
    verify_state_in_store(store, address2, None, 0, root);
    verify_state_in_store(store, address3, None, 0, root);

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
        4, /* expected_nodes_created */
        1, /* expected_nodes_retired */
        1, /* expected_blobs_retired */
    );
    verify_state_in_store(store, address1, Some(&value1_update), 1, root);
    verify_state_in_store(store, address2, Some(&value2), 1, root);
    verify_state_in_store(store, address3, Some(&value3), 1, root);
}

#[test]
fn test_retired_records() {
    let address1 = AccountAddress::new([1u8; ADDRESS_LENGTH]);
    let address2 = AccountAddress::new([2u8; ADDRESS_LENGTH]);
    let address3 = AccountAddress::new([3u8; ADDRESS_LENGTH]);
    let value1 = AccountStateBlob::from(vec![0x01]);
    let value2 = AccountStateBlob::from(vec![0x02]);
    let value2_update = AccountStateBlob::from(vec![0x12]);
    let value3 = AccountStateBlob::from(vec![0x03]);
    let value3_update = AccountStateBlob::from(vec![0x13]);

    let tmp_dir = TempPath::new();
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
        3, /* expected_nodes_created */
        2, /* expected_nodes_retired */
        1, /* expected_blobs_retired */
    );
    let root2 = put_account_state_set(
        store,
        vec![(address3, value3_update.clone())],
        2, /* version */
        2, /* expected_nodes_created */
        2, /* expected_nodes_retired */
        1, /* expected_blobs_retired */
    );

    // Verify.
    // Prune with limit=0, nothing is gone.
    {
        prune_stale_indices(
            store, 0, /* least_readable_version */
            1, /* target_least_readable_version */
            0, /* limit */
        );
        verify_state_in_store(store, address1, Some(&value1), 0, root0);
    }
    // Prune till version=1.
    {
        prune_stale_indices(
            store, 0,   /* least_readable_version */
            1,   /* target_least_readable_version */
            100, /* limit */
        );
        // root0 is gone.
        assert!(store
            .get_account_state_with_proof_by_version(address2, 0)
            .is_err());
        // root1 is still there.
        verify_state_in_store(store, address1, Some(&value1), 1, root1);
        verify_state_in_store(store, address2, Some(&value2_update), 1, root1);
        verify_state_in_store(store, address3, Some(&value3), 1, root1);
    }
    // Prune till version=2.
    {
        prune_stale_indices(
            store, 1,   /* least_readable_version */
            2,   /* target_least_readable_version */
            100, /* limit */
        );
        // root1 is gone.
        assert!(store
            .get_account_state_with_proof_by_version(address2, 1)
            .is_err());
        // root2 is still there.
        verify_state_in_store(store, address1, Some(&value1), 2, root2);
        verify_state_in_store(store, address2, Some(&value2_update), 2, root2);
        verify_state_in_store(store, address3, Some(&value3_update), 2, root2);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_get_account_iter(
        input in hash_map(any::<AccountAddress>(), any::<AccountStateBlob>(), 1..200)
    ) {
        // Convert to a vector so iteration order becomes deterministic.
        let kvs: Vec<_> = input.into_iter().collect();

        let tmp_dir = TempPath::new();
        let db = LibraDB::new(&tmp_dir);
        let store = &db.state_store;

        for (i, (key, value)) in kvs.iter().enumerate() {
            // Insert one key at each version.
            let mut cs = ChangeSet::new();
            let account_state_set: HashMap<_, _> = std::iter::once((*key, value.clone())).collect();
            store
                .put_account_state_sets(vec![account_state_set], i as Version, &mut cs)
                .unwrap();
            store.db.write_schemas(cs.batch).unwrap();
        }

        // Test iterator at each version.
        for i in 0..kvs.len() {
            let actual_values = db.get_account_iter(i as Version)
                .unwrap()
                .collect::<Result<Vec<_>>>()
                .unwrap();
            let mut expected_values: Vec<_> = kvs[..=i]
                .iter()
                .map(|(addr, account)| (addr.hash(), account.clone()))
                .collect();
            expected_values.sort_unstable_by_key(|item| item.0);
            prop_assert_eq!(actual_values, expected_values);
        }
    }
}

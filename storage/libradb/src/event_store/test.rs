// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::LibraDB;
use crypto::{hash::ACCUMULATOR_PLACEHOLDER_HASH, utils::keypair_strategy};
use itertools::Itertools;
use proptest::{
    collection::{hash_set, vec},
    prelude::*,
    strategy::Union,
};
use rand::{Rng, StdRng};
use std::collections::HashMap;
use tempfile::tempdir;
use types::{
    account_address::AccountAddress, contract_event::ContractEvent,
    proof::verify_event_accumulator_element, proptest_types::renumber_events,
};

fn save(store: &EventStore, version: Version, events: &[ContractEvent]) -> HashValue {
    let mut batch = SchemaBatch::new();
    let root_hash = store.put_events(version, events, &mut batch).unwrap();
    store.db.write_schemas(batch).unwrap();

    root_hash
}

#[test]
fn test_put_empty() {
    let tmp_dir = tempdir().unwrap();
    let db = LibraDB::new(&tmp_dir);
    let store = &db.event_store;
    let mut batch = SchemaBatch::new();
    assert_eq!(
        store.put_events(0, &[], &mut batch).unwrap(),
        *ACCUMULATOR_PLACEHOLDER_HASH
    );
}

#[test]
fn test_error_on_get_from_empty() {
    let tmp_dir = tempdir().unwrap();
    let db = LibraDB::new(&tmp_dir);
    let store = &db.event_store;

    assert!(store
        .get_event_with_proof_by_version_and_index(100, 0)
        .is_err());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_put_get_verify(events in vec(any::<ContractEvent>().no_shrink(), 1..100)) {
        let tmp_dir = tempdir().unwrap();
        let db = LibraDB::new(&tmp_dir);
        let store = &db.event_store;

        let root_hash = save(store, 100, &events);

        // get and verify each and every event with proof
        for (idx, expected_event) in events.iter().enumerate() {
            let (event, proof) = store
                .get_event_with_proof_by_version_and_index(100, idx as u64)
                .unwrap();
            prop_assert_eq!(&event, expected_event);
            verify_event_accumulator_element(root_hash, event.hash(),  idx as u64, &proof).unwrap();
        }
        // error on index >= num_events
        prop_assert!(store
            .get_event_with_proof_by_version_and_index(100, events.len() as u64)
            .is_err());
    }

}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1))]

    #[test]
    fn test_get_all_events_by_version(
        events1 in vec(any::<ContractEvent>().no_shrink(), 1..100),
        events2 in vec(any::<ContractEvent>().no_shrink(), 1..100),
        events3 in vec(any::<ContractEvent>().no_shrink(), 1..100),
    ) {

        let tmp_dir = tempdir().unwrap();
        let db = LibraDB::new(&tmp_dir);
        let store = &db.event_store;
        // Save 3 chunks at different versions
        save(store, 99 /*version*/, &events1);
        save(store, 100 /*version*/, &events2);
        save(store, 101 /*version*/, &events3);

        // Now get all events at each version and verify that it matches what is expected.
        let events_99 = store.get_events_by_version(99 /*version*/).unwrap();
        prop_assert_eq!(events_99, events1);

        let events_100 = store.get_events_by_version(100 /*version*/).unwrap();
        prop_assert_eq!(events_100, events2);

        let events_101 = store.get_events_by_version(101 /*version*/).unwrap();
        prop_assert_eq!(events_101, events3);

        // Now query a version that doesn't exist and verify that no results come back
        let events_102 = store.get_events_by_version(102 /*version*/).unwrap();
        prop_assert_eq!(events_102.len(), 0);
    }
}

fn traverse_events_by_access_path(
    store: &EventStore,
    access_path: &AccessPath,
    ledger_version: Version,
) -> Vec<ContractEvent> {
    const LIMIT: u64 = 3;

    let mut seq_num = 0;

    let mut event_keys = Vec::new();
    let mut last_batch_len = LIMIT;
    loop {
        let mut batch = store
            .lookup_events_by_access_path(access_path, seq_num, LIMIT, ledger_version)
            .unwrap();
        if last_batch_len < LIMIT {
            assert!(batch.is_empty());
        }
        if batch.is_empty() {
            break;
        }

        last_batch_len = batch.len() as u64;
        let first_seq = batch.first().unwrap().0;
        let last_seq = batch.last().unwrap().0;

        assert!(last_batch_len <= LIMIT);
        assert_eq!(seq_num, first_seq);
        assert_eq!(seq_num + last_batch_len - 1, last_seq);

        event_keys.extend(batch.iter());
        seq_num = last_seq + 1;
    }

    event_keys
        .into_iter()
        .map(|(_seq, ver, idx)| {
            store
                .get_event_with_proof_by_version_and_index(ver, idx)
                .unwrap()
                .0
        })
        .collect()
}

fn arb_event_batches() -> impl Strategy<Value = (Vec<AccessPath>, Vec<Vec<ContractEvent>>)> {
    (
        vec(any::<AccountAddress>(), 3),
        hash_set(any::<Vec<u8>>(), 3),
        (0..100usize),
    )
        .prop_flat_map(|(addresses, event_paths, num_batches)| {
            let all_possible_access_paths = addresses
                .iter()
                .cartesian_product(event_paths.iter())
                .map(|(address, event_path)| AccessPath::new(*address, event_path.clone()))
                .collect::<Vec<_>>();
            let access_path_strategy =
                Union::new(all_possible_access_paths.clone().into_iter().map(Just));

            (
                Just(all_possible_access_paths),
                vec(
                    vec(ContractEvent::strategy_impl(access_path_strategy), 0..10),
                    num_batches,
                ),
            )
        })
        .prop_map(|(all_possible_access_paths, event_batches)| {
            let mut seq_num_by_access_path = HashMap::new();
            let numbered_event_batches = event_batches
                .into_iter()
                .map(|events| renumber_events(&events, &mut seq_num_by_access_path))
                .collect::<Vec<_>>();

            (all_possible_access_paths, numbered_event_batches)
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_get_events_by_access_path((addresses, event_batches) in arb_event_batches().no_shrink()) {
        test_get_events_by_access_path_impl(addresses, event_batches);
    }
}

fn test_get_events_by_access_path_impl(
    access_paths: Vec<AccessPath>,
    event_batches: Vec<Vec<ContractEvent>>,
) {
    // Put into db.
    let tmp_dir = tempdir().unwrap();
    let db = LibraDB::new(&tmp_dir);
    let store = &db.event_store;

    let mut batch = SchemaBatch::new();
    event_batches.iter().enumerate().for_each(|(ver, events)| {
        store.put_events(ver as u64, events, &mut batch).unwrap();
    });
    db.commit(batch);
    let ledger_version_plus_one = event_batches.len() as u64;

    // Calculate expected event sequence per access_path.
    let mut events_by_access_path = HashMap::new();
    event_batches.into_iter().for_each(|batch| {
        batch.into_iter().for_each(|e| {
            let mut events = events_by_access_path
                .entry(e.access_path().clone())
                .or_insert_with(Vec::new);
            assert_eq!(events.len() as u64, e.sequence_number());
            events.push(e.clone());
        })
    });

    // Fetch and check.
    events_by_access_path
        .into_iter()
        .for_each(|(path, events)| {
            let traversed = traverse_events_by_access_path(&store, &path, ledger_version_plus_one);
            assert_eq!(events, traversed);
        });
}

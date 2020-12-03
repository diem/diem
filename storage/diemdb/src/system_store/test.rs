// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::{
    change_set::ChangeSet,
    ledger_counters::{LedgerCounter, LedgerCounterBumps},
    DiemDB,
};
use diem_temppath::TempPath;
use std::collections::HashMap;

fn bump_ledger_counters(
    store: &SystemStore,
    first_version: Version,
    last_version: Version,
    counter_bumps: HashMap<Version, LedgerCounterBumps>,
) -> LedgerCounters {
    let mut cs = ChangeSet::new_with_bumps(counter_bumps);
    let counters = store
        .bump_ledger_counters(first_version, last_version, &mut cs)
        .unwrap();
    store.db.write_schemas(cs.batch).unwrap();

    counters
}

fn create_bumps_map(
    first_version: Version,
    events_created_list: Vec<usize>,
) -> HashMap<Version, LedgerCounterBumps> {
    let mut bumps_map = HashMap::new();
    events_created_list
        .into_iter()
        .enumerate()
        .for_each(|(i, count)| {
            let mut bumps = LedgerCounterBumps::new();
            bumps.bump(LedgerCounter::EventsCreated, count);
            bumps_map.insert(first_version + i as u64, bumps);
        });
    bumps_map
}

#[test]
fn test_inc_ledger_counters() {
    let tmp_dir = TempPath::new();
    let db = DiemDB::new_for_test(&tmp_dir);
    let store = &db.system_store;

    // First batch, add to zeros.
    {
        let bumps = create_bumps_map(0, vec![3, 7]);

        let counters = bump_ledger_counters(
            store, 0, /* first_version */
            1, /* last_version */
            bumps,
        );
        assert_eq!(counters.get(LedgerCounter::EventsCreated), 10);
    }
    // Second batch, add to the first batch.
    {
        let bumps = create_bumps_map(2, vec![5; 9]);

        let counters = bump_ledger_counters(
            store, 2,  /* first_version */
            10, /* last_version */
            bumps,
        );
        assert_eq!(counters.get(LedgerCounter::EventsCreated), 55);
    }
    // Add to an old version
    {
        let bumps = create_bumps_map(4, vec![1, 2, 3, 4, 5]);

        let counters = bump_ledger_counters(
            store, 4, /* first_version */
            8, /* last_version */
            bumps,
        );
        assert_eq!(counters.get(LedgerCounter::EventsCreated), 35);
    }
    // Base version and some entries are missing, swallowing the error and adding to zeros.
    {
        let bumps = create_bumps_map(12, vec![5, 10]);

        let counters = bump_ledger_counters(
            store, 12, /* first_version */
            14, /* last_version */
            bumps,
        );
        assert_eq!(counters.get(LedgerCounter::EventsCreated), 15);
    }
    // if a counter at a version doesn't bump, it retains the same value as the last version.
    {
        let counters = bump_ledger_counters(
            store,
            15, /* first_version */
            15, /* last_version */
            HashMap::new(),
        );
        assert_eq!(counters.get(LedgerCounter::EventsCreated), 15);
    }
}

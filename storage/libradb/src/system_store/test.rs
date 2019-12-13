// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::{ledger_counters::LedgerCounter, LibraDB};
use libra_tools::tempdir::TempPath;

fn bump_ledger_counters(
    store: &SystemStore,
    first_version: Version,
    last_version: Version,
    bumps: LedgerCounterBumps,
) -> LedgerCounters {
    let mut batch = SchemaBatch::new();
    let counters = store
        .bump_ledger_counters(first_version, last_version, bumps, &mut batch)
        .unwrap();
    store.db.write_schemas(batch).unwrap();

    counters
}

#[test]
fn test_inc_ledger_counters() {
    let tmp_dir = TempPath::new();
    let db = LibraDB::new(&tmp_dir);
    let store = &db.system_store;

    // First batch, add to zeros.
    {
        let mut bumps = LedgerCounterBumps::new();
        bumps.bump(LedgerCounter::EventsCreated, 10);

        let counters = bump_ledger_counters(
            store, 0, /* first_version */
            1, /* last_version */
            bumps,
        );
        assert_eq!(counters.get(LedgerCounter::EventsCreated), 10);
    }
    // Second batch, add to the first batch.
    {
        let mut bumps = LedgerCounterBumps::new();
        bumps.bump(LedgerCounter::EventsCreated, 20);

        let counters = bump_ledger_counters(
            store, 2,  /* first_version */
            10, /* last_version */
            bumps,
        );
        assert_eq!(counters.get(LedgerCounter::EventsCreated), 30);
    }
    // Add to an old version.
    {
        let mut bumps = LedgerCounterBumps::new();
        bumps.bump(LedgerCounter::EventsCreated, 5);

        let counters = bump_ledger_counters(
            store, 2, /* first_version */
            8, /* last_version */
            bumps,
        );
        assert_eq!(counters.get(LedgerCounter::EventsCreated), 15);
    }
    // Base version missing, swallowing the error and adding to zeros.
    {
        let mut bumps = LedgerCounterBumps::new();
        bumps.bump(LedgerCounter::EventsCreated, 1);

        let counters = bump_ledger_counters(
            store, 3, /* first_version */
            8, /* last_version */
            bumps,
        );
        assert_eq!(counters.get(LedgerCounter::EventsCreated), 1);
    }
}

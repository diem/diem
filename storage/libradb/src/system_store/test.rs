use super::*;
use crate::{ledger_counters::LedgerCounter, LibraDB};
use tempfile::tempdir;

fn inc_ledger_counters(
    db: &LibraDB,
    first_version: Version,
    last_version: Version,
    diff: LedgerCounters,
) -> LedgerCounters {
    let mut batch = SchemaBatch::new();
    let counters = db
        .system_store
        .inc_ledger_counters(first_version, last_version, diff, &mut batch)
        .unwrap();
    db.commit(batch).unwrap();

    counters
}

#[test]
fn test_inc_ledger_counters() {
    let tmp_dir = tempdir().unwrap();
    let db = LibraDB::new(&tmp_dir);

    // First batch, add to zeros.
    {
        let mut diff = LedgerCounters::new();
        diff.inc(LedgerCounter::EventsCreated, 10);

        let counters = inc_ledger_counters(
            &db, 0, /* first_version */
            1, /* last_version */
            diff,
        );
        assert_eq!(counters.get(LedgerCounter::EventsCreated), 10);
    }
    // Second batch, add to the first batch.
    {
        let mut diff = LedgerCounters::new();
        diff.inc(LedgerCounter::EventsCreated, 20);

        let counters = inc_ledger_counters(
            &db, 2,  /* first_version */
            10, /* last_version */
            diff,
        );
        assert_eq!(counters.get(LedgerCounter::EventsCreated), 30);
    }
    // Add to an old version.
    {
        let mut diff = LedgerCounters::new();
        diff.inc(LedgerCounter::EventsCreated, 5);

        let counters = inc_ledger_counters(
            &db, 2, /* first_version */
            8, /* last_version */
            diff,
        );
        assert_eq!(counters.get(LedgerCounter::EventsCreated), 15);
    }
    // Base version missing, swallowing the error and adding to zeros.
    {
        let mut diff = LedgerCounters::new();
        diff.inc(LedgerCounter::EventsCreated, 1);

        let counters = inc_ledger_counters(
            &db, 3, /* first_version */
            8, /* last_version */
            diff,
        );
        assert_eq!(counters.get(LedgerCounter::EventsCreated), 1);
    }
}

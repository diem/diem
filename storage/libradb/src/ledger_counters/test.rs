use super::*;

#[test]
fn test_ledger_counters() {
    // Defaults to 0.
    let mut counters = LedgerCounters::new();
    assert_eq!(counters.get(LedgerCounter::StateBlobsCreated), 0);

    // Bump
    let mut bumps = LedgerCounterBumps::new();
    bumps
        .bump(LedgerCounter::StateBlobsCreated, 1)
        .bump(LedgerCounter::StateBlobsRetired, 1);
    counters.bump(bumps);
    assert_eq!(counters.get(LedgerCounter::StateBlobsCreated), 1);
    assert_eq!(counters.get(LedgerCounter::StateBlobsRetired), 1);

    // Bump again.
    let mut bumps = LedgerCounterBumps::new();
    bumps
        .bump(LedgerCounter::EventsCreated, 1)
        .bump(LedgerCounter::StateBlobsCreated, 1);
    counters.bump(bumps);
    assert_eq!(counters.get(LedgerCounter::EventsCreated), 1);
    assert_eq!(counters.get(LedgerCounter::StateBlobsCreated), 2);
    assert_eq!(counters.get(LedgerCounter::StateBlobsRetired), 1);
}

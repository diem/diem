use super::*;

#[test]
fn test_ledger_counters() {
    // Defaults to 0.
    let mut counters = LedgerCounters::new();
    assert_eq!(counters.get(LedgerCounter::StateBlobsCreated), 0);

    // inc()
    counters
        .inc(LedgerCounter::StateBlobsCreated, 1)
        .inc(LedgerCounter::StateBlobsRetired, 1);
    assert_eq!(counters.get(LedgerCounter::StateBlobsCreated), 1);
    assert_eq!(counters.get(LedgerCounter::StateBlobsRetired), 1);

    let mut counters_diff = LedgerCounters::new();
    counters_diff
        .inc(LedgerCounter::EventsCreated, 1)
        .inc(LedgerCounter::StateBlobsCreated, 1);

    // combine()
    counters.combine(counters_diff);
    assert_eq!(counters.get(LedgerCounter::EventsCreated), 1);
    assert_eq!(counters.get(LedgerCounter::StateBlobsCreated), 2);
    assert_eq!(counters.get(LedgerCounter::StateBlobsRetired), 1);
}

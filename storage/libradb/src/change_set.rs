use crate::ledger_counters::LedgerCounters;
use schemadb::SchemaBatch;

/// Structure that collects changes to be made to the DB in one transaction.
///
/// To be specific it carries a batch of db alternations and counter increases that'll be converted
/// to DB alternations on "sealing". This is required to be converted to `SealedChangeSet` before
/// committing to the DB.
pub(crate) struct ChangeSet {
    /// A batch of db alternations.
    pub batch: SchemaBatch,
    /// Counter increases to be make on commit.
    pub counters: LedgerCounters,
}

impl ChangeSet {
    /// Constructor.
    pub fn new() -> Self {
        Self {
            batch: SchemaBatch::new(),
            counters: LedgerCounters::new(),
        }
    }
}

/// ChangeSet that's ready to be committed to the DB.
///
/// This is a wrapper type just to make sure `ChangeSet` to be committed is sealed properly.
pub(crate) struct SealedChangeSet {
    /// A batch of db alternations.
    pub batch: SchemaBatch,
}

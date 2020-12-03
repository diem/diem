// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::ledger_counters::LedgerCounterBumps;
use diem_types::transaction::Version;
use schemadb::SchemaBatch;
use std::collections::HashMap;

/// Structure that collects changes to be made to the DB in one transaction.
///
/// To be specific it carries a batch of db alternations and counter increases that'll be converted
/// to DB alternations on "sealing". This is required to be converted to `SealedChangeSet` before
/// committing to the DB.
pub(crate) struct ChangeSet {
    /// A batch of db alternations.
    pub batch: SchemaBatch,
    /// Counter bumps to be made on commit.
    counter_bumps: HashMap<Version, LedgerCounterBumps>,
}

impl ChangeSet {
    /// Constructor.
    pub fn new() -> Self {
        Self {
            batch: SchemaBatch::new(),
            counter_bumps: HashMap::new(),
        }
    }

    pub fn counter_bumps(&mut self, version: Version) -> &mut LedgerCounterBumps {
        self.counter_bumps
            .entry(version)
            .or_insert_with(LedgerCounterBumps::new)
    }

    #[cfg(test)]
    pub fn new_with_bumps(counter_bumps: HashMap<Version, LedgerCounterBumps>) -> Self {
        Self {
            batch: SchemaBatch::new(),
            counter_bumps,
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

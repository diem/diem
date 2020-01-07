// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::OP_COUNTER;
use num_derive::ToPrimitive;
use num_traits::ToPrimitive;
use once_cell::sync::Lazy;
use prometheus::IntGaugeVec;
#[cfg(test)]
use proptest::{collection::hash_map, prelude::*};
#[cfg(test)]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, EnumIter};

// register Prometheus counters
pub static LIBRA_STORAGE_LEDGER: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        // metric name
        "libra_storage_ledger",
        // metric description
        "Libra storage ledger counters",
        // metric labels (dimensions)
        &["type"]
    )
    .unwrap()
});

/// Types of ledger counters.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, ToPrimitive, EnumIter, AsRefStr)]
#[cfg_attr(test, derive(Arbitrary))]
#[strum(serialize_all = "snake_case")]
pub(crate) enum LedgerCounter {
    EventsCreated = 101,

    NewStateLeaves = 201,
    StaleStateLeaves = 202,

    NewStateNodes = 301,
    StaleStateNodes = 302,
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
struct InnerLedgerCounters {
    counters: BTreeMap<u16, usize>,
}

impl InnerLedgerCounters {
    pub fn new() -> Self {
        Self {
            counters: BTreeMap::new(),
        }
    }

    fn raw_key(counter: LedgerCounter) -> u16 {
        counter
            .to_u16()
            .expect("LedgerCounter should convert to u16.")
    }

    fn get(&self, counter: LedgerCounter) -> usize {
        self.counters
            .get(&Self::raw_key(counter))
            .cloned()
            .unwrap_or(0)
    }

    fn inc(&mut self, counter: LedgerCounter, by: usize) -> &mut Self {
        self.raw_inc(Self::raw_key(counter), by)
    }

    fn raw_inc(&mut self, key: u16, by: usize) -> &mut Self {
        let value = self.counters.entry(key).or_insert(0);
        *value += by;

        self
    }
}

/// Represents `LedgerCounter` bumps yielded by saving a batch of transactions.
pub(crate) struct LedgerCounterBumps {
    bumps: InnerLedgerCounters,
}

impl LedgerCounterBumps {
    /// Construsts an empty set of bumps.
    pub fn new() -> Self {
        Self {
            bumps: InnerLedgerCounters::new(),
        }
    }

    /// Makes the bump of a certain counter bigger.
    ///
    /// If a bump has not already been recorded for the counter, assumes current value of 0.
    pub fn bump(&mut self, counter: LedgerCounter, by: usize) -> &mut Self {
        self.bumps.inc(counter, by);

        self
    }

    /// Get the current value of the bump of `counter`.
    ///
    /// Defaults to 0.
    #[cfg(test)]
    pub fn get(&mut self, counter: LedgerCounter) -> usize {
        self.bumps.get(counter)
    }
}

/// Represents ledger counter values at a certain version.
#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub(crate) struct LedgerCounters {
    counters: InnerLedgerCounters,
}

impl LedgerCounters {
    /// Constructs a new empty counter set.
    pub fn new() -> Self {
        Self {
            counters: InnerLedgerCounters::new(),
        }
    }

    /// Bump each counter in `bumps` with the value in `bumps`.
    pub fn bump(&mut self, bumps: LedgerCounterBumps) -> &mut Self {
        for (key, value) in bumps.bumps.counters.into_iter() {
            self.counters.raw_inc(key, value);
        }

        self
    }

    /// Bump Prometheus counters.
    pub fn bump_op_counters(&self) {
        for counter in LedgerCounter::iter() {
            OP_COUNTER.set(counter.as_ref(), self.get(counter));
            LIBRA_STORAGE_LEDGER
                .with_label_values(&[counter.as_ref()])
                .set(self.get(counter) as i64);
        }
    }

    /// Get the value of `counter`.
    pub fn get(&self, counter: LedgerCounter) -> usize {
        self.counters.get(counter)
    }
}

#[cfg(test)]
prop_compose! {
    pub(crate) fn ledger_counters_strategy()(
        counters_map in hash_map(any::<LedgerCounter>(), any::<usize>(), 0..3)
    ) -> LedgerCounters {
        let mut counters = InnerLedgerCounters::new();
        for (counter, value) in counters_map {
            counters.inc(counter, value);
        }

        LedgerCounters { counters }
    }
}

#[cfg(test)]
impl Arbitrary for LedgerCounters {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        ledger_counters_strategy().boxed()
    }
}

#[cfg(test)]
mod test;

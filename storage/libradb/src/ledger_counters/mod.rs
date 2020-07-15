// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{metrics::LIBRA_STORAGE_LEDGER, OP_COUNTER};
use num_derive::ToPrimitive;
use num_traits::ToPrimitive;
use num_variants::NumVariants;
#[cfg(test)]
use proptest::{collection::hash_map, prelude::*};
#[cfg(test)]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Types of ledger counters.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, ToPrimitive, NumVariants)]
#[cfg_attr(test, derive(Arbitrary))]
pub(crate) enum LedgerCounter {
    EventsCreated = 101,

    NewStateLeaves = 201,
    StaleStateLeaves = 202,

    NewStateNodes = 301,
    StaleStateNodes = 302,
}

impl LedgerCounter {
    const VARIANTS: [LedgerCounter; LedgerCounter::NUM_VARIANTS] = [
        LedgerCounter::EventsCreated,
        LedgerCounter::NewStateLeaves,
        LedgerCounter::StaleStateLeaves,
        LedgerCounter::NewStateNodes,
        LedgerCounter::StaleStateNodes,
    ];

    const STR_EVENTS_CREATED: &'static str = "events_created";
    const STR_NEW_STATE_LEAVES: &'static str = "new_state_leaves";
    const STR_STALE_STATE_LEAVES: &'static str = "stale_state_leaves";
    const STR_NEW_STATE_NODES: &'static str = "new_state_nodes";
    const STR_STALE_STATE_NODES: &'static str = "stale_state_nodes";

    pub fn name(self) -> &'static str {
        match self {
            Self::EventsCreated => Self::STR_EVENTS_CREATED,
            Self::NewStateLeaves => Self::STR_NEW_STATE_LEAVES,
            Self::StaleStateLeaves => Self::STR_STALE_STATE_LEAVES,
            Self::NewStateNodes => Self::STR_NEW_STATE_NODES,
            Self::StaleStateNodes => Self::STR_STALE_STATE_NODES,
        }
    }
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
        for counter in &LedgerCounter::VARIANTS {
            OP_COUNTER.set(counter.name(), self.get(*counter));
            LIBRA_STORAGE_LEDGER
                .with_label_values(&[counter.name()])
                .set(self.get(*counter) as i64);
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

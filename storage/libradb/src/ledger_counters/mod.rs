use canonical_serialization::{
    CanonicalDeserialize, CanonicalDeserializer, CanonicalSerialize, CanonicalSerializer,
};
use failure::prelude::*;
use num_derive::ToPrimitive;
use num_traits::ToPrimitive;
use proptest::{collection::hash_map, prelude::*};
use proptest_derive::Arbitrary;
use std::collections::BTreeMap;

#[derive(Arbitrary, Clone, Copy, Debug, Eq, Hash, PartialEq, ToPrimitive)]
pub(crate) enum LedgerCounter {
    EventsCreated = 101,

    StateBlobsCreated = 201,
    StateBlobsRetired = 202,
}

#[derive(Eq, PartialEq, Debug)]
pub(crate) struct LedgerCounters {
    counters: BTreeMap<u16, usize>,
}

impl LedgerCounters {
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

    pub fn get(&self, counter: LedgerCounter) -> usize {
        self.counters
            .get(&Self::raw_key(counter))
            .cloned()
            .unwrap_or(0)
    }

    pub fn inc(&mut self, counter: LedgerCounter, by: usize) -> &mut Self {
        self.raw_inc(Self::raw_key(counter), by)
    }

    fn raw_inc(&mut self, key: u16, by: usize) -> &mut Self {
        let value = self.counters.entry(key).or_insert(0);
        *value += by;

        self
    }

    pub fn combine(&mut self, rhs: Self) -> &mut Self {
        for (key, value) in rhs.counters.into_iter() {
            self.raw_inc(key, value);
        }

        self
    }
}

impl CanonicalSerialize for LedgerCounters {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer.encode_btreemap(&self.counters)?;
        Ok(())
    }
}

impl CanonicalDeserialize for LedgerCounters {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self> {
        let counters = deserializer.decode_btreemap::<u16, usize>()?;

        Ok(Self { counters })
    }
}

prop_compose! {
    fn ledger_counters_strategy()(
        counters_map in hash_map(any::<LedgerCounter>(), any::<usize>(), 0..3)
    ) -> LedgerCounters {
        let mut counters = LedgerCounters::new();
        for (counter, value) in counters_map {
            counters.inc(counter, value);
        }

        counters
    }
}

impl Arbitrary for LedgerCounters {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        ledger_counters_strategy().boxed()
    }
}

#[cfg(test)]
mod test;

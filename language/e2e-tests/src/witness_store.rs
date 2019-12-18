use itertools::Itertools;
use libra_types::account_address::AccountAddress;
use libra_types::channel::Witness;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct FakeWitnessStore {
    store: HashMap<AccountAddress, HashMap<u64, Witness>>,
}

impl FakeWitnessStore {
    pub fn add_witness(&mut self, account: AccountAddress, witness: Witness) {
        let entry = self.store.entry(account);
        match entry {
            Entry::Occupied(mut entry) => {
                let witnesses = entry.get_mut();
                witnesses.insert(witness.channel_sequence_number(), witness);
            }
            Entry::Vacant(entry) => {
                let mut witnesses = HashMap::new();
                witnesses.insert(witness.channel_sequence_number(), witness);
                entry.insert(witnesses);
            }
        }
    }

    pub fn get_witness(
        &self,
        account: &AccountAddress,
        channel_sequence_number: u64,
    ) -> Option<Witness> {
        self.store
            .get(account)
            .and_then(|witnesses| witnesses.get(&channel_sequence_number).cloned())
    }

    pub fn get_latest_witness(&self, account: &AccountAddress) -> Option<Witness> {
        self.store
            .get(account)
            .and_then(|witnesses| {
                witnesses
                    .iter()
                    .sorted_by(|first, second| first.0.cmp(second.0))
                    .collect_vec()
                    .pop()
            })
            .map(|item| item.1.clone())
    }
}

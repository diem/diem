use crate::{
    interface::Store,
    metrics::TIMER,
    types::{DummyValue, FrozenSmt, Kv, Proof},
};
use diem_crypto::HashValue;
use diem_types::transaction::Version;
use rayon::prelude::*;
use scratchpad::{AccountStatus, ProofRead};
use std::{collections::HashMap, convert::identity, sync::Arc};

pub struct ProofReader {
    proofs: HashMap<HashValue, Proof>,
}

impl ProofRead<DummyValue> for ProofReader {
    fn get_proof(&self, key: HashValue) -> Option<&Proof> {
        self.proofs.get(&key)
    }
}

pub struct ProofFetcher {
    pub store: Arc<dyn Store>,
}

impl ProofFetcher {
    pub fn new(store: Arc<dyn Store>) -> Self {
        Self { store }
    }

    pub fn fetch_proofs_for_missing(
        &self,
        smt: &FrozenSmt,
        persisted_version: Version,
        kvs: &[Kv],
    ) -> ProofReader {
        let _timer = TIMER
            .with_label_values(&["fetch_proofs_for_missing"])
            .start_timer();

        let proofs = kvs
            .par_iter()
            .with_min_len(100)
            .map(|(key, _v)| match smt.get(*key) {
                AccountStatus::ExistsInScratchPad(_) => None,
                AccountStatus::DoesNotExist => None,
                AccountStatus::ExistsInDB(depth) | AccountStatus::Unknown(depth) => Some((
                    *key,
                    self.store
                        .get_proof(*key, persisted_version, depth)
                        .unwrap(),
                )),
            })
            .filter_map(identity)
            .collect::<Vec<_>>()
            .into_iter()
            .collect();

        ProofReader { proofs }
    }
}

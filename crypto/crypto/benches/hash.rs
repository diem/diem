// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[macro_use]
extern crate criterion;

use libra_crypto::hash::{CryptoHash, CryptoHasher, HashValue};
use libra_crypto_derive::{CryptoHasher, LCSCryptoHash};
use serde::{Deserialize, Serialize};

const INPUT: &[u8] = &[1u8; 120];

// Main use case
#[derive(CryptoHasher, Serialize, Deserialize, LCSCryptoHash)]
struct HashBencher(&'static [u8]);

// Similar with an hand-written implementation of CryptoHash.
struct HashBencherBase(&'static [u8]);

impl CryptoHash for HashBencherBase {
    type Hasher = HashBencherHasher;
    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        // Assumes self.0.len() < 128
        state.update(&[self.0.len() as u8]);
        state.update(&self.0);
        state.finish()
    }
}

fn bench_hasher(c: &mut criterion::Criterion) {
    let expected_digest =
        HashValue::from_hex("72640fc4b2379de554a45871056f65b7c7ffe282841ae5f1774dc88bc91c3062")
            .unwrap();

    c.bench_function("hashing", |b| {
        b.iter(|| {
            let digest = CryptoHash::hash(criterion::black_box(&HashBencher(INPUT)));
            assert_eq!(digest, expected_digest);
        })
    });

    c.bench_function("hashing_base", |b| {
        b.iter(|| {
            let digest = CryptoHash::hash(criterion::black_box(&HashBencherBase(INPUT)));
            assert_eq!(digest, expected_digest);
        })
    });
}

criterion_group!(benches, bench_hasher);
criterion_main!(benches);

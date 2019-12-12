// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[macro_use]
extern crate criterion;

use libra_crypto::hash::{CryptoHash, CryptoHasher, HashValue, SparseMerkleInternalHasher};

struct HashBencher(&'static [u8]);

impl HashBencher {
    fn bench_hash(input: &'static [u8]) -> HashValue {
        let bench = Self(input);
        bench.hash()
    }
}

impl CryptoHash for HashBencher {
    type Hasher = SparseMerkleInternalHasher;
    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(self.0);
        state.finish()
    }
}

fn bench_hasher(c: &mut criterion::Criterion) {
    let expected_digest =
        HashValue::from_hex("a2a3adfaed90739641d57dc3d0f8e6231b0d038748dad0feaec765c9aa9676a6")
            .unwrap();

    c.bench_function("hashing", |b| {
        b.iter(|| {
            let digest = HashBencher::bench_hash(criterion::black_box(b"someinput"));
            assert_eq!(digest, expected_digest);
        })
    });
}

criterion_group!(benches, bench_hasher);
criterion_main!(benches);

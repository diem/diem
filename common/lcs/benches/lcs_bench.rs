// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use criterion::{criterion_group, criterion_main, Criterion};
use libra_canonical_serialization::to_bytes;
use std::collections::{BTreeMap, HashMap};

pub fn lcs_benchmark(c: &mut Criterion) {
    let mut btree_map = BTreeMap::new();
    let mut hash_map = HashMap::new();
    for i in 0u32..2000u32 {
        btree_map.insert(i, i);
        hash_map.insert(i, i);
    }
    c.bench_function("serialize btree map", |b| {
        b.iter(|| {
            to_bytes(&btree_map).unwrap();
        })
    });
    c.bench_function("serialize hash map", |b| {
        b.iter(|| {
            to_bytes(&hash_map).unwrap();
        })
    });
}

criterion_group!(benches, lcs_benchmark);
criterion_main!(benches);

// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use criterion::{criterion_group, criterion_main, Criterion};
use language_benchmarks::transactions::TransactionBencher;
use language_e2e_tests::account_universe::P2PTransferGen;
use proptest::prelude::*;

//
// Transaction benchmarks
//

fn peer_to_peer(c: &mut Criterion) {
    c.bench_function("peer_to_peer", |b| {
        let bencher = TransactionBencher::new(any_with::<P2PTransferGen>((1_000, 1_000_000)));
        bencher.bench(b)
    });
}

criterion_group!(txn_benches, peer_to_peer);

criterion_main!(txn_benches);

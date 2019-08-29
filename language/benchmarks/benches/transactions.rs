// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use criterion::{criterion_group, criterion_main, Criterion};
use language_benchmarks::{transactions::TransactionBencher, vm_bencher::VMBencher};
use language_e2e_tests::account_universe::P2PTransferGen;
use proptest::prelude::*;

#[allow(dead_code)]
fn peer_to_peer(c: &mut Criterion) {
    c.bench_function("peer_to_peer", |b| {
        let bencher = TransactionBencher::new(any_with::<P2PTransferGen>((1_000, 1_000_000)));
        bencher.bench(b)
    });
}

fn vm_bench(c: &mut Criterion) {
    println!("start benchmark");
    c.bench_function("vm_bench", |b| {
        let mut bencher = VMBencher::new();
        bencher.run_script();
        bencher.bench(b)
    });
    println!("end benchmark");
}

criterion_group!(benches, vm_bench);
criterion_main!(benches);

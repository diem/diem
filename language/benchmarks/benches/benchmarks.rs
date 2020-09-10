// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use criterion::{criterion_group, criterion_main, Criterion};
use language_benchmarks::{move_vm::bench, transactions::TransactionBencher};
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

//
// MoveVM benchmarks
//

fn arith(c: &mut Criterion) {
    bench(c, "arith");
}

fn call(c: &mut Criterion) {
    bench(c, "call");
}

criterion_group!(vm_benches, arith, call);

criterion_main!(vm_benches);

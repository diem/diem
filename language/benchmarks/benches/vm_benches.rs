// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use criterion::{criterion_group, criterion_main, Criterion};
use language_benchmarks::move_vm::bench;

//
// MoveVM benchmarks
//

fn arith(c: &mut Criterion) {
    bench(c, "arith");
}

fn call(c: &mut Criterion) {
    bench(c, "call");
}

fn natives(c: &mut Criterion) {
    bench(c, "natives");
}

criterion_group!(vm_benches, arith, call, natives);

criterion_main!(vm_benches);

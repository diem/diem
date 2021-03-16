// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use criterion::{criterion_group, criterion_main, measurement::Measurement, Criterion};
use language_benchmarks::{measurement::cpu_time_measurement, move_vm::bench};

//
// MoveVM benchmarks
//

fn arith<M: Measurement + 'static>(c: &mut Criterion<M>) {
    bench(c, "arith");
}

fn call<M: Measurement + 'static>(c: &mut Criterion<M>) {
    bench(c, "call");
}

fn natives<M: Measurement + 'static>(c: &mut Criterion<M>) {
    bench(c, "natives");
}

criterion_group!(
    name = vm_benches;
    config = cpu_time_measurement();
    targets = arith,
    call,
    natives
);

criterion_main!(vm_benches);

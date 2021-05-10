// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput};
use diem_crypto::{hash::SPARSE_MERKLE_PLACEHOLDER_HASH, HashValue};
use diem_types::account_state_blob::AccountStateBlob;
use executor_types::ProofReader;
use rand::{prelude::StdRng, Rng, SeedableRng};
use scratchpad::SparseMerkleTree;
use std::collections::HashMap;

fn rng() -> StdRng {
    let seed: &[_] = &[1, 2, 3, 4];
    let mut actual_seed = [0u8; 32];
    actual_seed[..seed.len()].copy_from_slice(&seed);

    StdRng::from_seed(actual_seed)
}

fn gen_value(rng: &mut StdRng) -> AccountStateBlob {
    rng.gen::<[u8; 32]>().to_vec().into()
}

fn insert_to_empty(c: &mut Criterion) {
    let mut rng = rng();

    let mut group = c.benchmark_group("insert to empty smt");
    for size in &[100usize, 1000, 10000] {
        let values = std::iter::repeat_with(|| (gen_value(&mut rng), gen_value(&mut rng)))
            .take(*size)
            .collect::<Vec<_>>();
        let small_batches = values
            .iter()
            .map(|(v1, v2)| {
                vec![
                    (HashValue::random_with_rng(&mut rng), v1),
                    (HashValue::random_with_rng(&mut rng), v2),
                ]
            })
            .collect::<Vec<_>>();
        let one_large_batch = small_batches.iter().cloned().flatten().collect::<Vec<_>>();
        let empty_smt = SparseMerkleTree::new(*SPARSE_MERKLE_PLACEHOLDER_HASH);
        let proof_reader = ProofReader::new(HashMap::new());

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_function(BenchmarkId::new("serial_update", size), |b| {
            b.iter_batched(
                || small_batches.clone(),
                |small_batches| {
                    empty_smt
                        .serial_update(small_batches, &proof_reader)
                        .unwrap();
                },
                BatchSize::LargeInput,
            )
        });
        group.bench_function(BenchmarkId::new("batches_update", size), |b| {
            b.iter_batched(
                || small_batches.clone(),
                |small_batches| {
                    empty_smt
                        .batches_update(small_batches, &proof_reader)
                        .unwrap();
                },
                BatchSize::LargeInput,
            )
        });
        group.bench_function(BenchmarkId::new("batch_update", size), |b| {
            b.iter_batched(
                || one_large_batch.clone(),
                |one_large_batch| {
                    empty_smt
                        .batch_update(one_large_batch, &proof_reader)
                        .unwrap();
                },
                BatchSize::LargeInput,
            )
        });
    }
    group.finish();
}

criterion_group!(benches, insert_to_empty);
criterion_main!(benches);

// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput};
use diem_crypto::{hash::SPARSE_MERKLE_PLACEHOLDER_HASH, HashValue};
use diem_types::account_state_blob::AccountStateBlob;
use itertools::zip_eq;
use rand::{distributions::Standard, prelude::StdRng, seq::IteratorRandom, Rng, SeedableRng};
use scratchpad::{test_utils::naive_smt::NaiveSmt, SparseMerkleTree};
use std::collections::HashSet;

type ProofReader = scratchpad::test_utils::proof_reader::ProofReader<AccountStateBlob>;

struct Block {
    smt: SparseMerkleTree<AccountStateBlob>,
    updates: Vec<Vec<(HashValue, AccountStateBlob)>>,
    proof_reader: ProofReader,
}

impl Block {
    fn updates(&self) -> Vec<Vec<(HashValue, &AccountStateBlob)>> {
        self.updates
            .iter()
            .map(|small_batch| small_batch.iter().map(|(k, v)| (*k, v)).collect())
            .collect()
    }

    fn updates_flat_batch(&self) -> Vec<(HashValue, &AccountStateBlob)> {
        self.updates().iter().flatten().cloned().collect()
    }
}

struct Group {
    name: String,
    blocks: Vec<Block>,
}

impl Group {
    fn run(&self, c: &mut Criterion) {
        let mut group = c.benchmark_group(&self.name);

        for block in &self.blocks {
            let block_size = block.updates.len();
            let small_batches = block.updates();
            let one_large_batch = block.updates_flat_batch();

            group.throughput(Throughput::Elements(block_size as u64));

            group.bench_function(BenchmarkId::new("serial_update", block_size), |b| {
                b.iter_batched(
                    || small_batches.clone(),
                    |small_batches| {
                        block
                            .smt
                            .serial_update(small_batches, &block.proof_reader)
                            .unwrap();
                    },
                    BatchSize::LargeInput,
                )
            });
            group.bench_function(BenchmarkId::new("batches_update", block_size), |b| {
                b.iter_batched(
                    || small_batches.clone(),
                    |small_batches| {
                        block
                            .smt
                            .batches_update(small_batches, &block.proof_reader)
                            .unwrap();
                    },
                    BatchSize::LargeInput,
                )
            });
            group.bench_function(
                BenchmarkId::new("batches_update__flat_batch", block_size),
                |b| {
                    b.iter_batched(
                        || one_large_batch.clone(),
                        |one_large_batch| {
                            block
                                .smt
                                .batches_update(vec![one_large_batch], &block.proof_reader)
                                .unwrap();
                        },
                        BatchSize::LargeInput,
                    )
                },
            );
            group.bench_function(BenchmarkId::new("batch_update", block_size), |b| {
                b.iter_batched(
                    || one_large_batch.clone(),
                    |one_large_batch| {
                        block
                            .smt
                            .batch_update(one_large_batch, &block.proof_reader)
                            .unwrap();
                    },
                    BatchSize::LargeInput,
                )
            });
        }
        group.finish();
    }
}

struct Benches {
    base_empty: Group,
    base_committed: Group,
    base_uncommitted: Group,
}

impl Benches {
    fn gen(block_sizes: &[usize]) -> Self {
        let mut rng = Self::rng();

        // 1 million possible keys
        let keys = std::iter::repeat_with(|| HashValue::random_with_rng(&mut rng))
            .take(1_000_000)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        // group: insert to an empty SMT
        let base_empty = Group {
            name: "insert to empty".into(),
            blocks: block_sizes
                .iter()
                .map(|block_size| Block {
                    smt: SparseMerkleTree::new(*SPARSE_MERKLE_PLACEHOLDER_HASH),
                    updates: Self::gen_updates(&mut rng, &keys, *block_size),
                    proof_reader: ProofReader::new(Vec::new()),
                })
                .collect(),
        };

        // all addresses with an existing value
        let values = std::iter::repeat_with(|| Self::gen_value(&mut rng))
            .take(keys.len())
            .collect::<Vec<_>>();
        let existing_state = zip_eq(&keys, &values)
            .map(|(key, value)| (*key, value))
            .collect::<Vec<_>>();
        let mut naive_base_smt = NaiveSmt::new(existing_state.as_slice());

        // group: insert to a committed SMT ("unknown" root)
        let base_committed = Group {
            name: "insert to committed base".into(),
            blocks: block_sizes
                .iter()
                .map(|block_size| {
                    let updates = Self::gen_updates(&mut rng, &keys, *block_size);
                    let proof_reader = Self::gen_proof_reader(&mut naive_base_smt, &updates);
                    Block {
                        smt: SparseMerkleTree::new(naive_base_smt.get_root_hash()),
                        updates,
                        proof_reader,
                    }
                })
                .collect(),
        };

        // group: insert to an uncommitted SMT (some structures in mem)
        let base_uncommitted = Group {
            name: "insert to uncommitted base".into(),
            blocks: base_committed
                .blocks
                .iter()
                .map(|base_block| {
                    // This is an SMT holding updates from the `base_committed` block in mem.
                    let updates = Self::gen_updates(&mut rng, &keys, base_block.updates.len());
                    let proof_reader = Self::gen_proof_reader(&mut naive_base_smt, &updates);

                    Block {
                        smt: base_block
                            .smt
                            .batch_update(base_block.updates_flat_batch(), &base_block.proof_reader)
                            .unwrap(),
                        updates,
                        proof_reader,
                    }
                })
                .collect(),
        };

        Self {
            base_empty,
            base_committed,
            base_uncommitted,
        }
    }

    fn run(&self, c: &mut Criterion) {
        self.base_empty.run(c);
        self.base_committed.run(c);
        self.base_uncommitted.run(c);
    }

    fn gen_updates(
        rng: &mut StdRng,
        keys: &[HashValue],
        block_size: usize,
    ) -> Vec<Vec<(HashValue, AccountStateBlob)>> {
        std::iter::repeat_with(|| vec![Self::gen_update(rng, keys), Self::gen_update(rng, keys)])
            .take(block_size)
            .collect()
    }

    fn gen_update(rng: &mut StdRng, keys: &[HashValue]) -> (HashValue, AccountStateBlob) {
        (*keys.iter().choose(rng).unwrap(), Self::gen_value(rng))
    }

    fn gen_value(rng: &mut StdRng) -> AccountStateBlob {
        rng.sample_iter(&Standard)
            .take(100)
            .collect::<Vec<u8>>()
            .into()
    }

    fn gen_proof_reader(
        naive_smt: &mut NaiveSmt,
        updates: &[Vec<(HashValue, AccountStateBlob)>],
    ) -> ProofReader {
        let proofs = updates
            .iter()
            .flatten()
            .map(|(key, _)| (*key, naive_smt.get_proof(key)))
            .collect();
        ProofReader::new(proofs)
    }

    fn rng() -> StdRng {
        let seed: &[_] = &[1, 2, 3, 4];
        let mut actual_seed = [0u8; 32];
        actual_seed[..seed.len()].copy_from_slice(&seed);

        StdRng::from_seed(actual_seed)
    }
}

fn sparse_merkle_benches(c: &mut Criterion) {
    // Fix Rayon threadpool size to 8, which is realistic as in the current production setting
    // and benchmarking result will be more stable across different machines.
    rayon::ThreadPoolBuilder::new()
        .num_threads(8)
        .thread_name(|index| format!("rayon-global-{}", index))
        .build_global()
        .expect("Failed to build rayon global thread pool.");

    Benches::gen(&[2, 4, 8, 16, 32, 100, 1000, 10000]).run(c);
}

criterion_group!(benches, sparse_merkle_benches);
criterion_main!(benches);

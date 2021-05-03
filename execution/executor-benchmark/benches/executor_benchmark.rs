// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use criterion::{criterion_group, criterion_main, measurement::Measurement, BatchSize, Criterion};
use executor_benchmark::{
    create_storage_service_and_executor, TransactionExecutor, TransactionGenerator,
};

pub const NUM_ACCOUNTS: usize = 1000;
pub const SMALL_BLOCK_SIZE: usize = 500;
pub const MEDIUM_BLOCK_SIZE: usize = 1000;
pub const LARGE_BLOCK_SIZE: usize = 1000;
pub const INITIAL_BALANCE: u64 = 1000000;

//
// Transaction benchmarks
//

fn executor_benchmark<M: Measurement + 'static>(c: &mut Criterion<M>) {
    let (config, genesis_key) = diem_genesis_tool::test_config();

    let (_db, executor) = create_storage_service_and_executor(&config);
    let parent_block_id = executor.committed_block_id();

    let mut generator = TransactionGenerator::new(genesis_key, NUM_ACCOUNTS);

    let mut executor = TransactionExecutor::new(executor, parent_block_id);
    let txns = generator.gen_account_creations(SMALL_BLOCK_SIZE);
    for txn_block in txns {
        executor.execute_block(txn_block);
    }
    let txns = generator.gen_mint_transactions(INITIAL_BALANCE, SMALL_BLOCK_SIZE);
    for txn_block in txns {
        executor.execute_block(txn_block);
    }

    c.bench_function("bench_p2p_small", |bencher| {
        bencher.iter_batched(
            || generator.gen_transfer_transactions(SMALL_BLOCK_SIZE, 1),
            |mut txn_block| executor.execute_block(txn_block.pop().unwrap()),
            BatchSize::LargeInput,
        )
    });

    c.bench_function("bench_p2p_medium", |bencher| {
        bencher.iter_batched(
            || generator.gen_transfer_transactions(MEDIUM_BLOCK_SIZE, 1),
            |mut txn_block| executor.execute_block(txn_block.pop().unwrap()),
            BatchSize::LargeInput,
        )
    });

    c.bench_function("bench_p2p_large", |bencher| {
        bencher.iter_batched(
            || generator.gen_transfer_transactions(LARGE_BLOCK_SIZE, 1),
            |mut txn_block| executor.execute_block(txn_block.pop().unwrap()),
            BatchSize::LargeInput,
        )
    });
}

criterion_group!(
    name = txn_benches;
    config = Criterion::default().sample_size(10);
    targets = executor_benchmark
);

criterion_main!(txn_benches);

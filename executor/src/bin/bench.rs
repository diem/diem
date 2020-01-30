// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

const NUM_ACCOUNTS: usize = 1_000_000;
const INIT_ACCOUNT_BALANCE: u64 = 1_000_000;
const BLOCK_SIZE: usize = 1000;
const NUM_TRANSFER_BLOCKS: usize = 1000;

fn main() {
    executor::benchmark::run_benchmark(
        NUM_ACCOUNTS,
        INIT_ACCOUNT_BALANCE,
        BLOCK_SIZE,
        NUM_TRANSFER_BLOCKS,
    );
}

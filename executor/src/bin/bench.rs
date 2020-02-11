// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(long, default_value = "1000000")]
    num_accounts: usize,

    #[structopt(long, default_value = "1000000")]
    init_account_balance: u64,

    #[structopt(long, default_value = "500")]
    block_size: usize,

    #[structopt(long, default_value = "1000")]
    num_transfer_blocks: usize,

    #[structopt(long, parse(from_os_str))]
    db_dir: Option<PathBuf>,
}

fn main() {
    let opt = Opt::from_args();

    let _logger = libra_logger::set_default_global_logger(
        true, /* async_drain */
        None, /* chan_size */
    );

    rayon::ThreadPoolBuilder::new()
        .thread_name(|index| format!("rayon-global-{}", index))
        .build_global()
        .expect("Failed to build rayon global thread pool.");

    executor::benchmark::run_benchmark(
        opt.num_accounts,
        opt.init_account_balance,
        opt.block_size,
        opt.num_transfer_blocks,
        opt.db_dir,
    );
}

// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(long, default_value = "2000")]
    num_updates: usize,

    #[structopt(long)]
    num_accounts: u64,

    #[structopt(short, default_value = "40")]
    blob_size: usize,

    #[structopt(long, parse(from_os_str))]
    db_dir: PathBuf,
}

fn main() {
    rayon::ThreadPoolBuilder::new()
        .thread_name(|index| format!("rayon-global-{}", index))
        .build_global()
        .expect("Failed to build rayon global thread pool.");

    let opt = Opt::from_args();
    scratchpad_benchmark::run_benchmark(
        opt.num_updates,
        opt.num_accounts,
        opt.blob_size,
        opt.db_dir,
    );
}

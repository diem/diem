// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short, default_value = "1000000")]
    num_accounts: usize,

    #[structopt(short, default_value = "1000000")]
    version: u64,

    #[structopt(short, default_value = "40")]
    blob_size: usize,

    #[structopt(long, parse(from_os_str))]
    db_dir: PathBuf,

    #[structopt(long)]
    prune_window: Option<u64>,
}

fn main() {
    let opt = Opt::from_args();

    diemdb_benchmark::run_benchmark(
        opt.num_accounts,
        opt.version,
        opt.blob_size,
        opt.db_dir,
        opt.prune_window,
    );
}

// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use backup_cli::{restore::restore_account_state, storage::local_fs::LocalFs};
use std::path::PathBuf;
use structopt::StructOpt;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    stream::StreamExt,
};

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(long, parse(from_os_str))]
    db_dir: PathBuf,
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();

    let mut lines = BufReader::new(tokio::io::stdin()).lines();

    println!("Input version:");
    let version = lines
        .next()
        .await
        .expect("Must provide version.")
        .expect("Failed to read from stdin.")
        .parse::<u64>()
        .expect("Version must be valid u64.");
    println!("Version: {}", version);

    println!("Input manifest:");
    let manifest = lines
        .next()
        .await
        .expect("Must provide manifest.")
        .expect("Failed to read from stdin.");
    println!("Manifest: {}", &manifest);

    // LocalFs uses absolute paths, so when reading we can construct it in any dir.
    let storage = LocalFs::new(PathBuf::from("."));
    restore_account_state(&storage, &manifest, version, &opt.db_dir)
        .await
        .expect("Failed restoring state.");

    println!("Finished restoring account state.");
}

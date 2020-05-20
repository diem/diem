// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use backup_cli::restore::restore_account_state;
use itertools::Itertools;
use libra_crypto::HashValue;
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

    println!("Input state root hash:");
    let root_hash_hex = lines
        .next()
        .await
        .expect("Must provide state root hash.")
        .expect("Failed to read from stdin.");
    let root_hash = HashValue::from_slice(
        &hex::decode(&root_hash_hex).expect("State root hash must be valid hex."),
    )
    .expect("Invalid root hash.");
    println!("State root hash: {:x}", root_hash);

    let file_handle_pair_iter = lines
        .collect::<Result<Vec<_>, _>>()
        .await
        .expect("Failed reading file handles.")
        .into_iter()
        .tuples::<(_, _)>();

    restore_account_state(version, root_hash, &opt.db_dir, file_handle_pair_iter)
        .await
        .expect("Failed restoring state.");

    println!("Finished restoring account state.");
}

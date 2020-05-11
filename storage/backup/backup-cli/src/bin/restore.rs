// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use backup_cli::restore_account_state;
use itertools::Itertools;
use libra_crypto::HashValue;
use std::{io::BufRead, path::PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(long, parse(from_os_str))]
    db_dir: PathBuf,
}

fn main() {
    let opt = Opt::from_args();

    let stdin = std::io::stdin();
    let mut iter = stdin.lock().lines();

    println!("Input version:");
    let version = iter
        .next()
        .expect("Must provide version.")
        .expect("Failed to read from stdin.")
        .parse::<u64>()
        .expect("Version must be valid u64.");
    println!("Version: {}", version);

    println!("Input state root hash:");
    let root_hash_hex = iter
        .next()
        .expect("Must provide state root hash.")
        .expect("Failed to read from stdin.");
    let root_hash = HashValue::from_slice(
        &hex::decode(&root_hash_hex).expect("State root hash must be valid hex."),
    )
    .expect("Invalid root hash.");
    println!("State root hash: {:x}", root_hash);

    let iter = iter.tuples().map(|(a, b)| Ok((a?, b?)));
    restore_account_state(version, root_hash, &opt.db_dir, iter);

    println!("Finished restoring account state.");
}

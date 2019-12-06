// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use generate_keypair::create_faucet_key_file;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Tool to generate public/private keypairs")]
struct Args {
    #[structopt(short = "o", long)]
    /// Output file path. Keypair is written to this file
    output: String,
}

fn main() {
    let args = Args::from_args();
    create_faucet_key_file(&args.output);
}

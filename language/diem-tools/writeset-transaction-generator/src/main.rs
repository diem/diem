// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Result};
use diem_types::{account_address::AccountAddress, transaction::Transaction};
use diem_writeset_generator::{
    encode_custom_script, encode_halt_network_transaction, encode_remove_validators_transaction,
};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// Path to the local DiemDB file
    #[structopt(long, short, parse(from_os_str))]
    output: PathBuf,
    #[structopt(subcommand)] // Note that we mark a field as a subcommand
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// List of addresses to remove from validator set
    #[structopt(name = "remove-validators")]
    RemoveValidators { addresses: Vec<AccountAddress> },
    /// Block the execution of any transaction in the network
    #[structopt(name = "halt-network")]
    HaltNetwork,
    /// Build a custom file in templates into admin script
    #[structopt(name = "build-custom-script")]
    BuildCustomScript { script_name: String, args: String },
}

fn save_transaction(txn: Transaction, path: PathBuf) -> Result<()> {
    let bytes = bcs::to_bytes(&txn).map_err(|_| format_err!("Transaction Serialize Error"))?;
    std::fs::write(path.as_path(), bytes.as_slice())
        .map_err(|_| format_err!("Unable to write to path"))
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let transaction = match opt.cmd {
        Command::RemoveValidators { addresses } => encode_remove_validators_transaction(addresses),
        Command::HaltNetwork => encode_halt_network_transaction(),
        Command::BuildCustomScript { script_name, args } => {
            encode_custom_script(&script_name, &serde_json::from_str(args.as_str())?)
        }
    };
    save_transaction(transaction, opt.output)
}

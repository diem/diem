// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Result};
use diem_types::{account_address::AccountAddress, transaction::Transaction};
use diem_writeset_generator::{
    encode_custom_script, encode_halt_network_payload, encode_remove_validators_payload,
};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// Path to the output serialized bytes
    #[structopt(long, short, parse(from_os_str))]
    output: PathBuf,
    /// Output as serialized WriteSet payload. Set this flag if this payload is submitted to AOS portal.
    #[structopt(long)]
    output_payload: bool,
    #[structopt(subcommand)]
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
    BuildCustomScript {
        script_name: String,
        args: String,
        execute_as: Option<AccountAddress>,
    },
}

fn save_bytes(bytes: Vec<u8>, path: PathBuf) -> Result<()> {
    std::fs::write(path.as_path(), bytes.as_slice())
        .map_err(|_| format_err!("Unable to write to path"))
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let payload = match opt.cmd {
        Command::RemoveValidators { addresses } => encode_remove_validators_payload(addresses),
        Command::HaltNetwork => encode_halt_network_payload(),
        Command::BuildCustomScript {
            script_name,
            args,
            execute_as,
        } => encode_custom_script(
            &script_name,
            &serde_json::from_str::<serde_json::Value>(args.as_str())?,
            execute_as,
        ),
    };
    if opt.output_payload {
        save_bytes(
            bcs::to_bytes(&payload).map_err(|_| format_err!("Transaction Serialize Error"))?,
            opt.output,
        )
    } else {
        save_bytes(
            bcs::to_bytes(&Transaction::GenesisTransaction(payload))
                .map_err(|_| format_err!("Transaction Serialize Error"))?,
            opt.output,
        )
    }
}

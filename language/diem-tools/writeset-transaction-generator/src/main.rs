// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Result};
use diem_types::{account_address::AccountAddress, transaction::Transaction};
use diem_validator_interface::{DiemValidatorInterface, JsonRpcDebuggerInterface};
use diem_writeset_generator::{
    create_release_writeset, encode_custom_script, encode_halt_network_payload,
    encode_remove_validators_payload, verify_payload_change,
};
use std::path::PathBuf;
use stdlib::build_stdlib;
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
    /// Create a release writeset by comparing local Diem Framework against a remote blockchain state.
    #[structopt(name = "create-release")]
    CreateDiemFrameworkRelease {
        /// Public JSON-rpc endpoint URL.
        url: String,
        /// Blockchain height
        version: u64,
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
        Command::CreateDiemFrameworkRelease { url, version } => {
            let remote = JsonRpcDebuggerInterface::new(url.as_str())?;
            let remote_modules = remote.get_diem_framework_modules_by_version(version)?;
            let local_modules = build_stdlib().into_iter().map(|(_, m)| m).collect();
            let payload = create_release_writeset(remote_modules, local_modules)?;
            verify_payload_change(&remote, Some(version), &payload)?;
            payload
        }
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

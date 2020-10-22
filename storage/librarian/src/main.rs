// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use anyhow::Result;
use libra_types::account_address::AccountAddress;
use libradb::librarian::Librarian;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(
    name = "Librarian",
    about = "A command line tool that offers multiple data access commands for LibraDB"
)]
struct Opt {
    #[structopt(long = "db", parse(from_os_str))]
    db_dir: PathBuf,

    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt)]
enum Command {
    /// Get the latest ledger version
    Status,
    /// Transaction related query
    Txn(TxnOp),
    /// Account related query
    Account(AccountCmd),
}

#[derive(StructOpt)]
enum TxnOp {
    Get {
        /// The version of the target transaction
        #[structopt(long, short)]
        version: u64,
    },
    Scan {
        /// The lower bound of the version range in transaction scan
        #[structopt(default_value = "0", long)]
        from: u64,
        /// The upper bound (inclusive) of the version range in transaction scan, by default the current ledger version
        #[structopt(long)]
        to: Option<u64>,
    },
}

#[derive(StructOpt)]
struct AccountCmd {
    /// The Account Address in hex
    #[structopt(short, long, short)]
    address: AccountAddress,
    #[structopt(subcommand)]
    account_op: AccountOp,
}

#[derive(StructOpt)]
enum AccountOp {
    Get {
        /// The version at which the account data is requested
        #[structopt(long, short)]
        version: u64,
    },
}

fn main() {
    if let Err(err) = run_cmd() {
        println!("Error: {}", err);
    }
}

fn run_cmd() -> Result<()> {
    let opt = Opt::from_args();
    let librarian = Librarian::new(opt.db_dir)?;
    match opt.cmd {
        Command::Status => {
            println!("{:?}", librarian.get_startup_info()?);
        }
        Command::Txn(txn_op) => match txn_op {
            TxnOp::Get { version } => {
                println!("{:?}", librarian.get_txn_by_version(version)?);
            }
            TxnOp::Scan { from, to } => {
                let to = match to {
                    Some(to) => to,
                    None => librarian.get_committed_version()?,
                };
                println!("{:?}", librarian.scan_txn_by_version(from, to)?);
            }
        },
        Command::Account(account_cmd) => {
            let address = account_cmd.address;
            match account_cmd.account_op {
                AccountOp::Get { version } => {
                    println!(
                        "{:?}",
                        librarian.get_account_state_by_version(address, version)?
                    );
                }
            }
        }
    }
    Ok(())
}

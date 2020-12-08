// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use anyhow::Result;

use diem_types::{account_address::AccountAddress, event::EventKey};
use diemdb::diemsum::Diemsum;
use serde::Serialize;
use serde_json::to_string_pretty;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(
    name = "Diemsum",
    about = "A command line tool that offers multiple data access commands for DiemDB"
)]
struct Opt {
    /// The parent dir of diemdb
    #[structopt(long = "db", parse(from_os_str))]
    db_dir: PathBuf,

    /// Whether output in json format
    #[structopt(long)]
    json: bool,

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
    /// Event related query
    #[structopt(flatten)]
    Event(EventCmd),
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
        #[structopt(default_value = "0", long = "from")]
        from_version: u64,
        /// The upper bound (inclusive) of the version range in transaction scan, by default the current ledger version
        #[structopt(long = "to")]
        to_version: Option<u64>,
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

#[derive(StructOpt)]
enum EventCmd {
    /// Look for a range of event stream of an EventKey
    EventByKey {
        /// The event key
        #[structopt(short, long = "key", short = "k")]
        event_key: EventKey,
        #[structopt(subcommand)]
        event_op: EventOp,
    },
    /// Look for events emitted by the txn at the specified version
    EventByVersion {
        /// The target version for the interested events
        #[structopt(long, short)]
        version: u64,
    },
}

#[derive(StructOpt)]
enum EventOp {
    Get {
        /// The sequence at which the event is requested
        #[structopt(long, short)]
        seq: u64,
    },
    Scan {
        /// The lower bound of the sequence range in event scan
        #[structopt(default_value = "0", long = "from")]
        from_seq: u64,
        /// The upper bound (inclusive) of the sequence range in event scan, by default the latest sequence
        #[structopt(long = "to")]
        to_seq: Option<u64>,
    },
}

fn main() {
    if let Err(err) = run_cmd() {
        println!("Error: {}", err);
    }
}

fn run_cmd() -> Result<()> {
    let opt = Opt::from_args();
    let diemsum = Diemsum::new(opt.db_dir)?;
    let is_json = opt.json;
    match opt.cmd {
        Command::Status => {
            print(diemsum.get_startup_info()?, is_json)?;
        }
        Command::Txn(txn_op) => match txn_op {
            TxnOp::Get { version } => {
                print(&diemsum.get_txn_by_version(version)?, is_json)?;
            }
            TxnOp::Scan {
                from_version,
                to_version,
            } => {
                let to_version = match to_version {
                    Some(to_version) => to_version,
                    None => diemsum.get_committed_version()?,
                };
                print(
                    diemsum.scan_txn_by_version(from_version, to_version)?,
                    is_json,
                )?;
            }
        },
        Command::Account(account_cmd) => {
            let address = account_cmd.address;
            match account_cmd.account_op {
                AccountOp::Get { version } => {
                    print(
                        diemsum.get_account_state_by_version(address, version)?,
                        is_json,
                    )?;
                }
            }
        }
        Command::Event(event_cmd) => match event_cmd {
            EventCmd::EventByKey {
                event_key,
                event_op,
            } => match event_op {
                EventOp::Get { seq } => {
                    print(diemsum.scan_events_by_seq(&event_key, seq, seq)?, is_json)?;
                }
                EventOp::Scan { from_seq, to_seq } => {
                    let to_seq = match to_seq {
                        Some(to_version) => to_version,
                        None => u64::max_value(),
                    };
                    print(
                        diemsum.scan_events_by_seq(&event_key, from_seq, to_seq)?,
                        is_json,
                    )?;
                }
            },
            EventCmd::EventByVersion { version } => {
                print(diemsum.get_events_by_version(version)?, is_json)?;
            }
        },
    }
    Ok(())
}

fn print<T>(data: T, is_json: bool) -> Result<()>
where
    T: Serialize + std::fmt::Debug,
{
    if is_json {
        println!("{}", to_string_pretty(&data)?);
    } else {
        println!("{:?}", data)
    }
    Ok(())
}

// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use anyhow::Result;
use libra_logger::info;
use libradb::LibraDB;
use std::path::PathBuf;
use storage_interface::DbReader;
use transaction_builder::get_transaction_name;

use libra_types::{
    account_address::AccountAddress, account_config::AccountResource, account_state::AccountState,
};
use std::convert::TryFrom;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(long, parse(from_os_str))]
    db: PathBuf,

    #[structopt(subcommand)] // Note that we mark a field as a subcommand
    cmd: Option<Command>,
}

#[derive(Debug, StructOpt)]
enum Command {
    #[structopt(name = "list-txns")]
    ListTXNs,
    #[structopt(name = "print-txn")]
    PrintTXN { version: u64 },
    #[structopt(name = "print-account")]
    PrintAccount {
        #[structopt(parse(try_from_str))]
        address: AccountAddress,
    },
    #[structopt(name = "list-accounts")]
    ListAccounts,
}

/// Print out latest information stored in the DB.
fn print_head(db: &LibraDB) -> Result<()> {
    let si = db
        .get_startup_info()
        .expect("Can't get startup info")
        .expect("StartupInfo is empty, database is empty.");

    let version = si.latest_ledger_info.ledger_info().version();
    info!("Version: {}", version);

    info!(
        "The latest ledger info: {}",
        si.latest_ledger_info.ledger_info()
    );

    info!("Signatures: {:?}", si.latest_ledger_info.signatures());

    info!("Current EpochState: {}", si.get_epoch_state());

    let backup = db.get_backup_handler();
    let iter = backup.get_account_iter(version)?;
    let num_account_state = iter.count();
    info!("Total Accounts: {}", num_account_state);

    print_txn(db, version);

    Ok(())
}

fn print_txn(db: &LibraDB, version: u64) {
    let tx = db
        .get_transaction_with_proof(version, version, false)
        .expect("Unable to load latest TXN");
    println!(
        "Transaction {}: {}",
        version,
        tx.transaction.format_for_client(get_transaction_name)
    );
}

fn print_account(db: &LibraDB, addr: AccountAddress) {
    let maybe_blob = db
        .get_latest_account_state(addr)
        .expect("Unable to read AccountState");
    if let Some(blob) = maybe_blob {
        match AccountResource::try_from(&blob) {
            Ok(r) => {
                println!("Account {}: {:?}", addr, r);
            }
            Err(e) => {
                info!(
                    "Account {} exists, but have no AccountResource: {}.",
                    addr, e
                );
            }
        }
    } else {
        info!("Account {} doesn't exists", addr);
    }
}

fn list_txns(db: &LibraDB) {
    let version = db
        .get_latest_version()
        .expect("Unable to get latest version");
    let backup = db.get_backup_handler();
    let iter = backup
        .get_transaction_iter(0, version as usize)
        .expect("Unable to get txn iter");
    for (v, tx) in iter.enumerate() {
        println!(
            "TXN {}: {}",
            v,
            tx.expect("Unable to read TX")
                .0
                .format_for_client(get_transaction_name)
        );
    }
}

fn list_accounts(db: &LibraDB) {
    let version = db
        .get_latest_version()
        .expect("Unable to get latest version");
    let backup = db.get_backup_handler();
    let iter = backup
        .get_account_iter(version)
        .expect("Unagle to get account iter");
    let mut num_account = 0;
    for res in iter {
        match res {
            Ok((_, blob)) => {
                let accs = AccountState::try_from(&blob).expect("Failed to read AccountState");
                let addr = accs
                    .get_account_address()
                    .expect("Could not get address from state");
                match addr {
                    Some(x) => {
                        num_account += 1;
                        println!("Address: {:?}", x);
                    }
                    None => println!("Skipping: No address for AccountState: {:?}", accs),
                }
            }
            Err(x) => println!("Got err iterating through AccountStateBlobs {:?}", x),
        }
    }
    info!("Total Accounts: {}", num_account);
}

fn main() {
    ::libra_logger::Logger::new().init();

    let opt = Opt::from_args();

    let p = opt.db.as_path();

    if !p.is_dir() {
        info!("Invalid Directory {:?}!", p);
        std::process::exit(-1);
    }

    let log_dir = tempfile::tempdir().expect("Unable to get temp dir");
    info!("Opening DB at: {:?}, log at {:?}", p, log_dir.path());

    let db =
        LibraDB::open(p, true /* readonly */, None /* pruner */).expect("Unable to open LibraDB");
    info!("DB opened successfully.");

    if let Some(cmd) = opt.cmd {
        match cmd {
            Command::ListTXNs => {
                list_txns(&db);
            }
            Command::PrintTXN { version } => {
                print_txn(&db, version);
            }
            Command::PrintAccount { address } => {
                print_account(&db, address);
            }
            Command::ListAccounts => {
                list_accounts(&db);
            }
        }
    } else {
        print_head(&db).expect("Unable to read information from DB");

        Opt::clap().print_help().unwrap();
        println!();
    }
}

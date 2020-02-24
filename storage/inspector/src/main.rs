// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use anyhow::{ensure, Result};
use libra_logger::prelude::*;
use libradb::LibraDB;
use std::path::PathBuf;
use transaction_builder::get_transaction_name;

use libra_crypto::hash::CryptoHash;
use libra_crypto::HashValue;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(long, parse(from_os_str))]
    db: PathBuf,
}

/// Print out latest information stored in the DB.
fn print_head(db: &LibraDB) {
    let version = db
        .get_latest_version()
        .expect("get_latest_version throw error");

    info!("Latest Version: {}", version);

    let si = db
        .get_startup_info()
        .expect("Can't get startup info")
        .expect("StartupInfo is empty, database is empty.");

    info!(
        "The latest ledger info: {}",
        si.latest_ledger_info.ledger_info()
    );

    info!("Signatures: {:?}", si.latest_ledger_info.signatures());

    info!(
        "Current Validator Set: {}",
        si.latest_validator_set
            .expect("Unable to determine validator set, DB incorrect")
    );

    let tx = db
        .get_transaction_with_proof(version, version, false)
        .expect("Unable to load latest TXN");
    info!(
        "Current Transaction: {}",
        tx.transaction.format_for_client(get_transaction_name)
    );
}

/// List all TXNs from DB and verify their proofs
fn verify_txns(db: &LibraDB) -> Result<()> {
    let si = db
        .get_startup_info()
        .expect("Can't get startup info")
        .expect("StartupInfo is empty, database is empty.");
    let ledger_info = si.latest_ledger_info.ledger_info();

    let latest_version = ledger_info.version();

    for n in (1..latest_version).rev() {
        let tx = db.get_transaction_with_proof(n, latest_version, true)?;

        let tx_info = tx.proof.transaction_info();

        let events_root_hash = tx.events.as_ref().map(|events| {
            let event_hashes: Vec<_> = events.iter().map(ContractEvent::hash).collect();
            InMemoryAccumulator::<EventAccumulatorHasher>::from_leaves(&event_hashes).root_hash()
        });

        // verify TX proof
        tx.proof
            .verify(ledger_info, tx.transaction.hash(), events_root_hash, n)?;
    }
    info!("Total TXNs verified: {}", latest_version);

    Ok(())
}

// Verify the last accounts state tree by iterating through all accounts
fn verify_latest_account_state(db: &LibraDB) -> Result<()> {
    let version = db.get_latest_version()?;
    let backup = db.get_backup_handler();

    let iter = backup.get_account_iter(version)?;

    let mut total = 0;
    for i in iter {
        let (addr_hash, blob) = i?;
        ensure!(addr_hash != HashValue::zero(), "Invalid address hash!");
        ensure!(blob.hash() != HashValue::zero(), "Invalid blob!");
        total += 1;
    }
    info!("Total account verified: {}", total);

    // TODO: verify the root hash is correct

    Ok(())
}

fn main() {
    let _logger = libra_logger::set_global_logger(false, None);

    let opt = Opt::from_args();

    let p = opt.db.as_path();

    if !p.is_dir() {
        info!("Invalid Directory {:?}!", p);
        std::process::exit(-1);
    }

    let log_dir = tempfile::tempdir().expect("Unable to get temp dir");
    info!("Opening DB at: {:?}, log at {:?}", p, log_dir.path());

    let db = LibraDB::open(p, true, Some(log_dir.path())).expect("Unable to open LibraDB");
    info!("DB opened successfully.");

    print_head(&db);

    verify_txns(&db).expect("Unable to verify all TXNs");

    verify_latest_account_state(&db).expect("Unable to verify account state");

    info!("Successfully verified database.");
}

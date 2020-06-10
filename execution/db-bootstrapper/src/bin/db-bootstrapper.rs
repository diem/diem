// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{ensure, format_err, Context, Result};
use executor::db_bootstrapper::calculate_genesis;
use libra_types::{transaction::Transaction, waypoint::Waypoint};
use libra_vm::LibraVM;
use libradb::LibraDB;
use std::{fs::File, io::Read, path::PathBuf};
use storage_interface::DbReaderWriter;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(
    name = "db-bootstrapper",
    about = "Calculate, verify and commit the genesis to local DB without a consensus among validators."
)]
struct Opt {
    #[structopt(parse(from_os_str))]
    db_dir: PathBuf,

    #[structopt(short, long, parse(from_os_str))]
    genesis_txn_file: PathBuf,

    #[structopt(short, long)]
    waypoint_to_verify: Option<Waypoint>,

    #[structopt(long, requires("waypoint-to-verify"))]
    commit: bool,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    let genesis_txn = load_genesis_txn(&opt.genesis_txn_file)
        .with_context(|| format_err!("Failed loading genesis txn."))?;
    let db = DbReaderWriter::new(
        LibraDB::open(
            &opt.db_dir,
            false, /* readonly */
            None,  /* pruner */
        )
        .with_context(|| format_err!("Failed to open DB."))?,
    );

    let tree_state = db
        .reader
        .get_latest_tree_state()
        .with_context(|| format_err!("Failed to get latest tree state."))?;
    let committer = calculate_genesis::<LibraVM>(&db, tree_state, &genesis_txn)
        .with_context(|| format_err!("Failed to calculate genesis."))?;
    println!("Successfully calculated genesis.");
    println!("{:?}", committer.waypoint());

    if let Some(waypoint) = opt.waypoint_to_verify {
        ensure!(
            waypoint == committer.waypoint(),
            "Waypoint verification failed. Expected {:?}, got {:?}.",
            waypoint,
            committer.waypoint(),
        );
        println!("Waypoint verified.");

        if opt.commit {
            committer
                .commit()
                .with_context(|| format_err!("Committing genesis to DB."))?;
            println!("Successfully committed genesis.")
        }
    }

    Ok(())
}

fn load_genesis_txn(path: &PathBuf) -> Result<Transaction> {
    let mut file = File::open(&path)?;
    let mut buffer = vec![];
    file.read_to_end(&mut buffer)?;

    Ok(lcs::from_bytes(&buffer)?)
}

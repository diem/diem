// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Context, Result};
use backup_cli::{
    restore::{GlobalRestoreOpt, StateSnapshotRestoreController, StateSnapshotRestoreOpt},
    storage::StorageOpt,
};
use libradb::LibraDB;
use std::sync::Arc;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Opt {
    #[structopt(flatten)]
    global: GlobalRestoreOpt,

    #[structopt(flatten)]
    state_snapshot: StateSnapshotRestoreOpt,

    #[structopt(subcommand)]
    storage: StorageOpt,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Opt::from_args();

    let db = Arc::new(
        LibraDB::open(
            opt.global.db_dir,
            false, /* read_only */
            None,  /* pruner */
        )
        .expect("Failed opening DB."),
    );
    let storage = opt.storage.init_storage().await?;
    StateSnapshotRestoreController::new(opt.state_snapshot, storage, db)
        .run()
        .await
        .context("Failed restoring state_snapshot.")?;

    println!("Finished restoring account state.");

    Ok(())
}

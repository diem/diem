// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Context, Result};
use backup_cli::{
    backup_types::{
        epoch_ending::restore::{EpochEndingRestoreController, EpochEndingRestoreOpt},
        state_snapshot::restore::{StateSnapshotRestoreController, StateSnapshotRestoreOpt},
        transaction::restore::{TransactionRestoreController, TransactionRestoreOpt},
    },
    storage::StorageOpt,
    utils::GlobalRestoreOpt,
};
use libradb::LibraDB;
use std::sync::Arc;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Opt {
    #[structopt(flatten)]
    global: GlobalRestoreOpt,

    #[structopt(subcommand)]
    restore_type: RestoreType,
}

#[derive(StructOpt)]
enum RestoreType {
    EpochEnding {
        #[structopt(flatten)]
        opt: EpochEndingRestoreOpt,
        #[structopt(subcommand)]
        storage: StorageOpt,
    },
    StateSnapshot {
        #[structopt(flatten)]
        opt: StateSnapshotRestoreOpt,
        #[structopt(subcommand)]
        storage: StorageOpt,
    },
    Transaction {
        #[structopt(flatten)]
        opt: TransactionRestoreOpt,
        #[structopt(subcommand)]
        storage: StorageOpt,
    },
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
    let restore_handler = Arc::new(db.get_restore_handler());

    match opt.restore_type {
        RestoreType::EpochEnding { opt, storage } => {
            EpochEndingRestoreController::new(opt, storage.init_storage().await?, restore_handler)
                .run()
                .await
                .map(|_| println!("Epoch ending information restore success."))
                .context("Failed restoring epoch ending information.")?;
        }
        RestoreType::StateSnapshot { opt, storage } => {
            StateSnapshotRestoreController::new(
                opt,
                storage.init_storage().await?,
                restore_handler,
            )
            .run()
            .await
            .map(|_| println!("State snapshot restore success."))
            .context("Failed restoring state snapshot.")?;
        }
        RestoreType::Transaction { opt, storage } => {
            TransactionRestoreController::new(opt, storage.init_storage().await?, restore_handler)
                .run()
                .await
                .map(|_| println!("Transactions restore success."))
                .context("Failed restoring state snapshot.")?;
        }
    }

    Ok(())
}

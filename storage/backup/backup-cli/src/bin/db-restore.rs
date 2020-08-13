// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use backup_cli::{
    backup_types::{
        epoch_ending::restore::{EpochEndingRestoreController, EpochEndingRestoreOpt},
        state_snapshot::restore::{StateSnapshotRestoreController, StateSnapshotRestoreOpt},
        transaction::restore::{TransactionRestoreController, TransactionRestoreOpt},
    },
    coordinators::restore::{RestoreCoordinator, RestoreCoordinatorOpt},
    storage::StorageOpt,
    utils::GlobalRestoreOpt,
};
use libra_secure_push_metrics::MetricsPusher;
use libradb::{GetRestoreHandler, LibraDB};
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
    Auto {
        #[structopt(flatten)]
        opt: RestoreCoordinatorOpt,
        #[structopt(subcommand)]
        storage: StorageOpt,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    libra_logger::Logger::new().init();
    MetricsPusher.start();

    let opt = Opt::from_args();
    let db = Arc::new(
        LibraDB::open(
            &opt.global.db_dir,
            false, /* read_only */
            None,  /* pruner */
        )
        .expect("Failed opening DB."),
    );
    let restore_handler = Arc::new(db.get_restore_handler());

    let global_opt = opt.global;
    match opt.restore_type {
        RestoreType::EpochEnding { opt, storage } => {
            EpochEndingRestoreController::new(
                opt,
                global_opt,
                storage.init_storage().await?,
                restore_handler,
            )
            .run()
            .await?;
        }
        RestoreType::StateSnapshot { opt, storage } => {
            StateSnapshotRestoreController::new(
                opt,
                global_opt,
                storage.init_storage().await?,
                restore_handler,
            )
            .run()
            .await?;
        }
        RestoreType::Transaction { opt, storage } => {
            TransactionRestoreController::new(
                opt,
                global_opt,
                storage.init_storage().await?,
                restore_handler,
            )
            .run()
            .await?;
        }
        RestoreType::Auto { opt, storage } => {
            RestoreCoordinator::new(
                opt,
                global_opt,
                storage.init_storage().await?,
                restore_handler,
            )
            .run()
            .await?;
        }
    }

    Ok(())
}

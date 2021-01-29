// Copyright (c) The Diem Core Contributors
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
    utils::{GlobalRestoreOpt, GlobalRestoreOptions},
};
use diem_logger::{prelude::*, Level, Logger};
use diem_secure_push_metrics::MetricsPusher;
use std::convert::TryInto;
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
    main_impl().await.map_err(|e| {
        error!("main_impl() failed: {}", e);
        e
    })
}

async fn main_impl() -> Result<()> {
    Logger::new().level(Level::Info).read_env().init();
    let _mp = MetricsPusher::start();

    let opt = Opt::from_args();
    let global_opt: GlobalRestoreOptions = opt.global.clone().try_into()?;

    match opt.restore_type {
        RestoreType::EpochEnding { opt, storage } => {
            EpochEndingRestoreController::new(opt, global_opt, storage.init_storage().await?)
                .run(None)
                .await?;
        }
        RestoreType::StateSnapshot { opt, storage } => {
            StateSnapshotRestoreController::new(
                opt,
                global_opt,
                storage.init_storage().await?,
                None, /* epoch_history */
            )
            .run()
            .await?;
        }
        RestoreType::Transaction { opt, storage } => {
            TransactionRestoreController::new(
                opt,
                global_opt,
                storage.init_storage().await?,
                None, /* epoch_history */
            )
            .run()
            .await?;
        }
        RestoreType::Auto { opt, storage } => {
            RestoreCoordinator::new(opt, global_opt, storage.init_storage().await?)
                .run()
                .await?;
        }
    }

    Ok(())
}

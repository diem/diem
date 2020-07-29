// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use backup_cli::{
    backup_types::{
        epoch_ending::backup::{EpochEndingBackupController, EpochEndingBackupOpt},
        state_snapshot::backup::{StateSnapshotBackupController, StateSnapshotBackupOpt},
        transaction::backup::{TransactionBackupController, TransactionBackupOpt},
    },
    metadata::{cache, cache::MetadataCacheOpt},
    storage::StorageOpt,
    utils::{
        backup_service_client::{BackupServiceClient, BackupServiceClientOpt},
        GlobalBackupOpt,
    },
};
use std::sync::Arc;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(about = "Libra backup tool.")]
enum Command {
    #[structopt(about = "Manually run one shot commands.")]
    OneShot(OneShotCommand),
}

#[derive(StructOpt)]
enum OneShotCommand {
    #[structopt(about = "Query the backup service builtin in the local Libra node.")]
    Query(OneShotQueryType),
    #[structopt(about = "Do a one shot backup.")]
    Backup(OneShotBackupOpt),
}

#[derive(StructOpt)]
enum OneShotQueryType {
    #[structopt(
        about = "Queries the latest epoch, committed version and synced version of the local Libra \
        node, via the backup service within it."
    )]
    NodeState(OneShotQueryNodeStateOpt),
    #[structopt(
        about = "Queries the latest epoch and versions of the existing backups in the storage."
    )]
    BackupStorageState(OneShotQueryBackupStorageStateOpt),
}

#[derive(StructOpt)]
struct OneShotQueryNodeStateOpt {
    #[structopt(flatten)]
    client: BackupServiceClientOpt,
}

#[derive(StructOpt)]
struct OneShotQueryBackupStorageStateOpt {
    #[structopt(flatten)]
    metadata_cache: MetadataCacheOpt,
    #[structopt(subcommand)]
    storage: StorageOpt,
}

#[derive(StructOpt)]
struct OneShotBackupOpt {
    #[structopt(flatten)]
    global: GlobalBackupOpt,

    #[structopt(flatten)]
    client: BackupServiceClientOpt,

    #[structopt(subcommand)]
    backup_type: BackupType,
}

#[derive(StructOpt)]
enum BackupType {
    EpochEnding {
        #[structopt(flatten)]
        opt: EpochEndingBackupOpt,
        #[structopt(subcommand)]
        storage: StorageOpt,
    },
    StateSnapshot {
        #[structopt(flatten)]
        opt: StateSnapshotBackupOpt,
        #[structopt(subcommand)]
        storage: StorageOpt,
    },
    Transaction {
        #[structopt(flatten)]
        opt: TransactionBackupOpt,
        #[structopt(subcommand)]
        storage: StorageOpt,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cmd = Command::from_args();
    match cmd {
        Command::OneShot(one_shot_cmd) => match one_shot_cmd {
            OneShotCommand::Query(typ) => match typ {
                OneShotQueryType::NodeState(opt) => {
                    let client = BackupServiceClient::new_with_opt(opt.client);
                    if let Some(db_state) = client.get_db_state().await? {
                        println!("{}", db_state)
                    } else {
                        println!("DB not bootstrapped.")
                    }
                }
                OneShotQueryType::BackupStorageState(opt) => {
                    let view = cache::sync_and_load(
                        &opt.metadata_cache,
                        opt.storage.init_storage().await?,
                    )
                    .await?;
                    println!("{}", view.get_storage_state())
                }
            },
            OneShotCommand::Backup(opt) => {
                let client = Arc::new(BackupServiceClient::new_with_opt(opt.client));
                let global_opt = opt.global;

                match opt.backup_type {
                    BackupType::EpochEnding { opt, storage } => {
                        EpochEndingBackupController::new(
                            opt,
                            global_opt,
                            client,
                            storage.init_storage().await?,
                        )
                        .run()
                        .await?;
                    }
                    BackupType::StateSnapshot { opt, storage } => {
                        StateSnapshotBackupController::new(
                            opt,
                            global_opt,
                            client,
                            storage.init_storage().await?,
                        )
                        .run()
                        .await?;
                    }
                    BackupType::Transaction { opt, storage } => {
                        TransactionBackupController::new(
                            opt,
                            global_opt,
                            client,
                            storage.init_storage().await?,
                        )
                        .run()
                        .await?;
                    }
                }
            }
        },
    }
    Ok(())
}

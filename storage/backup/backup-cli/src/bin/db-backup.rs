// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Context, Result};
use backup_cli::{
    backup::{
        BackupServiceClient, BackupServiceClientOpt, GlobalBackupOpt,
        StateSnapshotBackupController, StateSnapshotBackupOpt,
    },
    storage::StorageOpt,
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
    Query(OneShotQueryOpt),
    #[structopt(about = "Do a one shot backup.")]
    Backup(OneShotBackupOpt),
}

#[derive(StructOpt)]
struct OneShotQueryOpt {
    #[structopt(flatten)]
    client: BackupServiceClientOpt,
    #[structopt(
        long = "latest-version",
        help = "Queries the latest version of the ledger."
    )]
    latest_version: bool,
}

#[derive(StructOpt)]
struct OneShotBackupOpt {
    #[structopt(flatten)]
    global: GlobalBackupOpt,

    #[structopt(flatten)]
    state_snapshot: StateSnapshotBackupOpt,

    #[structopt(flatten)]
    client: BackupServiceClientOpt,

    #[structopt(subcommand)]
    storage: StorageOpt,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cmd = Command::from_args();
    match cmd {
        Command::OneShot(one_shot_cmd) => match one_shot_cmd {
            OneShotCommand::Query(opt) => {
                let client = BackupServiceClient::new_with_opt(opt.client);
                if opt.latest_version {
                    let (v, _) = client.get_latest_state_root().await?;
                    println!("latest-version: {}", v);
                }
            }
            OneShotCommand::Backup(opt) => {
                let client = Arc::new(BackupServiceClient::new_with_opt(opt.client));
                let storage = opt.storage.init_storage().await?;

                let manifest = StateSnapshotBackupController::new(
                    opt.state_snapshot,
                    opt.global,
                    client,
                    storage,
                )
                .run()
                .await
                .context("Failed to backup account state.")?;

                println!("Success. Manifest saved to {}", &manifest);
            }
        },
    }
    Ok(())
}

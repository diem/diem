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
struct Opt {
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
    let opt = Opt::from_args();
    let client = Arc::new(BackupServiceClient::new_with_opt(opt.client));
    let storage = opt.storage.init_storage().await?;

    let manifest =
        StateSnapshotBackupController::new(opt.state_snapshot, opt.global, client, storage)
            .run()
            .await
            .context("Failed to backup account state.")?;

    println!("Success. Manifest saved to {}", &manifest);

    Ok(())
}

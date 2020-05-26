// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use backup_cli::{
    backup::{
        BackupServiceClient, BackupServiceClientOpt, GlobalBackupOpt,
        StateSnapshotBackupController, StateSnapshotBackupOpt,
    },
    storage::local_fs::{LocalFs, LocalFsOpt},
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

    #[structopt(flatten)]
    storage: LocalFsOpt,
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();
    let client = Arc::new(BackupServiceClient::new_with_opt(opt.client));
    let storage = Arc::new(LocalFs::new_with_opt(opt.storage));

    let manifest =
        StateSnapshotBackupController::new(opt.state_snapshot, opt.global, client, storage)
            .run()
            .await
            .expect("Failed to backup account state.");

    println!("Success. Manifest saved to {}", &manifest);
}

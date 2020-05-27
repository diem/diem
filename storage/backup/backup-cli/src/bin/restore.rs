// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use backup_cli::{
    restore::{GlobalRestoreOpt, StateSnapshotRestoreController, StateSnapshotRestoreOpt},
    storage::local_fs::{LocalFs, LocalFsOpt},
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

    #[structopt(flatten)]
    storage: LocalFsOpt,
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();

    let db = Arc::new(
        LibraDB::open(
            opt.global.db_dir,
            false, /* read_only */
            None,  /* pruner */
        )
        .expect("Failed opening DB."),
    );
    let storage = Arc::new(LocalFs::new_with_opt(opt.storage));
    StateSnapshotRestoreController::new(opt.state_snapshot, storage, db)
        .run()
        .await
        .expect("Failed restoring state_snapshot.");

    println!("Finished restoring account state.");
}

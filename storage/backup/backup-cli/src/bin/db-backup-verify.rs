// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use backup_cli::{
    coordinators::verify::VerifyCoordinator, metadata::cache::MetadataCacheOpt, storage::StorageOpt,
};
use diem_logger::{prelude::*, Level, Logger};
use diem_secure_push_metrics::MetricsPusher;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Opt {
    #[structopt(flatten)]
    metadata_cache_opt: MetadataCacheOpt,
    #[structopt(subcommand)]
    storage: StorageOpt,
}

#[tokio::main]
async fn main() -> Result<()> {
    main_impl().await.map_err(|e| {
        error!("main_impl() failed: {}", e);
        e
    })
}

async fn main_impl() -> Result<()> {
    Logger::new().level(Level::Info).init();
    let _mp = MetricsPusher::start();

    let opt = Opt::from_args();
    VerifyCoordinator::new(opt.storage.init_storage().await?, opt.metadata_cache_opt)?
        .run()
        .await
}

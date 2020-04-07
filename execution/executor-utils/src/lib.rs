// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(any(test, feature = "testing"))]
pub mod test_helpers;

use executor::{db_bootstrapper::maybe_bootstrap_db, Executor};
use libra_config::config::NodeConfig;
use libra_vm::LibraVM;
use std::sync::Arc;
use storage_client::SyncStorageClient;
use storage_service::start_storage_service;
use tokio::runtime::Runtime;

pub fn create_storage_service_and_executor(config: &NodeConfig) -> (Runtime, Executor<LibraVM>) {
    let rt = start_storage_service(config);
    maybe_bootstrap_db::<LibraVM>(config).unwrap();

    let db_reader = Arc::new(SyncStorageClient::new(&config.storage.address));
    let db_writer = Arc::clone(&db_reader);
    let executor = Executor::new(db_reader, db_writer);

    (rt, executor)
}

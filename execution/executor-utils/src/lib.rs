// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(any(test, feature = "testing"))]
pub mod test_helpers;

use executor::{db_bootstrapper::maybe_bootstrap_db, Executor};
use libra_config::config::NodeConfig;
use libra_vm::LibraVM;
use std::sync::Arc;
use storage_client::{StorageReadServiceClient, StorageWriteServiceClient};
use storage_service::start_storage_service;
use tokio::runtime::Runtime;

pub fn create_storage_service_and_executor(config: &NodeConfig) -> (Runtime, Executor<LibraVM>) {
    let rt = start_storage_service(config);
    maybe_bootstrap_db::<LibraVM>(config).unwrap();

    let storage_read_client = Arc::new(StorageReadServiceClient::new(&config.storage.address));
    let storage_write_client = Arc::new(StorageWriteServiceClient::new(&config.storage.address));
    let executor = Executor::new(storage_read_client, storage_write_client);

    (rt, executor)
}

// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(any(test, feature = "testing"))]
pub mod test_helpers;

use executor::{db_bootstrapper::bootstrap_db_if_empty, Executor};
use libra_config::config::NodeConfig;
use libra_vm::LibraVM;
use storage_client::SyncStorageClient;
use storage_service::{init_libra_db, start_storage_service_with_db};
use tokio::runtime::Runtime;

pub fn create_storage_service_and_executor(config: &NodeConfig) -> (Runtime, Executor<LibraVM>) {
    let (db, db_rw) = init_libra_db(config);
    bootstrap_db_if_empty::<LibraVM>(&db_rw, config).unwrap();

    let rt = start_storage_service_with_db(config, db);
    let executor = Executor::new(SyncStorageClient::new(&config.storage.address).into());

    (rt, executor)
}

// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(any(test, feature = "testing"))]
pub mod test_helpers;

use executor::{db_bootstrapper::bootstrap_db_if_empty, Executor};
use libra_config::{config::NodeConfig, utils::get_genesis_txn};
use libra_vm::LibraVM;
use std::sync::Arc;
use storage_client::SyncStorageClient;
use storage_interface::DbReader;
use storage_service::{init_libra_db, start_storage_service_with_db};
use tokio::runtime::Runtime;

pub fn create_storage_service_and_executor(
    config: &NodeConfig,
) -> (Arc<dyn DbReader>, Runtime, Executor<LibraVM>) {
    let (db, db_rw) = init_libra_db(config);
    bootstrap_db_if_empty::<LibraVM>(&db_rw, get_genesis_txn(config).unwrap()).unwrap();

    let rt = start_storage_service_with_db(config, db.clone());
    let executor = Executor::new(SyncStorageClient::new(&config.storage.address).into());

    (db, rt, executor)
}

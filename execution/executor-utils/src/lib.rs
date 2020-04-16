// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(any(test, feature = "testing"))]
pub mod test_helpers;

use executor::{db_bootstrapper::maybe_bootstrap_db, Executor};
use libra_config::config::NodeConfig;
use libra_vm::LibraVM;
use storage_client::SyncStorageClient;
use storage_service::{init_libra_db, start_storage_service_with_db};
use tokio::runtime::Runtime;

pub fn create_storage_service_and_executor(config: &NodeConfig) -> (Runtime, Executor<LibraVM>) {
    let (arc_db, db_reader_writer) = init_libra_db(config);
    maybe_bootstrap_db::<LibraVM>(db_reader_writer, config).unwrap();

    let rt = start_storage_service_with_db(config, arc_db);
    let executor = Executor::new(SyncStorageClient::new(&config.storage.address).into());

    (rt, executor)
}

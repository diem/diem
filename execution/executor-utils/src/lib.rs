// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(any(test, feature = "testing"))]
pub mod test_helpers;

use executor::{db_bootstrapper::maybe_bootstrap_db, Executor};
use libra_config::config::NodeConfig;
use libra_vm::LibraVM;
use libradb::LibraDB;
use storage_interface::DbReaderWriter;

pub fn create_db_and_executor(config: &NodeConfig) -> (DbReaderWriter, Executor<LibraVM>) {
    let db = DbReaderWriter::new(LibraDB::new(&config.storage.dir()));
    maybe_bootstrap_db::<LibraVM>(db.clone(), config).unwrap();
    (db.clone(), Executor::new(db))
}

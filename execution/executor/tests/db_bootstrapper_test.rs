// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use config_builder::test_config;
use executor::db_bootstrapper::bootstrap_db_if_empty;
use libra_temppath::TempPath;
use libra_types::waypoint::Waypoint;
use libra_vm::LibraVM;
use libradb::LibraDB;
use storage_interface::DbReaderWriter;

#[test]
fn test_empty_db() {
    let (config, _) = test_config();
    let tmp_dir = TempPath::new();
    let db_rw = DbReaderWriter::new(LibraDB::new(&tmp_dir));

    // Executor won't be able to boot on empty db due to lack of StartupInfo.
    assert!(db_rw.reader.get_startup_info().unwrap().is_none());

    // Bootstrap empty DB.
    let waypoint = bootstrap_db_if_empty::<LibraVM>(db_rw.clone(), &config)
        .expect("Should not fail.")
        .expect("Should not be None.");
    let startup_info = db_rw
        .reader
        .get_startup_info()
        .expect("Should not fail.")
        .expect("Should not be None.");
    assert_eq!(
        Waypoint::new(startup_info.latest_ledger_info.ledger_info()).unwrap(),
        waypoint
    );

    // `bootstrap_db_if_empty()` does nothing on non-empty DB.
    assert!(bootstrap_db_if_empty::<LibraVM>(db_rw, &config)
        .unwrap()
        .is_none())
}

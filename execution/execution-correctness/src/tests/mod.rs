// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod local;
mod serializer;
mod spawned_process;
mod suite;
mod thread;

use executor::db_bootstrapper::bootstrap_db_if_empty;
use grpc_storage::service::start_simple_storage_service_with_db;
use libra_config::{config::NodeConfig, utils};
use libra_vm::LibraVM;
use libradb::LibraDB;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    thread::JoinHandle,
};
use storage_interface::DbReaderWriter;

fn start_storage_service() -> (NodeConfig, JoinHandle<()>) {
    let (mut config, _genesis_key) = config_builder::test_config();
    let tmp_dir = libra_temppath::TempPath::new();
    config.storage.dir = tmp_dir.path().to_path_buf();

    let server_port = utils::get_available_port();
    config.storage.simple_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), server_port);
    let (db, db_rw) = DbReaderWriter::wrap(LibraDB::new_for_test(&config.storage.dir()));
    bootstrap_db_if_empty::<LibraVM>(&db_rw, utils::get_genesis_txn(&config).unwrap()).unwrap();
    let handle = start_simple_storage_service_with_db(&config, db);
    (config, handle)
}

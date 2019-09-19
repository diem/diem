// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! End-to-end tests for the execution service.
//!
//! The non-test code in this crate is helper code and can be used by test code outside of
//! execution.

#[cfg(test)]
mod tests;

use config::config::NodeConfig;
use crypto::HashValue;
use execution_proto::proto::execution_grpc::create_execution;
use execution_service::ExecutionService;
use grpc_helpers::ServerHandle;
use grpcio::{EnvBuilder, ServerBuilder};
use std::{collections::HashMap, sync::Arc};
use storage_client::{StorageReadServiceClient, StorageWriteServiceClient};
use storage_service::start_storage_service;
use types::{crypto_proxies::LedgerInfoWithSignatures, ledger_info::LedgerInfo};

pub fn gen_block_id(index: u8) -> HashValue {
    HashValue::new([index; HashValue::LENGTH])
}

pub fn gen_ledger_info_with_sigs(
    version: u64,
    root_hash: HashValue,
    commit_block_id: HashValue,
) -> LedgerInfoWithSignatures {
    let ledger_info = LedgerInfo::new(
        version,
        root_hash,
        /* consensus_data_hash = */ HashValue::zero(),
        commit_block_id,
        0,
        /* timestamp = */ 0,
        None,
    );
    LedgerInfoWithSignatures::new(ledger_info, /* signatures = */ HashMap::new())
}

pub fn create_and_start_server(config: &NodeConfig) -> (ServerHandle, grpcio::Server) {
    let storage_server_handle = start_storage_service(config);

    let client_env = Arc::new(EnvBuilder::new().build());
    let storage_read_client = Arc::new(StorageReadServiceClient::new(
        Arc::clone(&client_env),
        &config.storage.address,
        config.storage.port,
    ));
    let storage_write_client = Arc::new(StorageWriteServiceClient::new(
        Arc::clone(&client_env),
        &config.storage.address,
        config.storage.port,
        None,
    ));

    let execution_service = create_execution(ExecutionService::new(
        storage_read_client,
        storage_write_client,
        config,
    ));
    let mut execution_server = ServerBuilder::new(Arc::new(EnvBuilder::new().build()))
        .register_service(execution_service)
        .bind(config.execution.address.clone(), config.execution.port)
        .build()
        .expect("Failed to create execution server.");
    execution_server.start();

    (storage_server_handle, execution_server)
}

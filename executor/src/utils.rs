use crate::{ExecutedTrees, Executor};
use libra_config::config::NodeConfig;
use std::sync::Arc;
use storage_client::{StorageRead, StorageReadServiceClient, StorageWriteServiceClient};
use storage_service::start_storage_service;
use tokio::runtime::Runtime;
use vm_runtime::LibraVM;

pub fn create_storage_service_and_executor(
    config: &NodeConfig,
) -> (Runtime, Executor<LibraVM>, ExecutedTrees) {
    let mut rt = start_storage_service(config);

    let storage_read_client = Arc::new(StorageReadServiceClient::new(&config.storage.address));
    let storage_write_client = Arc::new(StorageWriteServiceClient::new(&config.storage.address));

    let executor = Executor::new(storage_read_client, storage_write_client, config);

    let storage_read_client = Arc::new(StorageReadServiceClient::new(&config.storage.address));

    let startup_info = rt
        .block_on(storage_read_client.get_startup_info())
        .expect("unable to read ledger info from storage")
        .expect("startup info is None");
    let committed_trees = ExecutedTrees::from(startup_info.committed_tree_state);
    (rt, executor, committed_trees)
}

// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    local_client::LocalClient,
    process::ProcessService,
    remote_service::RemoteService,
    serializer::{SerializerClient, SerializerService},
    spawned_process::SpawnedProcess,
    thread::ThreadService,
};
use executor::Executor;
use executor_types::BlockExecutor;
use libra_config::config::{ExecutionCorrectnessService, NodeConfig};
use libra_vm::LibraVM;
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use storage_client::StorageClient;

enum ExecutionCorrectnessWrapper {
    Local(Arc<Mutex<Box<dyn BlockExecutor>>>),
    Process(ProcessService),
    Serializer(Arc<Mutex<SerializerService>>),
    SpawnedProcess(SpawnedProcess),
    Thread(ThreadService),
}

pub struct ExecutionCorrectnessManager {
    internal_execution_correctness: ExecutionCorrectnessWrapper,
}

impl ExecutionCorrectnessManager {
    pub fn new(config: &mut NodeConfig) -> Self {
        let storage_address = config.storage.address;
        match &config.execution.service {
            ExecutionCorrectnessService::Process(remote_service) => {
                Self::new_process(remote_service.server_address)
            }
            ExecutionCorrectnessService::SpawnedProcess(_) => Self::new_spawned_process(config),
            ExecutionCorrectnessService::Local => Self::new_local(storage_address),
            ExecutionCorrectnessService::Serializer => Self::new_serializer(storage_address),
            ExecutionCorrectnessService::Thread => Self::new_thread(storage_address),
        }
    }

    pub fn new_local(storage_address: SocketAddr) -> Self {
        let block_executor = Box::new(Executor::<LibraVM>::new(
            StorageClient::new(&storage_address).into(),
        ));
        Self {
            internal_execution_correctness: ExecutionCorrectnessWrapper::Local(Arc::new(
                Mutex::new(block_executor),
            )),
        }
    }

    pub fn new_process(server_addr: SocketAddr) -> Self {
        let process_service = ProcessService::new(server_addr);
        Self {
            internal_execution_correctness: ExecutionCorrectnessWrapper::Process(process_service),
        }
    }

    pub fn new_serializer(storage_address: SocketAddr) -> Self {
        let block_executor = Box::new(Executor::<LibraVM>::new(
            StorageClient::new(&storage_address).into(),
        ));
        let serializer_service = SerializerService::new(block_executor);
        Self {
            internal_execution_correctness: ExecutionCorrectnessWrapper::Serializer(Arc::new(
                Mutex::new(serializer_service),
            )),
        }
    }

    pub fn new_spawned_process(config: &NodeConfig) -> Self {
        let process = SpawnedProcess::new(config);
        Self {
            internal_execution_correctness: ExecutionCorrectnessWrapper::SpawnedProcess(process),
        }
    }

    pub fn new_thread(storage_address: SocketAddr) -> Self {
        let thread = ThreadService::new(storage_address);
        Self {
            internal_execution_correctness: ExecutionCorrectnessWrapper::Thread(thread),
        }
    }

    pub fn client(&self) -> Box<dyn BlockExecutor + Send + Sync> {
        match &self.internal_execution_correctness {
            ExecutionCorrectnessWrapper::Local(execution_correctness) => {
                Box::new(LocalClient::new(execution_correctness.clone()))
            }
            ExecutionCorrectnessWrapper::Process(process) => Box::new(process.client()),
            ExecutionCorrectnessWrapper::Serializer(serializer_service) => {
                Box::new(SerializerClient::new(serializer_service.clone()))
            }
            ExecutionCorrectnessWrapper::SpawnedProcess(process) => Box::new(process.client()),
            ExecutionCorrectnessWrapper::Thread(thread) => Box::new(thread.client()),
        }
    }
}

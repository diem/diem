// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    execution_correctness::ExecutionCorrectness,
    local::{LocalClient, LocalService},
    process::ProcessService,
    remote_service::RemoteService,
    serializer::{SerializerClient, SerializerService},
    spawned_process::SpawnedProcess,
    thread::ThreadService,
};
use executor::Executor;
use libra_config::config::{ExecutionCorrectnessService, NodeConfig};
use libra_crypto::ed25519::Ed25519PrivateKey;
use libra_global_constants::EXECUTION_KEY;
use libra_secure_storage::{CryptoStorage, Storage};
use libra_vm::LibraVM;
use std::{
    convert::TryInto,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use storage_client::StorageClient;

pub fn extract_execution_prikey(config: &mut NodeConfig) -> Option<Ed25519PrivateKey> {
    let backend = &config.execution.backend;
    let mut storage: Storage = backend.try_into().expect("Unable to initialize storage");
    if let Some(test_config) = config.test.as_mut() {
        if let Some(private_key) = test_config
            .execution_keypair
            .as_mut()
            .expect("Missing execution keypair in test config")
            .take_private()
        {
            storage
                .import_private_key(EXECUTION_KEY, private_key)
                .expect("Unable to insert execution key");
        }
    }
    storage.export_private_key(EXECUTION_KEY).ok()
}

enum ExecutionCorrectnessWrapper {
    Local(Arc<Mutex<LocalService>>),
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
        match &config.execution.service {
            ExecutionCorrectnessService::Process(remote_service) => {
                return Self::new_process(remote_service.server_address)
            }
            ExecutionCorrectnessService::SpawnedProcess(_) => {
                return Self::new_spawned_process(config)
            }
            _ => (),
        };

        let execution_prikey = extract_execution_prikey(config);
        let storage_address = config.storage.address;
        match &config.execution.service {
            ExecutionCorrectnessService::Local => {
                Self::new_local(storage_address, execution_prikey)
            }
            ExecutionCorrectnessService::Serializer => {
                Self::new_serializer(storage_address, execution_prikey)
            }
            ExecutionCorrectnessService::Thread => {
                Self::new_thread(storage_address, execution_prikey)
            }
            _ => unreachable!(
                "Unimplemented ExecutionCorrectnessService: {:?}",
                config.execution.service
            ),
        }
    }

    pub fn new_local(
        storage_address: SocketAddr,
        execution_prikey: Option<Ed25519PrivateKey>,
    ) -> Self {
        let block_executor = Box::new(Executor::<LibraVM>::new(
            StorageClient::new(&storage_address).into(),
        ));
        Self {
            internal_execution_correctness: ExecutionCorrectnessWrapper::Local(Arc::new(
                Mutex::new(LocalService::new(block_executor, execution_prikey)),
            )),
        }
    }

    pub fn new_process(server_addr: SocketAddr) -> Self {
        let process_service = ProcessService::new(server_addr);
        Self {
            internal_execution_correctness: ExecutionCorrectnessWrapper::Process(process_service),
        }
    }

    pub fn new_serializer(
        storage_address: SocketAddr,
        execution_prikey: Option<Ed25519PrivateKey>,
    ) -> Self {
        let block_executor = Box::new(Executor::<LibraVM>::new(
            StorageClient::new(&storage_address).into(),
        ));
        let serializer_service = SerializerService::new(block_executor, execution_prikey);
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

    pub fn new_thread(
        storage_address: SocketAddr,
        execution_prikey: Option<Ed25519PrivateKey>,
    ) -> Self {
        let thread = ThreadService::new(storage_address, execution_prikey);
        Self {
            internal_execution_correctness: ExecutionCorrectnessWrapper::Thread(thread),
        }
    }

    pub fn client(&self) -> Box<dyn ExecutionCorrectness + Send + Sync> {
        match &self.internal_execution_correctness {
            ExecutionCorrectnessWrapper::Local(local_service) => {
                Box::new(LocalClient::new(local_service.clone()))
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

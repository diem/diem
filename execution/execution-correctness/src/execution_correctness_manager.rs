// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    execution_correctness::ExecutionCorrectness,
    local::{LocalClient, LocalService},
    process::ProcessService,
    remote_service::RemoteService,
    serializer::{SerializerClient, SerializerService},
    thread::ThreadService,
};
use diem_config::config::{ExecutionCorrectnessService, NodeConfig};
use diem_crypto::ed25519::Ed25519PrivateKey;
use diem_global_constants::EXECUTION_KEY;
use diem_infallible::Mutex;
use diem_secure_storage::{CryptoStorage, Storage};
use diem_vm::DiemVM;
use executor::Executor;
use std::{convert::TryInto, net::SocketAddr, sync::Arc};
use storage_client::StorageClient;

pub fn extract_execution_prikey(config: &NodeConfig) -> Option<Ed25519PrivateKey> {
    let backend = &config.execution.backend;
    let mut storage: Storage = backend.try_into().expect("Unable to initialize storage");
    if let Some(test_config) = config.test.as_ref() {
        let private_key = test_config
            .execution_key
            .as_ref()
            .expect("Missing execution key in test config")
            .private_key();

        storage
            .import_private_key(EXECUTION_KEY, private_key)
            .expect("Unable to insert execution key");
    }
    if config.execution.sign_vote_proposal {
        Some(
            storage
                .export_private_key(EXECUTION_KEY)
                .expect("Missing execution_private_key in secure storage"),
        )
    } else {
        None
    }
}

enum ExecutionCorrectnessWrapper {
    Local(Arc<Mutex<LocalService>>),
    Process(ProcessService),
    Serializer(Arc<Mutex<SerializerService>>),
    Thread(ThreadService),
}

pub struct ExecutionCorrectnessManager {
    internal_execution_correctness: ExecutionCorrectnessWrapper,
}

impl ExecutionCorrectnessManager {
    pub fn new(config: &NodeConfig) -> Self {
        if let ExecutionCorrectnessService::Process(remote_service) = &config.execution.service {
            return Self::new_process(
                remote_service.server_address,
                config.execution.network_timeout_ms,
            );
        }

        let execution_prikey = extract_execution_prikey(config);
        let storage_address = config.storage.address;
        let timeout_ms = config.storage.timeout_ms;
        match &config.execution.service {
            ExecutionCorrectnessService::Local => {
                Self::new_local(storage_address, execution_prikey, timeout_ms)
            }
            ExecutionCorrectnessService::Serializer => {
                Self::new_serializer(storage_address, execution_prikey, timeout_ms)
            }
            ExecutionCorrectnessService::Thread => {
                Self::new_thread(storage_address, execution_prikey, timeout_ms)
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
        timeout: u64,
    ) -> Self {
        let block_executor = Box::new(Executor::<DiemVM>::new(
            StorageClient::new(&storage_address, timeout).into(),
        ));
        Self {
            internal_execution_correctness: ExecutionCorrectnessWrapper::Local(Arc::new(
                Mutex::new(LocalService::new(block_executor, execution_prikey)),
            )),
        }
    }

    pub fn new_process(server_addr: SocketAddr, network_timeout: u64) -> Self {
        let process_service = ProcessService::new(server_addr, network_timeout);
        Self {
            internal_execution_correctness: ExecutionCorrectnessWrapper::Process(process_service),
        }
    }

    pub fn new_serializer(
        storage_address: SocketAddr,
        execution_prikey: Option<Ed25519PrivateKey>,
        timeout: u64,
    ) -> Self {
        let block_executor = Box::new(Executor::<DiemVM>::new(
            StorageClient::new(&storage_address, timeout).into(),
        ));
        let serializer_service = SerializerService::new(block_executor, execution_prikey);
        Self {
            internal_execution_correctness: ExecutionCorrectnessWrapper::Serializer(Arc::new(
                Mutex::new(serializer_service),
            )),
        }
    }

    pub fn new_thread(
        storage_address: SocketAddr,
        execution_prikey: Option<Ed25519PrivateKey>,
        network_timeout: u64,
    ) -> Self {
        let thread = ThreadService::new(storage_address, execution_prikey, network_timeout);
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
            ExecutionCorrectnessWrapper::Thread(thread) => Box::new(thread.client()),
        }
    }
}

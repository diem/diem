// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::serializer::{
    ExecutionCorrectnessInput, SerializerClient, SerializerService, TSerializerClient,
};
use diem_crypto::ed25519::Ed25519PrivateKey;
use diem_logger::warn;
use diem_secure_net::{NetworkClient, NetworkServer};
use diem_vm::DiemVM;
use executor::Executor;
use executor_types::Error;
use std::net::SocketAddr;
use storage_client::StorageClient;

pub trait RemoteService {
    fn client(&self) -> SerializerClient {
        let network_client =
            NetworkClient::new("execution", self.server_address(), self.network_timeout());
        let service = Box::new(RemoteClient::new(network_client));
        SerializerClient::new_client(service)
    }

    fn server_address(&self) -> SocketAddr;
    fn network_timeout(&self) -> u64;
}

pub fn execute(
    storage_addr: SocketAddr,
    listen_addr: SocketAddr,
    prikey: Option<Ed25519PrivateKey>,
    network_timeout: u64,
) {
    let block_executor = Box::new(Executor::<DiemVM>::new(
        StorageClient::new(&storage_addr, network_timeout).into(),
    ));
    let mut serializer_service = SerializerService::new(block_executor, prikey);
    let mut network_server = NetworkServer::new("execution", listen_addr, network_timeout);

    loop {
        if let Err(e) = process_one_message(&mut network_server, &mut serializer_service) {
            warn!("Warning: Failed to process message: {}", e);
        }
    }
}

fn process_one_message(
    network_server: &mut NetworkServer,
    serializer_service: &mut SerializerService,
) -> Result<(), Error> {
    let request = network_server.read()?;
    let response = serializer_service.handle_message(request)?;
    network_server.write(&response)?;
    Ok(())
}

struct RemoteClient {
    network_client: NetworkClient,
}

impl RemoteClient {
    pub fn new(network_client: NetworkClient) -> Self {
        Self { network_client }
    }

    fn process_one_message(&mut self, input: &[u8]) -> Result<Vec<u8>, Error> {
        self.network_client.write(&input)?;
        self.network_client.read().map_err(|e| e.into())
    }
}

impl TSerializerClient for RemoteClient {
    fn request(&mut self, input: ExecutionCorrectnessInput) -> Result<Vec<u8>, Error> {
        let input_message = bcs::to_bytes(&input)?;
        loop {
            match self.process_one_message(&input_message) {
                Err(err) => warn!("Failed to communicate with LEC service: {}", err),
                Ok(value) => return Ok(value),
            }
        }
    }
}

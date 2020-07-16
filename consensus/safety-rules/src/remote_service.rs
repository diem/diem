// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    persistent_safety_storage::PersistentSafetyStorage,
    serializer::{SafetyRulesInput, SerializerClient, SerializerService, TSerializerClient},
    Error, SafetyRules,
};
use libra_logger::warn;
use libra_secure_net::{NetworkClient, NetworkServer, TIMEOUT};
use std::net::SocketAddr;

pub trait RemoteService {
    fn client(&self) -> SerializerClient {
        let network_client = NetworkClient::new(self.server_address(), TIMEOUT);
        let service = Box::new(RemoteClient::new(network_client));
        SerializerClient::new_client(service)
    }

    fn server_address(&self) -> SocketAddr;
}

pub fn execute(
    storage: PersistentSafetyStorage,
    listen_addr: SocketAddr,
    verify_vote_proposal_signature: bool,
) {
    let safety_rules = SafetyRules::new(storage, verify_vote_proposal_signature);
    let mut serializer_service = SerializerService::new(safety_rules);
    let mut network_server = NetworkServer::new(listen_addr, TIMEOUT);

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
    fn request(&mut self, input: SafetyRulesInput) -> Result<Vec<u8>, Error> {
        let input_message = lcs::to_bytes(&input)?;
        loop {
            match self.process_one_message(&input_message) {
                Err(err) => warn!("Failed to communicate with SafetyRules service: {}", err),
                Ok(value) => return Ok(value),
            }
        }
    }
}

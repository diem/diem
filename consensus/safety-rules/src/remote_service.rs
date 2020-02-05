// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    persistent_safety_storage::PersistentSafetyStorage,
    serializer::{SafetyRulesInput, SerializerClient, SerializerService, TSerializerClient},
    Error, SafetyRules,
};
use consensus_types::common::{Author, Payload};
use libra_secure_net::{NetworkClient, NetworkServer};
use std::{marker::PhantomData, net::SocketAddr};

pub trait RemoteService<T: Payload> {
    fn client(&self) -> SerializerClient<T> {
        let network_client = NetworkClient::connect(self.server_address()).unwrap();
        let service = Box::new(RemoteClient::new(network_client));
        SerializerClient::new_client(service)
    }

    fn server_address(&self) -> SocketAddr;
}

pub fn execute<T: Payload>(
    author: Author,
    storage: PersistentSafetyStorage,
    listen_addr: SocketAddr,
) {
    let safety_rules = SafetyRules::<T>::new(author, storage);
    let mut serializer_service = SerializerService::new(safety_rules);
    let mut network_server = NetworkServer::new(listen_addr);

    loop {
        if let Err(e) = process_one_message(&mut network_server, &mut serializer_service) {
            eprintln!("Warning: Failed to process message: {}", e);
        }
    }
}

fn process_one_message<T: Payload>(
    network_server: &mut NetworkServer,
    serializer_service: &mut SerializerService<T>,
) -> Result<(), Error> {
    let request = network_server.read()?;
    let response = serializer_service.handle_message(request)?;
    network_server.write(&response)?;
    Ok(())
}

struct RemoteClient<T> {
    network_client: NetworkClient,
    marker: PhantomData<T>,
}

impl<T> RemoteClient<T> {
    pub fn new(network_client: NetworkClient) -> Self {
        Self {
            network_client,
            marker: PhantomData,
        }
    }
}

impl<T: Payload> TSerializerClient<T> for RemoteClient<T> {
    fn request(&mut self, input: SafetyRulesInput<T>) -> Result<Vec<u8>, Error> {
        let input_message = lcs::to_bytes(&input)?;
        self.network_client.write(&input_message)?;
        let result = self.network_client.read()?;
        Ok(result)
    }
}

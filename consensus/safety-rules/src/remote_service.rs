// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    persistent_storage::PersistentStorage,
    serializer::{SafetyRulesInput, SerializerClient, SerializerService, TSerializerClient},
    Error, SafetyRules,
};
use consensus_types::common::Payload;
use libra_secure_net::{NetworkClient, NetworkServer};
use libra_types::crypto_proxies::ValidatorSigner;
use std::{marker::PhantomData, net::SocketAddr, sync::Arc};

pub trait RemoteService<T: Payload> {
    fn client(&self) -> SerializerClient<T> {
        let network_client = NetworkClient::connect(self.server_address()).unwrap();
        let service = Box::new(RemoteClient::new(network_client));
        SerializerClient::new_client(service)
    }

    fn server_address(&self) -> SocketAddr;
}

pub fn execute<T: Payload>(
    storage: Box<dyn PersistentStorage>,
    validator_signer: ValidatorSigner,
    listen_addr: SocketAddr,
) {
    let safety_rules = SafetyRules::<T>::new(storage, Arc::new(validator_signer));
    let mut serializer_service = SerializerService::new(safety_rules);
    let mut network_server = NetworkServer::new(listen_addr);

    loop {
        let request = network_server.read().unwrap();
        let response = serializer_service.handle_message(request).unwrap();
        network_server.write(&response).unwrap();
    }
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

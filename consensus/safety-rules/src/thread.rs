// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

//! This provides a execution separation between SafetyRules and Consensus without requiring the
//! use of processes. Rust does not support fork and so the mechanics to actually construct a
//! SafetyRules that would run together and be started by Consensus requires a separate binary and
//! making a call to start that via a command. This is a lightweight means of accomplishing a goal
//! in testing correctness of the communication layer between Consensus and SafetyRules.

use crate::{
    network::{NetworkClient, NetworkServer},
    persistent_storage::PersistentStorage,
    remote::{RemoteClient, RemoteService, SafetyRulesInput, TRemoteClient},
    Error, SafetyRules,
};
use consensus_types::common::Payload;
use libra_config::utils;
use libra_types::crypto_proxies::ValidatorSigner;
use std::{
    marker::PhantomData,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
    thread::{self, JoinHandle},
};

/// ThreadClient is the actual owner of the thread but in the context of Consenus and SafetyRules
/// is on the client side of the operations as it makes queries / requests to SafetyRules.
pub struct ThreadClient<T> {
    child: JoinHandle<()>,
    server_addr: SocketAddr,
    marker: PhantomData<T>,
}

impl<T: Payload> ThreadClient<T> {
    pub fn new(storage: Box<dyn PersistentStorage>, validator_signer: ValidatorSigner) -> Self {
        let listen_port = utils::get_available_port();
        let listen_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), listen_port);
        let server_addr = listen_addr;

        let child = thread::spawn(move || Self::execute(storage, validator_signer, listen_addr));

        Self {
            child,
            server_addr,
            marker: PhantomData,
        }
    }

    pub fn client(&self) -> RemoteClient<T> {
        let network_client = NetworkClient::connect(self.server_addr).unwrap();
        let service = Box::new(ThreadService::new(network_client));
        RemoteClient::new_client(service)
    }

    fn execute(
        storage: Box<dyn PersistentStorage>,
        validator_signer: ValidatorSigner,
        listen_addr: SocketAddr,
    ) {
        let safety_rules = SafetyRules::<T>::new(storage, Arc::new(validator_signer));
        let mut remote_service = RemoteService::new(safety_rules);
        let mut network_server = NetworkServer::new(listen_addr);

        loop {
            let request = network_server.read().unwrap();
            let response = remote_service.handle_message(request).unwrap();
            network_server.write(&response).unwrap();
        }
    }
}

/// ThreadService hosts SafetyRules
struct ThreadService<T> {
    network_client: NetworkClient,
    marker: PhantomData<T>,
}

impl<T> ThreadService<T> {
    pub fn new(network_client: NetworkClient) -> Self {
        Self {
            network_client,
            marker: PhantomData,
        }
    }
}

impl<T: Payload> TRemoteClient<T> for ThreadService<T> {
    fn request(&mut self, input: SafetyRulesInput<T>) -> Result<Vec<u8>, Error> {
        let input_message = lcs::to_bytes(&input)?;
        self.network_client.write(&input_message)?;
        let result = self.network_client.read()?;
        Ok(result)
    }
}

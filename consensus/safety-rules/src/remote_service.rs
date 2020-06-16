// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    persistent_safety_storage::PersistentSafetyStorage, serializer::SerializerService, Error,
    SafetyRules,
};
use consensus_types::common::Author;
use libra_logger::warn;
use libra_secure_net::NetworkServer;
use std::net::SocketAddr;

pub trait RemoteService {
    fn server_address(&self) -> SocketAddr;
}

pub fn execute(author: Author, storage: PersistentSafetyStorage, listen_addr: SocketAddr) {
    let safety_rules = SafetyRules::new(author, storage);
    let mut serializer_service = SerializerService::new(safety_rules);
    let mut network_server = NetworkServer::new(listen_addr);

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

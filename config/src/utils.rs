// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::config::NodeConfig;
use diem_types::{
    account_address::AccountAddress,
    network_address::{NetworkAddress, Protocol},
    transaction::{authenticator::AuthenticationKey, Transaction},
};
use get_if_addrs::get_if_addrs;
use std::net::{TcpListener, TcpStream};

pub fn default_validator_owner_auth_key_from_name(name: &[u8]) -> AuthenticationKey {
    let salt = "validator_owner::";
    let mut name_in_bytes = salt.as_bytes().to_vec();
    name_in_bytes.extend_from_slice(name);
    let hash = diem_crypto::HashValue::sha3_256_of(&name_in_bytes);
    AuthenticationKey::new(*hash.as_ref())
}

pub fn validator_owner_account_from_name(name: &[u8]) -> AccountAddress {
    default_validator_owner_auth_key_from_name(name).derived_address()
}

/// Return an ephemeral, available port. On unix systems, the port returned will be in the
/// TIME_WAIT state ensuring that the OS won't hand out this port for some grace period.
/// Callers should be able to bind to this port given they use SO_REUSEADDR.
pub fn get_available_port() -> u16 {
    const MAX_PORT_RETRIES: u32 = 1000;

    for _ in 0..MAX_PORT_RETRIES {
        if let Ok(port) = get_ephemeral_port() {
            return port;
        }
    }

    panic!("Error: could not find an available port");
}

fn get_ephemeral_port() -> ::std::io::Result<u16> {
    // Request a random available port from the OS
    let listener = TcpListener::bind(("localhost", 0))?;
    let addr = listener.local_addr()?;

    // Create and accept a connection (which we'll promptly drop) in order to force the port
    // into the TIME_WAIT state, ensuring that the port will be reserved from some limited
    // amount of time (roughly 60s on some Linux systems)
    let _sender = TcpStream::connect(addr)?;
    let _incoming = listener.accept()?;

    Ok(addr.port())
}

/// Extracts one local non-loopback IP address, if one exists. Otherwise returns None.
pub fn get_local_ip() -> Option<NetworkAddress> {
    get_if_addrs().ok().and_then(|if_addrs| {
        if_addrs
            .iter()
            .find(|if_addr| !if_addr.is_loopback())
            .map(|if_addr| NetworkAddress::from(Protocol::from(if_addr.ip())))
    })
}

pub fn get_available_port_in_multiaddr(is_ipv4: bool) -> NetworkAddress {
    let ip_proto = if is_ipv4 {
        Protocol::Ip4("0.0.0.0".parse().unwrap())
    } else {
        Protocol::Ip6("::1".parse().unwrap())
    };
    NetworkAddress::from(ip_proto).push(Protocol::Tcp(get_available_port()))
}

pub fn get_genesis_txn(config: &NodeConfig) -> Option<&Transaction> {
    config.execution.genesis.as_ref()
}

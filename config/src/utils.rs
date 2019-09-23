// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use get_if_addrs::get_if_addrs;
use parity_multiaddr::{Multiaddr, Protocol};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    collections::HashSet,
    hash::BuildHasher,
    net::{IpAddr, TcpListener, TcpStream},
};
use types::transaction::SCRIPT_HASH_LENGTH;

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
pub fn get_local_ip() -> Option<Multiaddr> {
    get_if_addrs().ok().and_then(|if_addrs| {
        if_addrs
            .into_iter()
            .find(|if_addr| !if_addr.is_loopback())
            .and_then(|if_addr| {
                let mut addr = Multiaddr::empty();
                match if_addr.ip() {
                    IpAddr::V4(a) => {
                        addr.push(Protocol::Ip4(a));
                    }
                    IpAddr::V6(a) => {
                        addr.push(Protocol::Ip6(a));
                    }
                }
                Some(addr)
            })
    })
}

pub fn deserialize_whitelist<'de, D>(
    deserializer: D,
) -> ::std::result::Result<HashSet<[u8; SCRIPT_HASH_LENGTH]>, D::Error>
where
    D: Deserializer<'de>,
{
    let whitelisted_scripts: Vec<String> = Deserialize::deserialize(deserializer)?;
    let whitelist = whitelisted_scripts
        .iter()
        .map(|s| {
            let mut hash = [0u8; SCRIPT_HASH_LENGTH];
            let decoded_hash =
                hex::decode(s).expect("Unable to decode script hash from configuration file.");
            assert_eq!(decoded_hash.len(), SCRIPT_HASH_LENGTH);
            hash.copy_from_slice(decoded_hash.as_slice());
            hash
        })
        .collect();
    Ok(whitelist)
}

pub fn serialize_whitelist<S, H>(
    whitelist: &HashSet<[u8; SCRIPT_HASH_LENGTH], H>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    H: BuildHasher,
{
    let encoded_whitelist: Vec<String> = whitelist.iter().map(hex::encode).collect();
    encoded_whitelist.serialize(serializer)
}

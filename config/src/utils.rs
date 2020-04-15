// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::config::NodeConfig;
use get_if_addrs::get_if_addrs;
use libra_types::{transaction::Transaction, PeerId};
use parity_multiaddr::{Multiaddr, Protocol};
use serde::{Serialize, Serializer};
use std::{
    collections::{BTreeMap, HashMap},
    hash::BuildHasher,
    net::{IpAddr, TcpListener, TcpStream},
};

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

pub fn get_available_port_in_multiaddr(is_ipv4: bool) -> Multiaddr {
    let mut addr = Multiaddr::empty();
    if is_ipv4 {
        addr.push(Protocol::Ip4("0.0.0.0".parse().unwrap()));
    } else {
        addr.push(Protocol::Ip6("::1".parse().unwrap()));
    }
    addr.push(Protocol::Tcp(get_available_port()));
    addr
}

/// Serialize HashMaps as BTreeMaps for consistent ordering
pub fn serialize_ordered_map<S, V, H>(
    value: &HashMap<PeerId, V, H>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    H: BuildHasher,
    V: Serialize,
{
    let ordered: BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
}

pub fn get_genesis_txn(config: &NodeConfig) -> anyhow::Result<&Transaction> {
    config
        .execution
        .genesis
        .as_ref()
        .ok_or_else(|| anyhow::format_err!("Genesis txn not present."))
}

// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    common::NetworkPublicKeys,
    protocols::{
        identity::{exchange_handshake, exchange_peerid},
        wire::handshake::v1::{HandshakeMsg, MessagingProtocolVersion, SupportedProtocols},
    },
};
use futures::io::{AsyncRead, AsyncWrite};
use libra_crypto::{traits::ValidCryptoMaterial, x25519};
use libra_logger::prelude::*;
use libra_network_address::NetworkAddress;
use libra_security_logger::{security_log, SecurityEvent};
use libra_types::PeerId;
use netcore::transport::{boxed, memory, tcp, ConnectionOrigin, TransportExt};
use noise::NoiseConfig;
use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    convert::TryFrom,
    fmt::Debug,
    io,
    sync::{Arc, Mutex, RwLock},
    time::Duration,
};

/// A timeout for the connection to open and complete all of the upgrade steps.
pub const TRANSPORT_TIMEOUT: Duration = Duration::from_secs(30);
/// Currently supported messaging protocol version.
/// TODO: Add ability to support more than one messaging protocol.
pub const SUPPORTED_MESSAGING_PROTOCOL: MessagingProtocolVersion = MessagingProtocolVersion::V1;
/// Global connection-id generator.
static CONNECTION_ID_GENERATOR: Lazy<Arc<Mutex<ConnectionIdGenerator>>> =
    Lazy::new(|| Arc::new(Mutex::new(ConnectionIdGenerator::new())));

const LIBRA_TCP_TRANSPORT: tcp::TcpTransport = tcp::TcpTransport {
    // Use default options.
    recv_buffer_size: None,
    send_buffer_size: None,
    ttl: None,
    keepalive: None,
    // Use TCP_NODELAY for libra tcp connections.
    nodelay: Some(true),
};

pub trait TSocket: AsyncRead + AsyncWrite + Send + Debug + Unpin + 'static {}
impl<T> TSocket for T where T: AsyncRead + AsyncWrite + Send + Debug + Unpin + 'static {}

/// Unique local identifier for a connection.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct ConnectionId(u32);

impl From<u32> for ConnectionId {
    fn from(i: u32) -> ConnectionId {
        ConnectionId(i)
    }
}

/// Generator of unique ConnectionIds.
struct ConnectionIdGenerator {
    next: ConnectionId,
}

impl ConnectionIdGenerator {
    fn new() -> ConnectionIdGenerator {
        Self {
            next: ConnectionId(0),
        }
    }

    fn next(&mut self) -> ConnectionId {
        let ret = self.next;
        self.next = ConnectionId(ret.0.wrapping_add(1));
        ret
    }
}

/// Metadata associated with an established connection.
#[derive(Clone, Debug)]
pub struct ConnectionMetadata {
    peer_id: PeerId,
    connection_id: ConnectionId,
    addr: NetworkAddress,
    origin: ConnectionOrigin,
    messaging_protocol: MessagingProtocolVersion,
    application_protocols: SupportedProtocols,
}

impl ConnectionMetadata {
    pub fn new(
        peer_id: PeerId,
        connection_id: ConnectionId,
        addr: NetworkAddress,
        origin: ConnectionOrigin,
        messaging_protocol: MessagingProtocolVersion,
        application_protocols: SupportedProtocols,
    ) -> ConnectionMetadata {
        ConnectionMetadata {
            peer_id,
            connection_id,
            addr,
            origin,
            messaging_protocol,
            application_protocols,
        }
    }

    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    pub fn connection_id(&self) -> ConnectionId {
        self.connection_id
    }

    pub fn addr(&self) -> &NetworkAddress {
        &self.addr
    }

    pub fn origin(&self) -> ConnectionOrigin {
        self.origin
    }
}

/// The `Connection` struct consists of connection metadata and the actual socket for
/// communication.
#[derive(Debug)]
pub struct Connection<TSocket> {
    pub socket: TSocket,
    pub metadata: ConnectionMetadata,
}

fn identity_key_to_peer_id(
    trusted_peers: &RwLock<HashMap<PeerId, NetworkPublicKeys>>,
    remote_static_key: &[u8],
) -> Option<PeerId> {
    for (peer_id, public_keys) in trusted_peers.read().unwrap().iter() {
        if public_keys.identity_public_key.to_bytes() == remote_static_key {
            return Some(*peer_id);
        }
    }

    None
}

/// temporary checks to make sure noise pubkeys are actually getting propagated correctly.
// TODO(philiphayes): remove this after Transport refactor
#[allow(dead_code)]
fn expect_noise_pubkey(
    addr: &NetworkAddress,
    remote_static_key: &[u8],
    origin: ConnectionOrigin,
) -> Result<(), io::Error> {
    if let ConnectionOrigin::Outbound = origin {
        let expected_remote_static_key = addr.find_noise_proto().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid NetworkAddress, no NoiseIk protocol: '{}'", addr),
            )
        })?;

        if remote_static_key != expected_remote_static_key.as_slice() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "remote static pubkey did not match expected: actual: {:?}, expected: {:?}",
                    remote_static_key, expected_remote_static_key,
                ),
            ));
        }
    }

    Ok(())
}

pub async fn perform_handshake<T: TSocket>(
    peer_id: PeerId,
    mut socket: T,
    addr: NetworkAddress,
    origin: ConnectionOrigin,
    own_handshake: &HandshakeMsg,
) -> Result<Connection<T>, io::Error> {
    let handshake_other = exchange_handshake(&own_handshake, &mut socket).await?;
    let intersecting_protocols = own_handshake.find_common_protocols(&handshake_other);
    match intersecting_protocols {
        None => {
            info!("No matching protocols found for connection with peer: {:?}. Handshake received: {:?}",
            peer_id.short_str(), handshake_other);
            Err(io::Error::new(
                io::ErrorKind::Other,
                "no matching messaging protocol",
            ))
        }
        Some((messaging_protocol, application_protocols)) => Ok(Connection {
            socket,
            metadata: ConnectionMetadata::new(
                peer_id,
                CONNECTION_ID_GENERATOR.lock().unwrap().next(),
                addr,
                origin,
                messaging_protocol,
                application_protocols,
            ),
        }),
    }
}

pub fn build_memory_noise_transport(
    identity_key: x25519::PrivateKey,
    trusted_peers: Arc<RwLock<HashMap<PeerId, NetworkPublicKeys>>>,
    application_protocols: SupportedProtocols,
) -> boxed::BoxedTransport<Connection<impl TSocket>, impl ::std::error::Error> {
    let memory_transport = memory::MemoryTransport::default();
    let noise_config = Arc::new(NoiseConfig::new(identity_key));
    let mut own_handshake = HandshakeMsg::new();
    own_handshake.add(SUPPORTED_MESSAGING_PROTOCOL, application_protocols);

    memory_transport
        .and_then(move |socket, _addr, origin| async move {
            let (remote_static_key, socket) =
                noise_config.upgrade_connection(socket, origin).await?;

            // TODO(philiphayes): reenable after seed peers are always fully rendered
            // expect_noise_pubkey(&addr, remote_static_key.as_slice(), origin)?;

            if let Some(peer_id) = identity_key_to_peer_id(&trusted_peers, &remote_static_key) {
                Ok((peer_id, socket))
            } else {
                Err(io::Error::new(io::ErrorKind::Other, "Not a trusted peer"))
            }
        })
        .and_then(move |(peer_id, socket), addr, origin| async move {
            perform_handshake(peer_id, socket, addr, origin, &own_handshake).await
        })
        .with_timeout(TRANSPORT_TIMEOUT)
        .boxed()
}

pub fn build_unauthenticated_memory_noise_transport(
    identity_key: x25519::PrivateKey,
    application_protocols: SupportedProtocols,
) -> boxed::BoxedTransport<Connection<impl TSocket>, impl ::std::error::Error> {
    let memory_transport = memory::MemoryTransport::default();
    let noise_config = Arc::new(NoiseConfig::new(identity_key));
    let mut own_handshake = HandshakeMsg::new();
    own_handshake.add(SUPPORTED_MESSAGING_PROTOCOL, application_protocols);

    memory_transport
        .and_then(move |socket, _addr, origin| {
            async move {
                let (remote_static_key, socket) =
                    noise_config.upgrade_connection(socket, origin).await?;

                // TODO(philiphayes): reenable after seed peers are always fully rendered
                // expect_noise_pubkey(&addr, remote_static_key.as_slice(), origin)?;

                // Generate PeerId from x25519::PublicKey.
                // Note: This is inconsistent with current types because AccountAddress is derived
                // from consensus key which is of type Ed25519PublicKey. Since AccountAddress does
                // not mean anything in a setting without remote authentication, we use the network
                // public key to generate a peer_id for the peer. The only reason this works is
                // that both are 32 bytes in size. If/when this condition no longer holds, we will
                // receive an error.
                let peer_id = PeerId::try_from(remote_static_key).unwrap();
                Ok((peer_id, socket))
            }
        })
        .and_then(move |(peer_id, socket), addr, origin| async move {
            perform_handshake(peer_id, socket, addr, origin, &own_handshake).await
        })
        .with_timeout(TRANSPORT_TIMEOUT)
        .boxed()
}

pub fn build_memory_transport(
    own_peer_id: PeerId,
    application_protocols: SupportedProtocols,
) -> boxed::BoxedTransport<Connection<impl TSocket>, impl ::std::error::Error> {
    let memory_transport = memory::MemoryTransport::default();
    let mut own_handshake = HandshakeMsg::new();
    own_handshake.add(SUPPORTED_MESSAGING_PROTOCOL, application_protocols);

    memory_transport
        .and_then(move |mut socket, _addr, _origin| async move {
            Ok((exchange_peerid(&own_peer_id, &mut socket).await?, socket))
        })
        .and_then(move |(peer_id, socket), addr, origin| async move {
            perform_handshake(peer_id, socket, addr, origin, &own_handshake).await
        })
        .with_timeout(TRANSPORT_TIMEOUT)
        .boxed()
}

//TODO(bmwill) Maybe create an Either Transport so we can merge the building of Memory + Tcp
pub fn build_tcp_noise_transport(
    identity_key: x25519::PrivateKey,
    trusted_peers: Arc<RwLock<HashMap<PeerId, NetworkPublicKeys>>>,
    application_protocols: SupportedProtocols,
) -> boxed::BoxedTransport<Connection<impl TSocket>, impl ::std::error::Error> {
    let noise_config = Arc::new(NoiseConfig::new(identity_key));
    let mut own_handshake = HandshakeMsg::new();
    own_handshake.add(SUPPORTED_MESSAGING_PROTOCOL, application_protocols);

    LIBRA_TCP_TRANSPORT
        .and_then(move |socket, _addr, origin| async move {
            let (remote_static_key, socket) =
                noise_config.upgrade_connection(socket, origin).await?;

            // TODO(philiphayes): reenable after seed peers are always fully rendered
            // expect_noise_pubkey(&addr, remote_static_key.as_slice(), origin)?;

            if let Some(peer_id) = identity_key_to_peer_id(&trusted_peers, &remote_static_key) {
                Ok((peer_id, socket))
            } else {
                security_log(SecurityEvent::InvalidNetworkPeer)
                    .error("UntrustedPeer")
                    .data(&trusted_peers)
                    .data(&remote_static_key)
                    .log();
                Err(io::Error::new(io::ErrorKind::Other, "Not a trusted peer"))
            }
        })
        .and_then(move |(peer_id, socket), addr, origin| async move {
            perform_handshake(peer_id, socket, addr, origin, &own_handshake).await
        })
        .with_timeout(TRANSPORT_TIMEOUT)
        .boxed()
}

// Transport based on TCP + Noise, but without remote authentication (i.e., any
// node is allowed to connect).
pub fn build_unauthenticated_tcp_noise_transport(
    identity_key: x25519::PrivateKey,
    application_protocols: SupportedProtocols,
) -> boxed::BoxedTransport<Connection<impl TSocket>, impl ::std::error::Error> {
    let noise_config = Arc::new(NoiseConfig::new(identity_key));
    let mut own_handshake = HandshakeMsg::new();
    own_handshake.add(SUPPORTED_MESSAGING_PROTOCOL, application_protocols);

    LIBRA_TCP_TRANSPORT
        .and_then(move |socket, _addr, origin| {
            async move {
                let (remote_static_key, socket) =
                    noise_config.upgrade_connection(socket, origin).await?;

                // TODO(philiphayes): reenable after seed peers are always fully rendered
                // expect_noise_pubkey(&addr, remote_static_key.as_slice(), origin)?;

                // Generate PeerId from x25519::PublicKey.
                // Note: This is inconsistent with current types because AccountAddress is derived
                // from consensus key which is of type Ed25519PublicKey. Since AccountAddress does
                // not mean anything in a setting without remote authentication, we use the network
                // public key to generate a peer_id for the peer. The only reason this works is that
                // both are 32 bytes in size. If/when this condition no longer holds, we will receive
                // an error.
                let peer_id = PeerId::try_from(remote_static_key).unwrap();
                Ok((peer_id, socket))
            }
        })
        .and_then(move |(peer_id, socket), addr, origin| async move {
            perform_handshake(peer_id, socket, addr, origin, &own_handshake).await
        })
        .with_timeout(TRANSPORT_TIMEOUT)
        .boxed()
}

pub fn build_tcp_transport(
    own_peer_id: PeerId,
    application_protocols: SupportedProtocols,
) -> boxed::BoxedTransport<Connection<impl TSocket>, impl ::std::error::Error> {
    let mut own_handshake = HandshakeMsg::new();
    own_handshake.add(SUPPORTED_MESSAGING_PROTOCOL, application_protocols);

    LIBRA_TCP_TRANSPORT
        .and_then(move |mut socket, _addr, _origin| async move {
            Ok((exchange_peerid(&own_peer_id, &mut socket).await?, socket))
        })
        .and_then(move |(peer_id, socket), addr, origin| async move {
            perform_handshake(peer_id, socket, addr, origin, &own_handshake).await
        })
        .with_timeout(TRANSPORT_TIMEOUT)
        .boxed()
}

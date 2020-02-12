// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    common::NetworkPublicKeys,
    protocols::identity::{exchange_identity, Identity},
};
use libra_crypto::{
    x25519::{X25519StaticPrivateKey, X25519StaticPublicKey},
    ValidKey,
};
use libra_security_logger::{security_log, SecurityEvent};
use libra_types::PeerId;
use netcore::{
    multiplexing::{yamux::Yamux, StreamMultiplexer},
    transport::{boxed, memory, tcp, TransportExt},
};
use noise::NoiseConfig;
use std::{
    collections::HashMap,
    convert::TryFrom,
    io,
    sync::{Arc, RwLock},
    time::Duration,
};

/// A timeout for the connection to open and complete all of the upgrade steps.
pub const TRANSPORT_TIMEOUT: Duration = Duration::from_secs(30);

const LIBRA_TCP_TRANSPORT: tcp::TcpTransport = tcp::TcpTransport {
    // Use default options.
    recv_buffer_size: None,
    send_buffer_size: None,
    ttl: None,
    keepalive: None,
    // Use TCP_NODELAY for libra tcp connections.
    nodelay: Some(true),
};

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

// Ensures that peer id in received identity is same as peer id derived from noise handshake.
fn match_peer_id(identity: Identity, peer_id: PeerId) -> Result<Identity, io::Error> {
    if identity.peer_id() != peer_id {
        security_log(SecurityEvent::InvalidNetworkPeer)
            .error("InvalidIdentity")
            .data(&identity)
            .data(&peer_id)
            .log();
        Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "PeerId received from Noise Handshake ({}) doesn't match one received from Identity Exchange ({})",
                    peer_id.short_str(),
                    identity.peer_id().short_str()
                )
        ))
    } else {
        Ok(identity)
    }
}

pub fn build_memory_noise_transport(
    own_identity: Identity,
    identity_keypair: (X25519StaticPrivateKey, X25519StaticPublicKey),
    trusted_peers: Arc<RwLock<HashMap<PeerId, NetworkPublicKeys>>>,
) -> boxed::BoxedTransport<(Identity, impl StreamMultiplexer), impl ::std::error::Error> {
    let memory_transport = memory::MemoryTransport::default();
    let noise_config = Arc::new(NoiseConfig::new(identity_keypair));

    memory_transport
        .and_then(move |socket, origin| async move {
            let (remote_static_key, socket) =
                noise_config.upgrade_connection(socket, origin).await?;
            if let Some(peer_id) = identity_key_to_peer_id(&trusted_peers, &remote_static_key) {
                Ok((peer_id, socket))
            } else {
                Err(io::Error::new(io::ErrorKind::Other, "Not a trusted peer"))
            }
        })
        .and_then(move |(peer_id, socket), origin| async move {
            let (identity, socket) = exchange_identity(&own_identity, socket, origin).await?;
            match_peer_id(identity, peer_id).and_then(|identity| Ok((identity, socket)))
        })
        .and_then(|(peer_id, socket), origin| async move {
            let muxer = Yamux::upgrade_connection(socket, origin).await?;
            Ok((peer_id, muxer))
        })
        .with_timeout(TRANSPORT_TIMEOUT)
        .boxed()
}

pub fn build_unauthenticated_memory_noise_transport(
    own_identity: Identity,
    identity_keypair: (X25519StaticPrivateKey, X25519StaticPublicKey),
) -> boxed::BoxedTransport<(Identity, impl StreamMultiplexer), impl ::std::error::Error> {
    let memory_transport = memory::MemoryTransport::default();
    let noise_config = Arc::new(NoiseConfig::new(identity_keypair));
    memory_transport
        .and_then(move |socket, origin| {
            async move {
                let (remote_static_key, socket) =
                    noise_config.upgrade_connection(socket, origin).await?;
                // Generate PeerId from X25519StaticPublicKey.
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
        .and_then(move |(peer_id, socket), origin| async move {
            let (identity, socket) = exchange_identity(&own_identity, socket, origin).await?;
            match_peer_id(identity, peer_id).and_then(|identity| Ok((identity, socket)))
        })
        .and_then(|(peer_id, socket), origin| async move {
            let muxer = Yamux::upgrade_connection(socket, origin).await?;
            Ok((peer_id, muxer))
        })
        .with_timeout(TRANSPORT_TIMEOUT)
        .boxed()
}

pub fn build_memory_transport(
    own_identity: Identity,
) -> boxed::BoxedTransport<(Identity, impl StreamMultiplexer), impl ::std::error::Error> {
    let memory_transport = memory::MemoryTransport::default();
    memory_transport
        .and_then(move |socket, origin| async move {
            Ok(exchange_identity(&own_identity, socket, origin).await?)
        })
        .and_then(|(identity, socket), origin| async move {
            let muxer = Yamux::upgrade_connection(socket, origin).await?;
            Ok((identity, muxer))
        })
        .with_timeout(TRANSPORT_TIMEOUT)
        .boxed()
}

//TODO(bmwill) Maybe create an Either Transport so we can merge the building of Memory + Tcp
pub fn build_tcp_noise_transport(
    own_identity: Identity,
    identity_keypair: (X25519StaticPrivateKey, X25519StaticPublicKey),
    trusted_peers: Arc<RwLock<HashMap<PeerId, NetworkPublicKeys>>>,
) -> boxed::BoxedTransport<(Identity, impl StreamMultiplexer), impl ::std::error::Error> {
    let noise_config = Arc::new(NoiseConfig::new(identity_keypair));

    LIBRA_TCP_TRANSPORT
        .and_then(move |socket, origin| async move {
            let (remote_static_key, socket) =
                noise_config.upgrade_connection(socket, origin).await?;
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
        .and_then(move |(peer_id, socket), origin| async move {
            let (identity, socket) = exchange_identity(&own_identity, socket, origin).await?;
            match_peer_id(identity, peer_id).and_then(|identity| Ok((identity, socket)))
        })
        .and_then(|(peer_id, socket), origin| async move {
            let muxer = Yamux::upgrade_connection(socket, origin).await?;
            Ok((peer_id, muxer))
        })
        .with_timeout(TRANSPORT_TIMEOUT)
        .boxed()
}

// Transport based on TCP + Noise, but without remote authentication (i.e., any
// node is allowed to connect).
pub fn build_unauthenticated_tcp_noise_transport(
    own_identity: Identity,
    identity_keypair: (X25519StaticPrivateKey, X25519StaticPublicKey),
) -> boxed::BoxedTransport<(Identity, impl StreamMultiplexer), impl ::std::error::Error> {
    let noise_config = Arc::new(NoiseConfig::new(identity_keypair));
    LIBRA_TCP_TRANSPORT
        .and_then(move |socket, origin| {
            async move {
                let (remote_static_key, socket) =
                    noise_config.upgrade_connection(socket, origin).await?;
                // Generate PeerId from X25519StaticPublicKey.
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
        .and_then(move |(peer_id, socket), origin| async move {
            let (identity, socket) = exchange_identity(&own_identity, socket, origin).await?;
            match_peer_id(identity, peer_id).and_then(|identity| Ok((identity, socket)))
        })
        .and_then(|(peer_id, socket), origin| async move {
            let muxer = Yamux::upgrade_connection(socket, origin).await?;
            Ok((peer_id, muxer))
        })
        .with_timeout(TRANSPORT_TIMEOUT)
        .boxed()
}

pub fn build_tcp_transport(
    own_identity: Identity,
) -> boxed::BoxedTransport<(Identity, impl StreamMultiplexer), impl ::std::error::Error> {
    LIBRA_TCP_TRANSPORT
        .and_then(move |socket, origin| async move {
            Ok(exchange_identity(&own_identity, socket, origin).await?)
        })
        .and_then(|(identity, socket), origin| async move {
            let muxer = Yamux::upgrade_connection(socket, origin).await?;
            Ok((identity, muxer))
        })
        .with_timeout(TRANSPORT_TIMEOUT)
        .boxed()
}

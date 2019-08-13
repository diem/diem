// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    common::NetworkPublicKeys,
    protocols::identity::{exchange_identity, Identity},
};
use logger::prelude::*;
use netcore::{
    multiplexing::{yamux::Yamux, StreamMultiplexer},
    transport::{boxed, memory, tcp, TransportExt},
};
use nextgen_crypto::{
    x25519::{X25519StaticPrivateKey, X25519StaticPublicKey},
    ValidKey,
};
use noise::NoiseConfig;
use std::{
    collections::HashMap,
    io,
    sync::{Arc, RwLock},
    time::Duration,
};
use types::PeerId;

/// A timeout for the connection to open and complete all of the upgrade steps.
const TRANSPORT_TIMEOUT: Duration = Duration::from_secs(30);

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

pub fn build_memory_noise_transport(
    own_identity: Identity,
    identity_keypair: (X25519StaticPrivateKey, X25519StaticPublicKey),
    trusted_peers: Arc<RwLock<HashMap<PeerId, NetworkPublicKeys>>>,
) -> boxed::BoxedTransport<(Identity, impl StreamMultiplexer), impl ::std::error::Error> {
    let memory_transport = memory::MemoryTransport::default();
    let noise_config = Arc::new(NoiseConfig::new(identity_keypair));

    memory_transport
        .and_then(move |socket, origin| {
            async move {
                let (remote_static_key, socket) =
                    noise_config.upgrade_connection(socket, origin).await?;

                if let Some(peer_id) = identity_key_to_peer_id(&trusted_peers, &remote_static_key) {
                    Ok((peer_id, socket))
                } else {
                    Err(io::Error::new(io::ErrorKind::Other, "Not a trusted peer"))
                }
            }
        })
        .and_then(|(peer_id, socket), origin| {
            async move {
                let muxer = Yamux::upgrade_connection(socket, origin).await?;
                Ok((peer_id, muxer))
            }
        })
        .and_then(move |(peer_id, muxer), origin| {
            async move {
                let (identity, muxer) = exchange_identity(&own_identity, muxer, origin).await?;

                if identity.peer_id() == peer_id {
                    Ok((identity, muxer))
                } else {
                    Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!(
                            "PeerId received from Noise Handshake ({}) doesn't match one received from Identity Exchange ({})",
                            peer_id.short_str(),
                            identity.peer_id().short_str()
                        )
                    ))
                }
            }
        })
        .with_timeout(TRANSPORT_TIMEOUT)
        .boxed()
}

pub fn build_memory_transport(
    own_identity: Identity,
) -> boxed::BoxedTransport<(Identity, impl StreamMultiplexer), impl ::std::error::Error> {
    let memory_transport = memory::MemoryTransport::default();

    memory_transport
        .and_then(|socket, origin| {
            async move {
                let muxer = Yamux::upgrade_connection(socket, origin).await?;
                Ok(muxer)
            }
        })
        .and_then(move |muxer, origin| {
            async move {
                let (identity, muxer) = exchange_identity(&own_identity, muxer, origin).await?;

                Ok((identity, muxer))
            }
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
    let tcp_transport = tcp::TcpTransport::default();
    let noise_config = Arc::new(NoiseConfig::new(identity_keypair));

    tcp_transport
        .and_then(move |socket, origin| {
            async move {
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
            }
        })
        .and_then(|(peer_id, socket), origin| {
            async move {
                let muxer = Yamux::upgrade_connection(socket, origin).await?;
                Ok((peer_id, muxer))
            }
        })
        .and_then(move |(peer_id, muxer), origin| {
            async move {
                let (identity, muxer) = exchange_identity(&own_identity, muxer, origin).await?;

                if identity.peer_id() == peer_id {
                    Ok((identity, muxer))
                } else {
                    security_log(SecurityEvent::InvalidNetworkPeer)
                        .error("InvalidIdentity")
                        .data(&identity)
                        .data(&peer_id)
                        .data(&origin)
                        .log();
                    Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!(
                            "PeerId received from Noise Handshake ({}) doesn't match one received from Identity Exchange ({})",
                            peer_id.short_str(),
                            identity.peer_id().short_str()
                        )
                    ))
                }
            }
        })
        .with_timeout(TRANSPORT_TIMEOUT)
        .boxed()
}

pub fn build_tcp_transport(
    own_identity: Identity,
) -> boxed::BoxedTransport<(Identity, impl StreamMultiplexer), impl ::std::error::Error> {
    let tcp_transport = tcp::TcpTransport::default();

    tcp_transport
        .and_then(|socket, origin| {
            async move {
                let muxer = Yamux::upgrade_connection(socket, origin).await?;
                Ok(muxer)
            }
        })
        .and_then(move |muxer, origin| {
            async move {
                let (identity, muxer) = exchange_identity(&own_identity, muxer, origin).await?;

                Ok((identity, muxer))
            }
        })
        .with_timeout(TRANSPORT_TIMEOUT)
        .boxed()
}

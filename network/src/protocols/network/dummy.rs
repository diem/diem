// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for validator_network.

use crate::{
    error::NetworkError,
    peer_manager::{ConnectionRequestSender, PeerManagerRequestSender},
    protocols::{
        network::{Event, NetworkEvents, NetworkSender},
        rpc::error::RpcError,
    },
    validator_network::network_builder::{NetworkBuilder, TransportType},
    NetworkPublicKeys, ProtocolId,
};
use channel::message_queues::QueueStyle;
use futures::{executor::block_on, StreamExt};
use libra_config::config::RoleType;
use libra_crypto::{
    ed25519::Ed25519PrivateKey, test_utils::TEST_SEED, x25519::X25519StaticPrivateKey, PrivateKey,
    Uniform,
};
use libra_types::PeerId;
use parity_multiaddr::Multiaddr;
use rand::{rngs::StdRng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Duration};
use tokio::runtime::Runtime;

const TEST_RPC_PROTOCOL: ProtocolId = ProtocolId::ConsensusRpc;
const TEST_DIRECT_SEND_PROTOCOL: ProtocolId = ProtocolId::ConsensusDirectSend;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DummyMsg(pub Vec<u8>);

fn add_to_network(network: &mut NetworkBuilder) -> (DummyNetworkSender, DummyNetworkEvents) {
    let (sender, receiver, connection_reqs_tx, connection_notifs_rx) = network
        .add_protocol_handler(
            vec![TEST_RPC_PROTOCOL],
            vec![TEST_DIRECT_SEND_PROTOCOL],
            QueueStyle::LIFO,
            None,
        );
    (
        DummyNetworkSender::new(sender, connection_reqs_tx),
        DummyNetworkEvents::new(receiver, connection_notifs_rx),
    )
}

/// TODO(davidiw): In DummyNetwork, replace DummyMsg with a Serde compatible type once migration
/// is complete
pub type DummyNetworkEvents = NetworkEvents<DummyMsg>;

#[derive(Clone)]
pub struct DummyNetworkSender {
    inner: NetworkSender<DummyMsg>,
}

impl DummyNetworkSender {
    pub fn new(
        peer_mgr_reqs_tx: PeerManagerRequestSender,
        connection_reqs_tx: ConnectionRequestSender,
    ) -> Self {
        Self {
            inner: NetworkSender::new(peer_mgr_reqs_tx, connection_reqs_tx),
        }
    }

    pub fn send_to(&mut self, recipient: PeerId, message: DummyMsg) -> Result<(), NetworkError> {
        let protocol = TEST_DIRECT_SEND_PROTOCOL;
        self.inner.send_to(recipient, protocol, message)
    }

    pub async fn send_rpc(
        &mut self,
        recipient: PeerId,
        message: DummyMsg,
        timeout: Duration,
    ) -> Result<DummyMsg, RpcError> {
        let protocol = TEST_RPC_PROTOCOL;
        self.inner
            .send_rpc(recipient, protocol, message, timeout)
            .await
    }
}

const HOUR_IN_MS: u64 = 60 * 60 * 1000;

pub struct DummyNetwork {
    pub runtime: Runtime,
    pub dialer_peer_id: PeerId,
    pub dialer_events: DummyNetworkEvents,
    pub dialer_sender: DummyNetworkSender,
    pub listener_peer_id: PeerId,
    pub listener_events: DummyNetworkEvents,
    pub listener_sender: DummyNetworkSender,
}

/// The following sets up a 2 peer network and verifies connectivity.
pub fn setup_network() -> DummyNetwork {
    let any: Multiaddr = "/ip4/127.0.0.1/tcp/0".parse().unwrap();
    let runtime = Runtime::new().unwrap();
    let (dialer_peer_id, dialer_addr) = (PeerId::random(), any.clone());
    let (listener_peer_id, listener_addr) = (PeerId::random(), any);

    // Setup keys for dialer.
    let mut rng = StdRng::from_seed(TEST_SEED);
    let dialer_signing_private_key = Ed25519PrivateKey::generate(&mut rng);
    let dialer_signing_public_key = dialer_signing_private_key.public_key();
    let dialer_identity_private_key = X25519StaticPrivateKey::generate(&mut rng);
    let dialer_identity_public_key = dialer_identity_private_key.public_key();

    // Setup keys for listener.
    let listener_signing_private_key = Ed25519PrivateKey::generate(&mut rng);
    let listener_signing_public_key = listener_signing_private_key.public_key();
    let listener_identity_private_key = X25519StaticPrivateKey::generate(&mut rng);
    let listener_identity_public_key = listener_identity_private_key.public_key();

    // Setup trusted peers.
    let trusted_peers: HashMap<_, _> = vec![
        (
            dialer_peer_id,
            NetworkPublicKeys {
                signing_public_key: dialer_signing_public_key.clone(),
                identity_public_key: dialer_identity_public_key.clone(),
            },
        ),
        (
            listener_peer_id,
            NetworkPublicKeys {
                signing_public_key: listener_signing_public_key.clone(),
                identity_public_key: listener_identity_public_key.clone(),
            },
        ),
    ]
    .into_iter()
    .collect();

    // Set up the listener network
    let mut network_builder = NetworkBuilder::new(
        runtime.handle().clone(),
        listener_peer_id,
        listener_addr,
        RoleType::Validator,
    );
    network_builder
        .transport(TransportType::TcpNoise(Some((
            listener_identity_private_key,
            listener_identity_public_key,
        ))))
        .trusted_peers(trusted_peers.clone())
        .signing_keys((listener_signing_private_key, listener_signing_public_key))
        .discovery_interval_ms(HOUR_IN_MS)
        .add_discovery();
    let (listener_sender, mut listener_events) = add_to_network(&mut network_builder);
    let listen_addr = network_builder.build();

    // Set up the dialer network
    let mut network_builder = NetworkBuilder::new(
        runtime.handle().clone(),
        dialer_peer_id,
        dialer_addr,
        RoleType::Validator,
    );
    network_builder
        .transport(TransportType::TcpNoise(Some((
            dialer_identity_private_key,
            dialer_identity_public_key,
        ))))
        .trusted_peers(trusted_peers)
        .signing_keys((dialer_signing_private_key, dialer_signing_public_key))
        .seed_peers(
            [(listener_peer_id, vec![listen_addr])]
                .iter()
                .cloned()
                .collect(),
        )
        .discovery_interval_ms(HOUR_IN_MS)
        .add_discovery();
    let (dialer_sender, mut dialer_events) = add_to_network(&mut network_builder);
    let _dialer_addr = network_builder.build();

    // Wait for establishing connection
    let first_dialer_event = block_on(dialer_events.next()).unwrap().unwrap();
    assert_eq!(first_dialer_event, Event::NewPeer(listener_peer_id));
    let first_listener_event = block_on(listener_events.next()).unwrap().unwrap();
    assert_eq!(first_listener_event, Event::NewPeer(dialer_peer_id));

    DummyNetwork {
        runtime,
        dialer_peer_id,
        dialer_events,
        dialer_sender,
        listener_peer_id,
        listener_events,
        listener_sender,
    }
}

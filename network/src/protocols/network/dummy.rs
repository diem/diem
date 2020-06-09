// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for validator_network.

use crate::{
    error::NetworkError,
    peer_manager::{ConnectionRequestSender, PeerManagerRequestSender},
    protocols::{
        network::{Event, NetworkEvents, NetworkSender, NewNetworkSender},
        rpc::error::RpcError,
    },
    validator_network::network_builder::{
        AuthenticationMode, NetworkBuilder, NETWORK_CHANNEL_SIZE,
    },
    ProtocolId,
};
use channel::message_queues::QueueStyle;
use futures::{executor::block_on, StreamExt};
use libra_config::{chain_id::ChainId, config::RoleType, network_id::NetworkId};
use libra_crypto::{test_utils::TEST_SEED, x25519, Uniform};
use libra_metrics::IntCounterVec;
use libra_network_address::NetworkAddress;
use libra_types::PeerId;
use rand::{rngs::StdRng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Duration};
use tokio::runtime::Runtime;

const TEST_RPC_PROTOCOL: ProtocolId = ProtocolId::ConsensusRpc;
const TEST_DIRECT_SEND_PROTOCOL: ProtocolId = ProtocolId::ConsensusDirectSend;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DummyMsg(pub Vec<u8>);

pub fn network_endpoint_config() -> (
    Vec<ProtocolId>,
    Vec<ProtocolId>,
    QueueStyle,
    usize,
    Option<&'static IntCounterVec>,
) {
    (
        vec![TEST_RPC_PROTOCOL],
        vec![TEST_DIRECT_SEND_PROTOCOL],
        QueueStyle::LIFO,
        NETWORK_CHANNEL_SIZE,
        None,
    )
}

/// TODO(davidiw): In DummyNetwork, replace DummyMsg with a Serde compatible type once migration
/// is complete
pub type DummyNetworkEvents = NetworkEvents<DummyMsg>;

#[derive(Clone)]
pub struct DummyNetworkSender {
    inner: NetworkSender<DummyMsg>,
}

impl NewNetworkSender for DummyNetworkSender {
    fn new(
        peer_mgr_reqs_tx: PeerManagerRequestSender,
        connection_reqs_tx: ConnectionRequestSender,
    ) -> Self {
        Self {
            inner: NetworkSender::new(peer_mgr_reqs_tx, connection_reqs_tx),
        }
    }
}

impl DummyNetworkSender {
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
    let runtime = Runtime::new().unwrap();
    let network_id = NetworkId::Validator;
    let chain_id = ChainId::default();
    let dialer_peer_id = PeerId::random();
    let listener_peer_id = PeerId::random();

    // Setup keys for dialer.
    let mut rng = StdRng::from_seed(TEST_SEED);
    let dialer_identity_private_key = x25519::PrivateKey::generate(&mut rng);
    let dialer_identity_public_key = dialer_identity_private_key.public_key();

    // Setup keys for listener.
    let listener_identity_private_key = x25519::PrivateKey::generate(&mut rng);
    let listener_identity_public_key = listener_identity_private_key.public_key();

    // Setup listen addresses
    let dialer_addr: NetworkAddress = "/ip4/127.0.0.1/tcp/0".parse().unwrap();
    let listener_addr: NetworkAddress = "/ip4/127.0.0.1/tcp/0".parse().unwrap();

    // Setup trusted peers.
    let trusted_peers: HashMap<_, _> = vec![
        (dialer_peer_id, dialer_identity_public_key),
        (listener_peer_id, listener_identity_public_key),
    ]
    .into_iter()
    .collect();

    // Set up the listener network
    let mut network_builder = NetworkBuilder::new(
        runtime.handle().clone(),
        chain_id.clone(),
        network_id.clone(),
        RoleType::Validator,
        listener_peer_id,
        listener_addr,
    );
    network_builder
        .authentication_mode(AuthenticationMode::Mutual(listener_identity_private_key))
        .trusted_peers(trusted_peers.clone())
        .add_connectivity_manager();
    let (listener_sender, mut listener_events) = network_builder
        .add_protocol_handler::<DummyNetworkSender, DummyNetworkEvents>(network_endpoint_config());
    let listener_addr = network_builder.build();

    // Set up the dialer network
    let mut network_builder = NetworkBuilder::new(
        runtime.handle().clone(),
        chain_id,
        network_id,
        RoleType::Validator,
        dialer_peer_id,
        dialer_addr,
    );
    network_builder
        .authentication_mode(AuthenticationMode::Mutual(dialer_identity_private_key))
        .trusted_peers(trusted_peers)
        .seed_peers(
            [(listener_peer_id, vec![listener_addr])]
                .iter()
                .cloned()
                .collect(),
        )
        .add_connectivity_manager();
    let (dialer_sender, mut dialer_events) = network_builder
        .add_protocol_handler::<DummyNetworkSender, DummyNetworkEvents>(network_endpoint_config());
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

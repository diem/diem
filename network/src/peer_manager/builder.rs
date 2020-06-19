// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    constants, counters,
    peer_manager::{
        conn_notifs_channel, ConnectionRequest, ConnectionRequestSender, PeerManager,
        PeerManagerNotification, PeerManagerRequest, PeerManagerRequestSender,
    },
    protocols::wire::handshake::v1::SupportedProtocols,
    transport::{self, Connection, LibraNetTransport, LIBRA_TCP_TRANSPORT},
    ProtocolId,
};
use channel::{self, libra_channel, message_queues::QueueStyle};
use libra_config::{chain_id::ChainId, config::HANDSHAKE_VERSION, network_id::NetworkContext};
use libra_crypto::x25519;
use libra_logger::prelude::*;
use libra_metrics::IntCounterVec;
use libra_network_address::NetworkAddress;
use libra_types::PeerId;
use netcore::transport::{memory, Transport};
use std::{
    clone::Clone,
    collections::HashMap,
    fmt::Debug,
    num::NonZeroUsize,
    sync::{Arc, RwLock},
};
use tokio::runtime::Handle;

#[derive(Debug)]
pub enum AuthenticationMode {
    /// Inbound and outbound connections are secured with NoiseIK; however, only
    /// clients/dialers will authenticate the servers/listeners. More specifically,
    /// dialers will pin the connection to a specific, expected pubkey while
    /// listeners will accept any inbound dialer's pubkey.
    ServerOnly(x25519::PrivateKey),
    /// Inbound and outbound connections are secured with NoiseIK. Both dialer and
    /// listener will only accept connections that successfully authenticate to a
    /// pubkey in their "trusted peers" set.
    Mutual(x25519::PrivateKey),
}

impl AuthenticationMode {
    /// Convenience method to retrieve the public key for the auth mode's inner
    /// network identity key.
    ///
    /// Note: this only works because all auth modes are Noise-based.
    pub fn public_key(&self) -> x25519::PublicKey {
        match self {
            AuthenticationMode::ServerOnly(key) | AuthenticationMode::Mutual(key) => {
                key.public_key()
            }
        }
    }
}

pub struct PeerManagerBuilder {
    chain_id: ChainId,
    network_context: Arc<NetworkContext>,
    // TODO(philiphayes): better support multiple listening addrs
    listen_address: NetworkAddress,
    trusted_peers: Arc<RwLock<HashMap<PeerId, x25519::PublicKey>>>,
    // An option to ensure at most one copy of the contained private key.
    authentication_mode: Option<AuthenticationMode>,
    channel_size: usize,
    direct_send_protocols: Vec<ProtocolId>,
    rpc_protocols: Vec<ProtocolId>,
    upstream_handlers:
        HashMap<ProtocolId, libra_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>>,
    connection_event_handlers: Vec<conn_notifs_channel::Sender>,
    pm_reqs_tx: libra_channel::Sender<(PeerId, ProtocolId), PeerManagerRequest>,
    pm_reqs_rx: libra_channel::Receiver<(PeerId, ProtocolId), PeerManagerRequest>,
    connection_reqs_tx: libra_channel::Sender<PeerId, ConnectionRequest>,
    connection_reqs_rx: libra_channel::Receiver<PeerId, ConnectionRequest>,
    max_concurrent_network_reqs: usize,
    max_concurrent_network_notifs: usize,
}

impl PeerManagerBuilder {
    pub fn create(
        chain_id: ChainId,
        network_context: Arc<NetworkContext>,
        // TODO(philiphayes): better support multiple listening addrs
        listen_address: NetworkAddress,
        trusted_peers: Arc<RwLock<HashMap<PeerId, x25519::PublicKey>>>,
        authentication_mode: AuthenticationMode,
        channel_size: usize,
        max_concurrent_network_reqs: usize,
        max_concurrent_network_notifs: usize,
    ) -> Self {
        // Setup channel to send requests to peer manager.
        let (pm_reqs_tx, pm_reqs_rx) = libra_channel::new(
            QueueStyle::FIFO,
            NonZeroUsize::new(constants::NETWORK_CHANNEL_SIZE).unwrap(),
            Some(&counters::PENDING_PEER_MANAGER_REQUESTS),
        );
        // Setup channel to send connection requests to peer manager.
        let (connection_reqs_tx, connection_reqs_rx) = libra_channel::new(
            QueueStyle::FIFO,
            NonZeroUsize::new(constants::NETWORK_CHANNEL_SIZE).unwrap(),
            None,
        );

        Self {
            chain_id,
            network_context,
            // TODO(philiphayes): better support multiple listening addrs
            listen_address,
            trusted_peers,
            authentication_mode: Some(authentication_mode),
            channel_size,
            direct_send_protocols: Vec::new(),
            rpc_protocols: Vec::new(),
            upstream_handlers: HashMap::new(),
            connection_event_handlers: Vec::new(),
            pm_reqs_tx,
            pm_reqs_rx,
            connection_reqs_tx,
            connection_reqs_rx,
            max_concurrent_network_reqs,
            max_concurrent_network_notifs,
        }
    }

    pub fn connection_reqs_tx(&self) -> libra_channel::Sender<PeerId, ConnectionRequest> {
        self.connection_reqs_tx.clone()
    }

    pub fn add_connection_event_listener(&mut self) -> conn_notifs_channel::Receiver {
        let (tx, rx) = conn_notifs_channel::new();
        self.connection_event_handlers.push(tx);
        rx
    }

    /// Create the configured transport and start PeerManager.
    /// Return the actual NetworkAddress over which this peer is listening.
    pub fn build(mut self, executor: &Handle) -> NetworkAddress {
        use libra_network_address::Protocol::*;

        let chain_id = self.chain_id.clone();
        let network_id = self.network_context.network_id().clone();
        let protos = self.supported_protocols();

        let (key, maybe_trusted_peers, peer_id) = match self
            .authentication_mode
            .take()
            .expect("AuthenticationMode must be set")
        {
            // validator-operated full node
            AuthenticationMode::ServerOnly(key)
                if self.network_context.peer_id() == PeerId::ZERO =>
            {
                let public_key = key.public_key();
                let peer_id = PeerId::from_identity_public_key(public_key);
                (key, None, peer_id)
            }
            // full node
            AuthenticationMode::ServerOnly(key) => (key, None, self.network_context.peer_id()),
            // validator
            AuthenticationMode::Mutual(key) => (
                key,
                Some(self.trusted_peers.clone()),
                self.network_context.peer_id(),
            ),
        };

        match self.listen_address.as_slice() {
            [Ip4(_), Tcp(_)] | [Ip6(_), Tcp(_)] => self.build_with_transport(
                LibraNetTransport::new(
                    LIBRA_TCP_TRANSPORT.clone(),
                    peer_id,
                    key,
                    maybe_trusted_peers,
                    HANDSHAKE_VERSION,
                    chain_id,
                    network_id,
                    protos,
                ),
                executor,
            ),
            [Memory(_)] => self.build_with_transport(
                LibraNetTransport::new(
                    memory::MemoryTransport,
                    peer_id,
                    key,
                    maybe_trusted_peers,
                    HANDSHAKE_VERSION,
                    chain_id,
                    network_id,
                    protos,
                ),
                executor,
            ),
            _ => panic!(
                "{} Unsupported listen_address: '{}', expected '/memory/<port>', \
                 '/ip4/<addr>/tcp/<port>', or '/ip6/<addr>/tcp/<port>'.",
                self.network_context, self.listen_address
            ),
        }
    }

    /// Given a transport build and launch PeerManager.
    /// Return the actual NetworkAddress over which this peer is listening.
    fn build_with_transport<TTransport, TSocket>(
        self,
        transport: TTransport,
        executor: &Handle,
    ) -> NetworkAddress
    where
        TTransport: Transport<Output = Connection<TSocket>> + Send + 'static,
        TSocket: transport::TSocket,
    {
        let peer_mgr = PeerManager::new(
            executor.clone(),
            transport,
            self.network_context.clone(),
            // TODO(philiphayes): peer manager should take `Vec<NetworkAddress>`
            // (which could be empty, like in client use case)
            self.listen_address.clone(),
            self.pm_reqs_rx,
            self.connection_reqs_rx,
            self.upstream_handlers,
            self.connection_event_handlers,
            self.max_concurrent_network_reqs,
            self.max_concurrent_network_notifs,
            self.channel_size,
        );

        // PeerManager constructor appends a public key to the listen_address.
        let listen_addr_with_key = peer_mgr.listen_addr().clone();

        // Split this off into a separate function
        executor.spawn(peer_mgr.start());
        debug!("{} Started peer manager", self.network_context);

        listen_addr_with_key
    }

    /// Add a handler for given protocols using raw bytes.
    pub fn add_protocol_handler(
        &mut self,
        rpc_protocols: Vec<ProtocolId>,
        direct_send_protocols: Vec<ProtocolId>,
        queue_preference: QueueStyle,
        max_queue_size_per_peer: usize,
        counter: Option<&'static IntCounterVec>,
    ) -> (
        PeerManagerRequestSender,
        libra_channel::Receiver<(PeerId, ProtocolId), PeerManagerNotification>,
        ConnectionRequestSender,
        conn_notifs_channel::Receiver,
    ) {
        self.direct_send_protocols
            .extend(direct_send_protocols.clone());
        self.rpc_protocols.extend(rpc_protocols.clone());
        let (network_notifs_tx, network_notifs_rx) = libra_channel::new(
            queue_preference,
            NonZeroUsize::new(max_queue_size_per_peer).unwrap(),
            counter,
        );
        for protocol in rpc_protocols
            .iter()
            .chain(direct_send_protocols.iter())
            .cloned()
        {
            self.upstream_handlers
                .insert(protocol, network_notifs_tx.clone());
        }
        let (connection_notifs_tx, connection_notifs_rx) = conn_notifs_channel::new();
        // Auto-subscribe all application level handlers to connection events.
        self.connection_event_handlers.push(connection_notifs_tx);
        (
            PeerManagerRequestSender::new(self.pm_reqs_tx.clone()),
            network_notifs_rx,
            ConnectionRequestSender::new(self.connection_reqs_tx.clone()),
            connection_notifs_rx,
        )
    }

    fn supported_protocols(&self) -> SupportedProtocols {
        self.direct_send_protocols
            .iter()
            .chain(&self.rpc_protocols)
            .into()
    }
}

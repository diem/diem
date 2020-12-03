// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    counters,
    noise::stream::NoiseStream,
    peer_manager::{
        conn_notifs_channel, ConnectionRequest, ConnectionRequestSender, PeerManager,
        PeerManagerNotification, PeerManagerRequest, PeerManagerRequestSender,
    },
    protocols::wire::handshake::v1::SupportedProtocols,
    transport::{self, Connection, DiemNetTransport, DIEM_TCP_TRANSPORT},
    ProtocolId,
};
use channel::{self, diem_channel, message_queues::QueueStyle};
use diem_config::{config::HANDSHAKE_VERSION, network_id::NetworkContext};
use diem_crypto::x25519;
use diem_infallible::RwLock;
use diem_logger::prelude::*;
use diem_metrics::IntCounterVec;
use diem_network_address::NetworkAddress;
use diem_types::{chain_id::ChainId, PeerId};
#[cfg(any(test, feature = "testing", feature = "fuzzing"))]
use netcore::transport::memory::MemoryTransport;
use netcore::transport::{
    tcp::{TcpSocket, TcpTransport},
    Transport,
};
use std::{
    clone::Clone,
    collections::{HashMap, HashSet},
    fmt::Debug,
    num::NonZeroUsize,
    sync::Arc,
};
use tokio::runtime::Handle;

// TODO:  This is the wrong logical location for this code to exist.  Determine the better location.
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

struct TransportContext {
    chain_id: ChainId,
    direct_send_protocols: Vec<ProtocolId>,
    rpc_protocols: Vec<ProtocolId>,
    authentication_mode: AuthenticationMode,
    trusted_peers: Arc<RwLock<HashMap<PeerId, HashSet<x25519::PublicKey>>>>,
}

impl TransportContext {
    pub fn new(
        chain_id: ChainId,
        direct_send_protocols: Vec<ProtocolId>,
        rpc_protocols: Vec<ProtocolId>,
        authentication_mode: AuthenticationMode,
        trusted_peers: Arc<RwLock<HashMap<PeerId, HashSet<x25519::PublicKey>>>>,
    ) -> Self {
        Self {
            chain_id,
            direct_send_protocols,
            rpc_protocols,
            authentication_mode,
            trusted_peers,
        }
    }

    fn supported_protocols(&self) -> SupportedProtocols {
        self.direct_send_protocols
            .iter()
            .chain(&self.rpc_protocols)
            .into()
    }

    fn augment_direct_send_protocols(
        &mut self,
        direct_send_protocols: Vec<ProtocolId>,
    ) -> &mut Self {
        self.direct_send_protocols.extend(direct_send_protocols);
        self
    }

    fn augment_rpc_protocols(&mut self, rpc_protocols: Vec<ProtocolId>) -> &mut Self {
        self.rpc_protocols.extend(rpc_protocols);
        self
    }
}

struct PeerManagerContext {
    // TODO(philiphayes): better support multiple listening addrs
    pm_reqs_tx: diem_channel::Sender<(PeerId, ProtocolId), PeerManagerRequest>,
    pm_reqs_rx: diem_channel::Receiver<(PeerId, ProtocolId), PeerManagerRequest>,
    connection_reqs_tx: diem_channel::Sender<PeerId, ConnectionRequest>,
    connection_reqs_rx: diem_channel::Receiver<PeerId, ConnectionRequest>,

    upstream_handlers:
        HashMap<ProtocolId, diem_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>>,
    connection_event_handlers: Vec<conn_notifs_channel::Sender>,

    max_concurrent_network_reqs: usize,
    max_concurrent_network_notifs: usize,
    channel_size: usize,
    inbound_connection_limit: usize,
}

impl PeerManagerContext {
    fn new(
        pm_reqs_tx: diem_channel::Sender<(PeerId, ProtocolId), PeerManagerRequest>,
        pm_reqs_rx: diem_channel::Receiver<(PeerId, ProtocolId), PeerManagerRequest>,
        connection_reqs_tx: diem_channel::Sender<PeerId, ConnectionRequest>,
        connection_reqs_rx: diem_channel::Receiver<PeerId, ConnectionRequest>,

        upstream_handlers: HashMap<
            ProtocolId,
            diem_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>,
        >,
        connection_event_handlers: Vec<conn_notifs_channel::Sender>,

        max_concurrent_network_reqs: usize,
        max_concurrent_network_notifs: usize,
        channel_size: usize,
        inbound_connection_limit: usize,
    ) -> Self {
        Self {
            pm_reqs_tx,
            pm_reqs_rx,
            connection_reqs_tx,
            connection_reqs_rx,

            upstream_handlers,
            connection_event_handlers,

            max_concurrent_network_reqs,
            max_concurrent_network_notifs,
            channel_size,
            inbound_connection_limit,
        }
    }

    fn add_upstream_handler(
        &mut self,
        protocol_id: ProtocolId,
        channel: diem_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>,
    ) -> &mut Self {
        self.upstream_handlers.insert(protocol_id, channel);
        self
    }

    pub fn add_connection_event_listener(&mut self) -> conn_notifs_channel::Receiver {
        let (tx, rx) = conn_notifs_channel::new();
        self.connection_event_handlers.push(tx);
        rx
    }
}

#[cfg(any(test, feature = "testing", feature = "fuzzing"))]
type MemoryPeerManager =
    PeerManager<DiemNetTransport<MemoryTransport>, NoiseStream<memsocket::MemorySocket>>;
type TcpPeerManager = PeerManager<DiemNetTransport<TcpTransport>, NoiseStream<TcpSocket>>;

#[derive(Debug, PartialEq, PartialOrd)]
enum State {
    CREATED,
    BUILT,
    STARTED,
}

pub struct PeerManagerBuilder {
    network_context: Arc<NetworkContext>,
    transport_context: Option<TransportContext>,
    peer_manager_context: Option<PeerManagerContext>,
    // TODO(philiphayes): better support multiple listening addrs
    // An option to ensure at most one copy of the contained private key.
    #[cfg(any(test, feature = "testing", feature = "fuzzing"))]
    memory_peer_manager: Option<MemoryPeerManager>,
    tcp_peer_manager: Option<TcpPeerManager>,
    // ListenAddress will be updated when the PeerManager is built
    listen_address: NetworkAddress,
    state: State,
    max_frame_size: usize,
    enable_proxy_protocol: bool,
}

impl PeerManagerBuilder {
    pub fn create(
        chain_id: ChainId,
        network_context: Arc<NetworkContext>,
        // TODO(philiphayes): better support multiple listening addrs
        listen_address: NetworkAddress,
        trusted_peers: Arc<RwLock<HashMap<PeerId, HashSet<x25519::PublicKey>>>>,
        authentication_mode: AuthenticationMode,
        channel_size: usize,
        max_concurrent_network_reqs: usize,
        max_concurrent_network_notifs: usize,
        max_frame_size: usize,
        enable_proxy_protocol: bool,
        inbound_connection_limit: usize,
    ) -> Self {
        // Setup channel to send requests to peer manager.
        let (pm_reqs_tx, pm_reqs_rx) = diem_channel::new(
            QueueStyle::FIFO,
            NonZeroUsize::new(channel_size).unwrap(),
            Some(&counters::PENDING_PEER_MANAGER_REQUESTS),
        );
        // Setup channel to send connection requests to peer manager.
        let (connection_reqs_tx, connection_reqs_rx) = diem_channel::new(
            QueueStyle::FIFO,
            NonZeroUsize::new(channel_size).unwrap(),
            None,
        );

        Self {
            network_context,
            transport_context: Some(TransportContext::new(
                chain_id,
                Vec::new(),
                Vec::new(),
                authentication_mode,
                trusted_peers,
            )),
            peer_manager_context: Some(PeerManagerContext::new(
                pm_reqs_tx,
                pm_reqs_rx,
                connection_reqs_tx,
                connection_reqs_rx,
                HashMap::new(),
                Vec::new(),
                max_concurrent_network_reqs,
                max_concurrent_network_notifs,
                channel_size,
                inbound_connection_limit,
            )),
            #[cfg(any(test, feature = "testing", feature = "fuzzing"))]
            memory_peer_manager: None,
            tcp_peer_manager: None,
            listen_address,
            state: State::CREATED,
            max_frame_size,
            enable_proxy_protocol,
        }
    }

    pub fn listen_address(&self) -> NetworkAddress {
        self.listen_address.clone()
    }

    pub fn connection_reqs_tx(&self) -> diem_channel::Sender<PeerId, ConnectionRequest> {
        self.peer_manager_context
            .as_ref()
            .expect("Cannot access connection_reqs once PeerManager has been built")
            .connection_reqs_tx
            .clone()
    }

    pub fn add_connection_event_listener(&mut self) -> conn_notifs_channel::Receiver {
        self.peer_manager_context
            .as_mut()
            .expect("Cannot add an event listener if PeerManager has already been built.")
            .add_connection_event_listener()
    }

    /// Create the configured transport and start PeerManager.
    /// Return the actual NetworkAddress over which this peer is listening.
    pub fn build(&mut self, executor: &Handle) -> &mut Self {
        assert_eq!(self.state, State::CREATED);
        self.state = State::BUILT;
        use diem_network_address::Protocol::*;

        let transport_context = self
            .transport_context
            .take()
            .expect("PeerManager can only be built once");

        let protos = transport_context.supported_protocols();
        let chain_id = transport_context.chain_id;

        let (key, maybe_trusted_peers) = match transport_context.authentication_mode {
            AuthenticationMode::ServerOnly(key) => (key, None),
            AuthenticationMode::Mutual(key) => (key, Some(transport_context.trusted_peers)),
        };

        match self.listen_address.as_slice() {
            [Ip4(_), Tcp(_)] | [Ip6(_), Tcp(_)] => {
                self.tcp_peer_manager = Some(self.build_with_transport(
                    DiemNetTransport::new(
                        DIEM_TCP_TRANSPORT.clone(),
                        self.network_context.clone(),
                        key,
                        maybe_trusted_peers,
                        HANDSHAKE_VERSION,
                        chain_id,
                        protos,
                        self.enable_proxy_protocol,
                    ),
                    executor,
                ))
            }
            #[cfg(any(test, feature = "testing", feature = "fuzzing"))]
            [Memory(_)] => {
                self.memory_peer_manager = Some(self.build_with_transport(
                    DiemNetTransport::new(
                        MemoryTransport,
                        self.network_context.clone(),
                        key,
                        maybe_trusted_peers,
                        HANDSHAKE_VERSION,
                        chain_id,
                        protos,
                        self.enable_proxy_protocol,
                    ),
                    executor,
                ))
            }
            _ => panic!(
                "{} Unsupported listen_address: '{}', expected '/memory/<port>', \
                 '/ip4/<addr>/tcp/<port>', or '/ip6/<addr>/tcp/<port>'.",
                self.network_context, self.listen_address
            ),
        };

        self
    }

    /// Given a transport build and launch PeerManager.
    /// Return the actual NetworkAddress over which this peer is listening.
    fn build_with_transport<TTransport, TSocket>(
        &mut self,
        transport: TTransport,
        executor: &Handle,
    ) -> PeerManager<TTransport, TSocket>
    where
        TTransport: Transport<Output = Connection<TSocket>> + Send + 'static,
        TSocket: transport::TSocket,
    {
        let pm_context = self
            .peer_manager_context
            .take()
            .expect("PeerManager can only be built once");

        let peer_mgr = PeerManager::new(
            executor.clone(),
            transport,
            self.network_context.clone(),
            // TODO(philiphayes): peer manager should take `Vec<NetworkAddress>`
            // (which could be empty, like in client use case)
            self.listen_address.clone(),
            pm_context.pm_reqs_rx,
            pm_context.connection_reqs_rx,
            pm_context.upstream_handlers,
            pm_context.connection_event_handlers,
            pm_context.max_concurrent_network_reqs,
            pm_context.max_concurrent_network_notifs,
            pm_context.channel_size,
            self.max_frame_size,
            pm_context.inbound_connection_limit,
        );

        // PeerManager constructor appends a public key to the listen_address.
        self.listen_address = peer_mgr.listen_addr().clone();

        peer_mgr
    }

    fn start_peer_manager<TTransport, TSocket>(
        &mut self,
        peer_manager: PeerManager<TTransport, TSocket>,
        executor: &Handle,
    ) where
        TTransport: Transport<Output = Connection<TSocket>> + Send + 'static,
        TSocket: transport::TSocket,
    {
        executor.spawn(peer_manager.start());
        debug!("{} Started peer manager", self.network_context);
    }

    pub fn start(&mut self, executor: &Handle) {
        assert_eq!(self.state, State::BUILT);
        self.state = State::STARTED;
        debug!("{} Starting Peer manager", self.network_context);
        #[cfg(any(test, feature = "testing", feature = "fuzzing"))]
        if let Some(memory_pm) = self.memory_peer_manager.take() {
            self.start_peer_manager(memory_pm, executor);
        };
        if let Some(tcp_pm) = self.tcp_peer_manager.take() {
            self.start_peer_manager(tcp_pm, executor);
        }
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
        diem_channel::Receiver<(PeerId, ProtocolId), PeerManagerNotification>,
        ConnectionRequestSender,
        conn_notifs_channel::Receiver,
    ) {
        self.transport_context
            .as_mut()
            .expect("Cannot add a protocol handler once PeerManager has been built")
            .augment_direct_send_protocols(direct_send_protocols.clone())
            .augment_rpc_protocols(rpc_protocols.clone());

        let (network_notifs_tx, network_notifs_rx) = diem_channel::new(
            queue_preference,
            NonZeroUsize::new(max_queue_size_per_peer).unwrap(),
            counter,
        );

        let pm_context = self
            .peer_manager_context
            .as_mut()
            .expect("Cannot add a protocol handler once PeerManager has been built");

        for protocol in rpc_protocols
            .iter()
            .chain(direct_send_protocols.iter())
            .cloned()
        {
            pm_context.add_upstream_handler(protocol, network_notifs_tx.clone());
        }
        let connection_notifs_rx = pm_context.add_connection_event_listener();

        (
            PeerManagerRequestSender::new(pm_context.pm_reqs_tx.clone()),
            network_notifs_rx,
            ConnectionRequestSender::new(pm_context.connection_reqs_tx.clone()),
            connection_notifs_rx,
        )
    }
}

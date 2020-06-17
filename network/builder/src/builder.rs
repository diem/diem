// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Remotely authenticated vs. unauthenticated network end-points:
//! ---------------------------------------------------
//! A network end-point operates with remote authentication if it only accepts connections
//! from a known set of peers (`trusted_peers`) identified by their network identity keys.
//! This does not mean that the other end-point of a connection also needs to operate with
//! authentication -- a network end-point running with remote authentication enabled will
//! connect to or accept connections from an end-point running in authenticated mode as
//! long as the latter is in its trusted peers set.
use channel::{self, libra_channel, message_queues::QueueStyle};
use futures::stream::StreamExt;
use libra_config::{
    chain_id::ChainId,
    config::{identity_key, peer_id, DiscoveryMethod, NetworkConfig, RoleType, HANDSHAKE_VERSION},
    network_id::{NetworkContext, NetworkId},
};
use libra_crypto::x25519;
use libra_logger::prelude::*;
use libra_metrics::IntCounterVec;
use libra_network_address::NetworkAddress;
use libra_types::{waypoint::Waypoint, PeerId};
use netcore::transport::{memory, Transport};
use network::{
    connectivity_manager::{ConnectivityManager, ConnectivityRequest},
    constants, counters,
    peer_manager::{
        conn_notifs_channel, ConnectionRequest, ConnectionRequestSender, PeerManager,
        PeerManagerNotification, PeerManagerRequest, PeerManagerRequestSender,
    },
    protocols::{
        discovery::{self, Discovery},
        health_checker::{self, HealthChecker},
        network::{NewNetworkEvents, NewNetworkSender},
        wire::handshake::v1::SupportedProtocols,
    },
    transport::{self, Connection, LibraNetTransport, LIBRA_TCP_TRANSPORT},
    ProtocolId,
};
use onchain_discovery::builder::OnchainDiscoveryBuilder;
use std::{
    clone::Clone,
    collections::HashMap,
    num::NonZeroUsize,
    sync::{Arc, RwLock},
    time::Duration,
};
use storage_interface::DbReader;
use tokio::{
    runtime::{Builder, Handle, Runtime},
    time::interval,
};
use tokio_retry::strategy::ExponentialBackoff;

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

/// Build Network module with custom configuration values.
/// Methods can be chained in order to set the configuration values.
/// MempoolNetworkHandler and ConsensusNetworkHandler are constructed by calling
/// [`NetworkBuilder::build`].  New instances of `NetworkBuilder` are obtained
/// via [`NetworkBuilder::new`].
// TODO(philiphayes): refactor NetworkBuilder and libra-node; current config is
// pretty tangled.
pub struct NetworkBuilder {
    executor: Handle,
    chain_id: ChainId,
    network_context: NetworkContext,
    // TODO(philiphayes): better support multiple listening addrs
    listen_address: NetworkAddress,
    advertised_address: Option<NetworkAddress>,
    seed_peers: HashMap<PeerId, Vec<NetworkAddress>>,
    trusted_peers: Arc<RwLock<HashMap<PeerId, x25519::PublicKey>>>,
    authentication_mode: Option<AuthenticationMode>,
    channel_size: usize,
    direct_send_protocols: Vec<ProtocolId>,
    rpc_protocols: Vec<ProtocolId>,
    discovery_interval_ms: u64,
    ping_interval_ms: u64,
    ping_timeout_ms: u64,
    ping_failures_tolerated: u64,
    upstream_handlers:
        HashMap<ProtocolId, libra_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>>,
    connection_event_handlers: Vec<conn_notifs_channel::Sender>,
    pm_reqs_tx: libra_channel::Sender<(PeerId, ProtocolId), PeerManagerRequest>,
    pm_reqs_rx: libra_channel::Receiver<(PeerId, ProtocolId), PeerManagerRequest>,
    connection_reqs_tx: libra_channel::Sender<PeerId, ConnectionRequest>,
    connection_reqs_rx: libra_channel::Receiver<PeerId, ConnectionRequest>,
    conn_mgr_reqs_tx: Option<channel::Sender<ConnectivityRequest>>,
    connectivity_check_interval_ms: u64,
    max_concurrent_network_reqs: usize,
    max_concurrent_network_notifs: usize,
    max_connection_delay_ms: u64,
}

impl NetworkBuilder {
    /// Return a new NetworkBuilder initialized with default configuration values.
    pub fn new(
        executor: Handle,
        chain_id: ChainId,
        network_id: NetworkId,
        role: RoleType,
        peer_id: PeerId,
        listen_address: NetworkAddress,
    ) -> NetworkBuilder {
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
        NetworkBuilder {
            executor,
            chain_id,
            network_context: NetworkContext::new(network_id, role, peer_id),
            listen_address,
            advertised_address: None,
            seed_peers: HashMap::new(),
            trusted_peers: Arc::new(RwLock::new(HashMap::new())),
            authentication_mode: None,
            channel_size: constants::NETWORK_CHANNEL_SIZE,
            direct_send_protocols: vec![],
            rpc_protocols: vec![],
            upstream_handlers: HashMap::new(),
            connection_event_handlers: Vec::new(),
            pm_reqs_tx,
            pm_reqs_rx,
            connection_reqs_tx,
            connection_reqs_rx,
            conn_mgr_reqs_tx: None,
            discovery_interval_ms: constants::DISCOVERY_INTERVAL_MS,
            ping_interval_ms: constants::PING_INTERVAL_MS,
            ping_timeout_ms: constants::PING_TIMEOUT_MS,
            ping_failures_tolerated: constants::PING_FAILURES_TOLERATED,
            connectivity_check_interval_ms: constants::CONNECTIVITY_CHECK_INTERNAL_MS,
            max_concurrent_network_reqs: constants::MAX_CONCURRENT_NETWORK_REQS,
            max_concurrent_network_notifs: constants::MAX_CONCURRENT_NETWORK_NOTIFS,
            max_connection_delay_ms: constants::MAX_CONNECTION_DELAY_MS,
        }
    }

    pub fn network_context(&self) -> &NetworkContext {
        &self.network_context
    }

    pub fn peer_id(&self) -> PeerId {
        self.network_context.peer_id()
    }

    /// Set network authentication mode.
    pub fn authentication_mode(&mut self, authentication_mode: AuthenticationMode) -> &mut Self {
        self.authentication_mode = Some(authentication_mode);
        self
    }

    /// Set an address to advertise, if different from the listen address
    pub fn advertised_address(&mut self, advertised_address: NetworkAddress) -> &mut Self {
        self.advertised_address = Some(advertised_address);
        self
    }

    /// Set trusted peers.
    pub fn trusted_peers(
        &mut self,
        trusted_peers: HashMap<PeerId, x25519::PublicKey>,
    ) -> &mut Self {
        *self.trusted_peers.write().unwrap() = trusted_peers;
        self
    }

    /// Set seed peers to bootstrap discovery
    pub fn seed_peers(&mut self, seed_peers: HashMap<PeerId, Vec<NetworkAddress>>) -> &mut Self {
        self.seed_peers = seed_peers;
        self
    }

    /// Set discovery ticker interval
    pub fn discovery_interval_ms(&mut self, discovery_interval_ms: u64) -> &mut Self {
        self.discovery_interval_ms = discovery_interval_ms;
        self
    }

    /// Set connectivity check ticker interval
    pub fn connectivity_check_interval_ms(
        &mut self,
        connectivity_check_interval_ms: u64,
    ) -> &mut Self {
        self.connectivity_check_interval_ms = connectivity_check_interval_ms;
        self
    }

    pub fn conn_mgr_reqs_tx(&self) -> Option<channel::Sender<ConnectivityRequest>> {
        self.conn_mgr_reqs_tx.clone()
    }

    fn supported_protocols(&self) -> SupportedProtocols {
        self.direct_send_protocols
            .iter()
            .chain(&self.rpc_protocols)
            .into()
    }

    /// Add a handler for given protocols using raw bytes.
    fn inner_add_protocol_handler(
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

    pub fn add_connection_event_listener(&mut self) -> conn_notifs_channel::Receiver {
        let (tx, rx) = conn_notifs_channel::new();
        self.connection_event_handlers.push(tx);
        rx
    }

    /// Add a [`ConnectivityManager`] to the network.
    ///
    /// [`ConnectivityManager`] is responsible for ensuring that we are connected
    /// to a node iff. it is an eligible node and maintaining persistent
    /// connections with all eligible nodes. A list of eligible nodes is received
    /// at initialization, and updates are received on changes to system membership.
    ///
    /// Note: a connectivity manager should only be added if the network is
    /// permissioned.
    pub fn add_connectivity_manager(&mut self) -> &mut Self {
        let (conn_mgr_reqs_tx, conn_mgr_reqs_rx) = channel::new(
            self.channel_size,
            &counters::PENDING_CONNECTIVITY_MANAGER_REQUESTS,
        );
        self.conn_mgr_reqs_tx = Some(conn_mgr_reqs_tx);
        let trusted_peers = self.trusted_peers.clone();
        let seed_peers = self.seed_peers.clone();
        let max_connection_delay_ms = self.max_connection_delay_ms;
        let connectivity_check_interval_ms = self.connectivity_check_interval_ms;
        let pm_conn_mgr_notifs_rx = self.add_connection_event_listener();
        let conn_mgr = self.executor.enter(|| {
            ConnectivityManager::new(
                self.network_context.clone(),
                trusted_peers,
                seed_peers,
                interval(Duration::from_millis(connectivity_check_interval_ms)).fuse(),
                ConnectionRequestSender::new(self.connection_reqs_tx.clone()),
                pm_conn_mgr_notifs_rx,
                conn_mgr_reqs_rx,
                ExponentialBackoff::from_millis(2).factor(1000),
                max_connection_delay_ms,
            )
        });
        self.executor.spawn(conn_mgr.start());
        self
    }

    /// Add the (gossip) [`Discovery`] protocol to the network.
    ///
    /// (gossip) [`Discovery`] discovers other eligible peers' network addresses
    /// by exchanging the full set of known peer network addresses with connected
    /// peers as a network protocol.
    ///
    /// This is for testing purposes only and should not be used in production networks.
    pub fn add_gossip_discovery(&mut self) -> &mut Self {
        let conn_mgr_reqs_tx = self
            .conn_mgr_reqs_tx()
            .expect("ConnectivityManager not enabled");
        // Get handles for network events and sender.
        let (discovery_network_tx, discovery_network_rx) =
            self.add_protocol_handler(discovery::network_endpoint_config());

        // TODO(philiphayes): the current setup for gossip discovery doesn't work
        // when we don't have an `advertised_address` set, since it uses the
        // `listen_address`, which might not be bound to a port yet. For example,
        // if our `listen_address` is "/ip6/::1/tcp/0" and `advertised_address` is
        // `None`, then this will set our `advertised_address` to something like
        // "/ip6/::1/tcp/0/ln-noise-ik/<pubkey>/ln-handshake/0", which is wrong
        // since the actual bound port will be something > 0.

        // TODO(philiphayes): in network_builder setup, only bind the channels.
        // wait until PeerManager is running to actual setup gossip discovery.

        let advertised_address = self
            .advertised_address
            .clone()
            .unwrap_or_else(|| self.listen_address.clone());
        let authentication_mode = self
            .authentication_mode
            .as_ref()
            .expect("Authentication Mode not set");
        let pubkey = authentication_mode.public_key();
        let advertised_address = advertised_address.append_prod_protos(pubkey, HANDSHAKE_VERSION);

        let addrs = vec![advertised_address];
        let discovery_interval_ms = self.discovery_interval_ms;
        let discovery = self.executor.enter(|| {
            Discovery::new(
                self.network_context.clone(),
                addrs,
                interval(Duration::from_millis(discovery_interval_ms)).fuse(),
                discovery_network_tx,
                discovery_network_rx,
                conn_mgr_reqs_tx,
            )
        });
        self.executor.spawn(discovery.start());
        debug!("{} Started discovery protocol actor", self.network_context);
        self
    }

    pub fn add_connection_monitoring(&mut self) -> &mut Self {
        // Initialize and start HealthChecker.
        let (hc_network_tx, hc_network_rx) =
            self.add_protocol_handler(health_checker::network_endpoint_config());
        let ping_interval_ms = self.ping_interval_ms;
        let ping_timeout_ms = self.ping_timeout_ms;
        let ping_failures_tolerated = self.ping_failures_tolerated;
        let health_checker = self.executor.enter(|| {
            HealthChecker::new(
                self.network_context.clone(),
                interval(Duration::from_millis(ping_interval_ms)).fuse(),
                hc_network_tx,
                hc_network_rx,
                Duration::from_millis(ping_timeout_ms),
                ping_failures_tolerated,
            )
        });
        self.executor.spawn(health_checker.start());
        debug!("{} Started health checker", self.network_context);
        self
    }

    /// Create the configured transport and start PeerManager.
    /// Return the actual NetworkAddress over which this peer is listening.
    pub fn build(mut self) -> NetworkAddress {
        use libra_network_address::Protocol::*;

        let chain_id = self.chain_id.clone();
        let network_id = self.network_context.network_id().clone();
        let protos = self.supported_protocols();

        let authentication_mode = self
            .authentication_mode
            .take()
            .expect("Authentication Mode not set");

        let (key, maybe_trusted_peers, peer_id) = match authentication_mode {
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
            [Ip4(_), Tcp(_)] | [Ip6(_), Tcp(_)] => {
                self.build_with_transport(LibraNetTransport::new(
                    LIBRA_TCP_TRANSPORT.clone(),
                    peer_id,
                    key,
                    maybe_trusted_peers,
                    HANDSHAKE_VERSION,
                    chain_id,
                    network_id,
                    protos,
                ))
            }
            [Memory(_)] => self.build_with_transport(LibraNetTransport::new(
                memory::MemoryTransport,
                peer_id,
                key,
                maybe_trusted_peers,
                HANDSHAKE_VERSION,
                chain_id,
                network_id,
                protos,
            )),
            _ => panic!(
                "{} Unsupported listen_address: '{}', expected '/memory/<port>', \
                 '/ip4/<addr>/tcp/<port>', or '/ip6/<addr>/tcp/<port>'.",
                self.network_context, self.listen_address
            ),
        }
    }

    /// Given a transport build and launch PeerManager.
    /// Return the actual NetworkAddress over which this peer is listening.
    fn build_with_transport<TTransport, TSocket>(self, transport: TTransport) -> NetworkAddress
    where
        TTransport: Transport<Output = Connection<TSocket>> + Send + 'static,
        TSocket: transport::TSocket,
    {
        let peer_mgr = PeerManager::new(
            self.executor.clone(),
            transport,
            self.network_context.clone(),
            // TODO(philiphayes): peer manager should take `Vec<NetworkAddress>`
            // (which could be empty, like in client use case)
            self.listen_address,
            self.pm_reqs_rx,
            self.connection_reqs_rx,
            self.upstream_handlers,
            self.connection_event_handlers,
            self.max_concurrent_network_reqs,
            self.max_concurrent_network_notifs,
            self.channel_size,
        );
        let listen_addr = peer_mgr.listen_addr().clone();

        self.executor.spawn(peer_mgr.start());
        debug!("{} Started peer manager", self.network_context);

        listen_addr
    }

    /// Adds a endpoints for the provided configuration.  Returns NetworkSender and NetworkEvent which
    /// can be attached to other components.
    pub fn add_protocol_handler<SenderT, EventT>(
        &mut self,
        (rpc_protocols, direct_send_protocols, queue_preference, max_queue_size_per_peer, counter): (
            Vec<ProtocolId>,
            Vec<ProtocolId>,
            QueueStyle,
            usize,
            Option<&'static IntCounterVec>,
        ),
    ) -> (SenderT, EventT)
    where
        EventT: NewNetworkEvents,
        SenderT: NewNetworkSender,
    {
        let (peer_mgr_reqs_tx, peer_mgr_reqs_rx, connection_reqs_tx, connection_notifs_rx) = self
            .inner_add_protocol_handler(
                rpc_protocols,
                direct_send_protocols,
                queue_preference,
                max_queue_size_per_peer,
                counter,
            );
        (
            SenderT::new(peer_mgr_reqs_tx, connection_reqs_tx),
            EventT::new(peer_mgr_reqs_rx, connection_notifs_rx),
        )
    }
}

pub fn setup_network(
    chain_id: &ChainId,
    role: RoleType,
    config: &mut NetworkConfig,
    libra_db: Arc<dyn DbReader>,
    waypoint: Waypoint,
) -> (Runtime, NetworkBuilder) {
    let runtime = Builder::new()
        .thread_name("network-")
        .threaded_scheduler()
        .enable_all()
        .build()
        .expect("Failed to start runtime. Won't be able to start networking.");

    let identity_key = identity_key(config);
    let peer_id = peer_id(config);

    let mut network_builder = NetworkBuilder::new(
        runtime.handle().clone(),
        chain_id.clone(),
        config.network_id.clone(),
        role,
        peer_id,
        config.listen_address.clone(),
    );
    network_builder.add_connection_monitoring();

    // Sanity check seed peer addresses.
    config
        .verify_seed_peer_addrs()
        .expect("Seed peer addresses must be well-formed");
    let seed_peers = config.seed_peers.clone();

    if config.mutual_authentication {
        let network_peers = config.network_peers.clone();
        let trusted_peers = if role == RoleType::Validator {
            // for validators, trusted_peers is empty will be populated from consensus
            HashMap::new()
        } else {
            network_peers
        };

        info!(
            "network setup: role: {}, seed_peers: {:?}, trusted_peers: {:?}",
            role, seed_peers, trusted_peers,
        );

        network_builder
            .authentication_mode(AuthenticationMode::Mutual(identity_key))
            .trusted_peers(trusted_peers)
            .seed_peers(seed_peers)
            .connectivity_check_interval_ms(config.connectivity_check_interval_ms)
            .add_connectivity_manager();
    } else {
        // Enforce the outgoing connection (dialer) verifies the identity of the listener (server)
        network_builder.authentication_mode(AuthenticationMode::ServerOnly(identity_key));
        if !seed_peers.is_empty() {
            network_builder
                .seed_peers(seed_peers)
                .add_connectivity_manager();
        }
    }

    match &config.discovery_method {
        DiscoveryMethod::Gossip(gossip_config) => {
            network_builder
                .advertised_address(gossip_config.advertised_address.clone())
                .discovery_interval_ms(gossip_config.discovery_interval_ms)
                .add_gossip_discovery();
        }
        DiscoveryMethod::Onchain => {
            let (network_tx, discovery_events) = network_builder.add_protocol_handler(
                onchain_discovery::network_interface::network_endpoint_config(),
            );
            let onchain_discovery_builder = OnchainDiscoveryBuilder::build(
                network_builder
                    .conn_mgr_reqs_tx()
                    .expect("ConnectivityManager must be installed"),
                network_tx,
                discovery_events,
                network_builder.network_context().clone(),
                libra_db,
                waypoint,
                runtime.handle(),
            );
            onchain_discovery_builder.start(runtime.handle());
        }
        DiscoveryMethod::None => {}
    }

    (runtime, network_builder)
}

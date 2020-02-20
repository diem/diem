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
use crate::{
    common::NetworkPublicKeys,
    connectivity_manager::{ConnectivityManager, ConnectivityRequest},
    counters,
    peer_manager::{
        conn_status_channel, PeerManager, PeerManagerNotification, PeerManagerRequest,
        PeerManagerRequestSender,
    },
    protocols::{
        discovery::{Discovery, PeerInfo},
        health_checker::HealthChecker,
        identity::Identity,
    },
    transport::*,
    validator_network, ProtocolId,
};
use channel::message_queues::QueueStyle;
use channel::{self, libra_channel};
use futures::stream::StreamExt;
use libra_config::config::RoleType;
use libra_crypto::{
    ed25519::*,
    x25519::{X25519StaticPrivateKey, X25519StaticPublicKey},
};
use libra_logger::prelude::*;
use libra_metrics::IntCounterVec;
use libra_types::{validator_signer::ValidatorSigner, PeerId};
use netcore::multiplexing::StreamMultiplexer;
use netcore::transport::Transport;
use parity_multiaddr::Multiaddr;
use std::clone::Clone;
use std::iter::FromIterator;
use std::{
    collections::{HashMap, HashSet},
    num::NonZeroUsize,
    sync::{Arc, RwLock},
    time::Duration,
};
use tokio::runtime::Handle;
use tokio::time::interval;
use tokio_retry::strategy::ExponentialBackoff;

pub const NETWORK_CHANNEL_SIZE: usize = 1024;
pub const DISCOVERY_INTERVAL_MS: u64 = 1000;
pub const PING_INTERVAL_MS: u64 = 1000;
pub const PING_TIMEOUT_MS: u64 = 10_000;
pub const DISOVERY_MSG_TIMEOUT_MS: u64 = 10_000;
pub const CONNECTIVITY_CHECK_INTERNAL_MS: u64 = 5000;
pub const INBOUND_RPC_TIMEOUT_MS: u64 = 10_000;
pub const MAX_CONCURRENT_OUTBOUND_RPCS: u32 = 100;
pub const MAX_CONCURRENT_INBOUND_RPCS: u32 = 100;
pub const PING_FAILURES_TOLERATED: u64 = 10;
pub const MAX_CONCURRENT_NETWORK_REQS: usize = 100;
pub const MAX_CONCURRENT_NETWORK_NOTIFS: usize = 100;
pub const MAX_CONNECTION_DELAY_MS: u64 = 10 * 60 * 1000 /* 10 minutes */;

/// The type of the transport layer, i.e., running on memory or TCP stream,
/// with or without Noise encryption
pub enum TransportType {
    Memory,
    MemoryNoise(Option<(X25519StaticPrivateKey, X25519StaticPublicKey)>),
    PermissionlessMemoryNoise(Option<(X25519StaticPrivateKey, X25519StaticPublicKey)>),
    Tcp,
    TcpNoise(Option<(X25519StaticPrivateKey, X25519StaticPublicKey)>),
    PermissionlessTcpNoise(Option<(X25519StaticPrivateKey, X25519StaticPublicKey)>),
}

/// Build Network module with custom configuration values.
/// Methods can be chained in order to set the configuration values.
/// MempoolNetworkHandler and ConsensusNetworkHandler are constructed by calling
/// [`NetworkBuilder::build`].  New instances of `NetworkBuilder` are obtained
/// via [`NetworkBuilder::new`].
pub struct NetworkBuilder {
    executor: Handle,
    peer_id: PeerId,
    addr: Multiaddr,
    role: RoleType,
    advertised_address: Option<Multiaddr>,
    seed_peers: HashMap<PeerId, PeerInfo>,
    trusted_peers: Arc<RwLock<HashMap<PeerId, NetworkPublicKeys>>>,
    transport: TransportType,
    channel_size: usize,
    direct_send_protocols: Vec<ProtocolId>,
    rpc_protocols: Vec<ProtocolId>,
    discovery_interval_ms: u64,
    ping_interval_ms: u64,
    ping_timeout_ms: u64,
    ping_failures_tolerated: u64,
    upstream_handlers:
        HashMap<ProtocolId, libra_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>>,
    connection_event_handlers: Vec<conn_status_channel::Sender>,
    pm_reqs_tx: libra_channel::Sender<(PeerId, ProtocolId), PeerManagerRequest>,
    pm_reqs_rx: libra_channel::Receiver<(PeerId, ProtocolId), PeerManagerRequest>,
    conn_mgr_reqs_tx: Option<channel::Sender<ConnectivityRequest>>,
    connectivity_check_interval_ms: u64,
    inbound_rpc_timeout_ms: u64,
    max_concurrent_outbound_rpcs: u32,
    max_concurrent_inbound_rpcs: u32,
    max_concurrent_network_reqs: usize,
    max_concurrent_network_notifs: usize,
    max_connection_delay_ms: u64,
    signing_keys: Option<(Ed25519PrivateKey, Ed25519PublicKey)>,
    enable_remote_authentication: bool,
}

impl NetworkBuilder {
    /// Return a new NetworkBuilder initialized with default configuration values.
    pub fn new(
        executor: Handle,
        peer_id: PeerId,
        addr: Multiaddr,
        role: RoleType,
    ) -> NetworkBuilder {
        // Setup channel to send requests to peer manager.
        let (pm_reqs_tx, pm_reqs_rx) = libra_channel::new(
            QueueStyle::FIFO,
            NonZeroUsize::new(NETWORK_CHANNEL_SIZE).unwrap(),
            Some(&counters::PENDING_PEER_MANAGER_REQUESTS),
        );
        NetworkBuilder {
            executor,
            peer_id,
            addr,
            role,
            advertised_address: None,
            seed_peers: HashMap::new(),
            trusted_peers: Arc::new(RwLock::new(HashMap::new())),
            channel_size: NETWORK_CHANNEL_SIZE,
            direct_send_protocols: vec![],
            rpc_protocols: vec![],
            upstream_handlers: HashMap::new(),
            connection_event_handlers: Vec::new(),
            pm_reqs_tx,
            pm_reqs_rx,
            conn_mgr_reqs_tx: None,
            transport: TransportType::Memory,
            discovery_interval_ms: DISCOVERY_INTERVAL_MS,
            ping_interval_ms: PING_INTERVAL_MS,
            ping_timeout_ms: PING_TIMEOUT_MS,
            ping_failures_tolerated: PING_FAILURES_TOLERATED,
            connectivity_check_interval_ms: CONNECTIVITY_CHECK_INTERNAL_MS,
            inbound_rpc_timeout_ms: INBOUND_RPC_TIMEOUT_MS,
            max_concurrent_outbound_rpcs: MAX_CONCURRENT_OUTBOUND_RPCS,
            max_concurrent_inbound_rpcs: MAX_CONCURRENT_INBOUND_RPCS,
            max_concurrent_network_reqs: MAX_CONCURRENT_NETWORK_REQS,
            max_concurrent_network_notifs: MAX_CONCURRENT_NETWORK_NOTIFS,
            max_connection_delay_ms: MAX_CONNECTION_DELAY_MS,
            signing_keys: None,
            enable_remote_authentication: true,
        }
    }

    /// Set transport type, i.e., Memory or Tcp transports.
    pub fn transport(&mut self, transport: TransportType) -> &mut Self {
        self.transport = transport;
        self
    }

    /// Set and address to advertise, if different from the listen address
    pub fn advertised_address(&mut self, advertised_address: Multiaddr) -> &mut Self {
        self.advertised_address = Some(advertised_address);
        self
    }

    /// Set trusted peers.
    pub fn trusted_peers(
        &mut self,
        trusted_peers: HashMap<PeerId, NetworkPublicKeys>,
    ) -> &mut Self {
        *self.trusted_peers.write().unwrap() = trusted_peers;
        self
    }

    /// Set signing keys of local node.
    pub fn signing_keys(&mut self, keys: (Ed25519PrivateKey, Ed25519PublicKey)) -> &mut Self {
        self.signing_keys = Some(keys);
        self
    }

    /// Set seed peers to bootstrap discovery
    pub fn seed_peers(&mut self, seed_peers: HashMap<PeerId, Vec<Multiaddr>>) -> &mut Self {
        self.seed_peers = seed_peers
            .into_iter()
            .map(|(peer_id, seed_addrs)| (peer_id, PeerInfo::new(seed_addrs, 0)))
            .collect();
        self
    }

    /// Set discovery ticker interval
    pub fn discovery_interval_ms(&mut self, discovery_interval_ms: u64) -> &mut Self {
        self.discovery_interval_ms = discovery_interval_ms;
        self
    }

    /// Set ping interval.
    pub fn ping_interval_ms(&mut self, ping_interval_ms: u64) -> &mut Self {
        self.ping_interval_ms = ping_interval_ms;
        self
    }

    /// Set number of ping failures tolerated.
    pub fn ping_failures_tolerated(&mut self, ping_failures_tolerated: u64) -> &mut Self {
        self.ping_failures_tolerated = ping_failures_tolerated;
        self
    }

    /// Set ping timeout.
    pub fn ping_timeout_ms(&mut self, ping_timeout_ms: u64) -> &mut Self {
        self.ping_timeout_ms = ping_timeout_ms;
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

    /// Set inbound rpc timeout.
    pub fn inbound_rpc_timeout_ms(&mut self, inbound_rpc_timeout_ms: u64) -> &mut Self {
        self.inbound_rpc_timeout_ms = inbound_rpc_timeout_ms;
        self
    }

    /// The maximum number of concurrent outbound rpc requests we will service.
    pub fn max_concurrent_outbound_rpcs(&mut self, max_concurrent_outbound_rpcs: u32) -> &mut Self {
        self.max_concurrent_outbound_rpcs = max_concurrent_outbound_rpcs;
        self
    }

    /// The maximum number of concurrent inbound rpc requests we will service.
    pub fn max_concurrent_inbound_rpcs(&mut self, max_concurrent_inbound_rpcs: u32) -> &mut Self {
        self.max_concurrent_inbound_rpcs = max_concurrent_inbound_rpcs;
        self
    }

    /// The maximum number of concurrent NetworkRequests we will service in NetworkProvider.
    pub fn max_concurrent_network_reqs(&mut self, max_concurrent_network_reqs: usize) -> &mut Self {
        self.max_concurrent_network_reqs = max_concurrent_network_reqs;
        self
    }

    /// The maximum number of concurrent Notifications from each actor we will service in
    /// NetworkProvider.
    pub fn max_concurrent_network_notifs(
        &mut self,
        max_concurrent_network_notifs: usize,
    ) -> &mut Self {
        self.max_concurrent_network_notifs = max_concurrent_network_notifs;
        self
    }

    /// The maximum duration (in milliseconds) we should wait before dialing a peer we should
    /// connect to.
    pub fn max_connection_delay_ms(&mut self, max_connection_delay_ms: u64) -> &mut Self {
        self.max_connection_delay_ms = max_connection_delay_ms;
        self
    }

    /// Set the size of the channels between different network actors.
    pub fn channel_size(&mut self, channel_size: usize) -> &mut Self {
        self.channel_size = channel_size;
        self
    }

    /// Set the enable_remote_authentication flag to make the network operate with remote authentication.
    pub fn enable_remote_authentication(
        &mut self,
        enable_remote_authentication: bool,
    ) -> &mut Self {
        self.enable_remote_authentication = enable_remote_authentication;
        self
    }

    pub fn conn_mgr_reqs_tx(&self) -> Option<channel::Sender<ConnectivityRequest>> {
        self.conn_mgr_reqs_tx.clone()
    }

    fn supported_protocols(&self) -> Vec<ProtocolId> {
        self.direct_send_protocols
            .iter()
            .chain(&self.rpc_protocols)
            .cloned()
            .collect()
    }

    /// Add a handler for given protocols using raw bytes.
    pub fn add_protocol_handler(
        &mut self,
        rpc_protocols: Vec<ProtocolId>,
        direct_send_protocols: Vec<ProtocolId>,
        queue_preference: QueueStyle,
        counter: Option<&'static IntCounterVec>,
    ) -> (
        libra_channel::Sender<(PeerId, ProtocolId), PeerManagerRequest>,
        libra_channel::Receiver<(PeerId, ProtocolId), PeerManagerNotification>,
        conn_status_channel::Receiver,
    ) {
        self.direct_send_protocols
            .extend(direct_send_protocols.clone());
        self.rpc_protocols.extend(rpc_protocols.clone());
        let (network_notifs_tx, network_notifs_rx) = libra_channel::new(
            queue_preference,
            NonZeroUsize::new(self.channel_size).unwrap(),
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
        let (control_notifs_tx, control_notifs_rx) = conn_status_channel::new();
        // Auto-subscribe all application level handlers to connection events.
        self.connection_event_handlers.push(control_notifs_tx);
        (
            self.pm_reqs_tx.clone(),
            network_notifs_rx,
            control_notifs_rx,
        )
    }

    pub fn add_connection_event_listener(
        &mut self,
    ) -> (
        libra_channel::Sender<(PeerId, ProtocolId), PeerManagerRequest>,
        conn_status_channel::Receiver,
    ) {
        let (tx, rx) = conn_status_channel::new();
        self.connection_event_handlers.push(tx);
        (self.pm_reqs_tx.clone(), rx)
    }

    pub fn add_discovery(&mut self) -> &mut Self {
        // We start the connectivity_manager module only if the network is
        // permissioned.
        // Initialize and start connectivity manager.
        let (conn_mgr_reqs_tx, conn_mgr_reqs_rx) = channel::new(
            self.channel_size,
            &counters::PENDING_CONNECTIVITY_MANAGER_REQUESTS,
        );
        self.conn_mgr_reqs_tx = Some(conn_mgr_reqs_tx.clone());
        // peer_event_handlers.push(pm_conn_mgr_notifs_tx);
        let trusted_peers = self.trusted_peers.clone();
        let max_connection_delay_ms = self.max_connection_delay_ms;
        let connectivity_check_interval_ms = self.connectivity_check_interval_ms;
        let (pm_reqs_tx, pm_conn_mgr_notifs_rx) = self.add_connection_event_listener();
        let conn_mgr = self.executor.enter(|| {
            ConnectivityManager::new(
                trusted_peers,
                interval(Duration::from_millis(connectivity_check_interval_ms)).fuse(),
                PeerManagerRequestSender::new(pm_reqs_tx),
                pm_conn_mgr_notifs_rx,
                conn_mgr_reqs_rx,
                ExponentialBackoff::from_millis(2).factor(1000),
                max_connection_delay_ms,
            )
        });
        self.executor.spawn(conn_mgr.start());

        // We start the discovery module only if the network is permissioned.
        // Note: We use the `enable_remote_authentication` flag as a proxy for whether we need to run the
        // discovery module or not. We should make this more explicit eventually.
        // Initialize and start Discovery actor.
        let (signing_private_key, _signing_public_key) =
            self.signing_keys.take().expect("Signing keys not set");
        // Setup signer from keys.
        let signer = ValidatorSigner::new(self.peer_id, signing_private_key);
        // Get handles for network events and sender.
        let (discovery_network_tx, discovery_network_rx) =
            validator_network::discovery::add_to_network(self);
        let peer_id = self.peer_id;
        let addrs = vec![self
            .advertised_address
            .clone()
            .unwrap_or_else(|| self.addr.clone())];
        let seed_peers = self.seed_peers.clone();
        let trusted_peers = self.trusted_peers.clone();
        let role = self.role;
        let discovery_interval_ms = self.discovery_interval_ms;
        let discovery = self.executor.enter(|| {
            Discovery::new(
                peer_id,
                role,
                addrs,
                signer,
                seed_peers,
                trusted_peers,
                interval(Duration::from_millis(discovery_interval_ms)).fuse(),
                discovery_network_tx,
                discovery_network_rx,
                conn_mgr_reqs_tx,
            )
        });
        self.executor.spawn(discovery.start());
        debug!("Started discovery protocol actor");
        self
    }

    pub fn add_connection_monitoring(&mut self) -> &mut Self {
        // Initialize and start HealthChecker.
        let (hc_network_tx, hc_network_rx) =
            validator_network::health_checker::add_to_network(self);
        let ping_interval_ms = self.ping_interval_ms;
        let ping_timeout_ms = self.ping_timeout_ms;
        let ping_failures_tolerated = self.ping_failures_tolerated;
        let health_checker = self.executor.enter(|| {
            HealthChecker::new(
                interval(Duration::from_millis(ping_interval_ms)).fuse(),
                hc_network_tx,
                hc_network_rx,
                Duration::from_millis(ping_timeout_ms),
                ping_failures_tolerated,
            )
        });
        self.executor.spawn(health_checker.start());
        debug!("Started health checker");
        self
    }

    /// Create the configured transport and start PeerManager.
    /// Return the actual Multiaddr over which this peer is listening.
    pub fn build(mut self) -> Multiaddr {
        let identity = Identity::new(self.peer_id, self.supported_protocols(), self.role);
        // Build network based on the transport type
        let trusted_peers = self.trusted_peers.clone();
        match self.transport {
            TransportType::Memory => self.build_with_transport(build_memory_transport(identity)),
            TransportType::MemoryNoise(ref mut keys) => {
                let keys = keys.take().expect("Identity keys not set");
                self.build_with_transport(build_memory_noise_transport(
                    identity,
                    keys,
                    trusted_peers,
                ))
            }
            TransportType::PermissionlessMemoryNoise(ref mut keys) => {
                let keys = keys.take().expect("Identity keys not set");
                self.build_with_transport(build_unauthenticated_memory_noise_transport(
                    identity, keys,
                ))
            }
            TransportType::Tcp => self.build_with_transport(build_tcp_transport(identity)),
            TransportType::TcpNoise(ref mut keys) => {
                let keys = keys.take().expect("Identity keys not set");
                self.build_with_transport(build_tcp_noise_transport(identity, keys, trusted_peers))
            }
            TransportType::PermissionlessTcpNoise(ref mut keys) => {
                let keys = keys.take().expect("Identity keys not set");
                self.build_with_transport(build_unauthenticated_tcp_noise_transport(identity, keys))
            }
        }
    }

    /// Given a transport build and launch PeerManager.
    /// Return the actual Multiaddr over which this peer is listening.
    fn build_with_transport<TTransport, TMuxer>(self, transport: TTransport) -> Multiaddr
    where
        TTransport: Transport<Output = (Identity, TMuxer)> + Send + 'static,
        TMuxer: StreamMultiplexer + 'static,
    {
        let peer_mgr = PeerManager::new(
            self.executor.clone(),
            transport,
            self.peer_id,
            self.addr,
            self.pm_reqs_rx,
            HashSet::from_iter(self.rpc_protocols.into_iter()),
            HashSet::from_iter(self.direct_send_protocols.into_iter()),
            self.upstream_handlers,
            self.connection_event_handlers,
            self.max_concurrent_network_reqs,
            self.max_concurrent_network_notifs,
            self.channel_size,
        );
        let listen_addr = peer_mgr.listen_addr().clone();
        self.executor.spawn(peer_mgr.start());
        debug!("Started peer manager");
        listen_addr
    }
}

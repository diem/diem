// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Permission-less vs Permissioned network end-points:
//! ---------------------------------------------------
//! A network end-point is permissioned if it only wants to accept connections from a known set
//! of peers (`trusted_peers`) identified by their network identity keys. This does not mean
//! that the other end-point of a connection also needs to run in permissioned mode --
//! a network end-point running in permissioned mode will connect to or accept connections from
//! an end-point running in permissionless mode as long as the latter is in its trusted peers
//! set.
use crate::{
    common::NetworkPublicKeys,
    connectivity_manager::ConnectivityManager,
    counters,
    interface::{LibraNetworkProvider, NetworkProvider},
    peer_manager::{PeerManager, PeerManagerRequestSender},
    proto::PeerInfo,
    protocols::{
        direct_send::DirectSend, discovery::Discovery, health_checker::HealthChecker,
        identity::Identity, rpc::Rpc,
    },
    transport::*,
    validator_network::{DISCOVERY_DIRECT_SEND_PROTOCOL, HEALTH_CHECKER_RPC_PROTOCOL},
    ProtocolId,
};
use channel;
use futures::StreamExt;
use libra_config::config::RoleType;
use libra_crypto::{
    ed25519::*,
    x25519::{X25519StaticPrivateKey, X25519StaticPublicKey},
};
use libra_logger::prelude::*;
use libra_types::{validator_signer::ValidatorSigner, PeerId};
use netcore::{multiplexing::StreamMultiplexer, transport::boxed::BoxedTransport};
use parity_multiaddr::Multiaddr;
use std::{
    collections::HashMap,
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
pub const MAX_CONCURRENT_NETWORK_REQS: u32 = 100;
pub const MAX_CONCURRENT_NETWORK_NOTIFS: u32 = 100;
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
    discovery_msg_timeout_ms: u64,
    ping_interval_ms: u64,
    ping_timeout_ms: u64,
    ping_failures_tolerated: u64,
    connectivity_check_interval_ms: u64,
    inbound_rpc_timeout_ms: u64,
    max_concurrent_outbound_rpcs: u32,
    max_concurrent_inbound_rpcs: u32,
    max_concurrent_network_reqs: u32,
    max_concurrent_network_notifs: u32,
    max_connection_delay_ms: u64,
    signing_keys: Option<(Ed25519PrivateKey, Ed25519PublicKey)>,
    is_permissioned: bool,
    health_checker_enabled: bool,
}

impl NetworkBuilder {
    /// Return a new NetworkBuilder initialized with default configuration values.
    pub fn new(
        executor: Handle,
        peer_id: PeerId,
        addr: Multiaddr,
        role: RoleType,
    ) -> NetworkBuilder {
        NetworkBuilder {
            executor,
            peer_id,
            addr,
            role,
            advertised_address: None,
            seed_peers: HashMap::new(),
            trusted_peers: Arc::new(RwLock::new(HashMap::new())),
            channel_size: NETWORK_CHANNEL_SIZE,
            direct_send_protocols: vec![ProtocolId::from_static(DISCOVERY_DIRECT_SEND_PROTOCOL)],
            rpc_protocols: vec![ProtocolId::from_static(HEALTH_CHECKER_RPC_PROTOCOL)],
            transport: TransportType::Memory,
            discovery_interval_ms: DISCOVERY_INTERVAL_MS,
            discovery_msg_timeout_ms: DISOVERY_MSG_TIMEOUT_MS,
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
            is_permissioned: true,
            health_checker_enabled: true,
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
            .map(|(peer_id, seed_addrs)| {
                let mut peer_info = PeerInfo::default();
                peer_info.epoch = 0;
                peer_info.addrs = seed_addrs
                    .into_iter()
                    .map(|addr| addr.as_ref().into())
                    .collect();
                (peer_id, peer_info)
            })
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

    /// Set discovery message timeout.
    pub fn discovery_msg_timeout_ms(&mut self, discovery_msg_timeout_ms: u64) -> &mut Self {
        self.discovery_msg_timeout_ms = discovery_msg_timeout_ms;
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
    pub fn max_concurrent_network_reqs(&mut self, max_concurrent_network_reqs: u32) -> &mut Self {
        self.max_concurrent_network_reqs = max_concurrent_network_reqs;
        self
    }

    /// The maximum number of concurrent Notifications from each actor we will service in
    /// NetworkProvider.
    pub fn max_concurrent_network_notifs(
        &mut self,
        max_concurrent_network_notifs: u32,
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

    /// Set the protocol IDs that DirectSend actor subscribes.
    pub fn direct_send_protocols(&mut self, protocols: Vec<ProtocolId>) -> &mut Self {
        self.direct_send_protocols = protocols;
        self.direct_send_protocols
            .push(ProtocolId::from_static(DISCOVERY_DIRECT_SEND_PROTOCOL));
        self
    }

    /// Set the protocol IDs that RPC actor subscribes.
    pub fn rpc_protocols(&mut self, protocols: Vec<ProtocolId>) -> &mut Self {
        self.rpc_protocols = protocols;
        self.rpc_protocols
            .push(ProtocolId::from_static(HEALTH_CHECKER_RPC_PROTOCOL));
        self
    }

    /// Set the is_permissioned flag to make the network permissioned or permission-less.
    pub fn permissioned(&mut self, is_permissioned: bool) -> &mut Self {
        self.is_permissioned = is_permissioned;
        self
    }

    fn supported_protocols(&self) -> Vec<ProtocolId> {
        self.direct_send_protocols
            .iter()
            .chain(&self.rpc_protocols)
            .cloned()
            .collect()
    }

    /// Enable or disable the health checker protocol in this network instance.
    // TODO(philiphayes): remember to remove this
    #[allow(dead_code)]
    fn health_checker_enabled(&mut self, enabled: bool) -> &mut Self {
        self.health_checker_enabled = enabled;
        let health_checker_protocol = ProtocolId::from_static(HEALTH_CHECKER_RPC_PROTOCOL);
        if enabled {
            self.rpc_protocols.push(health_checker_protocol);
        } else {
            // TODO(philiphayes): replace with `Vec::remove_item` when it's stable.
            let maybe_idx = self
                .rpc_protocols
                .iter()
                .position(|x| *x == health_checker_protocol);
            maybe_idx.map(|idx| self.rpc_protocols.remove(idx));
        }
        self
    }

    /// Create the configured `NetworkBuilder`
    /// Return the constructed Mempool and Consensus Sender+Events
    pub fn build(&mut self) -> (Multiaddr, Box<dyn LibraNetworkProvider>) {
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
                self.build_with_transport(build_permissionless_memory_noise_transport(
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
                self.build_with_transport(build_permissionless_tcp_noise_transport(identity, keys))
            }
        }
    }

    /// Given a transport build and launch the NetworkProvider and all subcomponents
    /// Return the constructed Mempool and Consensus Sender+Events
    fn build_with_transport(
        &mut self,
        transport: BoxedTransport<
            (Identity, impl StreamMultiplexer + 'static),
            impl ::std::error::Error + Send + Sync + 'static,
        >,
    ) -> (Multiaddr, Box<dyn LibraNetworkProvider>) {
        // Initialize lists of protocol handlers and peer event handlers.
        let mut peer_event_handlers = vec![];
        let mut protocol_handlers = HashMap::new();
        // Setup channel to send requests to peer manager.
        let (pm_reqs_tx, pm_reqs_rx) =
            channel::new(self.channel_size, &counters::PENDING_PEER_MANAGER_REQUESTS);

        // Initialize and start DirectSend actor.
        let (pm_ds_notifs_tx, pm_ds_notifs_rx) = channel::new(
            self.channel_size,
            &counters::PENDING_PEER_MANAGER_DIRECT_SEND_NOTIFICATIONS,
        );
        let direct_send_handlers = self
            .direct_send_protocols
            .iter()
            .map(|p| (p.clone(), pm_ds_notifs_tx.clone()));
        protocol_handlers.extend(direct_send_handlers);
        let (ds_reqs_tx, ds_reqs_rx) =
            channel::new(self.channel_size, &counters::PENDING_DIRECT_SEND_REQUESTS);
        let (ds_net_notifs_tx, ds_net_notifs_rx) = channel::new(
            self.channel_size,
            &counters::PENDING_DIRECT_SEND_NOTIFICATIONS,
        );
        let ds = DirectSend::new(
            self.executor.clone(),
            ds_reqs_rx,
            ds_net_notifs_tx,
            pm_ds_notifs_rx,
            PeerManagerRequestSender::new(pm_reqs_tx.clone()),
        );
        self.executor.spawn(ds.start());
        debug!("Started direct send actor");

        // Initialize and start RPC actor.
        let (pm_rpc_notifs_tx, pm_rpc_notifs_rx) = channel::new(
            self.channel_size,
            &counters::PENDING_PEER_MANAGER_RPC_NOTIFICATIONS,
        );
        let rpc_handlers = self
            .rpc_protocols
            .iter()
            .map(|p| (p.clone(), pm_rpc_notifs_tx.clone()));
        protocol_handlers.extend(rpc_handlers);
        let (rpc_net_notifs_tx, rpc_net_notifs_rx) =
            channel::new(self.channel_size, &counters::PENDING_RPC_NOTIFICATIONS);
        let (rpc_reqs_tx, rpc_reqs_rx) =
            channel::new(self.channel_size, &counters::PENDING_RPC_REQUESTS);
        let rpc = Rpc::new(
            self.executor.clone(),
            rpc_reqs_rx,
            pm_rpc_notifs_rx,
            PeerManagerRequestSender::new(pm_reqs_tx.clone()),
            rpc_net_notifs_tx,
            Duration::from_millis(self.inbound_rpc_timeout_ms),
            self.max_concurrent_outbound_rpcs,
            self.max_concurrent_inbound_rpcs,
        );
        self.executor.spawn(rpc.start());
        debug!("Started RPC actor");

        // We start the connectivity_manager module only if the network is
        // permissioned.
        let mut net_conn_mgr_reqs_tx = None;
        if self.is_permissioned {
            // Initialize and start connectivity manager.
            let (conn_mgr_reqs_tx, conn_mgr_reqs_rx) = channel::new(
                self.channel_size,
                &counters::PENDING_CONNECTIVITY_MANAGER_REQUESTS,
            );
            net_conn_mgr_reqs_tx = Some(conn_mgr_reqs_tx);
            let (pm_conn_mgr_notifs_tx, pm_conn_mgr_notifs_rx) = channel::new(
                self.channel_size,
                &counters::PENDING_PEER_MANAGER_CONNECTIVITY_MANAGER_NOTIFICATIONS,
            );
            peer_event_handlers.push(pm_conn_mgr_notifs_tx);
            let trusted_peers = self.trusted_peers.clone();
            let max_connection_delay_ms = self.max_connection_delay_ms;
            let connectivity_check_interval_ms = self.connectivity_check_interval_ms;
            let pm_reqs_tx = pm_reqs_tx.clone();
            let f = async move {
                let conn_mgr = ConnectivityManager::new(
                    trusted_peers,
                    interval(Duration::from_millis(connectivity_check_interval_ms)).fuse(),
                    PeerManagerRequestSender::new(pm_reqs_tx.clone()),
                    pm_conn_mgr_notifs_rx,
                    conn_mgr_reqs_rx,
                    ExponentialBackoff::from_millis(2).factor(1000 /* seconds */),
                    max_connection_delay_ms,
                );
                conn_mgr.start().await
            };
            self.executor.spawn(f);
            debug!("Started connection manager");
        }

        let pm_net_reqs_tx = pm_reqs_tx;
        let (pm_net_notifs_tx, pm_net_notifs_rx) = channel::new(
            self.channel_size,
            &counters::PENDING_PEER_MANAGER_NET_NOTIFICATIONS,
        );
        peer_event_handlers.push(pm_net_notifs_tx);
        let peer_mgr = PeerManager::new(
            transport,
            self.executor.clone(),
            self.peer_id,
            self.addr.clone(),
            pm_reqs_rx,
            protocol_handlers,
            peer_event_handlers,
        );
        let listen_addr = peer_mgr.listen_addr().clone();
        self.executor.spawn(peer_mgr.start());
        debug!("Started peer manager");

        // Setup communication channels.
        let (network_reqs_tx, network_reqs_rx) =
            channel::new(self.channel_size, &counters::PENDING_NETWORK_REQUESTS);
        let mut network_provider = NetworkProvider::new(
            pm_net_reqs_tx,
            pm_net_notifs_rx,
            rpc_reqs_tx,
            rpc_net_notifs_rx,
            ds_reqs_tx,
            ds_net_notifs_rx,
            net_conn_mgr_reqs_tx.clone(),
            network_reqs_rx,
            network_reqs_tx,
            self.max_concurrent_network_reqs,
            self.max_concurrent_network_notifs,
            self.channel_size,
        );

        if self.health_checker_enabled {
            // Initialize and start HealthChecker.
            let (hc_network_tx, hc_network_rx) = network_provider
                .add_health_checker(vec![ProtocolId::from_static(HEALTH_CHECKER_RPC_PROTOCOL)]);
            let ping_interval_ms = self.ping_interval_ms;
            let ping_timeout_ms = self.ping_timeout_ms;
            let ping_failures_tolerated = self.ping_failures_tolerated;
            let f = async move {
                let health_checker = HealthChecker::new(
                    interval(Duration::from_millis(ping_interval_ms)).fuse(),
                    hc_network_tx,
                    hc_network_rx,
                    Duration::from_millis(ping_timeout_ms),
                    ping_failures_tolerated,
                );
                health_checker.start().await
            };
            self.executor.spawn(f);
            debug!("Started health checker");
        }

        // We start the discovery module only if the network is permissioned.
        // Note: We use the `is_permissioned` flag as a proxy for whether we need to run the
        // discovery module or not. We should make this more explicit eventually.
        if self.is_permissioned {
            // Initialize and start Discovery actor.
            let (signing_private_key, _signing_public_key) =
                self.signing_keys.take().expect("Signing keys not set");
            // Setup signer from keys.
            let signer = ValidatorSigner::new(self.peer_id, signing_private_key);
            // Get handles for network events and sender.
            let (discovery_network_tx, discovery_network_rx) = network_provider.add_discovery(
                vec![ProtocolId::from_static(DISCOVERY_DIRECT_SEND_PROTOCOL)],
            );
            let peer_id = self.peer_id;
            let role = self.role;
            let addrs = vec![self
                .advertised_address
                .clone()
                .unwrap_or_else(|| self.addr.clone())];
            let seed_peers = self.seed_peers.clone();
            let trusted_peers = self.trusted_peers.clone();
            let discovery_interval_ms = self.discovery_interval_ms;
            let discovery_msg_timeout_ms = self.discovery_msg_timeout_ms;
            let f = async move {
                let discovery = Discovery::new(
                    peer_id,
                    role,
                    addrs,
                    signer,
                    seed_peers,
                    trusted_peers,
                    interval(Duration::from_millis(discovery_interval_ms)).fuse(),
                    discovery_network_tx,
                    discovery_network_rx,
                    net_conn_mgr_reqs_tx.take().unwrap(),
                    Duration::from_millis(discovery_msg_timeout_ms),
                );
                discovery.start().await
            };
            self.executor.spawn(f);
            debug!("Started discovery protocol actor");
        }
        (listen_addr, Box::new(network_provider))
    }
}

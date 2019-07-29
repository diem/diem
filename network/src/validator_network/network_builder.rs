// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    common::NetworkPublicKeys,
    connectivity_manager::ConnectivityManager,
    counters,
    interface::NetworkProvider,
    peer_manager::{PeerManager, PeerManagerRequestSender},
    proto::PeerInfo,
    protocols::{
        direct_send::DirectSend,
        discovery::{Discovery, DISCOVERY_PROTOCOL_NAME},
        health_checker::{HealthChecker, PING_PROTOCOL_NAME},
        identity::Identity,
        rpc::Rpc,
    },
    transport::{
        build_memory_noise_transport, build_memory_transport, build_tcp_noise_transport,
        build_tcp_transport,
    },
    validator_network::{
        ConsensusNetworkEvents, ConsensusNetworkSender, MempoolNetworkEvents, MempoolNetworkSender,
    },
    ProtocolId,
};
use channel;
use crypto::x25519::{X25519PrivateKey, X25519PublicKey};
use futures::{compat::Compat01As03, FutureExt, StreamExt, TryFutureExt};
use netcore::{multiplexing::StreamMultiplexer, transport::boxed::BoxedTransport};
use nextgen_crypto::ed25519::*;
use parity_multiaddr::Multiaddr;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::Duration,
};
use tokio::runtime::TaskExecutor;
use tokio_retry::strategy::ExponentialBackoff;
use tokio_timer::Interval;
use types::{validator_signer::ValidatorSigner, PeerId};

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
pub const CONSENSUS_INBOUND_MSG_TIMOUT_MS: u64 = 60 * 1000; // 1 minute
pub const MEMPOOL_INBOUND_MSG_TIMOUT_MS: u64 = 60 * 1000; // 1 minute

/// The type of the transport layer, i.e., running on memory or TCP stream,
/// with or without Noise encryption
pub enum TransportType {
    Memory,
    MemoryNoise,
    Tcp,
    TcpNoise,
}

/// Build Network module with custom configuration values.
/// Methods can be chained in order to set the configuration values.
/// MempoolNetworkHandler and ConsensusNetworkHandler are constructed by calling
/// [`NetworkBuilder::build`].  New instances of `NetworkBuilder` are obtained
/// via [`NetworkBuilder::new`].
pub struct NetworkBuilder {
    executor: TaskExecutor,
    peer_id: PeerId,
    addr: Multiaddr,
    advertised_address: Option<Multiaddr>,
    seed_peers: HashMap<PeerId, PeerInfo>,
    trusted_peers: Arc<RwLock<HashMap<PeerId, NetworkPublicKeys>>>,
    transport: TransportType,
    channel_size: usize,
    mempool_protocols: Vec<ProtocolId>,
    consensus_protocols: Vec<ProtocolId>,
    direct_send_protocols: Vec<ProtocolId>,
    rpc_protocols: Vec<ProtocolId>,
    discovery_interval_ms: u64,
    discovery_msg_timeout_ms: u64,
    ping_interval_ms: u64,
    ping_timeout_ms: u64,
    ping_failures_tolerated: u64,
    connectivity_check_interval_ms: u64,
    inbound_rpc_timeout_ms: u64,
    consensus_inbound_msg_timout_ms: u64,
    mempool_inbound_msg_timout_ms: u64,
    max_concurrent_outbound_rpcs: u32,
    max_concurrent_inbound_rpcs: u32,
    max_concurrent_network_reqs: u32,
    max_concurrent_network_notifs: u32,
    max_connection_delay_ms: u64,
    signing_keys: Option<(Ed25519PrivateKey, Ed25519PublicKey)>,
    identity_keys: Option<(X25519PrivateKey, X25519PublicKey)>,
}

impl NetworkBuilder {
    /// Return a new NetworkBuilder initialized with default configuration values.
    pub fn new(executor: TaskExecutor, peer_id: PeerId, addr: Multiaddr) -> NetworkBuilder {
        NetworkBuilder {
            executor,
            peer_id,
            addr,
            advertised_address: None,
            seed_peers: HashMap::new(),
            trusted_peers: Arc::new(RwLock::new(HashMap::new())),
            channel_size: NETWORK_CHANNEL_SIZE,
            mempool_protocols: vec![],
            consensus_protocols: vec![],
            direct_send_protocols: vec![],
            rpc_protocols: vec![],
            transport: TransportType::Memory,
            discovery_interval_ms: DISCOVERY_INTERVAL_MS,
            discovery_msg_timeout_ms: DISOVERY_MSG_TIMEOUT_MS,
            ping_interval_ms: PING_INTERVAL_MS,
            ping_timeout_ms: PING_TIMEOUT_MS,
            ping_failures_tolerated: PING_FAILURES_TOLERATED,
            connectivity_check_interval_ms: CONNECTIVITY_CHECK_INTERNAL_MS,
            inbound_rpc_timeout_ms: INBOUND_RPC_TIMEOUT_MS,
            consensus_inbound_msg_timout_ms: CONSENSUS_INBOUND_MSG_TIMOUT_MS,
            mempool_inbound_msg_timout_ms: MEMPOOL_INBOUND_MSG_TIMOUT_MS,
            max_concurrent_outbound_rpcs: MAX_CONCURRENT_OUTBOUND_RPCS,
            max_concurrent_inbound_rpcs: MAX_CONCURRENT_INBOUND_RPCS,
            max_concurrent_network_reqs: MAX_CONCURRENT_NETWORK_REQS,
            max_concurrent_network_notifs: MAX_CONCURRENT_NETWORK_NOTIFS,
            max_connection_delay_ms: MAX_CONNECTION_DELAY_MS,
            signing_keys: None,
            identity_keys: None,
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

    /// Set identity keys of local node.
    pub fn identity_keys(&mut self, keys: (X25519PrivateKey, X25519PublicKey)) -> &mut Self {
        self.identity_keys = Some(keys);
        self
    }

    /// Set seed peers to bootstrap discovery
    pub fn seed_peers(&mut self, seed_peers: HashMap<PeerId, Vec<Multiaddr>>) -> &mut Self {
        self.seed_peers = seed_peers
            .into_iter()
            .map(|(peer_id, seed_addrs)| {
                let mut peer_info = PeerInfo::new();
                peer_info.set_epoch(0);
                peer_info.set_addrs(
                    seed_addrs
                        .into_iter()
                        .map(|addr| addr.as_ref().into())
                        .collect(),
                );
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

    /// Set the protocol IDs that Mempool subscribes.
    pub fn mempool_protocols(&mut self, protocols: Vec<ProtocolId>) -> &mut Self {
        self.mempool_protocols = protocols;
        self
    }

    /// Set the protocol IDs that Consensus subscribes.
    pub fn consensus_protocols(&mut self, protocols: Vec<ProtocolId>) -> &mut Self {
        self.consensus_protocols = protocols;
        self
    }

    /// Set the protocol IDs that DirectSend actor subscribes.
    pub fn direct_send_protocols(&mut self, protocols: Vec<ProtocolId>) -> &mut Self {
        self.direct_send_protocols = protocols;
        self
    }

    /// Set the protocol IDs that RPC actor subscribes.
    pub fn rpc_protocols(&mut self, protocols: Vec<ProtocolId>) -> &mut Self {
        self.rpc_protocols = protocols;
        self
    }

    fn supported_protocols(&self) -> Vec<ProtocolId> {
        self.direct_send_protocols
            .iter()
            .chain(&self.rpc_protocols)
            .chain(&vec![
                ProtocolId::from_static(DISCOVERY_PROTOCOL_NAME),
                ProtocolId::from_static(PING_PROTOCOL_NAME),
            ])
            .cloned()
            .collect()
    }

    /// Create the configured `NetworkBuilder`
    /// Return the constructed Mempool and Consensus Sender+Events
    pub fn build(
        &mut self,
    ) -> (
        (MempoolNetworkSender, MempoolNetworkEvents),
        (ConsensusNetworkSender, ConsensusNetworkEvents),
        Multiaddr,
    ) {
        let identity = Identity::new(self.peer_id, self.supported_protocols());
        // Build network based on the transport type
        let own_identity_keys = self.identity_keys.take().expect("Identity keys not set");
        let trusted_peers = self.trusted_peers.clone();
        match self.transport {
            TransportType::Memory => self.build_with_transport(build_memory_transport(identity)),
            TransportType::MemoryNoise => self.build_with_transport(build_memory_noise_transport(
                identity,
                own_identity_keys,
                trusted_peers,
            )),
            TransportType::Tcp => self.build_with_transport(build_tcp_transport(identity)),
            TransportType::TcpNoise => self.build_with_transport(build_tcp_noise_transport(
                identity,
                own_identity_keys,
                trusted_peers,
            )),
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
    ) -> (
        (MempoolNetworkSender, MempoolNetworkEvents),
        (ConsensusNetworkSender, ConsensusNetworkEvents),
        Multiaddr,
    ) {
        // Construct Mempool and Consensus network interfaces
        let (network_reqs_tx, network_reqs_rx) =
            channel::new(self.channel_size, &counters::PENDING_NETWORK_REQUESTS);
        let (mempool_tx, mempool_rx) = channel::new_with_timeout(
            self.channel_size,
            &counters::PENDING_MEMPOOL_NETWORK_EVENTS,
            Duration::from_millis(self.consensus_inbound_msg_timout_ms),
        );
        let (consensus_tx, consensus_rx) = channel::new_with_timeout(
            self.channel_size,
            &counters::PENDING_CONSENSUS_NETWORK_EVENTS,
            Duration::from_millis(self.mempool_inbound_msg_timout_ms),
        );

        let mempool_network_sender = MempoolNetworkSender::new(network_reqs_tx.clone());
        let mempool_network_events = MempoolNetworkEvents::new(mempool_rx);
        let consensus_network_sender = ConsensusNetworkSender::new(network_reqs_tx.clone());
        let consensus_network_events = ConsensusNetworkEvents::new(consensus_rx);
        // Initialize and start NetworkProvider.
        let (pm_reqs_tx, pm_reqs_rx) =
            channel::new(self.channel_size, &counters::PENDING_PEER_MANAGER_REQUESTS);
        let (pm_net_notifs_tx, pm_net_notifs_rx) = channel::new(
            self.channel_size,
            &counters::PENDING_PEER_MANAGER_NET_NOTIFICATIONS,
        );
        let (ds_reqs_tx, ds_reqs_rx) =
            channel::new(self.channel_size, &counters::PENDING_DIRECT_SEND_REQUESTS);
        let (ds_net_notifs_tx, ds_net_notifs_rx) = channel::new(
            self.channel_size,
            &counters::PENDING_DIRECT_SEND_NOTIFICATIONS,
        );
        let (conn_mgr_reqs_tx, conn_mgr_reqs_rx) = channel::new(
            self.channel_size,
            &counters::PENDING_CONNECTIVITY_MANAGER_REQUESTS,
        );
        let (rpc_reqs_tx, rpc_reqs_rx) =
            channel::new(self.channel_size, &counters::PENDING_RPC_REQUESTS);
        let (rpc_net_notifs_tx, rpc_net_notifs_rx) =
            channel::new(self.channel_size, &counters::PENDING_RPC_NOTIFICATIONS);

        let mempool_handlers = self
            .mempool_protocols
            .iter()
            .map(|p| (p.clone(), mempool_tx.clone()));
        let consensus_handlers = self
            .consensus_protocols
            .iter()
            .map(|p| (p.clone(), consensus_tx.clone()));
        let upstream_handlers = mempool_handlers.chain(consensus_handlers).collect();

        let validator_network = NetworkProvider::new(
            pm_net_notifs_rx,
            rpc_reqs_tx,
            rpc_net_notifs_rx,
            ds_reqs_tx,
            ds_net_notifs_rx,
            conn_mgr_reqs_tx.clone(),
            network_reqs_rx,
            upstream_handlers,
            self.max_concurrent_network_reqs,
            self.max_concurrent_network_notifs,
        );
        self.executor
            .spawn(validator_network.start().boxed().unit_error().compat());

        // Initialize and start PeerManager.
        let (pm_ds_notifs_tx, pm_ds_notifs_rx) = channel::new(
            self.channel_size,
            &counters::PENDING_PEER_MANAGER_DIRECT_SEND_NOTIFICATIONS,
        );
        let (pm_rpc_notifs_tx, pm_rpc_notifs_rx) = channel::new(
            self.channel_size,
            &counters::PENDING_PEER_MANAGER_RPC_NOTIFICATIONS,
        );
        let (pm_discovery_notifs_tx, pm_discovery_notifs_rx) = channel::new(
            self.channel_size,
            &counters::PENDING_PEER_MANAGER_DISCOVERY_NOTIFICATIONS,
        );
        let (pm_ping_notifs_tx, pm_ping_notifs_rx) = channel::new(
            self.channel_size,
            &counters::PENDING_PEER_MANAGER_PING_NOTIFICATIONS,
        );
        let (pm_conn_mgr_notifs_tx, pm_conn_mgr_notifs_rx) = channel::new(
            self.channel_size,
            &counters::PENDING_PEER_MANAGER_CONNECTIVITY_MANAGER_NOTIFICATIONS,
        );

        let direct_send_handlers = self
            .direct_send_protocols
            .iter()
            .map(|p| (p.clone(), pm_ds_notifs_tx.clone()));
        let rpc_handlers = self
            .rpc_protocols
            .iter()
            .map(|p| (p.clone(), pm_rpc_notifs_tx.clone()));
        let discovery_handler = vec![(
            ProtocolId::from_static(DISCOVERY_PROTOCOL_NAME),
            pm_discovery_notifs_tx.clone(),
        )];
        let ping_handler = vec![(
            ProtocolId::from_static(PING_PROTOCOL_NAME),
            pm_ping_notifs_tx.clone(),
        )];
        let protocol_handlers = direct_send_handlers
            .chain(rpc_handlers)
            .chain(discovery_handler)
            .chain(ping_handler)
            .collect();

        let peer_mgr = PeerManager::new(
            transport,
            self.executor.clone(),
            self.peer_id,
            self.addr.clone(),
            pm_reqs_rx,
            protocol_handlers,
            vec![
                pm_net_notifs_tx,
                pm_conn_mgr_notifs_tx,
                pm_ping_notifs_tx,
                pm_discovery_notifs_tx,
            ],
        );
        let listen_addr = peer_mgr.listen_addr().clone();
        self.executor
            .spawn(peer_mgr.start().boxed().unit_error().compat());

        // Initialize and start DirectSend actor.
        let ds = DirectSend::new(
            self.executor.clone(),
            ds_reqs_rx,
            ds_net_notifs_tx,
            pm_ds_notifs_rx,
            PeerManagerRequestSender::new(pm_reqs_tx.clone()),
        );
        self.executor
            .spawn(ds.start().boxed().unit_error().compat());

        // Initialize and start RPC actor.
        let rpc = Rpc::new(
            rpc_reqs_rx,
            pm_rpc_notifs_rx,
            PeerManagerRequestSender::new(pm_reqs_tx.clone()),
            rpc_net_notifs_tx,
            Duration::from_millis(self.inbound_rpc_timeout_ms),
            self.max_concurrent_outbound_rpcs,
            self.max_concurrent_inbound_rpcs,
        );
        self.executor
            .spawn(rpc.start().boxed().unit_error().compat());

        let conn_mgr = ConnectivityManager::new(
            self.trusted_peers.clone(),
            Compat01As03::new(Interval::new_interval(Duration::from_millis(
                self.connectivity_check_interval_ms,
            )))
            .fuse(),
            PeerManagerRequestSender::new(pm_reqs_tx.clone()),
            pm_conn_mgr_notifs_rx,
            conn_mgr_reqs_rx,
            ExponentialBackoff::from_millis(2).factor(1000 /* seconds */),
            self.max_connection_delay_ms,
        );
        self.executor
            .spawn(conn_mgr.start().boxed().unit_error().compat());

        // Setup signer from keys.
        let (signing_private_key, _signing_public_key) =
            self.signing_keys.take().expect("Signing keys not set");
        let signer = ValidatorSigner::new(self.peer_id, signing_private_key);
        // Initialize and start Discovery actor.
        let discovery = Discovery::new(
            self.peer_id,
            vec![self
                .advertised_address
                .clone()
                .unwrap_or_else(|| listen_addr.clone())],
            signer,
            self.seed_peers.clone(),
            self.trusted_peers.clone(),
            Compat01As03::new(Interval::new_interval(Duration::from_millis(
                self.discovery_interval_ms,
            )))
            .fuse(),
            PeerManagerRequestSender::new(pm_reqs_tx.clone()),
            pm_discovery_notifs_rx,
            conn_mgr_reqs_tx.clone(),
            Duration::from_millis(self.discovery_msg_timeout_ms),
        );
        self.executor
            .spawn(discovery.start().boxed().unit_error().compat());

        // Initialize and start HealthChecker.
        let health_checker = HealthChecker::new(
            Compat01As03::new(Interval::new_interval(Duration::from_millis(
                self.ping_interval_ms,
            )))
            .fuse(),
            PeerManagerRequestSender::new(pm_reqs_tx.clone()),
            pm_ping_notifs_rx,
            Duration::from_millis(self.ping_timeout_ms),
            self.ping_failures_tolerated,
        );
        self.executor
            .spawn(health_checker.start().boxed().unit_error().compat());

        (
            (mempool_network_sender, mempool_network_events),
            (consensus_network_sender, consensus_network_events),
            listen_addr,
        )
    }
}

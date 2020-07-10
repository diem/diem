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
use channel::{self, message_queues::QueueStyle};
use libra_config::{
    chain_id::ChainId,
    config::{DiscoveryMethod, NetworkConfig, RoleType, HANDSHAKE_VERSION},
    network_id::{NetworkContext, NetworkId},
};
use libra_crypto::x25519;
use libra_logger::prelude::*;
use libra_metrics::IntCounterVec;
use libra_network_address::{
    encrypted::{TEST_SHARED_VAL_NETADDR_KEY, TEST_SHARED_VAL_NETADDR_KEY_VERSION},
    NetworkAddress,
};
use libra_types::PeerId;
use network::{
    connectivity_manager::{builder::ConnectivityManagerBuilder, ConnectivityRequest},
    constants,
    peer_manager::{
        builder::{AuthenticationMode, PeerManagerBuilder},
        conn_notifs_channel, ConnectionRequestSender,
    },
    protocols::{
        discovery::{self, builder::DiscoveryBuilder},
        health_checker::{self, builder::HealthCheckerBuilder},
        network::{NewNetworkEvents, NewNetworkSender},
    },
    ProtocolId,
};
use network_simple_onchain_discovery::{
    builder::ConfigurationChangeListenerBuilder, gen_simple_discovery_reconfig_subscription,
};
use std::{
    clone::Clone,
    collections::{HashMap, HashSet},
    iter,
    sync::{Arc, RwLock},
};
use subscription_service::ReconfigSubscription;
use tokio::runtime::{Builder, Handle, Runtime};

/// Build Network module with custom configuration values.
/// Methods can be chained in order to set the configuration values.
/// MempoolNetworkHandler and ConsensusNetworkHandler are constructed by calling
/// [`NetworkBuilder::build`].  New instances of `NetworkBuilder` are obtained
/// via [`NetworkBuilder::new`].
// TODO(philiphayes): refactor NetworkBuilder and libra-node; current config is
// pretty tangled.
pub struct NetworkBuilder {
    executor: Handle,
    network_context: Arc<NetworkContext>,
    trusted_peers: Arc<RwLock<HashMap<PeerId, HashSet<x25519::PublicKey>>>>,
    seed_addrs: HashMap<PeerId, Vec<NetworkAddress>>,
    seed_pubkeys: HashMap<PeerId, HashSet<x25519::PublicKey>>,
    channel_size: usize,
    connectivity_check_interval_ms: u64,
    max_connection_delay_ms: u64,
    /// For now full node connections are limited by
    max_fullnode_connections: usize,

    configuration_change_listener_builder: Option<ConfigurationChangeListenerBuilder>,
    connectivity_manager_builder: Option<ConnectivityManagerBuilder>,
    discovery_builder: Option<DiscoveryBuilder>,
    health_checker_builder: Option<HealthCheckerBuilder>,
    peer_manager_builder: PeerManagerBuilder,

    reconfig_subscriptions: Vec<ReconfigSubscription>,
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
        authentication_mode: AuthenticationMode,
    ) -> NetworkBuilder {
        // TODO: Pass network_context in as a constructed object.
        let network_context = Arc::new(NetworkContext::new(network_id, role, peer_id));
        let trusted_peers = Arc::new(RwLock::new(HashMap::new()));

        // A network cannot exist without a PeerManager
        // TODO:  construct this in create and pass it to new() as a parameter
        let peer_manager_builder = PeerManagerBuilder::create(
            chain_id,
            network_context.clone(),
            listen_address,
            trusted_peers.clone(),
            authentication_mode,
            // TODO:  move to a config
            constants::NETWORK_CHANNEL_SIZE,
            // TODO:  Move to a config
            constants::MAX_CONCURRENT_NETWORK_REQS,
            // TODO:  Move to a config
            constants::MAX_CONCURRENT_NETWORK_NOTIFS,
        );

        NetworkBuilder {
            executor,
            network_context,
            trusted_peers,
            seed_addrs: HashMap::new(),
            seed_pubkeys: HashMap::new(),
            channel_size: constants::NETWORK_CHANNEL_SIZE,
            connectivity_check_interval_ms: constants::CONNECTIVITY_CHECK_INTERNAL_MS,
            max_connection_delay_ms: constants::MAX_CONNECTION_DELAY_MS,
            max_fullnode_connections: constants::MAX_FULLNODE_CONNECTIONS,
            configuration_change_listener_builder: None,
            connectivity_manager_builder: None,
            discovery_builder: None,
            health_checker_builder: None,
            peer_manager_builder,
            reconfig_subscriptions: vec![],
        }
    }

    pub fn create(
        chain_id: &ChainId,
        role: RoleType,
        config: &mut NetworkConfig,
    ) -> (Runtime, NetworkBuilder) {
        let runtime = Builder::new()
            .thread_name("network-")
            .threaded_scheduler()
            .enable_all()
            .build()
            .expect("Failed to start runtime. Won't be able to start networking.");

        let peer_id = config.peer_id();
        let identity_key = config.identity_key();

        let authentication_mode = if config.mutual_authentication {
            AuthenticationMode::Mutual(identity_key)
        } else {
            AuthenticationMode::ServerOnly(identity_key)
        };
        let pubkey = authentication_mode.public_key();

        let mut network_builder = NetworkBuilder::new(
            runtime.handle().clone(),
            chain_id.clone(),
            config.network_id.clone(),
            role,
            peer_id,
            config.listen_address.clone(),
            authentication_mode,
        );
        network_builder
            .seed_addrs(config.seed_addrs.clone())
            .seed_pubkeys(config.seed_pubkeys.clone())
            .connectivity_check_interval_ms(config.connectivity_check_interval_ms)
            .add_connection_monitoring(
                // TODO: Move these values into NetworkConfig
                constants::PING_INTERVAL_MS,
                constants::PING_TIMEOUT_MS,
                constants::PING_FAILURES_TOLERATED,
            );

        // Sanity check seed addresses.
        config
            .verify_seed_addrs()
            .expect("Seed addresses must be well-formed");

        // Don't turn on connectivity manager if we're a public-facing server,
        // for example.
        //
        // Cases that require connectivity manager:
        //
        // 1) mutual authentication networks currently require connmgr to set the
        //    trusted peers set.
        // 2) networks with a discovery protocol need connmgr to connect to newly
        //    discovered peers.
        // 3) if we have seed peers, then we need connmgr to connect to them.
        // TODO(philiphayes): could probably use a better way to specify these cases
        if config.mutual_authentication
            || config.discovery_method != DiscoveryMethod::None
            || !config.seed_addrs.is_empty()
        {
            network_builder.add_connectivity_manager();
        }

        match &config.discovery_method {
            DiscoveryMethod::Gossip(gossip_config) => {
                network_builder.add_gossip_discovery(
                    gossip_config.advertised_address.clone(),
                    gossip_config.discovery_interval_ms,
                    pubkey,
                );
                // HACK: gossip relies on on-chain discovery for the eligible peers update.
                if role == RoleType::Validator {
                    network_builder.add_configuration_change_listener(role);
                }
            }
            DiscoveryMethod::Onchain => {
                network_builder.add_configuration_change_listener(role);
            }
            DiscoveryMethod::None => {}
        }

        (runtime, network_builder)
    }

    pub fn reconfig_subscriptions(&mut self) -> &mut Vec<ReconfigSubscription> {
        &mut self.reconfig_subscriptions
    }

    pub fn network_context(&self) -> Arc<NetworkContext> {
        self.network_context.clone()
    }

    pub fn peer_id(&self) -> PeerId {
        self.network_context.peer_id()
    }

    /// Set additional public keys for seed peers to bootstrap discovery.
    pub fn seed_pubkeys(
        &mut self,
        seed_pubkeys: HashMap<PeerId, HashSet<x25519::PublicKey>>,
    ) -> &mut Self {
        self.seed_pubkeys = seed_pubkeys;
        self
    }

    /// Set addresses of seed peers to bootstrap discovery
    pub fn seed_addrs(&mut self, seed_addrs: HashMap<PeerId, Vec<NetworkAddress>>) -> &mut Self {
        self.seed_addrs = seed_addrs;
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
        match self.connectivity_manager_builder.as_ref() {
            Some(conn_mgr_builder) => Some(conn_mgr_builder.conn_mgr_reqs_tx()),
            None => None,
        }
    }

    pub fn add_connection_event_listener(&mut self) -> conn_notifs_channel::Receiver {
        self.peer_manager_builder.add_connection_event_listener()
    }

    fn build_peer_manager(&mut self) -> &mut Self {
        self.peer_manager_builder.build(&self.executor);
        self
    }

    fn start_peer_manager(&mut self) -> &mut Self {
        self.peer_manager_builder.start(&self.executor);
        self
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
        let trusted_peers = self.trusted_peers.clone();
        let seed_addrs = self.seed_addrs.clone();
        let mut seed_pubkeys = self.seed_pubkeys.clone();

        let max_connection_delay_ms = self.max_connection_delay_ms;
        let connectivity_check_interval_ms = self.connectivity_check_interval_ms;
        let pm_conn_mgr_notifs_rx = self.add_connection_event_listener();
        let connection_limit = if let RoleType::FullNode = self.network_context.role() {
            Some(self.max_fullnode_connections)
        } else {
            None
        };

        // union pubkeys from addrs with pubkeys directly in config
        let addr_pubkeys_iter = seed_addrs.iter().map(|(peer_id, addrs)| {
            let pubkey_set: HashSet<_> = addrs
                .iter()
                .filter_map(NetworkAddress::find_noise_proto)
                .collect();
            (*peer_id, pubkey_set)
        });
        for (peer_id, pubkey_set) in addr_pubkeys_iter {
            seed_pubkeys.entry(peer_id).or_default().extend(pubkey_set);
        }

        self.connectivity_manager_builder = Some(ConnectivityManagerBuilder::create(
            self.network_context(),
            trusted_peers,
            seed_addrs,
            seed_pubkeys,
            connectivity_check_interval_ms,
            // TODO:  move this into a config
            2, // Legacy hardcoded value,
            max_connection_delay_ms,
            self.channel_size,
            ConnectionRequestSender::new(self.peer_manager_builder.connection_reqs_tx()),
            pm_conn_mgr_notifs_rx,
            connection_limit,
        ));
        self.build_connectivity_manager()
            .start_connectivity_manager()
    }

    fn build_connectivity_manager(&mut self) -> &mut Self {
        if let Some(builder) = self.connectivity_manager_builder.as_mut() {
            builder.build(&self.executor);
        }
        self
    }

    fn start_connectivity_manager(&mut self) -> &mut Self {
        if let Some(builder) = self.connectivity_manager_builder.as_mut() {
            builder.start(&self.executor);
        }
        self
    }

    fn add_configuration_change_listener(&mut self, role: RoleType) -> &mut Self {
        let conn_mgr_reqs_tx = self
            .conn_mgr_reqs_tx()
            .expect("ConnectivityManager must be installed for validator");
        let (simple_discovery_reconfig_subscription, simple_discovery_reconfig_rx) =
            gen_simple_discovery_reconfig_subscription();
        self.reconfig_subscriptions
            .push(simple_discovery_reconfig_subscription);

        // TODO(philiphayes): remove once we get real keys from key manager
        let shared_val_netaddr_key_map: HashMap<_, _> = iter::once((
            TEST_SHARED_VAL_NETADDR_KEY_VERSION,
            TEST_SHARED_VAL_NETADDR_KEY,
        ))
        .collect();

        self.configuration_change_listener_builder =
            Some(ConfigurationChangeListenerBuilder::create(
                role,
                shared_val_netaddr_key_map,
                conn_mgr_reqs_tx,
                simple_discovery_reconfig_rx,
            ));
        self.build_configuration_change_listener()
            .start_configuration_change_listener()
    }

    fn build_configuration_change_listener(&mut self) -> &mut Self {
        if let Some(configuration_change_listener) =
            self.configuration_change_listener_builder.as_mut()
        {
            configuration_change_listener.build();
        }
        self
    }

    fn start_configuration_change_listener(&mut self) -> &mut Self {
        if let Some(configuration_change_listener) =
            self.configuration_change_listener_builder.as_mut()
        {
            configuration_change_listener.start(&self.executor);
        }
        self
    }

    /// Add the (gossip) [`Discovery`] protocol to the network.
    ///
    /// (gossip) [`Discovery`] discovers other eligible peers' network addresses
    /// by exchanging the full set of known peer network addresses with connected
    /// peers as a network protocol.
    ///
    /// This is for testing purposes only and should not be used in production networks.
    // TODO:  remove the pub qualifier
    pub fn add_gossip_discovery(
        &mut self,
        advertised_address: NetworkAddress,
        discovery_interval_ms: u64,
        pubkey: x25519::PublicKey,
    ) -> &mut Self {
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

        let advertised_address = advertised_address.append_prod_protos(pubkey, HANDSHAKE_VERSION);

        let addrs = vec![advertised_address];

        self.discovery_builder = Some(DiscoveryBuilder::create(
            self.network_context(),
            addrs,
            discovery_interval_ms,
            discovery_network_tx,
            discovery_network_rx,
            conn_mgr_reqs_tx,
        ));
        self.build_gossip_discovery().start_gossip_discovery();
        self
    }

    fn build_gossip_discovery(&mut self) -> &mut Self {
        if let Some(discovery_builder) = self.discovery_builder.as_mut() {
            discovery_builder.build(&self.executor);
            debug!("{} Built Gossip Discovery", self.network_context());
        }
        self
    }

    fn start_gossip_discovery(&mut self) -> &mut Self {
        if let Some(discovery_builder) = self.discovery_builder.as_mut() {
            discovery_builder.start(&self.executor);
            debug!("{} Started gossip discovery", self.network_context());
        }
        self
    }

    /// Add a HealthChecker to the network.
    pub fn add_connection_monitoring(
        &mut self,
        ping_interval_ms: u64,
        ping_timeout_ms: u64,
        ping_failures_tolerated: u64,
    ) -> &mut Self {
        // Initialize and start HealthChecker.
        let (hc_network_tx, hc_network_rx) =
            self.add_protocol_handler(health_checker::network_endpoint_config());

        self.health_checker_builder = Some(HealthCheckerBuilder::create(
            self.network_context(),
            ping_interval_ms,
            ping_timeout_ms,
            ping_failures_tolerated,
            hc_network_tx,
            hc_network_rx,
        ));
        debug!("{} Created health checker", self.network_context);
        self.build_connection_monitoring()
    }

    /// Build the HealthChecker, if it has been added.
    fn build_connection_monitoring(&mut self) -> &mut Self {
        if let Some(health_checker) = self.health_checker_builder.as_mut() {
            health_checker.build(&self.executor);
            debug!("{} Built health checker", self.network_context);
        };
        self.start_connection_monitoring()
    }

    /// Star the built HealthChecker.
    fn start_connection_monitoring(&mut self) -> &mut Self {
        if let Some(health_checker) = self.health_checker_builder.as_mut() {
            health_checker.start(&self.executor);
            debug!("{} Started health checker", self.network_context);
        };
        self
    }

    /// Create the configured transport and start PeerManager.
    /// Return the actual NetworkAddress over which this peer is listening.
    pub fn build(mut self) -> NetworkAddress {
        self.build_peer_manager().start_peer_manager();
        self.peer_manager_builder.listen_address()
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
        let (peer_mgr_reqs_tx, peer_mgr_reqs_rx, connection_reqs_tx, connection_notifs_rx) =
            self.peer_manager_builder.add_protocol_handler(
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

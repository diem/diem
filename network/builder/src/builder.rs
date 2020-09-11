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
    config::{DiscoveryMethod, GossipConfig, NetworkConfig, RoleType, HANDSHAKE_VERSION},
    network_id::NetworkContext,
};
use libra_crypto::x25519;
use libra_logger::prelude::*;
use libra_metrics::IntCounterVec;
use libra_network_address::NetworkAddress;
use libra_network_address_encryption::Encryptor;
use libra_types::{chain_id::ChainId, PeerId};
use network::{
    connectivity_manager::{builder::ConnectivityManagerBuilder, ConnectivityRequest},
    constants,
    logging::NetworkSchema,
    peer_manager::{
        builder::{AuthenticationMode, PeerManagerBuilder},
        conn_notifs_channel, ConnectionRequestSender,
    },
    protocols::{
        gossip_discovery::{self, builder::GossipDiscoveryBuilder},
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
    sync::{Arc, RwLock},
};
use subscription_service::ReconfigSubscription;
use tokio::runtime::Handle;

#[derive(Debug, PartialEq, PartialOrd)]
enum State {
    CREATED,
    BUILT,
    STARTED,
}

/// Build Network module with custom configuration values.
/// Methods can be chained in order to set the configuration values.
/// MempoolNetworkHandler and ConsensusNetworkHandler are constructed by calling
/// [`NetworkBuilder::build`].  New instances of `NetworkBuilder` are obtained
/// via [`NetworkBuilder::create`].
pub struct NetworkBuilder {
    state: State,
    executor: Option<Handle>,
    network_context: Arc<NetworkContext>,

    configuration_change_listener_builder: Option<ConfigurationChangeListenerBuilder>,
    connectivity_manager_builder: Option<ConnectivityManagerBuilder>,
    discovery_builder: Option<GossipDiscoveryBuilder>,
    health_checker_builder: Option<HealthCheckerBuilder>,
    peer_manager_builder: PeerManagerBuilder,

    // (StateSync) ReconfigSubscriptions required by internal Network components.
    reconfig_subscriptions: Vec<ReconfigSubscription>,
}

impl NetworkBuilder {
    /// Return a new NetworkBuilder initialized with default configuration values.
    // TODO:  Remove `pub`.  NetworkBuilder should only be created thorugh `::create()`
    pub fn new(
        chain_id: ChainId,
        trusted_peers: Arc<RwLock<HashMap<PeerId, HashSet<x25519::PublicKey>>>>,
        network_context: Arc<NetworkContext>,
        listen_address: NetworkAddress,
        authentication_mode: AuthenticationMode,
        max_frame_size: usize,
    ) -> Self {
        // A network cannot exist without a PeerManager
        // TODO:  construct this in create and pass it to new() as a parameter. The complication is manual construction of NetworkBuilder in various tests.
        let peer_manager_builder = PeerManagerBuilder::create(
            chain_id,
            network_context.clone(),
            listen_address,
            trusted_peers,
            authentication_mode,
            // TODO: Encode this value in NetworkConfig
            constants::NETWORK_CHANNEL_SIZE,
            // TODO: Encode this value in NetworkConfig
            constants::MAX_CONCURRENT_NETWORK_REQS,
            // TODO: Encode this value in NetworkConfig
            constants::MAX_CONCURRENT_NETWORK_NOTIFS,
            max_frame_size,
        );

        NetworkBuilder {
            state: State::CREATED,
            executor: None,
            network_context,
            configuration_change_listener_builder: None,
            connectivity_manager_builder: None,
            discovery_builder: None,
            health_checker_builder: None,
            peer_manager_builder,
            reconfig_subscriptions: vec![],
        }
    }

    /// Create a new NetworkBuilder based on the provided configuration.
    pub fn create(chain_id: ChainId, role: RoleType, config: &NetworkConfig) -> NetworkBuilder {
        let peer_id = config.peer_id();
        let identity_key = config.identity_key();
        let pubkey = libra_crypto::PrivateKey::public_key(&identity_key);

        let authentication_mode = if config.mutual_authentication {
            AuthenticationMode::Mutual(identity_key)
        } else {
            AuthenticationMode::ServerOnly(identity_key)
        };

        let network_context = Arc::new(NetworkContext::new(
            config.network_id.clone(),
            role,
            peer_id,
        ));

        let trusted_peers = Arc::new(RwLock::new(HashMap::new()));

        let mut network_builder = NetworkBuilder::new(
            chain_id,
            trusted_peers.clone(),
            network_context,
            config.listen_address.clone(),
            authentication_mode,
            config.max_frame_size,
        );

        network_builder.add_connection_monitoring(
            // TODO: Encode this value in NetworkConfig
            constants::PING_INTERVAL_MS,
            // TODO: Encode this value in NetworkConfig
            constants::PING_TIMEOUT_MS,
            // TODO: Encode this value in NetworkConfig
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
        // TODO:  Why not add ConnectivityManager always?
        if config.mutual_authentication
            || config.discovery_method != DiscoveryMethod::None
            || !config.seed_addrs.is_empty()
        {
            network_builder.add_connectivity_manager(
                config.seed_addrs.clone(),
                config.seed_pubkeys.clone(), // TODO: this should be encoded in network config
                trusted_peers,
                constants::MAX_FULLNODE_CONNECTIONS,
                // TODO: this should be encoded in network_config
                constants::MAX_CONNECTION_DELAY_MS,
                config.connectivity_check_interval_ms,
                constants::NETWORK_CHANNEL_SIZE,
            );
        }

        match &config.discovery_method {
            DiscoveryMethod::Gossip(gossip_config) => {
                network_builder.add_gossip_discovery(gossip_config.clone(), pubkey);
                // HACK: gossip relies on on-chain discovery for the eligible peers update.
                // TODO:  it should be safe to enable the configuraction_change_listener always.
                if role == RoleType::Validator {
                    network_builder.add_configuration_change_listener(role, config.encryptor());
                }
            }
            DiscoveryMethod::Onchain => {
                network_builder.add_configuration_change_listener(role, config.encryptor());
            }
            DiscoveryMethod::None => {}
        }

        network_builder
    }

    /// Create the configured Networking components.
    pub fn build(&mut self, executor: Handle) -> &mut Self {
        assert_eq!(self.state, State::CREATED);
        self.state = State::BUILT;
        self.executor = Some(executor);
        self.build_peer_manager()
            .build_configuration_change_listener()
            .build_gossip_discovery()
            .build_connectivity_manager()
            .build_connection_monitoring()
    }

    /// Start the built Networking components.
    pub fn start(&mut self) -> &mut Self {
        assert_eq!(self.state, State::BUILT);
        self.state = State::STARTED;
        self.start_peer_manager()
            .start_connectivity_manager()
            .start_connection_monitoring()
            .start_gossip_discovery()
            .start_configuration_change_listener()
    }

    pub fn reconfig_subscriptions(&mut self) -> &mut Vec<ReconfigSubscription> {
        &mut self.reconfig_subscriptions
    }

    pub fn network_context(&self) -> Arc<NetworkContext> {
        self.network_context.clone()
    }

    fn conn_mgr_reqs_tx(&self) -> Option<channel::Sender<ConnectivityRequest>> {
        match self.connectivity_manager_builder.as_ref() {
            Some(conn_mgr_builder) => Some(conn_mgr_builder.conn_mgr_reqs_tx()),
            None => None,
        }
    }

    fn add_connection_event_listener(&mut self) -> conn_notifs_channel::Receiver {
        self.peer_manager_builder.add_connection_event_listener()
    }

    pub fn listen_address(&self) -> NetworkAddress {
        self.peer_manager_builder.listen_address()
    }

    fn build_peer_manager(&mut self) -> &mut Self {
        self.peer_manager_builder
            .build(self.executor.as_mut().expect("Executor must exist"));
        self
    }

    fn start_peer_manager(&mut self) -> &mut Self {
        self.peer_manager_builder
            .start(self.executor.as_mut().expect("Executor must exist"));
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
    pub fn add_connectivity_manager(
        &mut self,
        seed_addrs: HashMap<PeerId, Vec<NetworkAddress>>,
        mut seed_pubkeys: HashMap<PeerId, HashSet<x25519::PublicKey>>,
        trusted_peers: Arc<RwLock<HashMap<PeerId, HashSet<x25519::PublicKey>>>>,
        max_fullnode_connections: usize,
        max_connection_delay_ms: u64,
        connectivity_check_interval_ms: u64,
        channel_size: usize,
    ) -> &mut Self {
        let pm_conn_mgr_notifs_rx = self.add_connection_event_listener();
        let connection_limit = if let RoleType::FullNode = self.network_context.role() {
            Some(max_fullnode_connections)
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
            channel_size,
            ConnectionRequestSender::new(self.peer_manager_builder.connection_reqs_tx()),
            pm_conn_mgr_notifs_rx,
            connection_limit,
        ));
        self
    }

    fn build_connectivity_manager(&mut self) -> &mut Self {
        if let Some(builder) = self.connectivity_manager_builder.as_mut() {
            builder.build(self.executor.as_mut().expect("Executor must exist"));
        }
        self
    }

    fn start_connectivity_manager(&mut self) -> &mut Self {
        if let Some(builder) = self.connectivity_manager_builder.as_mut() {
            builder.start(self.executor.as_mut().expect("Executor must exist"));
        }
        self
    }

    fn add_configuration_change_listener(
        &mut self,
        role: RoleType,
        encryptor: Encryptor,
    ) -> &mut Self {
        let conn_mgr_reqs_tx = self
            .conn_mgr_reqs_tx()
            .expect("ConnectivityManager must be installed for validator");
        let (simple_discovery_reconfig_subscription, simple_discovery_reconfig_rx) =
            gen_simple_discovery_reconfig_subscription();
        self.reconfig_subscriptions
            .push(simple_discovery_reconfig_subscription);

        self.configuration_change_listener_builder =
            Some(ConfigurationChangeListenerBuilder::create(
                role,
                encryptor,
                conn_mgr_reqs_tx,
                simple_discovery_reconfig_rx,
            ));
        self
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
            configuration_change_listener
                .start(self.executor.as_mut().expect("Executor must exist"));
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
        gossip_config: GossipConfig,
        pubkey: x25519::PublicKey,
    ) -> &mut Self {
        let conn_mgr_reqs_tx = self
            .conn_mgr_reqs_tx()
            .expect("ConnectivityManager not enabled");
        // Get handles for network events and sender.
        let (discovery_network_tx, discovery_network_rx) =
            self.add_protocol_handler(gossip_discovery::network_endpoint_config());

        // TODO(philiphayes): the current setup for gossip discovery doesn't work
        // when we don't have an `advertised_address` set, since it uses the
        // `listen_address`, which might not be bound to a port yet. For example,
        // if our `listen_address` is "/ip6/::1/tcp/0" and `advertised_address` is
        // `None`, then this will set our `advertised_address` to something like
        // "/ip6/::1/tcp/0/ln-noise-ik/<pubkey>/ln-handshake/0", which is wrong
        // since the actual bound port will be something > 0.

        // TODO:  move this logic into DiscoveryBuilder::create
        let advertised_address = gossip_config
            .advertised_address
            .append_prod_protos(pubkey, HANDSHAKE_VERSION);

        let addrs = vec![advertised_address];

        self.discovery_builder = Some(GossipDiscoveryBuilder::create(
            self.network_context(),
            addrs,
            gossip_config.discovery_interval_ms,
            discovery_network_tx,
            discovery_network_rx,
            conn_mgr_reqs_tx,
        ));
        self
    }

    fn build_gossip_discovery(&mut self) -> &mut Self {
        if let Some(discovery_builder) = self.discovery_builder.as_mut() {
            discovery_builder.build(self.executor.as_mut().expect("Executor must exist"));
            debug!(
                NetworkSchema::new(&self.network_context),
                "{} Built Gossip Discovery", self.network_context
            );
        }
        self
    }

    fn start_gossip_discovery(&mut self) -> &mut Self {
        if let Some(discovery_builder) = self.discovery_builder.as_mut() {
            discovery_builder.start(self.executor.as_mut().expect("Executor must exist"));
            debug!(
                NetworkSchema::new(&self.network_context),
                "{} Started gossip discovery", self.network_context
            );
        }
        self
    }

    /// Add a HealthChecker to the network.
    // TODO: remove the pub qualifier
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
        debug!(
            NetworkSchema::new(&self.network_context),
            "{} Created health checker", self.network_context
        );
        self
    }

    /// Build the HealthChecker, if it has been added.
    fn build_connection_monitoring(&mut self) -> &mut Self {
        if let Some(health_checker) = self.health_checker_builder.as_mut() {
            health_checker.build(self.executor.as_mut().expect("Executor must exist"));
            debug!(
                NetworkSchema::new(&self.network_context),
                "{} Built health checker", self.network_context
            );
        };
        self
    }

    /// Star the built HealthChecker.
    fn start_connection_monitoring(&mut self) -> &mut Self {
        if let Some(health_checker) = self.health_checker_builder.as_mut() {
            health_checker.start(self.executor.as_mut().expect("Executor must exist"));
            debug!(
                NetworkSchema::new(&self.network_context),
                "{} Started health checker", self.network_context
            );
        };
        self
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

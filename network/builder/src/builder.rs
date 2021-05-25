// Copyright (c) The Diem Core Contributors
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
use diem_config::{
    config::{
        DiscoveryMethod, NetworkConfig, Peer, PeerRole, PeerSet, RateLimitConfig, RoleType,
        CONNECTION_BACKOFF_BASE, CONNECTIVITY_CHECK_INTERVAL_MS, MAX_CONCURRENT_NETWORK_REQS,
        MAX_CONNECTION_DELAY_MS, MAX_FRAME_SIZE, MAX_FULLNODE_OUTBOUND_CONNECTIONS,
        MAX_INBOUND_CONNECTIONS, NETWORK_CHANNEL_SIZE,
    },
    network_id::NetworkContext,
};
use diem_crypto::x25519::PublicKey;
use diem_infallible::RwLock;
use diem_logger::prelude::*;
use diem_metrics::IntCounterVec;
use diem_network_address_encryption::Encryptor;
use diem_time_service::TimeService;
use diem_types::{chain_id::ChainId, network_address::NetworkAddress};
use network::{
    connectivity_manager::{builder::ConnectivityManagerBuilder, ConnectivityRequest},
    logging::NetworkSchema,
    peer_manager::{
        builder::{AuthenticationMode, PeerManagerBuilder},
        conn_notifs_channel, ConnectionRequestSender,
    },
    protocols::{
        health_checker::{self, builder::HealthCheckerBuilder},
        network::{NewNetworkEvents, NewNetworkSender},
    },
    ProtocolId,
};
use network_simple_onchain_discovery::{
    builder::ValidatorSetChangeListenerBuilder, gen_simple_discovery_reconfig_subscription,
};
use std::{clone::Clone, collections::HashMap, sync::Arc};
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
    time_service: TimeService,
    network_context: Arc<NetworkContext>,

    configuration_change_listener_builder: Option<ValidatorSetChangeListenerBuilder>,
    connectivity_manager_builder: Option<ConnectivityManagerBuilder>,
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
        trusted_peers: Arc<RwLock<PeerSet>>,
        network_context: Arc<NetworkContext>,
        time_service: TimeService,
        listen_address: NetworkAddress,
        authentication_mode: AuthenticationMode,
        max_frame_size: usize,
        enable_proxy_protocol: bool,
        network_channel_size: usize,
        max_concurrent_network_reqs: usize,
        inbound_connection_limit: usize,
        inbound_rate_limit_config: Option<RateLimitConfig>,
        outbound_rate_limit_config: Option<RateLimitConfig>,
    ) -> Self {
        // A network cannot exist without a PeerManager
        // TODO:  construct this in create and pass it to new() as a parameter. The complication is manual construction of NetworkBuilder in various tests.
        let peer_manager_builder = PeerManagerBuilder::create(
            chain_id,
            network_context.clone(),
            time_service.clone(),
            listen_address,
            trusted_peers,
            authentication_mode,
            network_channel_size,
            max_concurrent_network_reqs,
            max_frame_size,
            enable_proxy_protocol,
            inbound_connection_limit,
            inbound_rate_limit_config,
            outbound_rate_limit_config,
        );

        NetworkBuilder {
            state: State::CREATED,
            executor: None,
            time_service,
            network_context,
            configuration_change_listener_builder: None,
            connectivity_manager_builder: None,
            health_checker_builder: None,
            peer_manager_builder,
            reconfig_subscriptions: vec![],
        }
    }

    pub fn new_for_test(
        chain_id: ChainId,
        seeds: PeerSet,
        trusted_peers: Arc<RwLock<PeerSet>>,
        network_context: Arc<NetworkContext>,
        time_service: TimeService,
        listen_address: NetworkAddress,
        authentication_mode: AuthenticationMode,
    ) -> NetworkBuilder {
        let mutual_authentication = matches!(authentication_mode, AuthenticationMode::Mutual(_));

        let mut builder = NetworkBuilder::new(
            chain_id,
            trusted_peers.clone(),
            network_context,
            time_service,
            listen_address,
            authentication_mode,
            MAX_FRAME_SIZE,
            false, /* Disable proxy protocol */
            NETWORK_CHANNEL_SIZE,
            MAX_CONCURRENT_NETWORK_REQS,
            MAX_INBOUND_CONNECTIONS,
            None,
            None,
        );

        builder.add_connectivity_manager(
            seeds,
            trusted_peers,
            MAX_FULLNODE_OUTBOUND_CONNECTIONS,
            CONNECTION_BACKOFF_BASE,
            MAX_CONNECTION_DELAY_MS,
            CONNECTIVITY_CHECK_INTERVAL_MS,
            NETWORK_CHANNEL_SIZE,
            mutual_authentication,
        );

        builder
    }

    /// Create a new NetworkBuilder based on the provided configuration.
    pub fn create(
        chain_id: ChainId,
        role: RoleType,
        config: &NetworkConfig,
        time_service: TimeService,
    ) -> NetworkBuilder {
        let peer_id = config.peer_id();
        let identity_key = config.identity_key();
        let pubkey = identity_key.public_key();

        let authentication_mode = if config.mutual_authentication {
            AuthenticationMode::Mutual(identity_key)
        } else {
            AuthenticationMode::ServerOnly(identity_key)
        };

        let network_context = Arc::new(NetworkContext::new(
            role,
            config.network_id.clone(),
            peer_id,
        ));

        let trusted_peers = Arc::new(RwLock::new(HashMap::new()));

        let mut network_builder = NetworkBuilder::new(
            chain_id,
            trusted_peers.clone(),
            network_context,
            time_service,
            config.listen_address.clone(),
            authentication_mode,
            config.max_frame_size,
            config.enable_proxy_protocol,
            config.network_channel_size,
            config.max_concurrent_network_reqs,
            config.max_inbound_connections,
            config.inbound_rate_limit_config,
            config.outbound_rate_limit_config,
        );

        network_builder.add_connection_monitoring(
            config.ping_interval_ms,
            config.ping_timeout_ms,
            config.ping_failures_tolerated,
        );

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
            || !config.seeds.is_empty()
        {
            let seeds = merge_seeds(config);

            network_builder.add_connectivity_manager(
                seeds,
                trusted_peers,
                config.max_outbound_connections,
                config.connection_backoff_base,
                config.max_connection_delay_ms,
                config.connectivity_check_interval_ms,
                config.network_channel_size,
                config.mutual_authentication,
            );
        }

        match &config.discovery_method {
            DiscoveryMethod::Onchain => {
                network_builder.add_validator_set_listener(pubkey, config.encryptor());
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
            .start_validator_set_listener()
    }

    pub fn reconfig_subscriptions(&mut self) -> &mut Vec<ReconfigSubscription> {
        &mut self.reconfig_subscriptions
    }

    pub fn network_context(&self) -> Arc<NetworkContext> {
        self.network_context.clone()
    }

    pub fn conn_mgr_reqs_tx(&self) -> Option<channel::Sender<ConnectivityRequest>> {
        self.connectivity_manager_builder
            .as_ref()
            .map(|conn_mgr_builder| conn_mgr_builder.conn_mgr_reqs_tx())
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
        seeds: PeerSet,
        trusted_peers: Arc<RwLock<PeerSet>>,
        max_outbound_connections: usize,
        connection_backoff_base: u64,
        max_connection_delay_ms: u64,
        connectivity_check_interval_ms: u64,
        channel_size: usize,
        mutual_authentication: bool,
    ) -> &mut Self {
        let pm_conn_mgr_notifs_rx = self.add_connection_event_listener();
        let outbound_connection_limit = if !self.network_context.network_id().is_validator_network()
        {
            Some(max_outbound_connections)
        } else {
            None
        };

        self.connectivity_manager_builder = Some(ConnectivityManagerBuilder::create(
            self.network_context(),
            self.time_service.clone(),
            trusted_peers,
            seeds,
            connectivity_check_interval_ms,
            connection_backoff_base,
            max_connection_delay_ms,
            channel_size,
            ConnectionRequestSender::new(self.peer_manager_builder.connection_reqs_tx()),
            pm_conn_mgr_notifs_rx,
            outbound_connection_limit,
            mutual_authentication,
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

    fn add_validator_set_listener(
        &mut self,
        pubkey: PublicKey,
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
            Some(ValidatorSetChangeListenerBuilder::create(
                self.network_context.clone(),
                pubkey,
                encryptor,
                conn_mgr_reqs_tx,
                simple_discovery_reconfig_rx,
            ));
        self
    }

    fn start_validator_set_listener(&mut self) -> &mut Self {
        if let Some(configuration_change_listener) =
            self.configuration_change_listener_builder.as_mut()
        {
            configuration_change_listener
                .start(self.executor.as_mut().expect("Executor must exist"));
        }
        self
    }

    /// Add a HealthChecker to the network.
    fn add_connection_monitoring(
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
            self.time_service.clone(),
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

/// Retrieve and merge seeds so that they have all keys associated
fn merge_seeds(config: &NetworkConfig) -> PeerSet {
    config.verify_seeds().expect("Seeds must be well formed");
    let mut seeds = config.seeds.clone();

    // Merge old seed configuration with new seed configuration
    // TODO(gnazario): Once fully migrated, remove `seed_addrs`
    config
        .seed_addrs
        .iter()
        .map(|(peer_id, addrs)| {
            (
                peer_id,
                Peer::from_addrs(PeerRole::ValidatorFullNode, addrs.clone()),
            )
        })
        .for_each(|(peer_id, peer)| {
            seeds
                .entry(*peer_id)
                // Sad clone due to Rust not realizing these are two distinct paths
                .and_modify(|seed| seed.extend(peer.clone()).unwrap())
                .or_insert(peer);
        });

    // Pull public keys out of addresses
    seeds.values_mut().for_each(
        |Peer {
             addresses, keys, ..
         }| {
            addresses
                .iter()
                .filter_map(NetworkAddress::find_noise_proto)
                .for_each(|pubkey| {
                    keys.insert(pubkey);
                });
        },
    );
    seeds
}

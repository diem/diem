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
use futures_util::stream::Fuse;
use libra_config::{
    config::{DiscoveryMethod, NetworkConfig, RoleType, HANDSHAKE_VERSION},
    network_id::{NetworkContext, NetworkId},
};
use libra_crypto::x25519;
use libra_logger::prelude::*;
use libra_metrics::IntCounterVec;
use libra_network_address::NetworkAddress;
use libra_secure_storage::config;
use libra_types::{waypoint::Waypoint, PeerId};
use netcore::transport::{memory, Transport};
use network::{
    connectivity_manager::{ConnectivityManager, ConnectivityManagerConfig, ConnectivityRequest},
    constants, counters,
    peer_manager::{
        conn_notifs_channel, ConnectionRequest, ConnectionRequestSender, PeerManager,
        PeerManagerConfig, PeerManagerNotification, PeerManagerRequest, PeerManagerRequestSender,
    },
    protocols::{
        discovery::{self, Discovery, DiscoveryConfig},
        health_checker::{self, HealthChecker, HealthCheckerConfig},
        network::{NewNetworkEvents, NewNetworkSender},
        wire::handshake::v1::SupportedProtocols,
    },
    transport::{self, Connection, LibraNetTransport, LIBRA_TCP_TRANSPORT},
    ProtocolId,
};
use network_simple_onchain_discovery::{
    gen_simple_discovery_reconfig_subscription, ConfigurationChangeListener,
};
use onchain_discovery::builder::{OnchainDiscoveryBuilder, OnchainDiscoveryBuilderConfig};
use std::{
    clone::Clone,
    collections::HashMap,
    num::NonZeroUsize,
    sync::{Arc, RwLock},
    time::Duration,
};
use storage_interface::DbReader;
use subscription_service::ReconfigSubscription;
use tokio::{
    runtime::Handle,
    time::{interval, Interval},
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
    network_context: NetworkContext,
    // TODO(philiphayes): better support multiple listening addrs
    listen_address: NetworkAddress,
    advertised_address: Option<NetworkAddress>,
    trusted_peers: Arc<RwLock<HashMap<PeerId, x25519::PublicKey>>>,
    authentication_mode: Option<AuthenticationMode>,

    //
    //  State used to build PeerManager.  Potentially impacted by network-external components.
    //
    // Cumulative collection of direct_send_protocols used by all client components.
    direct_send_protocols: Vec<ProtocolId>,
    // Cumulative collection of rpc_protocols used by all client components.
    rpc_protocols: Vec<ProtocolId>,
    // Root Sender for all components that connect to PeerManager for sending.
    pm_reqs_tx: libra_channel::Sender<(PeerId, ProtocolId), PeerManagerRequest>,
    // Root Sender for all components that connect to PeerManager for connecting.
    connection_reqs_tx: libra_channel::Sender<PeerId, ConnectionRequest>,
    // Root Sender for all components that connect to ConnectivityManager.
    conn_mgr_reqs_tx: Option<channel::Sender<ConnectivityRequest>>,

    //
    //  Component configurations
    //
    // ConnectivityManager configuration.
    conn_mgr_cfg: Option<ConnectivityManagerConfig<Fuse<Interval>, ExponentialBackoff>>,
    // Indicate if the ConfigurationChagneListener has been added or not.
    config_change_listener_cfg: bool,
    // Gossip Discovery configuration.
    discovery_cfg: Option<DiscoveryConfig<Fuse<Interval>>>,
    // Onchain Discovery configuration.
    onchain_discovery_cfg: Option<OnchainDiscoveryBuilderConfig>,
    // HealthChecker configuration.
    health_checker_cfg: Option<HealthCheckerConfig<Fuse<Interval>>>,
    // PeerManager configuration
    peer_mgr_cfg: Option<PeerManagerConfig>,

    //
    //  Internal components
    //
    // Internal ConnectivityManager component.
    conn_mgr: Option<ConnectivityManager<Fuse<Interval>, ExponentialBackoff>>,
    // ConfigurationChangeListener associated with ConnectivityManager
    config_change_listener: Option<ConfigurationChangeListener>,
    // Internal Discovery component.
    discovery: Option<Discovery<Fuse<Interval>>>,
    // Internal Onchain discovery channel
    onchain_discovery: Option<OnchainDiscoveryBuilder>,
    // Internal HealthChecker component.
    health_checker: Option<HealthChecker<Fuse<Interval>>>,
    // Internal PeerManager component.
    // TODO:  Actually slam a PeerManager in this space.
    //    peer_mgr: Option<u64>,

    // Boolean indicating whether or not the network has been built.
    built: bool,

    // Boolean indicating whether or not the built network being built has been started.
    started: bool,
}

impl NetworkBuilder {
    /// Return a new NetworkBuilder initialized with default configuration values.
    // TODO:  make this call private.  It should only be invoked through create.
    pub fn new(
        network_id: NetworkId,
        peer_id: PeerId,
        role: RoleType,
        listen_address: NetworkAddress,
    ) -> NetworkBuilder {
        // Setup channel to send/receive requests to/from peer manager.
        // pm_re
        let (pm_reqs_tx, pm_reqs_rx) = libra_channel::new(
            QueueStyle::FIFO,
            NonZeroUsize::new(constants::NETWORK_CHANNEL_SIZE).unwrap(),
            Some(&counters::PENDING_PEER_MANAGER_REQUESTS),
        );
        // Setup channel to send/receive connection requests to/from peer manager.
        let (connection_reqs_tx, connection_reqs_rx) = libra_channel::new(
            QueueStyle::FIFO,
            NonZeroUsize::new(constants::NETWORK_CHANNEL_SIZE).unwrap(),
            None,
        );

        let network_context = NetworkContext::new(network_id, role, peer_id);
        let peer_mgr_cfg = PeerManagerConfig::new(
            network_context.clone(),
            listen_address.clone(),
            constants::NETWORK_CHANNEL_SIZE,
            constants::MAX_CONCURRENT_NETWORK_REQS,
            constants::MAX_CONCURRENT_NETWORK_NOTIFS,
            pm_reqs_rx,
            connection_reqs_rx,
        );

        NetworkBuilder {
            network_context,
            listen_address,
            advertised_address: None,
            trusted_peers: Arc::new(RwLock::new(HashMap::new())),
            authentication_mode: None,
            direct_send_protocols: vec![],
            rpc_protocols: vec![],
            pm_reqs_tx,
            connection_reqs_tx,
            conn_mgr_reqs_tx: None,
            conn_mgr_cfg: None,
            config_change_listener_cfg: false,
            discovery_cfg: None,
            onchain_discovery_cfg: None,
            health_checker_cfg: None,
            peer_mgr_cfg: Some(peer_mgr_cfg),
            conn_mgr: None,
            config_change_listener: None,
            discovery: None,
            onchain_discovery: None,
            health_checker: None,
            //            peer_mgr: None,
            started: false,
            built: false,
        }
    }

    pub fn network_context(&self) -> &NetworkContext {
        &self.network_context
    }

    /// The primary public interface for creating a NetworkBuilder.
    /// Construct a NetworkBuilder in which the core networking components are configured
    pub fn create(
        config: &mut NetworkConfig,
        role: RoleType,
        libra_db: Arc<dyn DbReader>,
        waypoint: Waypoint,
    ) -> NetworkBuilder {
        let identity_key = config::identity_key(config);
        let peer_id = config::peer_id(config);

        let mut network_builder = NetworkBuilder::new(
            config.network_id.clone(),
            peer_id,
            role,
            config.listen_address.clone(),
        );
        // TODO:  these ping parameters should be provided as part of network_config.
        network_builder.add_connection_monitoring(
            constants::PING_INTERVAL_MS,
            constants::PING_TIMEOUT_MS,
            constants::PING_FAILURES_TOLERATED,
        );

        // Sanity check seed peer addresses.
        config
            .seed_peers
            .verify_libranet_addrs()
            .expect("Seed peer addresses must be well-formed");
        let seed_peers = config.seed_peers.seed_peers.clone();

        if config.mutual_authentication {
            let network_peers = config.network_peers.peers.clone();
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
                .add_connectivity_manager(
                    config.connectivity_check_interval_ms,
                    constants::MAX_CONNECTION_DELAY_MS,
                    seed_peers,
                );
            if role == RoleType::Validator {
                network_builder.add_configuration_change_listener();
            }
        } else {
            // Enforce the outgoing connection (dialer) verifies the identity of the listener (server)
            network_builder.authentication_mode(AuthenticationMode::ServerOnly(identity_key));
            if !seed_peers.is_empty() {
                network_builder.add_connectivity_manager(
                    config.connectivity_check_interval_ms,
                    constants::MAX_CONNECTION_DELAY_MS,
                    seed_peers,
                );
            }
        }

        match &config.discovery_method {
            DiscoveryMethod::Gossip(gossip_config) => {
                network_builder
                    .advertised_address(gossip_config.advertised_address.clone())
                    .add_gossip_discovery(gossip_config.discovery_interval_ms);
            }
            DiscoveryMethod::Onchain => {
                network_builder.add_onchain_discovery(waypoint, libra_db);
            }
            DiscoveryMethod::None => (),
        };

        network_builder
    }

    /// Build all internal components and prepare the network for use.
    /// Return any ReconfigurationSubscriptions.
    // TODO:  remove the runtime parameter
    pub fn build(&mut self, runtime: &Handle) -> (NetworkAddress, Vec<ReconfigSubscription>) {
        // Can only build the network once.
        assert!(!self.built);
        self.built = true;

        let mut reconfig_subscriptions = Vec::new();

        self.build_health_checker();
        reconfig_subscriptions.append(&mut self.build_connectivity_manager());
        self.build_gossip_discovery();
        self.build_onchain_discovery(runtime);

        // Must be last because it also starts the PeerManager.
        // TODO:  make this simply 'build'.
        let listen_address = self.build_and_start_peer_manager(runtime);

        (listen_address, reconfig_subscriptions)
    }

    /// Starts the network and the internal components.
    pub fn start(&mut self, executor: &Handle) {
        // Network must be built before it can be started.
        assert!(self.built);
        // Network can be started at most once.
        assert!(!self.started);
        self.started = true;

        self.start_health_checker(executor);

        self.start_connectivity_manager(executor);

        self.start_gossip_discovery(executor);

        self.start_onchain_discovery(executor);

        debug!("Started all internal components");
    }

    // TODO:  Make private/Remove
    pub fn peer_id(&self) -> PeerId {
        self.network_context.peer_id()
    }

    /// Set network authentication mode.
    // TODO: Make private/Remove
    pub fn authentication_mode(&mut self, authentication_mode: AuthenticationMode) -> &mut Self {
        self.authentication_mode = Some(authentication_mode);
        self
    }

    /// Set an address to advertise, if different from the listen address
    // TODO: Make private/Remove
    pub fn advertised_address(&mut self, advertised_address: NetworkAddress) -> &mut Self {
        self.advertised_address = Some(advertised_address);
        self
    }

    /// Set trusted peers.
    // TODO:Make private/Remove
    pub fn trusted_peers(
        &mut self,
        trusted_peers: HashMap<PeerId, x25519::PublicKey>,
    ) -> &mut Self {
        *self.trusted_peers.write().unwrap() = trusted_peers;
        self
    }

    // TODO:  remove entirely and move SimpleOnchainDiscovery inside
    pub fn conn_mgr_reqs_tx(&self) -> Option<channel::Sender<ConnectivityRequest>> {
        self.conn_mgr_reqs_tx.clone()
    }

    fn supported_protocols(&self) -> SupportedProtocols {
        self.direct_send_protocols
            .iter()
            .chain(&self.rpc_protocols)
            .into()
    }

    /// Adds a endpoints for the provided configuration.  Returns NetworkSender and NetworkEvent which
    /// can be attached to other components.  Used to attach both internal and external components.
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
        // Cannot modify the configuration or update wires once the network has been built.
        assert!(!self.built);
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

    /// Add a handler for given protocols using raw bytes.
    /// Utility function for `add_protocol_handler`
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
            if let Some(ref mut config) = self.peer_mgr_cfg {
                config.add_upstream_handler(protocol, network_notifs_tx.clone());
            }
        }
        // Auto-subscribe all application level handlers to connection events.
        let connection_notifs_rx = self.add_connection_event_listener();
        (
            PeerManagerRequestSender::new(self.pm_reqs_tx.clone()),
            network_notifs_rx,
            ConnectionRequestSender::new(self.connection_reqs_tx.clone()),
            connection_notifs_rx,
        )
    }

    /// Add a connection event handler to the `PeerManagerConfig`.
    fn add_connection_event_listener(&mut self) -> conn_notifs_channel::Receiver {
        let (tx, rx) = conn_notifs_channel::new();
        if let Some(ref mut config) = self.peer_mgr_cfg {
            config.add_connection_event_handler(tx);
        }
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
    // TODO: make private
    pub fn add_connectivity_manager(
        &mut self,
        connectivity_check_interval_ms: u64,
        max_connection_delay_ms: u64,
        seed_peers: HashMap<PeerId, Vec<NetworkAddress>>,
    ) -> &mut Self {
        // Cannot modify the configuration or update wires once the network has been built.
        assert!(!self.built);
        let conn_mgr_cfg = ConnectivityManagerConfig::new(
            self.network_context.clone(),
            self.trusted_peers.clone(),
            seed_peers,
            interval(Duration::from_millis(connectivity_check_interval_ms)).fuse(),
            ExponentialBackoff::from_millis(2).factor(1000),
            max_connection_delay_ms,
        );

        self.conn_mgr_cfg = Some(conn_mgr_cfg);
        self
    }

    /// Build the configured ['ConnectivityManager'] if it exists;  do nothing if not.
    /// If the ConfigurationChangeListener is additionally configured, it is built as well.
    fn build_connectivity_manager(&mut self) -> Vec<ReconfigSubscription> {
        let mut reconfig_subscriptions = Vec::new();
        if let Some(conn_mgr_cfg) = self.conn_mgr_cfg.take() {
            let (conn_mgr_reqs_tx, conn_mgr_reqs_rx) = channel::new(
                constants::NETWORK_CHANNEL_SIZE,
                &counters::PENDING_CONNECTIVITY_MANAGER_REQUESTS,
            );

            let pm_conn_mgr_notifs_rx = self.add_connection_event_listener();
            let conn_mgr = ConnectivityManager::new(
                conn_mgr_cfg,
                ConnectionRequestSender::new(self.connection_reqs_tx.clone()),
                pm_conn_mgr_notifs_rx,
                conn_mgr_reqs_rx,
            );

            self.conn_mgr = Some(conn_mgr);

            // Build the ConfigurationChangeListener
            if self.config_change_listener_cfg {
                let (simple_discovery_reconfig_subscription, simple_discovery_reconfig_rx) =
                    gen_simple_discovery_reconfig_subscription();
                reconfig_subscriptions.push(simple_discovery_reconfig_subscription);
                self.config_change_listener = Some(ConfigurationChangeListener::new(
                    conn_mgr_reqs_tx,
                    simple_discovery_reconfig_rx,
                    self.network_context.role(),
                ));
            }
        }
        reconfig_subscriptions
    }

    /// Start the built ['ConnectivityManager'] if it exists; do nothing if not.
    /// Additionally starts the ['ConfigurationChangeListener'] if present.
    //  TODO:  Either forcefully merge ConfigurationChangeListener into a ConnectivityManagerBuilder or decouple entirely.
    fn start_connectivity_manager(&mut self, executor: &Handle) {
        match self.conn_mgr.take() {
            Some(conn_mgr) => {
                // Start the ConnectivityManager
                executor.spawn(executor.enter(|| conn_mgr).start());
                debug!("Started ConnectivityManager");
                if let Some(listener) = self.config_change_listener.take() {
                    // Start the ConfigurationChangeListener
                    executor.spawn(executor.enter(|| listener).start());
                    debug!("Started NetworkConnectivityListener");
                }
            }
            None => debug!("No ConnectivityManager"),
        }
    }

    fn add_configuration_change_listener(&mut self) {
        self.config_change_listener_cfg = true;
    }

    /// Add the (gossip) [`Discovery`] protocol to the network.
    ///
    /// (gossip) [`Discovery`] discovers other eligible peers' network addresses
    /// by exchanging the full set of known peer network addresses with connected
    /// peers as a network protocol.
    ///
    /// This is for testing purposes only and should not be used in production networks.
    pub fn add_gossip_discovery(&mut self, discovery_interval_ms: u64) -> &mut Self {
        // Cannot modify the configuration or update wires once the network has been built.
        assert!(!self.built);
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

        self.discovery_cfg = Some(DiscoveryConfig::new(
            self.network_context.clone(),
            vec![advertised_address],
            interval(Duration::from_millis(discovery_interval_ms)).fuse(),
        ));
        self
    }

    /// Build the configured ['Discovery'] if it exists; do nothing if not.
    fn build_gossip_discovery(&mut self) {
        if let Some(discovery_cfg) = self.discovery_cfg.take() {
            let conn_mgr_reqs_tx = self
                .conn_mgr_reqs_tx()
                .expect("ConnectivityManager not enabled");
            // Get handles for network events and sender.
            let (discovery_network_tx, discovery_network_rx) =
                self.add_protocol_handler(discovery::network_endpoint_config());

            let discovery = Discovery::new(
                discovery_cfg,
                discovery_network_tx,
                discovery_network_rx,
                conn_mgr_reqs_tx,
            );
            self.discovery = Some(discovery);
        }
    }

    /// Start the built ['Discovery'] if it exists; do nothing if not.
    fn start_gossip_discovery(&mut self, executor: &Handle) {
        match self.discovery.take() {
            Some(discovery) => {
                executor.spawn(executor.enter(|| discovery).start());
                debug!("Started Discovery");
            }
            None => debug!("No Discovery"),
        }
    }

    /// Add the configuration for ['OnchainDiscovery'] to the network.
    fn add_onchain_discovery(
        &mut self,
        waypoint: Waypoint,
        libra_db: Arc<dyn DbReader>,
    ) -> &mut Self {
        // Cannot modify the configuration or update wires once the network has been built.
        assert!(!self.built);
        let (network_tx, discovery_events) = self
            .add_protocol_handler(onchain_discovery::network_interface::network_endpoint_config());

        let onchain_discovery_builder_cfg = OnchainDiscoveryBuilderConfig::new(
            self.conn_mgr_reqs_tx()
                .expect("connectivitymanager must be installed"),
            network_tx,
            discovery_events,
            self.network_context.clone(),
            waypoint,
            libra_db,
        );
        self.onchain_discovery_cfg = Some(onchain_discovery_builder_cfg);
        self
    }

    /// Build the configured ['OnchainDiscovery'] if it exists; do nothing if not.
    fn build_onchain_discovery(&mut self, runtime: &Handle) {
        if let Some(config) = self.onchain_discovery_cfg.take() {
            let onchain_discovery_builder = OnchainDiscoveryBuilder::build(config, &runtime);
            self.onchain_discovery = Some(onchain_discovery_builder);
        }
    }

    /// Start the built ['OnchainDiscovery'] if it exists; do nothing if not.
    fn start_onchain_discovery(&mut self, executor: &Handle) {
        match self.onchain_discovery.take() {
            Some(onchain_discovery) => {
                onchain_discovery.start(executor);
                debug!("Started OnchainDiscovery")
            }
            None => debug!("No OnchainDiscovery"),
        }
    }
    /// Add a configuration for ['HealthChecker'].
    fn add_connection_monitoring(
        &mut self,
        ping_interval_ms: u64,
        ping_timeout_ms: u64,
        ping_failures_tolerated: u64,
    ) -> &mut Self {
        // Cannot modify the configuration or update wires once the network has been built.
        assert!(!self.built);
        self.health_checker_cfg = Some(HealthCheckerConfig::new(
            self.network_context.clone(),
            interval(Duration::from_millis(ping_interval_ms)).fuse(),
            Duration::from_millis(ping_timeout_ms),
            ping_failures_tolerated,
        ));
        debug!("Configured HealthChecker");
        self
    }

    /// Build the configured ['HealthChecker'] if it exists; if not do nothing.
    fn build_health_checker(&mut self) {
        if let Some(health_checker_cfg) = self.health_checker_cfg.take() {
            // Initialize and start HealthChecker.
            let (hc_network_tx, hc_network_rx) =
                self.add_protocol_handler(health_checker::network_endpoint_config());

            self.health_checker = Some(HealthChecker::new(
                health_checker_cfg,
                hc_network_tx,
                hc_network_rx,
            ));
            debug!("Built the HealthChecker");
        }
    }

    /// Starts the built ['HealthChecker'] if it exists; does nothing otherwise.
    fn start_health_checker(&mut self, executor: &Handle) {
        match self.health_checker.take() {
            Some(health_checker) => {
                executor.spawn(executor.enter(|| health_checker).start());
                debug!("Started HealthChecker");
            }
            None => debug!("No HealthChecker"),
        }
    }

    /// Create the configured transport and start PeerManager.
    /// Return the actual NetworkAddress over which this peer is listening.
    // NOTE:  given supported_protocols, it is probably important that this is not built
    // until *ALL* wires have been configured/created.
    fn build_and_start_peer_manager(&mut self, runtime: &Handle) -> NetworkAddress {
        use libra_network_address::Protocol::*;

        let network_id = self.network_context.network_id().clone();
        let protos = self.supported_protocols();

        let authentication_mode = self
            .authentication_mode
            .take()
            .expect("Authentication Mode not set");

        let (key, maybe_trusted_peers, peer_id) = match authentication_mode {
            // validator-operated full node
            AuthenticationMode::ServerOnly(key)
                if self.network_context.peer_id() == PeerId::default() =>
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
                    network_id,
                    protos,
                ),
                runtime,
            ),
            [Memory(_)] => self.build_with_transport(
                LibraNetTransport::new(
                    memory::MemoryTransport,
                    peer_id,
                    key,
                    maybe_trusted_peers,
                    HANDSHAKE_VERSION,
                    network_id,
                    protos,
                ),
                runtime,
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
    // Note that this is a separate utility function because the type for Transport is
    // configuration dependent.
    fn build_with_transport<TTransport, TSocket>(
        &mut self,
        transport: TTransport,
        executor: &Handle,
    ) -> NetworkAddress
    where
        TTransport: Transport<Output = Connection<TSocket>> + Send + 'static,
        TSocket: transport::TSocket,
    {
        let peer_mgr = PeerManager::new(
            self.peer_mgr_cfg
                .take()
                .expect("PeerManager must be configured"),
            executor.clone(),
            transport,
        );
        let listen_addr = peer_mgr.listen_addr().clone();

        // TODO:  Store the PeerManager in NetworkBuilder so the start can be decoupled from the build.
        executor.spawn(peer_mgr.start());
        debug!("Started peer manager");
        listen_addr
    }
}

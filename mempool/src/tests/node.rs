// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    core_mempool::{CoreMempool, TimelineState},
    network::{MempoolNetworkEvents, MempoolSyncMsg},
    shared_mempool::{
        network::MempoolNetworkSender, start_shared_mempool, types::SharedMempoolNotification,
    },
    tests::common::TestTransaction,
};
use channel::{diem_channel, message_queues::QueueStyle};
use diem_config::{
    config::{NodeConfig, PeerNetworkId, PeerRole, RoleType},
    network_id::{NetworkContext, NetworkId, NodeNetworkId},
};
use diem_infallible::{Mutex, MutexGuard, RwLock};
use diem_types::{account_address::AccountAddress, transaction::GovernanceRole, PeerId};
use enum_dispatch::enum_dispatch;
use futures::{
    channel::mpsc::{self, unbounded, UnboundedReceiver},
    FutureExt, StreamExt,
};
use mempool_notifications::MempoolNotifier;
use netcore::transport::ConnectionOrigin;
use network::{
    peer_manager::{
        conn_notifs_channel, ConnectionNotification, ConnectionRequestSender,
        PeerManagerNotification, PeerManagerRequest, PeerManagerRequestSender,
    },
    protocols::network::{NetworkEvents, NewNetworkEvents, NewNetworkSender},
    transport::ConnectionMetadata,
    DisconnectReason, ProtocolId,
};
use rand::rngs::StdRng;
use std::{collections::HashMap, sync::Arc};
use storage_interface::mock::MockDbReader;
use tokio::runtime::{Builder, Runtime};
use vm_validator::mocks::mock_vm_validator::MockVMValidator;

type MempoolNetworkHandle = (
    NodeNetworkId,
    MempoolNetworkSender,
    NetworkEvents<MempoolSyncMsg>,
);

/// This is a simple node identifier for testing
/// This keeps track of the `NodeType` and a simple index
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialOrd, PartialEq)]
pub struct NodeId {
    pub node_type: NodeType,
    num: u32,
}

impl NodeId {
    pub(crate) fn new(node_type: NodeType, num: u32) -> Self {
        NodeId { node_type, num }
    }
}

/// Yet another type, to determine the differences between
/// Validators, ValidatorFullNodes, and FullNodes
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialOrd, PartialEq)]
pub enum NodeType {
    Validator,
    ValidatorFullNode,
    FullNode,
}

/// A union type for all types of simulated nodes
#[enum_dispatch(NodeInfoTrait)]
#[derive(Clone, Debug)]
pub enum NodeInfo {
    Validator(ValidatorNodeInfo),
    ValidatorFull(ValidatorFullNodeInfo),
    Full(FullNodeInfo),
}

/// Accessors to the union type of all simulated nodes
/// TODO: This really only supports 2 networks, but there are currently nodes with 3
#[enum_dispatch]
pub trait NodeInfoTrait {
    /// `PeerId` for primary network interface
    fn primary_peer_id(&self) -> PeerId;
    /// `NetworkId` for primary network interface
    fn primary_network(&self) -> NetworkId;
    /// `RoleType` of the `Node`
    fn role(&self) -> RoleType;
    /// `PeerRole` for use in the upstream / downstream peers
    fn peer_role(&self) -> PeerRole;
    /// `PeerId` for secondary network interface
    fn secondary_peer_id(&self) -> Option<PeerId>;
    /// `NetworkId` for secondary network interface
    fn secondary_network(&self) -> Option<NetworkId>;

    /// `PeerId` by using the `bool` input for simplicity
    fn peer_id(&self, is_primary: bool) -> PeerId {
        if is_primary {
            self.primary_peer_id()
        } else {
            self.secondary_peer_id().unwrap()
        }
    }

    /// `NetworkId` by using the `bool` input for simplicity
    fn network(&self, is_primary: bool) -> NetworkId {
        if is_primary {
            self.primary_network()
        } else {
            self.secondary_network().unwrap()
        }
    }

    fn network_context(&self, is_primary: bool) -> NetworkContext {
        let role = self.role();
        let network = self.network(is_primary);
        let peer_id = self.peer_id(is_primary);
        NetworkContext::new(role, network, peer_id)
    }
}

#[derive(Clone, Debug)]
pub struct ValidatorNodeInfo {
    primary_peer_id: PeerId,
}

impl ValidatorNodeInfo {
    fn new(peer_id: PeerId) -> Self {
        ValidatorNodeInfo {
            primary_peer_id: peer_id,
        }
    }
}

impl NodeInfoTrait for ValidatorNodeInfo {
    fn primary_peer_id(&self) -> PeerId {
        self.primary_peer_id
    }

    fn primary_network(&self) -> NetworkId {
        NetworkId::Validator
    }

    fn role(&self) -> RoleType {
        RoleType::Validator
    }

    fn peer_role(&self) -> PeerRole {
        PeerRole::Validator
    }

    fn secondary_peer_id(&self) -> Option<PeerId> {
        None
    }

    fn secondary_network(&self) -> Option<NetworkId> {
        None
    }
}

#[derive(Clone, Debug)]
pub struct ValidatorFullNodeInfo {
    primary_peer_id: PeerId,
    secondary_peer_id: PeerId,
}

impl ValidatorFullNodeInfo {
    fn new(primary_peer_id: PeerId, secondary_peer_id: PeerId) -> Self {
        ValidatorFullNodeInfo {
            primary_peer_id,
            secondary_peer_id,
        }
    }
}

impl NodeInfoTrait for ValidatorFullNodeInfo {
    fn primary_peer_id(&self) -> PeerId {
        self.primary_peer_id
    }

    fn primary_network(&self) -> NetworkId {
        NetworkId::vfn_network()
    }

    fn role(&self) -> RoleType {
        RoleType::FullNode
    }

    fn peer_role(&self) -> PeerRole {
        PeerRole::ValidatorFullNode
    }

    fn secondary_peer_id(&self) -> Option<PeerId> {
        Some(self.secondary_peer_id)
    }

    fn secondary_network(&self) -> Option<NetworkId> {
        Some(NetworkId::Public)
    }
}

#[derive(Clone, Debug)]
pub struct FullNodeInfo {
    peer_id: PeerId,
    peer_role: PeerRole,
}

impl FullNodeInfo {
    fn new(peer_id: PeerId, peer_role: PeerRole) -> Self {
        FullNodeInfo { peer_id, peer_role }
    }
}

impl NodeInfoTrait for FullNodeInfo {
    fn primary_peer_id(&self) -> PeerId {
        self.peer_id
    }

    fn primary_network(&self) -> NetworkId {
        NetworkId::Public
    }

    fn role(&self) -> RoleType {
        RoleType::FullNode
    }

    fn peer_role(&self) -> PeerRole {
        self.peer_role
    }

    fn secondary_peer_id(&self) -> Option<PeerId> {
        None
    }

    fn secondary_network(&self) -> Option<NetworkId> {
        None
    }
}

/// Provides a `NodeInfo` and `NodeConfig` for a validator
pub fn validator_config(rng: &mut StdRng, account_idx: u32) -> (ValidatorNodeInfo, NodeConfig) {
    let config =
        NodeConfig::random_with_template(account_idx, &NodeConfig::default_for_validator(), rng);

    let peer_id = config
        .validator_network
        .as_ref()
        .expect("Validator must have a validator network")
        .peer_id();
    (ValidatorNodeInfo::new(peer_id), config)
}

/// Provides a `NodeInfo` and `NodeConfig` for a ValidatorFullNode
pub fn vfn_config(rng: &mut StdRng, account_idx: u32) -> (ValidatorFullNodeInfo, NodeConfig) {
    let vfn_config = NodeConfig::random_with_template(
        account_idx,
        &NodeConfig::default_for_validator_full_node(),
        rng,
    );

    let primary_peer_id = vfn_config
        .full_node_networks
        .iter()
        .find(|network| network.network_id.is_vfn_network())
        .expect("VFN must have a VFN network")
        .peer_id();

    let secondary_peer_id = vfn_config
        .full_node_networks
        .iter()
        .filter(|network| network.network_id == NetworkId::Public)
        .last()
        .unwrap()
        .peer_id();
    (
        ValidatorFullNodeInfo::new(primary_peer_id, secondary_peer_id),
        vfn_config,
    )
}

/// Provides a `NodeInfo` and `NodeConfig` for a public full node
pub fn public_full_node_config(
    rng: &mut StdRng,
    account_idx: u32,
    peer_role: PeerRole,
) -> (FullNodeInfo, NodeConfig) {
    let fn_config = NodeConfig::random_with_template(
        account_idx,
        &NodeConfig::default_for_public_full_node(),
        rng,
    );

    let peer_id = fn_config
        .full_node_networks
        .iter()
        .find(|network| network.network_id == NetworkId::Public)
        .expect("Full Node must have a public network")
        .peer_id();
    (FullNodeInfo::new(peer_id, peer_role), fn_config)
}

/// A struct representing a node, it's network interfaces, mempool, and a mempool event subscriber
pub struct Node {
    /// The identifying Node
    node_info: NodeInfo,
    /// `CoreMempool` for this node
    mempool: Arc<Mutex<CoreMempool>>,
    /// Network interfaces for a node
    network_interfaces: HashMap<NetworkId, NodeNetworkInterface>,
    /// Tokio runtime
    runtime: Arc<Runtime>,
    /// Subscriber for mempool events
    subscriber: UnboundedReceiver<SharedMempoolNotification>,
}

/// Reimplement `NodeInfoTrait` for simplicity
impl NodeInfoTrait for Node {
    fn primary_peer_id(&self) -> AccountAddress {
        self.node_info.primary_peer_id()
    }

    fn primary_network(&self) -> NetworkId {
        self.node_info.primary_network()
    }

    fn role(&self) -> RoleType {
        self.node_info.role()
    }

    fn peer_role(&self) -> PeerRole {
        self.node_info.peer_role()
    }

    fn secondary_peer_id(&self) -> Option<AccountAddress> {
        self.node_info.secondary_peer_id()
    }

    fn secondary_network(&self) -> Option<NetworkId> {
        self.node_info.secondary_network()
    }
}

impl Node {
    /// Sets up a single node by starting up mempool and any network handles
    pub fn new(node: NodeInfo, config: NodeConfig) -> Node {
        let (network_interfaces, network_handles) = setup_node_network_interfaces(&node);
        let (mempool, runtime, subscriber) = start_node_mempool(config, network_handles);

        Node {
            node_info: node,
            mempool,
            network_interfaces,
            runtime: Arc::new(runtime),
            subscriber,
        }
    }

    /// Retrieves a `CoreMempool`
    pub fn mempool(&self) -> MutexGuard<CoreMempool> {
        self.mempool.lock()
    }

    /// Queues transactions for sending on a node.  Must use `broadcast_txns` to send to other nodes
    pub fn add_txns(&self, txns: Vec<TestTransaction>) {
        let mut mempool = self.mempool();
        for txn in txns {
            let transaction = txn.make_signed_transaction_with_max_gas_amount(5);
            mempool.add_txn(
                transaction.clone(),
                0,
                transaction.gas_unit_price(),
                0,
                TimelineState::NotReady,
                GovernanceRole::NonGovernanceRole,
            );
        }
    }

    /// Commits transactions and removes them from the local mempool, stops them from being broadcasted later
    pub fn commit_txns(&self, txns: Vec<TestTransaction>) {
        if let NodeInfo::Validator(_) = self.node_info {
            let mut mempool = self.mempool();
            for txn in txns {
                mempool.remove_transaction(
                    &TestTransaction::get_address(txn.address),
                    txn.sequence_number,
                    false,
                );
            }
        } else {
            panic!("Can't commit transactions on anything but a validator");
        }
    }

    /// Notifies the `Node` of a `new_peer`
    pub fn send_new_peer_event(
        &mut self,
        is_primary: bool,
        new_peer: PeerId,
        peer_role: PeerRole,
        origin: ConnectionOrigin,
    ) {
        let metadata = ConnectionMetadata::mock_with_role_and_origin(new_peer, peer_role, origin);
        let network_context = self.network_context(is_primary);
        let notif = ConnectionNotification::NewPeer(metadata, Arc::new(network_context));
        self.send_connection_event(is_primary, notif)
    }

    /// Notifies the `Node` of a `lost_peer`
    pub fn send_lost_peer_event(&mut self, is_primary: bool, lost_peer: PeerId) {
        let notif = ConnectionNotification::LostPeer(
            ConnectionMetadata::mock(lost_peer),
            NetworkContext::mock(),
            DisconnectReason::ConnectionLost,
        );
        self.send_connection_event(is_primary, notif)
    }

    /// Sends a connection event, and waits for the notification to arrive
    fn send_connection_event(&mut self, is_primary: bool, notif: ConnectionNotification) {
        self.send_network_notif(is_primary, notif);
        self.wait_for_event(SharedMempoolNotification::PeerStateChange);
    }

    /// Waits for a specific `SharedMempoolNotification` event
    pub fn wait_for_event(&mut self, expected: SharedMempoolNotification) {
        let event = self.runtime.block_on(self.subscriber.next()).unwrap();
        if event == expected {
            return;
        }

        panic!("Failed to get expected event '{:?}'", expected)
    }

    /// Checks that there are no `SharedMempoolNotification`s on the subscriber
    pub fn check_no_subscriber_events(&mut self) {
        assert!(self.subscriber.select_next_some().now_or_never().is_none())
    }

    /// Checks that a node has no pending messages to send.
    pub fn check_no_network_messages_sent(&mut self, is_primary: bool) {
        self.check_no_subscriber_events();
        assert!(self
            .get_network_interface(is_primary)
            .network_reqs_rx
            .select_next_some()
            .now_or_never()
            .is_none())
    }

    /// Retrieves a network interface for a specific `NetworkId` based on whether it's the primary network
    fn get_network_interface(&mut self, is_primary: bool) -> &mut NodeNetworkInterface {
        self.network_interfaces
            .get_mut(&self.network(is_primary))
            .unwrap()
    }

    /// Retrieves the next network request `PeerManagerRequest`
    pub fn get_next_network_req(&mut self, is_primary: bool) -> PeerManagerRequest {
        let runtime = self.runtime.clone();
        self.get_network_interface(is_primary)
            .get_next_network_req(runtime)
    }

    /// Send network request `PeerManagerNotification` from a remote peer to the local node
    pub fn send_network_req(
        &mut self,
        is_primary: bool,
        protocol: ProtocolId,
        notif: PeerManagerNotification,
    ) {
        self.get_network_interface(is_primary)
            .send_network_req(protocol, notif);
    }

    /// Sends a `ConnectionNotification` to the local node
    pub fn send_network_notif(&mut self, is_primary: bool, notif: ConnectionNotification) {
        self.get_network_interface(is_primary)
            .send_connection_notif(notif)
    }
}

/// A simplistic view of the entire network stack for a given `NetworkId`
/// Allows us to mock out the network without dealing with the details
pub struct NodeNetworkInterface {
    /// Peer request receiver for messages
    pub(crate) network_reqs_rx: diem_channel::Receiver<(PeerId, ProtocolId), PeerManagerRequest>,
    /// Peer notification sender for sending outgoing messages to other peers
    pub(crate) network_notifs_tx:
        diem_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>,
    /// Sender for connecting / disconnecting peers
    pub(crate) network_conn_event_notifs_tx: conn_notifs_channel::Sender,
}

impl NodeNetworkInterface {
    fn get_next_network_req(&mut self, runtime: Arc<Runtime>) -> PeerManagerRequest {
        runtime.block_on(self.network_reqs_rx.next()).unwrap()
    }

    fn send_network_req(&mut self, protocol: ProtocolId, message: PeerManagerNotification) {
        let remote_peer_id = match &message {
            PeerManagerNotification::RecvRpc(peer_id, _) => *peer_id,
            PeerManagerNotification::RecvMessage(peer_id, _) => *peer_id,
        };

        self.network_notifs_tx
            .push((remote_peer_id, protocol), message)
            .unwrap()
    }

    /// Send a notification specifying, where a remote peer has it's state changed
    fn send_connection_notif(&mut self, notif: ConnectionNotification) {
        let peer_id = match &notif {
            ConnectionNotification::NewPeer(metadata, _) => metadata.remote_peer_id,
            ConnectionNotification::LostPeer(metadata, _, _) => metadata.remote_peer_id,
        };

        self.network_conn_event_notifs_tx
            .push(peer_id, notif)
            .unwrap()
    }
}

// Below here are static functions to help build a new `Node`

/// Sets up the network handles for a `Node`
fn setup_node_network_interfaces(
    node: &NodeInfo,
) -> (
    HashMap<NetworkId, NodeNetworkInterface>,
    Vec<MempoolNetworkHandle>,
) {
    let (network_interface, network_handle) = setup_node_network_interface(PeerNetworkId(
        NodeNetworkId::new(node.primary_network(), 0),
        node.primary_peer_id(),
    ));

    let mut network_handles = vec![network_handle];
    let mut network_interfaces = HashMap::new();
    network_interfaces.insert(node.primary_network(), network_interface);

    // Add Secondary network if the node has one
    if let Some(secondary_network_id) = node.secondary_network() {
        let (network_interface, network_handle) = setup_node_network_interface(PeerNetworkId(
            NodeNetworkId::new(secondary_network_id.clone(), 1),
            node.secondary_peer_id().unwrap(),
        ));
        network_handles.push(network_handle);
        network_interfaces.insert(secondary_network_id, network_interface);
    }

    (network_interfaces, network_handles)
}

/// Builds a single network interface with associated queues, and attaches it to the top level network
fn setup_node_network_interface(
    peer_network_id: PeerNetworkId,
) -> (NodeNetworkInterface, MempoolNetworkHandle) {
    static MAX_QUEUE_SIZE: usize = 8;
    let (network_reqs_tx, network_reqs_rx) =
        diem_channel::new(QueueStyle::FIFO, MAX_QUEUE_SIZE, None);
    let (connection_reqs_tx, _) = diem_channel::new(QueueStyle::FIFO, MAX_QUEUE_SIZE, None);
    let (network_notifs_tx, network_notifs_rx) =
        diem_channel::new(QueueStyle::FIFO, MAX_QUEUE_SIZE, None);
    let (network_conn_event_notifs_tx, conn_status_rx) = conn_notifs_channel::new();
    let network_sender = MempoolNetworkSender::new(
        PeerManagerRequestSender::new(network_reqs_tx),
        ConnectionRequestSender::new(connection_reqs_tx),
    );
    let network_events = MempoolNetworkEvents::new(network_notifs_rx, conn_status_rx);

    (
        NodeNetworkInterface {
            network_reqs_rx,
            network_notifs_tx,
            network_conn_event_notifs_tx,
        },
        (peer_network_id.network_id(), network_sender, network_events),
    )
}

/// Starts up the mempool resources for a single node
fn start_node_mempool(
    config: NodeConfig,
    network_handles: Vec<MempoolNetworkHandle>,
) -> (
    Arc<Mutex<CoreMempool>>,
    Runtime,
    UnboundedReceiver<SharedMempoolNotification>,
) {
    let mempool = Arc::new(Mutex::new(CoreMempool::new(&config)));
    let (sender, subscriber) = unbounded();
    let (_ac_endpoint_sender, ac_endpoint_receiver) = mpsc::channel(1_024);
    let (_consensus_sender, consensus_events) = mpsc::channel(1_024);
    let (_mempool_notifier, mempool_listener) = MempoolNotifier::new();
    let (_reconfig_events, reconfig_events_receiver) = diem_channel::new(QueueStyle::LIFO, 1, None);

    let runtime = Builder::new_multi_thread()
        .thread_name("shared-mem")
        .enable_all()
        .build()
        .expect("[shared mempool] failed to create runtime");
    start_shared_mempool(
        runtime.handle(),
        &config,
        Arc::clone(&mempool),
        network_handles,
        ac_endpoint_receiver,
        consensus_events,
        mempool_listener,
        reconfig_events_receiver,
        Arc::new(MockDbReader),
        Arc::new(RwLock::new(MockVMValidator)),
        vec![sender],
    );

    (mempool, runtime, subscriber)
}

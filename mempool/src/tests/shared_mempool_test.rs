// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    core_mempool::{CoreMempool, TimelineState},
    mocks::MockSharedMempool,
    network::{MempoolNetworkEvents, MempoolNetworkSender, MempoolSyncMsg},
    shared_mempool::{start_shared_mempool, types::SharedMempoolNotification},
    tests::common::{batch_add_signed_txn, TestTransaction},
    CommitNotification, CommittedTransaction, ConsensusRequest,
};
use channel::{self, diem_channel, message_queues::QueueStyle};
use diem_config::{
    config::{NetworkConfig, NodeConfig, RoleType, UpstreamConfig},
    network_id::{NetworkContext, NetworkId, NodeNetworkId},
};
use diem_infallible::{Mutex, RwLock};
use diem_network_address::NetworkAddress;
use diem_types::{
    transaction::{GovernanceRole, SignedTransaction},
    PeerId,
};
use futures::{
    channel::{
        mpsc::{self, unbounded, UnboundedReceiver},
        oneshot,
    },
    executor::block_on,
    future::FutureExt,
    sink::SinkExt,
    StreamExt,
};
use netcore::transport::ConnectionOrigin;
use network::{
    peer_manager::{
        conn_notifs_channel, ConnectionNotification, ConnectionRequestSender,
        PeerManagerNotification, PeerManagerRequest, PeerManagerRequestSender,
    },
    protocols::network::{NewNetworkEvents, NewNetworkSender},
    DisconnectReason, ProtocolId,
};
use std::{
    collections::{HashMap, HashSet},
    num::NonZeroUsize,
    sync::Arc,
};
use storage_interface::mock::MockDbReader;
use tokio::runtime::{Builder, Runtime};
use vm_validator::mocks::mock_vm_validator::MockVMValidator;

#[derive(Default)]
struct SharedMempoolNetwork {
    mempools: HashMap<PeerId, Arc<Mutex<CoreMempool>>>,
    network_reqs_rxs:
        HashMap<PeerId, diem_channel::Receiver<(PeerId, ProtocolId), PeerManagerRequest>>,
    network_notifs_txs:
        HashMap<PeerId, diem_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>>,
    network_conn_event_notifs_txs: HashMap<PeerId, conn_notifs_channel::Sender>,
    runtimes: HashMap<PeerId, Runtime>,
    subscribers: HashMap<PeerId, UnboundedReceiver<SharedMempoolNotification>>,
    peer_ids: HashMap<PeerId, PeerId>,
}

// start a shared mempool for a node `peer_id` with config `config`
// and add it to `smp` network
fn init_single_shared_mempool(
    smp: &mut SharedMempoolNetwork,
    peer_id: PeerId,
    network_id: NetworkId,
    config: NodeConfig,
) {
    let mempool = Arc::new(Mutex::new(CoreMempool::new(&config)));
    let (network_reqs_tx, network_reqs_rx) =
        diem_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
    let (connection_reqs_tx, _) =
        diem_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
    let (network_notifs_tx, network_notifs_rx) =
        diem_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
    let (conn_status_tx, conn_status_rx) = conn_notifs_channel::new();
    let network_sender = MempoolNetworkSender::new(
        PeerManagerRequestSender::new(network_reqs_tx),
        ConnectionRequestSender::new(connection_reqs_tx),
    );
    let network_events = MempoolNetworkEvents::new(network_notifs_rx, conn_status_rx);
    let (sender, subscriber) = unbounded();
    let (_ac_endpoint_sender, ac_endpoint_receiver) = mpsc::channel(1_024);
    let network_handles = vec![(
        NodeNetworkId::new(network_id, 0),
        network_sender,
        network_events,
    )];
    let (_consensus_sender, consensus_events) = mpsc::channel(1_024);
    let (_state_sync_sender, state_sync_events) = mpsc::channel(1_024);
    let (_reconfig_events, reconfig_events_receiver) =
        diem_channel::new(QueueStyle::LIFO, NonZeroUsize::new(1).unwrap(), None);

    let runtime = Builder::new()
        .thread_name("shared-mem")
        .threaded_scheduler()
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
        state_sync_events,
        reconfig_events_receiver,
        Arc::new(MockDbReader),
        Arc::new(RwLock::new(MockVMValidator)),
        vec![sender],
    );

    smp.mempools.insert(peer_id, mempool);
    smp.network_reqs_rxs.insert(peer_id, network_reqs_rx);
    smp.network_notifs_txs.insert(peer_id, network_notifs_tx);
    smp.network_conn_event_notifs_txs
        .insert(peer_id, conn_status_tx);
    smp.subscribers.insert(peer_id, subscriber);
    smp.runtimes.insert(peer_id, runtime);
}

// first PeerId in `network_ids` will be key in SharedMempoolNetwork
fn init_smp_multiple_networks(
    smp: &mut SharedMempoolNetwork,
    network_ids: Vec<(NetworkId, PeerId)>,
    config: NodeConfig,
) {
    let mempool = Arc::new(Mutex::new(CoreMempool::new(&config)));

    let mut network_handles = vec![];
    for (idx, (network_id, peer_id)) in network_ids.iter().enumerate() {
        let (network_reqs_tx, network_reqs_rx) =
            diem_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
        let (connection_reqs_tx, _) =
            diem_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
        let (network_notifs_tx, network_notifs_rx) =
            diem_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
        let (conn_status_tx, conn_status_rx) = conn_notifs_channel::new();
        let network_sender = MempoolNetworkSender::new(
            PeerManagerRequestSender::new(network_reqs_tx),
            ConnectionRequestSender::new(connection_reqs_tx),
        );
        let network_events = MempoolNetworkEvents::new(network_notifs_rx, conn_status_rx);
        network_handles.push((
            NodeNetworkId::new(network_id.clone(), idx),
            network_sender,
            network_events,
        ));

        smp.network_reqs_rxs.insert(*peer_id, network_reqs_rx);
        smp.network_notifs_txs.insert(*peer_id, network_notifs_tx);
        smp.network_conn_event_notifs_txs
            .insert(*peer_id, conn_status_tx);
    }

    let (sender, subscriber) = unbounded();
    let (_ac_endpoint_sender, ac_endpoint_receiver) = mpsc::channel(1_024);
    let (_consensus_sender, consensus_events) = mpsc::channel(1_024);
    let (_state_sync_sender, state_sync_events) = mpsc::channel(1_024);
    let (_reconfig_events, reconfig_events_receiver) =
        diem_channel::new(QueueStyle::LIFO, NonZeroUsize::new(1).unwrap(), None);

    let runtime = Builder::new()
        .thread_name("shared-mem")
        .threaded_scheduler()
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
        state_sync_events,
        reconfig_events_receiver,
        Arc::new(MockDbReader),
        Arc::new(RwLock::new(MockVMValidator)),
        vec![sender],
    );

    let main_peer_id = network_ids[0].clone().1;
    smp.subscribers.insert(main_peer_id, subscriber);
    smp.mempools.insert(main_peer_id, mempool);
    smp.runtimes.insert(main_peer_id, runtime);
    for (_network_id, alter_peer_id) in network_ids.into_iter().skip(1) {
        smp.peer_ids.insert(alter_peer_id, main_peer_id);
    }
}

impl SharedMempoolNetwork {
    fn bootstrap_validator_network(
        validator_nodes_count: u32,
        broadcast_batch_size: usize,
        mempool_size: Option<usize>,
        max_broadcasts_per_peer: Option<usize>,
        max_ack_timeout: bool,
    ) -> (Self, Vec<PeerId>) {
        let mut smp = Self::default();
        let mut peers = vec![];

        for _ in 0..validator_nodes_count {
            let mut config = NodeConfig::random();
            config.validator_network = Some(NetworkConfig::network_with_id(NetworkId::Validator));
            let peer_id = config.validator_network.as_ref().unwrap().peer_id();
            config.mempool.shared_mempool_batch_size = broadcast_batch_size;
            // set the ack timeout duration to 0 to avoid sleeping to test rebroadcast scenario (broadcast must timeout for this)
            config.mempool.shared_mempool_ack_timeout_ms =
                if max_ack_timeout { u64::MAX } else { 0 };
            config.mempool.capacity = mempool_size.unwrap_or(config.mempool.capacity);
            config.mempool.max_broadcasts_per_peer =
                max_broadcasts_per_peer.unwrap_or(config.mempool.max_broadcasts_per_peer);

            init_single_shared_mempool(&mut smp, peer_id, NetworkId::Validator, config);

            peers.push(peer_id);
        }
        (smp, peers)
    }

    /// creates a shared mempool network of one full node and one validator
    /// returns the newly created SharedMempoolNetwork, and the ID of validator and full node, in that order
    fn bootstrap_vfn_network(
        broadcast_batch_size: usize,
        validator_mempool_size: Option<usize>,
        validator_account_txn_limit: Option<usize>,
    ) -> (Self, PeerId, PeerId) {
        let mut smp = Self::default();

        // declare peers in network
        let validator = PeerId::random();
        let full_node = PeerId::random();

        // validator config
        let mut config = NodeConfig::random();
        config.mempool.shared_mempool_batch_size = broadcast_batch_size;
        config.mempool.shared_mempool_backoff_interval_ms = 50;
        // set the ack timeout duration to 0 to avoid sleeping to test rebroadcast scenario (broadcast must timeout for this)
        config.mempool.shared_mempool_ack_timeout_ms = 0;
        if let Some(capacity) = validator_mempool_size {
            config.mempool.capacity = capacity
        }
        if let Some(capacity_per_user) = validator_account_txn_limit {
            config.mempool.capacity_per_user = capacity_per_user;
        }
        init_single_shared_mempool(&mut smp, validator, NetworkId::vfn_network(), config);

        // full node
        let mut fn_config = NodeConfig::random();
        fn_config.base.role = RoleType::FullNode;
        fn_config.mempool.shared_mempool_batch_size = broadcast_batch_size;
        fn_config.mempool.shared_mempool_backoff_interval_ms = 50;
        // set the ack timeout duration to 0 to avoid sleeping to test rebroadcast scenario (broadcast must timeout for this)
        fn_config.mempool.shared_mempool_ack_timeout_ms = 0;

        let mut upstream_config = UpstreamConfig::default();
        upstream_config.networks.push(NetworkId::vfn_network());
        fn_config.upstream = upstream_config;
        init_single_shared_mempool(&mut smp, full_node, NetworkId::vfn_network(), fn_config);

        (smp, validator, full_node)
    }

    fn add_txns(&mut self, peer_id: &PeerId, txns: Vec<TestTransaction>) {
        let mut mempool = self.mempools.get(peer_id).unwrap().lock();
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

    fn commit_txns(&mut self, peer_id: &PeerId, txns: Vec<TestTransaction>) {
        let mut mempool = self.mempools.get(peer_id).unwrap().lock();
        for txn in txns {
            mempool.remove_transaction(
                &TestTransaction::get_address(txn.address),
                txn.sequence_number,
                false,
            );
        }
    }

    fn send_new_peer_event(&mut self, receiver: &PeerId, new_peer: &PeerId, inbound: bool) {
        let origin = if inbound {
            ConnectionOrigin::Inbound
        } else {
            ConnectionOrigin::Outbound
        };
        let notif = ConnectionNotification::NewPeer(
            *new_peer,
            NetworkAddress::mock(),
            origin,
            NetworkContext::mock(),
        );
        self.send_connection_event(receiver, notif)
    }

    fn send_lost_peer_event(&mut self, receiver: &PeerId, lost_peer: &PeerId) {
        let notif = ConnectionNotification::LostPeer(
            *lost_peer,
            NetworkAddress::mock(),
            ConnectionOrigin::Inbound,
            DisconnectReason::ConnectionLost,
        );
        self.send_connection_event(receiver, notif)
    }

    fn send_connection_event(&mut self, peer: &PeerId, notif: ConnectionNotification) {
        let conn_notifs_tx = self.network_conn_event_notifs_txs.get_mut(peer).unwrap();
        conn_notifs_tx.push(*peer, notif).unwrap();
        self.wait_for_event(peer, SharedMempoolNotification::PeerStateChange);
    }

    fn wait_for_event(&mut self, peer_id: &PeerId, event: SharedMempoolNotification) {
        let main_peer_id = self.peer_ids.get(peer_id).unwrap_or(peer_id);
        let subscriber = self.subscribers.get_mut(main_peer_id).unwrap();
        assert_eq!(block_on(subscriber.next()).unwrap(), event);
    }

    fn check_no_events(&mut self, peer_id: &PeerId) {
        let main_peer_id = self.peer_ids.get(peer_id).unwrap_or(peer_id);
        let subscriber = self.subscribers.get_mut(main_peer_id).unwrap();

        assert!(subscriber.select_next_some().now_or_never().is_none());
    }

    // checks that a node has no pending messages to send
    fn assert_no_message_sent(&mut self, peer: &PeerId) {
        self.check_no_events(peer);

        // await next message from node
        let network_reqs_rx = self.network_reqs_rxs.get_mut(peer).unwrap();
        assert!(network_reqs_rx.select_next_some().now_or_never().is_none());
    }

    /// delivers next broadcast message from `peer`
    fn deliver_message(
        &mut self,
        peer: &PeerId,
        num_messages: usize,
        check_txns_in_mempool: bool, // check whether all txns in this broadcast are accepted into recipient's mempool
        execute_send: bool, // if true, actually delivers msg to remote peer; else, drop the message (useful for testing unreliable msg delivery)
        drop_ack: bool,     // if true, drop ack from remote peer to this peer
    ) -> (Vec<SignedTransaction>, PeerId) {
        // await broadcast notification
        for _ in 0..num_messages {
            self.wait_for_event(peer, SharedMempoolNotification::Broadcast);
        }

        // await next message from node
        let network_reqs_rx = self.network_reqs_rxs.get_mut(peer).unwrap();
        let network_req = block_on(network_reqs_rx.next()).unwrap();

        if let PeerManagerRequest::SendMessage(peer_id, msg) = network_req {
            let sync_msg = bcs::from_bytes(&msg.mdata).unwrap();
            if let MempoolSyncMsg::BroadcastTransactionsRequest { transactions, .. } = sync_msg {
                if !execute_send {
                    return (transactions, peer_id);
                }

                // send it to peer
                let receiver_network_notif_tx = self.network_notifs_txs.get_mut(&peer_id).unwrap();
                receiver_network_notif_tx
                    .push(
                        (*peer, ProtocolId::MempoolDirectSend),
                        PeerManagerNotification::RecvMessage(*peer, msg),
                    )
                    .unwrap();

                // await message delivery
                self.wait_for_event(&peer_id, SharedMempoolNotification::NewTransactions);

                // verify transaction was inserted into Mempool
                if check_txns_in_mempool {
                    let mempool = self.mempools.get(&peer_id).unwrap();
                    let block = mempool.lock().get_block(100, HashSet::new());
                    for txn in transactions.iter() {
                        assert!(block.contains(txn));
                    }
                }

                // deliver ACK for this request
                if !drop_ack {
                    self.deliver_response(&peer_id);
                }
                (transactions, peer_id)
            } else {
                panic!("did not receive expected BroadcastTransactionsRequest");
            }
        } else {
            panic!("peer {:?} didn't broadcast transaction", peer)
        }
    }

    /// delivers broadcast ACK from `peer`
    fn deliver_response(&mut self, peer: &PeerId) {
        self.wait_for_event(&peer, SharedMempoolNotification::ACK);
        let network_reqs_rx = self.network_reqs_rxs.get_mut(peer).unwrap();
        let network_req = block_on(network_reqs_rx.next()).unwrap();

        if let PeerManagerRequest::SendMessage(peer_id, msg) = network_req {
            let sync_msg = bcs::from_bytes(&msg.mdata).unwrap();
            if let MempoolSyncMsg::BroadcastTransactionsResponse { .. } = sync_msg {
                // send it to peer
                let receiver_network_notif_tx = self.network_notifs_txs.get_mut(&peer_id).unwrap();
                receiver_network_notif_tx
                    .push(
                        (*peer, ProtocolId::MempoolDirectSend),
                        PeerManagerNotification::RecvMessage(*peer, msg),
                    )
                    .unwrap();
            } else {
                panic!("did not receive expected broadcast ACK");
            }
        } else {
            panic!("peer {:?} did not ACK broadcast", peer);
        }
    }

    fn exist_in_metrics_cache(&self, peer_id: &PeerId, txn: &TestTransaction) -> bool {
        let mempool = self.mempools.get(peer_id).unwrap().lock();
        mempool
            .metrics_cache
            .get(&(
                TestTransaction::get_address(txn.address),
                txn.sequence_number,
            ))
            .is_some()
    }
}

#[test]
fn test_basic_flow() {
    let (mut smp, peers) =
        SharedMempoolNetwork::bootstrap_validator_network(2, 1, None, None, false);
    let (peer_a, peer_b) = (peers.get(0).unwrap(), peers.get(1).unwrap());
    smp.add_txns(
        &peer_a,
        vec![
            TestTransaction::new(1, 0, 1),
            TestTransaction::new(1, 1, 1),
            TestTransaction::new(1, 2, 1),
        ],
    );

    // A discovers new peer B
    smp.send_new_peer_event(peer_a, peer_b, true);

    for seq in 0..3 {
        // A attempts to send message
        let transactions = smp.deliver_message(&peer_a, 1, true, true, false).0;
        assert_eq!(transactions.get(0).unwrap().sequence_number(), seq);
    }
}

#[test]
fn test_metric_cache_ignore_shared_txns() {
    let (mut smp, peers) =
        SharedMempoolNetwork::bootstrap_validator_network(2, 1, None, None, false);
    let (peer_a, peer_b) = (peers.get(0).unwrap(), peers.get(1).unwrap());

    let txns = vec![
        TestTransaction::new(1, 0, 1),
        TestTransaction::new(1, 1, 1),
        TestTransaction::new(1, 2, 1),
    ];
    smp.add_txns(
        &peer_a,
        vec![txns[0].clone(), txns[1].clone(), txns[2].clone()],
    );
    // Check if txns's creation timestamp exist in peer_a's metrics_cache.
    assert_eq!(smp.exist_in_metrics_cache(&peer_a, &txns[0]), true);
    assert_eq!(smp.exist_in_metrics_cache(&peer_a, &txns[1]), true);
    assert_eq!(smp.exist_in_metrics_cache(&peer_a, &txns[2]), true);

    // Let peer_a discover new peer_b.
    smp.send_new_peer_event(peer_a, peer_b, true);
    smp.send_new_peer_event(peer_b, peer_a, false);

    for txn in txns.iter().take(3) {
        // Let peer_a share txns with peer_b
        let (_transaction, rx_peer) = smp.deliver_message(&peer_a, 1, true, true, false);
        // Check if txns's creation timestamp exist in peer_b's metrics_cache.
        assert_eq!(smp.exist_in_metrics_cache(&rx_peer, txn), false);
    }
}

#[test]
fn test_interruption_in_sync() {
    let (mut smp, peers) =
        SharedMempoolNetwork::bootstrap_validator_network(3, 1, None, None, false);
    let (peer_a, peer_b, peer_c) = (
        peers.get(0).unwrap(),
        peers.get(1).unwrap(),
        peers.get(2).unwrap(),
    );
    smp.add_txns(&peer_a, vec![TestTransaction::new(1, 0, 1)]);

    // A discovers first peer
    smp.send_new_peer_event(peer_a, peer_b, true);

    // make sure first txn delivered to first peer
    assert_eq!(
        *peer_b,
        smp.deliver_message(&peer_a, 1, true, true, false).1
    );

    // A discovers second peer
    smp.send_new_peer_event(peer_a, peer_c, true);

    // make sure first txn delivered to second peer
    assert_eq!(
        *peer_c,
        smp.deliver_message(&peer_a, 1, true, true, false).1
    );

    // A loses connection to B
    smp.send_lost_peer_event(peer_a, peer_b);

    // only C receives following transactions
    smp.add_txns(&peer_a, vec![TestTransaction::new(1, 1, 1)]);
    let (txn, peer_id) = smp.deliver_message(&peer_a, 1, true, true, false);
    assert_eq!(peer_id, *peer_c);
    assert_eq!(txn.get(0).unwrap().sequence_number(), 1);

    smp.add_txns(&peer_a, vec![TestTransaction::new(1, 2, 1)]);
    let (txn, peer_id) = smp.deliver_message(&peer_a, 1, true, true, false);
    assert_eq!(peer_id, *peer_c);
    assert_eq!(txn.get(0).unwrap().sequence_number(), 2);

    // A reconnects to B
    smp.send_new_peer_event(&peer_a, peer_b, true);

    // B should receive transaction 2
    let (txn, peer_id) = smp.deliver_message(&peer_a, 1, true, true, false);
    assert_eq!(peer_id, *peer_b);
    assert_eq!(txn.get(0).unwrap().sequence_number(), 1);
}

#[test]
fn test_ready_transactions() {
    let (mut smp, peers) =
        SharedMempoolNetwork::bootstrap_validator_network(2, 1, None, None, false);
    let (peer_a, peer_b) = (peers.get(0).unwrap(), peers.get(1).unwrap());
    smp.add_txns(
        &peer_a,
        vec![TestTransaction::new(1, 0, 1), TestTransaction::new(1, 2, 1)],
    );
    // first message delivery
    smp.send_new_peer_event(peer_a, peer_b, true);
    smp.deliver_message(&peer_a, 1, true, true, false);

    // add txn1 to Mempool
    smp.add_txns(&peer_a, vec![TestTransaction::new(1, 1, 1)]);
    // txn1 unlocked txn2. Now all transactions can go through in correct order
    let txn = &smp.deliver_message(&peer_a, 1, true, true, false).0;
    assert_eq!(txn.get(0).unwrap().sequence_number(), 1);
    let txn = &smp.deliver_message(&peer_a, 1, true, true, false).0;
    assert_eq!(txn.get(0).unwrap().sequence_number(), 2);
}

#[test]
fn test_broadcast_self_transactions() {
    let (mut smp, peers) =
        SharedMempoolNetwork::bootstrap_validator_network(2, 1, None, None, false);
    let (peer_a, peer_b) = (peers.get(0).unwrap(), peers.get(1).unwrap());
    smp.add_txns(&peer_a, vec![TestTransaction::new(0, 0, 1)]);

    // A and B discover each other
    smp.send_new_peer_event(peer_a, peer_b, true);
    smp.send_new_peer_event(peer_b, peer_a, true);

    // A sends txn to B
    smp.deliver_message(&peer_a, 1, true, true, false);

    // add new txn to B
    smp.add_txns(&peer_b, vec![TestTransaction::new(1, 0, 1)]);

    // verify that A will receive only second transaction from B
    let (txn, _) = smp.deliver_message(&peer_b, 1, true, true, false);
    assert_eq!(
        txn.get(0).unwrap().sender(),
        TestTransaction::get_address(1)
    );
}

#[test]
fn test_broadcast_dependencies() {
    let (mut smp, peers) =
        SharedMempoolNetwork::bootstrap_validator_network(2, 1, None, None, false);
    let (peer_a, peer_b) = (peers.get(0).unwrap(), peers.get(1).unwrap());
    // Peer A has transactions with sequence numbers 0 and 2
    smp.add_txns(
        &peer_a,
        vec![TestTransaction::new(0, 0, 1), TestTransaction::new(0, 2, 1)],
    );
    // Peer B has txn1
    smp.add_txns(&peer_b, vec![TestTransaction::new(0, 1, 1)]);

    // A and B discover each other
    smp.send_new_peer_event(peer_a, peer_b, true);
    smp.send_new_peer_event(peer_b, peer_a, false);

    // B receives 0
    smp.deliver_message(&peer_a, 1, true, true, false);
    // now B can broadcast 1
    let txn = smp.deliver_message(&peer_b, 1, true, true, false).0;
    assert_eq!(txn.get(0).unwrap().sequence_number(), 1);
    // now A can broadcast 2
    let txn = smp.deliver_message(&peer_a, 1, true, true, false).0;
    assert_eq!(txn.get(0).unwrap().sequence_number(), 2);
}

#[test]
fn test_broadcast_updated_transaction() {
    let (mut smp, peers) =
        SharedMempoolNetwork::bootstrap_validator_network(2, 1, None, None, false);
    let (peer_a, peer_b) = (peers.get(0).unwrap(), peers.get(1).unwrap());

    // Peer A has a transaction with sequence number 0 and gas price 1
    smp.add_txns(&peer_a, vec![TestTransaction::new(0, 0, 1)]);

    // A and B discover each other
    smp.send_new_peer_event(peer_a, peer_b, false);
    smp.send_new_peer_event(peer_b, peer_a, true);

    // B receives 0
    let txn = smp.deliver_message(&peer_a, 1, true, true, false).0;
    assert_eq!(txn.get(0).unwrap().sequence_number(), 0);
    assert_eq!(txn.get(0).unwrap().gas_unit_price(), 1);

    // Update the gas price of the transaction with sequence 0 after B has already received 0
    smp.add_txns(&peer_a, vec![TestTransaction::new(0, 0, 5)]);

    // trigger send from A to B and check B has updated gas price for sequence 0
    let txn = smp.deliver_message(&peer_a, 1, true, true, false).0;
    assert_eq!(txn.get(0).unwrap().sequence_number(), 0);
    assert_eq!(txn.get(0).unwrap().gas_unit_price(), 5);
}

#[test]
fn test_consensus_events_rejected_txns() {
    let smp = MockSharedMempool::new(None);

    // add txns 1, 2, 3, 4
    // txn 1: committed successfully
    // txn 2: not committed but older than gc block timestamp
    // txn 3: not committed and newer than block timestamp
    let committed_txn =
        TestTransaction::new(0, 0, 1).make_signed_transaction_with_expiration_time(0);
    let kept_txn = TestTransaction::new(1, 0, 1).make_signed_transaction(); // not committed or cleaned out by block timestamp gc
    let txns = vec![
        committed_txn.clone(),
        TestTransaction::new(0, 1, 1).make_signed_transaction_with_expiration_time(0),
        kept_txn.clone(),
    ];
    // add txns to mempool
    {
        let mut pool = smp.mempool.lock();
        assert!(batch_add_signed_txn(&mut pool, txns).is_ok());
    }

    // send commit notif
    let committed_txns = vec![CommittedTransaction {
        sender: committed_txn.sender(),
        sequence_number: committed_txn.sequence_number(),
    }];
    let (callback, callback_rcv) = oneshot::channel();
    let req = ConsensusRequest::RejectNotification(committed_txns, callback);
    let mut consensus_sender = smp.consensus_sender.clone();
    block_on(async {
        assert!(consensus_sender.send(req).await.is_ok());
        assert!(callback_rcv.await.is_ok());
    });

    // check mempool
    let mut pool = smp.mempool.lock();
    let (timeline, _) = pool.read_timeline(0, 10);
    assert_eq!(timeline.len(), 1);
    assert_eq!(timeline.get(0).unwrap(), &kept_txn);
}

#[test]
fn test_state_sync_events_committed_txns() {
    let (mut state_sync_sender, state_sync_events) = mpsc::channel(1_024);
    let smp = MockSharedMempool::new(Some(state_sync_events));

    // add txns 1, 2, 3, 4
    // txn 1: committed successfully
    // txn 2: not committed but older than gc block timestamp
    // txn 3: not committed and newer than block timestamp
    let committed_txn =
        TestTransaction::new(0, 0, 1).make_signed_transaction_with_expiration_time(0);
    let kept_txn = TestTransaction::new(1, 0, 1).make_signed_transaction(); // not committed or cleaned out by block timestamp gc
    let txns = vec![
        committed_txn.clone(),
        TestTransaction::new(0, 1, 1).make_signed_transaction_with_expiration_time(0),
        kept_txn.clone(),
    ];
    // add txns to mempool
    {
        let mut pool = smp.mempool.lock();
        assert!(batch_add_signed_txn(&mut pool, txns).is_ok());
    }

    // send commit notif
    let committed_txns = vec![CommittedTransaction {
        sender: committed_txn.sender(),
        sequence_number: committed_txn.sequence_number(),
    }];

    let (callback, callback_rcv) = oneshot::channel();
    let req = CommitNotification {
        transactions: committed_txns,
        block_timestamp_usecs: 1,
        callback,
    };
    block_on(async {
        assert!(state_sync_sender.send(req).await.is_ok());
        assert!(callback_rcv.await.is_ok());
    });

    // check mempool
    let mut pool = smp.mempool.lock();
    let (timeline, _) = pool.read_timeline(0, 10);
    assert_eq!(timeline.len(), 1);
    assert_eq!(timeline.get(0).unwrap(), &kept_txn);
}

// tests VFN properly identifying upstream peers in a network with both upstream and downstream peers
#[test]
fn test_vfn_multi_network() {
    let vfn_0 = PeerId::random();
    let vfn_0_public_network_id = PeerId::random();
    let vfn_1 = PeerId::random();
    let vfn_1_public_network_id = PeerId::random();
    let pfn = PeerId::random();

    let mut vfn_0_config = NodeConfig::default();
    vfn_0_config.base.role = RoleType::FullNode;
    vfn_0_config.mempool.shared_mempool_batch_size = 1;
    vfn_0_config.upstream.networks = vec![NetworkId::vfn_network(), NetworkId::Public];
    let mut vfn_1_config = NodeConfig::default();
    vfn_1_config.base.role = RoleType::FullNode;
    let mut pfn_config = NodeConfig::default();
    pfn_config.base.role = RoleType::FullNode;
    pfn_config.upstream.networks = vec![NetworkId::Public];

    let mut smp = SharedMempoolNetwork::default();
    init_smp_multiple_networks(
        &mut smp,
        vec![
            (NetworkId::vfn_network(), vfn_0),
            (NetworkId::Public, vfn_0_public_network_id),
        ],
        vfn_0_config,
    );
    init_smp_multiple_networks(
        &mut smp,
        vec![
            (NetworkId::vfn_network(), vfn_1),
            (NetworkId::Public, vfn_1_public_network_id),
        ],
        vfn_1_config,
    );
    init_single_shared_mempool(&mut smp, pfn, NetworkId::Public, pfn_config);

    //vfn 0 discovers pfn as inbound
    smp.send_new_peer_event(&vfn_0_public_network_id, &pfn, true);
    // vfn 0 discovers vfn 1 as outbound
    smp.send_new_peer_event(&vfn_0_public_network_id, &vfn_1_public_network_id, false);

    // add txn to fn_0
    smp.add_txns(&vfn_0, vec![TestTransaction::new(1, 0, 1)]);

    // vfn should broadcast to failover upstream vfn in public network
    assert_eq!(
        vfn_1_public_network_id,
        smp.deliver_message(&vfn_0_public_network_id, 1, false, true, false)
            .1
    );

    // vfn should not broadcast to downstream peer in public network
    smp.assert_no_message_sent(&vfn_0_public_network_id);
    // sanity check, vfn doesn't broadcast to incorrect network
    smp.assert_no_message_sent(&vfn_0);
}

#[test]
fn test_fn_failover() {
    // test vfn failing over to fallback network

    // set up network with fn with 1 primary v upstream and 3 fallback upstream fn
    let v_0 = PeerId::random();
    let fn_0 = PeerId::random();
    let fn_0_fallback_network_id = PeerId::random();
    let fn_1 = PeerId::random();
    let fn_2 = PeerId::random();
    let fn_3 = PeerId::random();
    let fallback_peers = vec![fn_1, fn_2, fn_3];

    let v0_config = NodeConfig::default();
    let mut fn_0_config = NodeConfig::default();
    fn_0_config.base.role = RoleType::FullNode;
    fn_0_config.mempool.default_failovers = 0;
    fn_0_config.mempool.shared_mempool_batch_size = 1;
    fn_0_config.upstream.networks = vec![NetworkId::vfn_network(), NetworkId::Public];
    let mut fn_1_config = NodeConfig::default();
    fn_1_config.base.role = RoleType::FullNode;
    fn_1_config.mempool.default_failovers = 0;
    let mut fn_2_config = NodeConfig::default();
    fn_2_config.base.role = RoleType::FullNode;
    fn_2_config.mempool.default_failovers = 0;
    let mut fn_3_config = NodeConfig::default();
    fn_3_config.base.role = RoleType::FullNode;
    fn_3_config.mempool.default_failovers = 0;

    let mut smp = SharedMempoolNetwork::default();
    init_single_shared_mempool(&mut smp, v_0, NetworkId::Validator, v0_config);
    init_smp_multiple_networks(
        &mut smp,
        vec![
            (NetworkId::vfn_network(), fn_0),
            (NetworkId::Public, fn_0_fallback_network_id),
        ],
        fn_0_config,
    );
    init_single_shared_mempool(&mut smp, fn_1, NetworkId::Public, fn_1_config);
    init_single_shared_mempool(&mut smp, fn_2, NetworkId::Public, fn_2_config);
    init_single_shared_mempool(&mut smp, fn_3, NetworkId::Public, fn_3_config);

    // fn_0 discovers primary and fallback upstream peers
    smp.send_new_peer_event(&fn_0, &v_0, true);
    for fallback_peer in fallback_peers.iter() {
        smp.send_new_peer_event(&fn_0_fallback_network_id, fallback_peer, false);
    }

    // add txn to fn_0
    smp.add_txns(&fn_0, vec![TestTransaction::new(1, 0, 1)]);

    // make sure it delivers txn to primary peer
    let recipient_peer = smp.deliver_message(&fn_0, 1, true, true, false).1;
    assert_eq!(recipient_peer, v_0);
    // check that no messages have been sent to fallback upstream peers
    smp.assert_no_message_sent(&fn_0_fallback_network_id);

    // bring v down
    smp.send_lost_peer_event(&fn_0, &v_0);

    // add txn to fn_0
    smp.add_txns(&fn_0, vec![TestTransaction::new(1, 1, 1)]);

    // make sure it delivers txn to fallback peer
    let mut actual_fallback_recipients = vec![];
    for _ in 0..fallback_peers.len() {
        let (txn, fallback_recipient) =
            smp.deliver_message(&fn_0_fallback_network_id, 1, true, true, false);
        assert!(fallback_peers.contains(&fallback_recipient));
        assert_eq!(txn.get(0).unwrap().sequence_number(), 0);
        // check that no messages have been sent to primary upstream peer
        assert!(!actual_fallback_recipients.contains(&fallback_recipient));
        actual_fallback_recipients.push(fallback_recipient);
    }
    smp.assert_no_message_sent(&fn_0);
    assert_eq!(actual_fallback_recipients.len(), fallback_peers.len());

    // add some more txns to fn_0
    smp.add_txns(
        &fn_0,
        vec![TestTransaction::new(1, 2, 1), TestTransaction::new(1, 3, 1)],
    );
    let expected_seq_nums = vec![1, 2, 3];
    for seq_num in expected_seq_nums {
        let mut actual_fallback_recipients = vec![];
        for _ in 0..fallback_peers.len() {
            let (txn, fallback_recipient) =
                smp.deliver_message(&fn_0_fallback_network_id, 1, true, true, false);
            assert!(fallback_peers.contains(&fallback_recipient));
            assert!(!actual_fallback_recipients.contains(&fallback_recipient));
            actual_fallback_recipients.push(fallback_recipient);
            // check sequence number
            assert_eq!(txn.get(0).unwrap().sequence_number(), seq_num);
        }
    }
    // check that no messages have been sent to primary upstream peer
    smp.assert_no_message_sent(&fn_0);

    // bring down one fallback peer
    smp.send_lost_peer_event(&fn_0_fallback_network_id, &fn_1);

    // add txn
    smp.add_txns(&fn_0, vec![TestTransaction::new(1, 4, 1)]);
    // make sure it gets broadcast to another fallback peer
    for _ in 0..2 {
        let mut actual_fallback_recipients = vec![];
        let (txn, fallback_recipient) =
            smp.deliver_message(&fn_0_fallback_network_id, 1, true, true, false);
        assert_ne!(fn_1, fallback_recipient);
        assert!(fallback_peers.contains(&fallback_recipient));
        assert!(!actual_fallback_recipients.contains(&fallback_recipient));
        actual_fallback_recipients.push(fallback_recipient);

        // check txn seq num
        assert_eq!(txn.get(0).unwrap().sequence_number(), 4);
    }
    // check that no messages have been sent to primary upstream peer
    smp.assert_no_message_sent(&fn_0);

    // bring down all upstream peers
    for fallback_peer in fallback_peers.iter() {
        if fallback_peer != &fn_1 {
            smp.send_lost_peer_event(&fn_0_fallback_network_id, fallback_peer);
        }
    }
    // check that no messages get sent
    smp.add_txns(&fn_0, vec![TestTransaction::new(1, 5, 1)]);
    smp.assert_no_message_sent(&fn_0);
    smp.assert_no_message_sent(&fn_0_fallback_network_id);

    // bring back v up
    smp.send_new_peer_event(&fn_0, &v_0, true);

    // add txn
    smp.add_txns(&fn_0, vec![TestTransaction::new(1, 6, 1)]);
    // check that we don't broadcast to failovers, only v

    for expected_seq_num in 0..=3 {
        let (txn, recipient) = smp.deliver_message(&fn_0, 1, true, true, false);
        assert_eq!(recipient, v_0);
        assert_eq!(txn.get(0).unwrap().sequence_number(), expected_seq_num);
        // check that no messages have been sent to fallback peers
        smp.assert_no_message_sent(&fn_0_fallback_network_id);
    }

    // bring back all fallback peers back
    for fallback_peer in fallback_peers.iter() {
        smp.send_new_peer_event(&fn_0_fallback_network_id, fallback_peer, false);
    }

    for expected_seq_num in 4..=6 {
        let (txn, recipient) = smp.deliver_message(&fn_0, 1, true, true, false);
        assert_eq!(recipient, v_0);
        assert_eq!(txn.get(0).unwrap().sequence_number(), expected_seq_num);
        // check that no messages have been sent to fallback peers
        smp.assert_no_message_sent(&fn_0_fallback_network_id);
    }
}

#[test]
fn test_rebroadcast_mempool_is_full() {
    let (mut smp, val, full_node) = SharedMempoolNetwork::bootstrap_vfn_network(3, Some(5), None);
    let mut all_txns = vec![];
    for i in 0..8 {
        all_txns.push(TestTransaction::new(1, i, 1));
    }
    smp.add_txns(&full_node, all_txns.clone());

    // FN discovers new peer V
    smp.send_new_peer_event(&full_node, &val, true);

    let (txns, _recipient) = smp.deliver_message(&full_node, 1, true, true, false);
    let seq_nums = txns
        .iter()
        .map(|txn| txn.sequence_number())
        .collect::<Vec<_>>();
    assert_eq!(vec![0, 1, 2], seq_nums);

    smp.add_txns(&full_node, vec![TestTransaction::new(1, 5, 1)]);

    let expected_broadcast = vec![3, 4, 5];

    for _ in 0..2 {
        let (txns, _recipient) = smp.deliver_message(&full_node, 1, false, true, false);
        let seq_nums = txns
            .iter()
            .map(|txn| txn.sequence_number())
            .collect::<Vec<_>>();
        assert_eq!(expected_broadcast, seq_nums);
    }

    // Test getting out of rebroadcasting mode: checking we can move past rebroadcasting after receiving non-retry ACK
    // make space for retried txns
    smp.commit_txns(&val, all_txns[..1].to_vec());

    // send retry batch again, this time should be processed
    smp.deliver_message(&full_node, 1, true, true, false);

    // retry batch sent above should be processed successfully, and FN should move on to broadcasting later txns
    let (txns, _recipient) = smp.deliver_message(&full_node, 1, false, true, false);
    let seq_nums = txns
        .iter()
        .map(|txn| txn.sequence_number())
        .collect::<Vec<_>>();
    assert_eq!(vec![6, 7], seq_nums);
}

#[test]
fn test_rebroadcast_missing_ack() {
    // create 2 validators A and B
    let (mut smp, peers) =
        SharedMempoolNetwork::bootstrap_validator_network(2, 1, None, None, false);
    let (peer_a, peer_b) = (peers.get(0).unwrap(), peers.get(1).unwrap());
    let pool_txns = vec![
        TestTransaction::new(1, 0, 1),
        TestTransaction::new(1, 1, 1),
        TestTransaction::new(1, 2, 1),
    ];
    smp.add_txns(&peer_a, pool_txns.clone());

    // A and B discover each other
    smp.send_new_peer_event(peer_a, peer_b, true);
    smp.send_new_peer_event(peer_b, peer_a, false);

    // test that for txn broadcasts that don't receive an ACK, A rebroadcasts the unACK'ed batch of txns
    for _ in 0..3 {
        let (txns, _recipient) = smp.deliver_message(&peer_a, 1, true, false, false);
        let seq_nums = txns
            .iter()
            .map(|txn| txn.sequence_number())
            .collect::<Vec<_>>();
        assert_eq!(seq_nums, vec![0]);
    }

    // getting out of rebroadcasting mode scenario 1: B sends back ACK eventually
    let (txns, _recipient) = smp.deliver_message(&peer_a, 1, true, true, false);
    let seq_nums = txns
        .iter()
        .map(|txn| txn.sequence_number())
        .collect::<Vec<_>>();
    assert_eq!(seq_nums, vec![0]);

    for _ in 0..3 {
        let (txns, _recipient) = smp.deliver_message(&peer_a, 1, true, false, false);
        let seq_nums = txns
            .iter()
            .map(|txn| txn.sequence_number())
            .collect::<Vec<_>>();
        assert_eq!(seq_nums, vec![1]);
    }

    // getting out of rebroadcasting mode scenario 2: txns in unACK'ed batch gets committed
    smp.commit_txns(&peer_a, vec![pool_txns[1].clone()]);

    let (txns, _recipient) = smp.deliver_message(&peer_a, 1, true, false, false);
    let seq_nums = txns
        .iter()
        .map(|txn| txn.sequence_number())
        .collect::<Vec<_>>();
    assert_eq!(seq_nums, vec![2]);
}

#[test]
fn test_max_broadcast_limit() {
    // create 2 validators A and B
    let (mut smp, peers) =
        SharedMempoolNetwork::bootstrap_validator_network(2, 1, None, Some(3), true);
    let (peer_a, peer_b) = (peers.get(0).unwrap(), peers.get(1).unwrap());
    let mut pool_txns = vec![];
    for seq_num in 0..6 {
        pool_txns.push(TestTransaction::new(1, seq_num, 1));
    }
    smp.add_txns(&peer_a, pool_txns);

    // A and B discover each other
    smp.send_new_peer_event(peer_a, peer_b, true);
    smp.send_new_peer_event(peer_b, peer_a, false);

    // test that for mempool broadcasts txns up till max broadcast, even if they are not ACK'ed
    let (txns, _recipient) = smp.deliver_message(&peer_a, 1, true, true, true);
    let seq_nums = txns
        .iter()
        .map(|txn| txn.sequence_number())
        .collect::<Vec<_>>();
    assert_eq!(seq_nums, vec![0]);
    for expected_seq_num in 1..3 {
        let (txns, _recipient) = smp.deliver_message(&peer_a, 1, true, false, false);
        let seq_nums = txns
            .iter()
            .map(|txn| txn.sequence_number())
            .collect::<Vec<_>>();
        assert_eq!(seq_nums, vec![expected_seq_num]);
    }

    // check that mempool doesn't broadcast more than max_broadcasts_per_peer, even
    // if there are more txns in mempool
    for _ in 0..10 {
        smp.assert_no_message_sent(&peer_a);
    }

    // deliver ACK from B to A
    // this should unblock A to send more broadcasts
    smp.deliver_response(&peer_b);
    let (txns, _recipient) = smp.deliver_message(&peer_a, 1, false, true, true);
    let seq_nums = txns
        .iter()
        .map(|txn| txn.sequence_number())
        .collect::<Vec<_>>();
    assert_eq!(seq_nums, vec![3]);

    // check that mempool doesn't broadcast more than max_broadcasts_per_peer, even
    // if there are more txns in mempool
    for _ in 0..10 {
        smp.assert_no_message_sent(&peer_a);
    }
}

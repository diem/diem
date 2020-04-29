// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    core_mempool::{CoreMempool, TimelineState},
    mocks::MockSharedMempool,
    network::{MempoolNetworkEvents, MempoolNetworkSender, MempoolSyncMsg},
    shared_mempool::{
        start_shared_mempool,
        types::{SharedMempoolNotification, SyncEvent},
    },
    tests::common::{batch_add_signed_txn, TestTransaction},
    CommitNotification, CommittedTransaction, ConsensusRequest,
};
use channel::{self, libra_channel, message_queues::QueueStyle};
use futures::{
    channel::{
        mpsc::{self, unbounded, UnboundedReceiver, UnboundedSender},
        oneshot,
    },
    executor::block_on,
    future::FutureExt,
    sink::SinkExt,
    StreamExt,
};
use libra_config::config::{NetworkConfig, NodeConfig, PeerNetworkId, RoleType, UpstreamConfig};
use libra_network_address::NetworkAddress;
use libra_types::{transaction::SignedTransaction, PeerId};
use network::{
    peer_manager::{
        conn_status_channel, ConnectionRequestSender, ConnectionStatusNotification,
        PeerManagerNotification, PeerManagerRequest, PeerManagerRequestSender,
    },
    DisconnectReason, ProtocolId,
};
use std::{
    collections::{HashMap, HashSet},
    num::NonZeroUsize,
    sync::{Arc, Mutex, RwLock},
    time::Duration,
};
use storage_interface::mock::MockDbReader;
use tokio::runtime::{Builder, Runtime};
use vm_validator::mocks::mock_vm_validator::MockVMValidator;

#[derive(Default)]
struct SharedMempoolNetwork {
    mempools: HashMap<PeerId, Arc<Mutex<CoreMempool>>>,
    network_reqs_rxs:
        HashMap<PeerId, libra_channel::Receiver<(PeerId, ProtocolId), PeerManagerRequest>>,
    network_notifs_txs:
        HashMap<PeerId, libra_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>>,
    network_conn_event_notifs_txs: HashMap<PeerId, conn_status_channel::Sender>,
    runtimes: HashMap<PeerId, Runtime>,
    subscribers: HashMap<PeerId, UnboundedReceiver<SharedMempoolNotification>>,
    timers: HashMap<PeerId, UnboundedSender<SyncEvent>>,
    peer_ids: HashMap<PeerId, PeerId>,
}

// start a shared mempool for a node `peer_id` with config `config`
// and add it to `smp` network
fn init_single_shared_mempool(smp: &mut SharedMempoolNetwork, peer_id: PeerId, config: NodeConfig) {
    let mempool = Arc::new(Mutex::new(CoreMempool::new(&config)));
    let (network_reqs_tx, network_reqs_rx) =
        libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
    let (connection_reqs_tx, _) =
        libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
    let (network_notifs_tx, network_notifs_rx) =
        libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
    let (conn_status_tx, conn_status_rx) = conn_status_channel::new();
    let network_sender = MempoolNetworkSender::new(
        PeerManagerRequestSender::new(network_reqs_tx),
        ConnectionRequestSender::new(connection_reqs_tx),
    );
    let network_events = MempoolNetworkEvents::new(network_notifs_rx, conn_status_rx);
    let (sender, subscriber) = unbounded();
    let (timer_sender, timer_receiver) = unbounded();
    let (_ac_endpoint_sender, ac_endpoint_receiver) = mpsc::channel(1_024);
    let network_handles = vec![(peer_id, network_sender, network_events)];
    let (_consensus_sender, consensus_events) = mpsc::channel(1_024);
    let (_state_sync_sender, state_sync_events) = mpsc::channel(1_024);
    let (_reconfig_events, reconfig_events_receiver) =
        libra_channel::new(QueueStyle::LIFO, NonZeroUsize::new(1).unwrap(), None);

    let runtime = Builder::new()
        .thread_name("shared-mem-")
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
        Some(timer_receiver.map(|_| SyncEvent).boxed()),
    );

    smp.mempools.insert(peer_id, mempool);
    smp.network_reqs_rxs.insert(peer_id, network_reqs_rx);
    smp.network_notifs_txs.insert(peer_id, network_notifs_tx);
    smp.network_conn_event_notifs_txs
        .insert(peer_id, conn_status_tx);
    smp.subscribers.insert(peer_id, subscriber);
    smp.timers.insert(peer_id, timer_sender);
    smp.runtimes.insert(peer_id, runtime);
}

// first PeerId in `network_ids` will be key in SharedMempoolNetwork
fn init_smp_multiple_networks(
    smp: &mut SharedMempoolNetwork,
    network_ids: Vec<PeerId>,
    config: NodeConfig,
) {
    let mempool = Arc::new(Mutex::new(CoreMempool::new(&config)));

    let mut network_handles = vec![];
    for peer_id in network_ids.iter() {
        let (network_reqs_tx, network_reqs_rx) =
            libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
        let (connection_reqs_tx, _) =
            libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
        let (network_notifs_tx, network_notifs_rx) =
            libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
        let (conn_status_tx, conn_status_rx) = conn_status_channel::new();
        let network_sender = MempoolNetworkSender::new(
            PeerManagerRequestSender::new(network_reqs_tx),
            ConnectionRequestSender::new(connection_reqs_tx),
        );
        let network_events = MempoolNetworkEvents::new(network_notifs_rx, conn_status_rx);
        network_handles.push((*peer_id, network_sender, network_events));

        smp.network_reqs_rxs.insert(*peer_id, network_reqs_rx);
        smp.network_notifs_txs.insert(*peer_id, network_notifs_tx);
        smp.network_conn_event_notifs_txs
            .insert(*peer_id, conn_status_tx);
    }

    let (sender, subscriber) = unbounded();
    let (timer_sender, timer_receiver) = unbounded();
    let (_ac_endpoint_sender, ac_endpoint_receiver) = mpsc::channel(1_024);
    let (_consensus_sender, consensus_events) = mpsc::channel(1_024);
    let (_state_sync_sender, state_sync_events) = mpsc::channel(1_024);
    let (_reconfig_events, reconfig_events_receiver) =
        libra_channel::new(QueueStyle::LIFO, NonZeroUsize::new(1).unwrap(), None);

    let runtime = Builder::new()
        .thread_name("shared-mem-")
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
        Some(timer_receiver.map(|_| SyncEvent).boxed()),
    );

    let main_peer_id = network_ids[0];
    smp.subscribers.insert(main_peer_id, subscriber);
    smp.timers.insert(main_peer_id, timer_sender);
    smp.mempools.insert(main_peer_id, mempool);
    smp.runtimes.insert(main_peer_id, runtime);
    for network_id in network_ids.into_iter().skip(1) {
        smp.peer_ids.insert(network_id, main_peer_id);
    }
}

impl SharedMempoolNetwork {
    fn bootstrap_validator_network(
        validator_nodes_count: u32,
        broadcast_batch_size: usize,
    ) -> (Self, Vec<PeerId>) {
        let mut smp = Self::default();
        let mut peers = vec![];

        for _ in 0..validator_nodes_count {
            let peer_id = PeerId::random();
            let mut validator_network_config = NetworkConfig::default();
            validator_network_config.peer_id = peer_id;
            let mut config = NodeConfig::random();
            config.validator_network = Some(validator_network_config);
            config.mempool.shared_mempool_batch_size = broadcast_batch_size;
            config.upstream = UpstreamConfig::default();
            config.upstream.primary_networks.push(peer_id);

            init_single_shared_mempool(&mut smp, peer_id, config);

            peers.push(peer_id);
        }
        (smp, peers)
    }

    /// creates a shared mempool network of one full node and one validator
    /// returns the newly created SharedMempoolNetwork, and the ID of validator and full node, in that order
    fn bootstrap_vfn_network(broadcast_batch_size: usize) -> (Self, PeerId, PeerId) {
        let mut smp = Self::default();

        // declare peers in network
        let validator = PeerId::random();
        let full_node = PeerId::random();

        // validator config
        let mut config = NodeConfig::random();
        config.mempool.shared_mempool_batch_size = broadcast_batch_size;

        let mut upstream_config = UpstreamConfig::default();
        upstream_config.primary_networks.push(validator);
        config.upstream = upstream_config;
        init_single_shared_mempool(&mut smp, validator, config);

        // full node
        let mut fn_config = NodeConfig::random();
        fn_config.base.role = RoleType::FullNode;
        fn_config.mempool.shared_mempool_batch_size = broadcast_batch_size;

        let mut upstream_config = UpstreamConfig::default();
        upstream_config
            .upstream_peers
            .insert(PeerNetworkId(full_node, validator));
        fn_config.upstream = upstream_config;
        init_single_shared_mempool(&mut smp, full_node, fn_config);

        (smp, validator, full_node)
    }

    fn add_txns(&mut self, peer_id: &PeerId, txns: Vec<TestTransaction>) {
        let mut mempool = self.mempools.get(peer_id).unwrap().lock().unwrap();
        for txn in txns {
            let transaction = txn.make_signed_transaction_with_max_gas_amount(5);
            mempool.add_txn(
                transaction.clone(),
                0,
                transaction.gas_unit_price(),
                0,
                TimelineState::NotReady,
                false,
            );
        }
    }

    fn send_connection_event(&mut self, peer: &PeerId, notif: ConnectionStatusNotification) {
        let conn_notifs_tx = self.network_conn_event_notifs_txs.get_mut(peer).unwrap();
        conn_notifs_tx.push(*peer, notif).unwrap();
        self.wait_for_event(peer, SharedMempoolNotification::PeerStateChange);
    }

    fn wait_for_event(&mut self, peer_id: &PeerId, event: SharedMempoolNotification) {
        let main_peer_id = self.peer_ids.get(peer_id).unwrap_or(peer_id);
        let subscriber = self.subscribers.get_mut(main_peer_id).unwrap();
        while block_on(subscriber.next()).unwrap() != event {
            continue;
        }
    }

    // checks that a node has no pending messages to send
    fn assert_no_message_sent(&mut self, peer: &PeerId) {
        let main_peer_id = self.peer_ids.get(peer).unwrap_or(peer);
        // emulate timer tick
        self.timers
            .get(main_peer_id)
            .unwrap()
            .unbounded_send(SyncEvent)
            .unwrap();

        // await next message from node
        let network_reqs_rx = self.network_reqs_rxs.get_mut(peer).unwrap();
        assert!(network_reqs_rx.select_next_some().now_or_never().is_none());
    }

    fn deliver_multi_network_message(
        &mut self,
        recipients: Vec<(PeerId, usize)>,
    ) -> Vec<(Vec<SignedTransaction>, PeerId)> {
        let first_peer_id = &recipients[0].0;
        let main_peer_id = self.peer_ids.get(first_peer_id).unwrap_or(first_peer_id);
        // emulate timer tick
        self.timers
            .get(main_peer_id)
            .unwrap()
            .unbounded_send(SyncEvent)
            .unwrap();

        let mut broadcasts = vec![];

        for (peer, count) in recipients {
            for _ in 0..count {
                // await next message from node
                let network_reqs_rx = self.network_reqs_rxs.get_mut(&peer).unwrap();
                let network_req = block_on(network_reqs_rx.next()).unwrap();

                if let PeerManagerRequest::SendMessage(peer_id, msg) = network_req {
                    let sync_msg = lcs::from_bytes(&msg.mdata).unwrap();
                    if let MempoolSyncMsg::BroadcastTransactionsRequest { transactions, .. } =
                        sync_msg
                    {
                        // send it to peer
                        let receiver_network_notif_tx =
                            self.network_notifs_txs.get_mut(&peer_id).unwrap();
                        receiver_network_notif_tx
                            .push(
                                (peer, ProtocolId::MempoolDirectSend),
                                PeerManagerNotification::RecvMessage(peer, msg),
                            )
                            .unwrap();

                        // await message delivery
                        self.wait_for_event(&peer_id, SharedMempoolNotification::NewTransactions);

                        // verify transaction was inserted into Mempool
                        let mempool = self.mempools.get(&peer_id).unwrap();
                        let block = mempool.lock().unwrap().get_block(100, HashSet::new());
                        for txn in transactions.iter() {
                            assert!(block.contains(txn));
                        }

                        // deliver ACK for this request
                        self.deliver_response(&peer_id);
                        broadcasts.push((transactions, peer_id));
                    } else {
                        panic!("did not receive expected BroadcastTransactionsRequest");
                    }
                } else {
                    panic!("peer {:?} didn't broadcast transaction", peer)
                }
            }
            let network_reqs_rx = self.network_reqs_rxs.get_mut(&peer).unwrap();
            assert!(network_reqs_rx.select_next_some().now_or_never().is_none());
        }
        broadcasts
    }

    /// delivers next broadcast message from `peer`
    fn deliver_message(&mut self, peer: &PeerId) -> (Vec<SignedTransaction>, PeerId) {
        println!("delivering");
        let main_peer_id = self.peer_ids.get(peer).unwrap_or(peer);
        // emulate timer tick
        self.timers
            .get(main_peer_id)
            .unwrap()
            .unbounded_send(SyncEvent)
            .unwrap();

        // await next message from node
        let network_reqs_rx = self.network_reqs_rxs.get_mut(peer).unwrap();
        let network_req = block_on(network_reqs_rx.next()).unwrap();

        if let PeerManagerRequest::SendMessage(peer_id, msg) = network_req {
            let sync_msg = lcs::from_bytes(&msg.mdata).unwrap();
            if let MempoolSyncMsg::BroadcastTransactionsRequest { transactions, .. } = sync_msg {
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
                let mempool = self.mempools.get(&peer_id).unwrap();
                let block = mempool.lock().unwrap().get_block(100, HashSet::new());
                for txn in transactions.iter() {
                    assert!(block.contains(txn));
                }

                // deliver ACK for this request
                self.deliver_response(&peer_id);
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
        let network_reqs_rx = self.network_reqs_rxs.get_mut(peer).unwrap();
        let network_req = block_on(network_reqs_rx.next()).unwrap();

        if let PeerManagerRequest::SendMessage(peer_id, msg) = network_req {
            let sync_msg = lcs::from_bytes(&msg.mdata).unwrap();
            if let MempoolSyncMsg::BroadcastTransactionsResponse { .. } = sync_msg {
                // send it to peer
                let receiver_network_notif_tx = self.network_notifs_txs.get_mut(&peer_id).unwrap();
                receiver_network_notif_tx
                    .push(
                        (*peer, ProtocolId::MempoolDirectSend),
                        PeerManagerNotification::RecvMessage(*peer, msg),
                    )
                    .unwrap();

                // await ACK delivery
                self.wait_for_event(&peer_id, SharedMempoolNotification::ACK);
            } else {
                panic!("did not receive expected broadcast ACK");
            }
        } else {
            panic!("peer {:?} did not ACK broadcast", peer);
        }
    }

    fn exist_in_metrics_cache(&self, peer_id: &PeerId, txn: &TestTransaction) -> bool {
        let mempool = self.mempools.get(peer_id).unwrap().lock().unwrap();
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
    let (mut smp, peers) = SharedMempoolNetwork::bootstrap_validator_network(2, 1);
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
    smp.send_connection_event(
        &peer_a,
        ConnectionStatusNotification::NewPeer(*peer_b, NetworkAddress::mock()),
    );

    for seq in 0..3 {
        // A attempts to send message
        let transactions = smp.deliver_message(&peer_a).0;
        assert_eq!(transactions.get(0).unwrap().sequence_number(), seq);
    }
}

#[test]
fn test_metric_cache_ignore_shared_txns() {
    let (mut smp, peers) = SharedMempoolNetwork::bootstrap_validator_network(2, 1);
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
    smp.send_connection_event(
        &peer_a,
        ConnectionStatusNotification::NewPeer(*peer_b, NetworkAddress::mock()),
    );
    for txn in txns.iter().take(3) {
        // Let peer_a share txns with peer_b
        let (_transaction, rx_peer) = smp.deliver_message(&peer_a);
        // Check if txns's creation timestamp exist in peer_b's metrics_cache.
        assert_eq!(smp.exist_in_metrics_cache(&rx_peer, txn), false);
    }
}

#[test]
fn test_interruption_in_sync() {
    let (mut smp, peers) = SharedMempoolNetwork::bootstrap_validator_network(3, 1);
    let (peer_a, peer_b, peer_c) = (
        peers.get(0).unwrap(),
        peers.get(1).unwrap(),
        peers.get(2).unwrap(),
    );
    smp.add_txns(&peer_a, vec![TestTransaction::new(1, 0, 1)]);

    // A discovers 2 peers
    smp.send_connection_event(
        &peer_a,
        ConnectionStatusNotification::NewPeer(*peer_b, NetworkAddress::mock()),
    );
    smp.send_connection_event(
        &peer_a,
        ConnectionStatusNotification::NewPeer(*peer_c, NetworkAddress::mock()),
    );

    // make sure it delivered first transaction to both nodes
    let mut peers = vec![
        smp.deliver_message(&peer_a).1,
        smp.deliver_message(&peer_a).1,
    ];
    peers.sort();
    let mut expected_peers = vec![*peer_b, *peer_c];
    expected_peers.sort();
    assert_eq!(peers, expected_peers);

    // A loses connection to B
    smp.send_connection_event(
        &peer_a,
        ConnectionStatusNotification::LostPeer(
            *peer_b,
            NetworkAddress::mock(),
            DisconnectReason::ConnectionLost,
        ),
    );

    // only C receives following transactions
    smp.add_txns(&peer_a, vec![TestTransaction::new(1, 1, 1)]);
    let (txn, peer_id) = smp.deliver_message(&peer_a);
    assert_eq!(peer_id, *peer_c);
    assert_eq!(txn.get(0).unwrap().sequence_number(), 1);

    smp.add_txns(&peer_a, vec![TestTransaction::new(1, 2, 1)]);
    let (txn, peer_id) = smp.deliver_message(&peer_a);
    assert_eq!(peer_id, *peer_c);
    assert_eq!(txn.get(0).unwrap().sequence_number(), 2);

    // A reconnects to B
    smp.send_connection_event(
        &peer_a,
        ConnectionStatusNotification::NewPeer(*peer_b, NetworkAddress::mock()),
    );

    // B should receive transaction 2
    let (txn, peer_id) = smp.deliver_message(&peer_a);
    assert_eq!(peer_id, *peer_b);
    assert_eq!(txn.get(0).unwrap().sequence_number(), 1);
}

#[test]
fn test_ready_transactions() {
    let (mut smp, peers) = SharedMempoolNetwork::bootstrap_validator_network(2, 1);
    let (peer_a, peer_b) = (peers.get(0).unwrap(), peers.get(1).unwrap());
    smp.add_txns(
        &peer_a,
        vec![TestTransaction::new(1, 0, 1), TestTransaction::new(1, 2, 1)],
    );
    // first message delivery
    smp.send_connection_event(
        &peer_a,
        ConnectionStatusNotification::NewPeer(*peer_b, NetworkAddress::mock()),
    );
    smp.deliver_message(&peer_a);

    // add txn1 to Mempool
    smp.add_txns(&peer_a, vec![TestTransaction::new(1, 1, 1)]);
    // txn1 unlocked txn2. Now all transactions can go through in correct order
    let txn = &smp.deliver_message(&peer_a).0;
    assert_eq!(txn.get(0).unwrap().sequence_number(), 1);
    let txn = &smp.deliver_message(&peer_a).0;
    assert_eq!(txn.get(0).unwrap().sequence_number(), 2);
}

#[test]
fn test_broadcast_self_transactions() {
    let (mut smp, peers) = SharedMempoolNetwork::bootstrap_validator_network(2, 1);
    let (peer_a, peer_b) = (peers.get(0).unwrap(), peers.get(1).unwrap());
    smp.add_txns(&peer_a, vec![TestTransaction::new(0, 0, 1)]);

    // A and B discover each other
    smp.send_connection_event(
        &peer_a,
        ConnectionStatusNotification::NewPeer(*peer_b, NetworkAddress::mock()),
    );
    smp.send_connection_event(
        &peer_b,
        ConnectionStatusNotification::NewPeer(*peer_a, NetworkAddress::mock()),
    );

    // A sends txn to B
    smp.deliver_message(&peer_a);

    // add new txn to B
    smp.add_txns(&peer_b, vec![TestTransaction::new(1, 0, 1)]);

    // verify that A will receive only second transaction from B
    let (txn, _) = smp.deliver_message(&peer_b);
    assert_eq!(
        txn.get(0).unwrap().sender(),
        TestTransaction::get_address(1)
    );
}

#[test]
fn test_broadcast_dependencies() {
    let (mut smp, peers) = SharedMempoolNetwork::bootstrap_validator_network(2, 1);
    let (peer_a, peer_b) = (peers.get(0).unwrap(), peers.get(1).unwrap());
    // Peer A has transactions with sequence numbers 0 and 2
    smp.add_txns(
        &peer_a,
        vec![TestTransaction::new(0, 0, 1), TestTransaction::new(0, 2, 1)],
    );
    // Peer B has txn1
    smp.add_txns(&peer_b, vec![TestTransaction::new(0, 1, 1)]);

    // A and B discover each other
    smp.send_connection_event(
        &peer_a,
        ConnectionStatusNotification::NewPeer(*peer_b, NetworkAddress::mock()),
    );
    smp.send_connection_event(
        &peer_b,
        ConnectionStatusNotification::NewPeer(*peer_a, NetworkAddress::mock()),
    );

    // B receives 0
    smp.deliver_message(&peer_a);
    // now B can broadcast 1
    let txn = smp.deliver_message(&peer_b).0;
    assert_eq!(txn.get(0).unwrap().sequence_number(), 1);
    // now A can broadcast 2
    let txn = smp.deliver_message(&peer_a).0;
    assert_eq!(txn.get(0).unwrap().sequence_number(), 2);
}

#[test]
fn test_broadcast_updated_transaction() {
    let (mut smp, peers) = SharedMempoolNetwork::bootstrap_validator_network(2, 1);
    let (peer_a, peer_b) = (peers.get(0).unwrap(), peers.get(1).unwrap());

    // Peer A has a transaction with sequence number 0 and gas price 1
    smp.add_txns(&peer_a, vec![TestTransaction::new(0, 0, 1)]);

    // A and B discover each other
    smp.send_connection_event(
        &peer_a,
        ConnectionStatusNotification::NewPeer(*peer_b, NetworkAddress::mock()),
    );
    smp.send_connection_event(
        &peer_b,
        ConnectionStatusNotification::NewPeer(*peer_a, NetworkAddress::mock()),
    );

    // B receives 0
    let txn = smp.deliver_message(&peer_a).0;
    assert_eq!(txn.get(0).unwrap().sequence_number(), 0);
    assert_eq!(txn.get(0).unwrap().gas_unit_price(), 1);

    // Update the gas price of the transaction with sequence 0 after B has already received 0
    smp.add_txns(&peer_a, vec![TestTransaction::new(0, 0, 5)]);

    // trigger send from A to B and check B has updated gas price for sequence 0
    let txn = smp.deliver_message(&peer_a).0;
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
    let committed_txn = TestTransaction::new(0, 0, 1)
        .make_signed_transaction_with_expiration_time(Duration::from_secs(0));
    let kept_txn = TestTransaction::new(1, 0, 1).make_signed_transaction(); // not committed or cleaned out by block timestamp gc
    let txns = vec![
        committed_txn.clone(),
        TestTransaction::new(0, 1, 1)
            .make_signed_transaction_with_expiration_time(Duration::from_secs(0)),
        kept_txn.clone(),
    ];
    // add txns to mempool
    {
        let mut pool = smp
            .mempool
            .lock()
            .expect("[mempool test] failed to acquire lock");
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
    let mut pool = smp
        .mempool
        .lock()
        .expect("[mempool test] failed to acquire mempool lock");
    let (timeline, _) = pool.read_timeline(0, 10);
    assert_eq!(timeline.len(), 1);
    assert!(timeline.contains(&kept_txn));
}

#[test]
fn test_state_sync_events_committed_txns() {
    let (mut state_sync_sender, state_sync_events) = mpsc::channel(1_024);
    let smp = MockSharedMempool::new(Some(state_sync_events));

    // add txns 1, 2, 3, 4
    // txn 1: committed successfully
    // txn 2: not committed but older than gc block timestamp
    // txn 3: not committed and newer than block timestamp
    let committed_txn = TestTransaction::new(0, 0, 1)
        .make_signed_transaction_with_expiration_time(Duration::from_secs(0));
    let kept_txn = TestTransaction::new(1, 0, 1).make_signed_transaction(); // not committed or cleaned out by block timestamp gc
    let txns = vec![
        committed_txn.clone(),
        TestTransaction::new(0, 1, 1)
            .make_signed_transaction_with_expiration_time(Duration::from_secs(0)),
        kept_txn.clone(),
    ];
    // add txns to mempool
    {
        let mut pool = smp
            .mempool
            .lock()
            .expect("[mempool test] failed to acquire lock");
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
    let mut pool = smp
        .mempool
        .lock()
        .expect("[mempool test] failed to acquire mempool lock");
    let (timeline, _) = pool.read_timeline(0, 10);
    assert_eq!(timeline.len(), 1);
    assert!(timeline.contains(&kept_txn));
}

#[test]
fn test_broadcast_ack_single_account_single_peer() {
    let batch_size = 3;
    let (mut smp, validator, full_node) = SharedMempoolNetwork::bootstrap_vfn_network(3);

    // add txns to FN
    // txns from single account
    let mut all_txns = vec![];
    for i in 0..10 {
        all_txns.push(TestTransaction::new(1, i, 1));
    }
    smp.add_txns(&full_node, all_txns.clone());

    // FN discovers new peer V
    smp.send_connection_event(
        &full_node,
        ConnectionStatusNotification::NewPeer(validator, NetworkAddress::mock()),
    );

    // deliver messages until FN mempool is empty
    let mut remaining_txn_index = batch_size;
    while remaining_txn_index < all_txns.len() + 1 {
        // deliver message
        let (_transactions, recipient) = smp.deliver_message(&full_node);
        assert_eq!(validator, recipient);

        // check that txns on FN have been GC'ed
        let mempool = smp.mempools.get(&full_node).unwrap();
        let block = mempool.lock().unwrap().get_block(100, HashSet::new());
        let remaining_txns = &all_txns[remaining_txn_index..];
        assert_eq!(block.len(), remaining_txns.len());
        for txn in remaining_txns {
            assert!(block.contains(&txn.make_signed_transaction_with_max_gas_amount(5)));
        }

        // update remaining txn index
        if remaining_txn_index == all_txns.len() {
            remaining_txn_index += batch_size;
        } else {
            remaining_txn_index = std::cmp::min(remaining_txn_index + batch_size, all_txns.len());
        }
    }
}

#[test]
fn test_broadcast_ack_multiple_accounts_single_peer() {
    let (mut smp, validator, full_node) = SharedMempoolNetwork::bootstrap_vfn_network(3);

    let all_txns = vec![
        TestTransaction::new(0, 0, 1),
        TestTransaction::new(0, 1, 1),
        TestTransaction::new(1, 0, 1),
        TestTransaction::new(1, 1, 1),
        TestTransaction::new(0, 2, 1),
    ];
    smp.add_txns(&full_node, all_txns.clone());

    // full node discovers new validator peer
    smp.send_connection_event(
        &full_node,
        ConnectionStatusNotification::NewPeer(validator, NetworkAddress::mock()),
    );

    // deliver message
    let (_transactions, recipient) = smp.deliver_message(&full_node);
    assert_eq!(validator, recipient);

    // check that txns have been GC'ed
    let mempool = smp.mempools.get(&full_node).unwrap();
    let remaining_txns = &all_txns[3..];
    let block = mempool.lock().unwrap().get_block(100, HashSet::new());
    assert_eq!(remaining_txns.len(), block.len());
    for txn in remaining_txns {
        assert!(block.contains(&txn.make_signed_transaction_with_max_gas_amount(5)));
    }
}

// primary_peers = k, fallbacks > 0
#[test]
fn test_k_policy_broadcast_no_fallback() {
    // all nodes
    let v_0 = PeerId::random();
    let fn_0 = PeerId::random();
    let fn_0_fallback_network_id = PeerId::random();
    let v_1 = PeerId::random();
    let fn_1 = PeerId::random();

    // v0 config
    let v0_config = NodeConfig::default();
    let v1_config = NodeConfig::default();

    // fn_0 has v0 as primary upstream peer and v1 as fallback upstream peer
    let mut fn_0_config = NodeConfig::default();
    fn_0_config.mempool.shared_mempool_batch_size = 1;
    fn_0_config
        .mempool
        .shared_mempool_min_broadcast_recipient_count = Some(1);
    fn_0_config
        .upstream
        .upstream_peers
        .insert(PeerNetworkId(fn_0, v_0));
    fn_0_config
        .upstream
        .fallback_networks
        .push(fn_0_fallback_network_id);

    let mut fn_1_config = NodeConfig::default();
    fn_1_config.mempool.shared_mempool_batch_size = 1;
    fn_1_config
        .upstream
        .upstream_peers
        .insert(PeerNetworkId(fn_1, v_1));

    let mut smp = SharedMempoolNetwork::default();
    init_single_shared_mempool(&mut smp, v_0, v0_config);
    init_single_shared_mempool(&mut smp, v_1, v1_config);
    init_single_shared_mempool(&mut smp, fn_1, fn_1_config);
    init_smp_multiple_networks(&mut smp, vec![fn_0, fn_0_fallback_network_id], fn_0_config);

    // fn_0 discovers primary and fallback upstream peers
    smp.send_connection_event(
        &fn_0,
        ConnectionStatusNotification::NewPeer(v_0, NetworkAddress::mock()),
    );
    smp.send_connection_event(
        &fn_0_fallback_network_id,
        ConnectionStatusNotification::NewPeer(fn_1, NetworkAddress::mock()),
    );

    // add txn to fn_0
    smp.add_txns(&fn_0, vec![TestTransaction::new(1, 0, 1)]);

    // make sure it delivers txn to primary peer
    let peers = vec![smp.deliver_message(&fn_0).1];
    assert_eq!(peers[0], v_0);
    // check that no messages have been sent to fallback upstream peer
    smp.assert_no_message_sent(&fn_0_fallback_network_id);
}

// primary_peers < k, fallback > 0, primary_peers + fallbacks = k
#[test]
fn test_k_policy_broadcast_to_fallback() {
    // all nodes
    let v_0 = PeerId::random();
    let fn_0 = PeerId::random();
    let fn_0_fallback_network_id = PeerId::random();
    let v_1 = PeerId::random();
    let fn_1 = PeerId::random();

    // v0 config
    let v0_config = NodeConfig::default();
    let v1_config = NodeConfig::default();

    // fn_0 has v0 as primary upstream peer and v1 as fallback upstream peer
    let mut fn_0_config = NodeConfig::default();
    fn_0_config.mempool.shared_mempool_batch_size = 1;
    fn_0_config
        .mempool
        .shared_mempool_min_broadcast_recipient_count = Some(2);
    fn_0_config
        .upstream
        .upstream_peers
        .insert(PeerNetworkId(fn_0, v_0));
    fn_0_config
        .upstream
        .fallback_networks
        .push(fn_0_fallback_network_id);

    let mut fn_1_config = NodeConfig::default();
    fn_1_config.mempool.shared_mempool_batch_size = 1;
    fn_1_config
        .upstream
        .upstream_peers
        .insert(PeerNetworkId(fn_1, v_1));

    let mut smp = SharedMempoolNetwork::default();
    init_single_shared_mempool(&mut smp, v_0, v0_config);
    init_single_shared_mempool(&mut smp, v_1, v1_config);
    init_single_shared_mempool(&mut smp, fn_1, fn_1_config);
    init_smp_multiple_networks(&mut smp, vec![fn_0, fn_0_fallback_network_id], fn_0_config);

    // fn_0 discovers primary and fallback upstream peers
    smp.send_connection_event(
        &fn_0,
        ConnectionStatusNotification::NewPeer(v_0, NetworkAddress::mock()),
    );
    smp.send_connection_event(
        &fn_0_fallback_network_id,
        ConnectionStatusNotification::NewPeer(fn_1, NetworkAddress::mock()),
    );

    // add txn to fn_0
    smp.add_txns(&fn_0, vec![TestTransaction::new(1, 0, 1)]);

    // make sure it delivers txn to both nodes
    let peers: Vec<PeerId> = smp
        .deliver_multi_network_message(vec![(fn_0, 1), (fn_0_fallback_network_id, 1)])
        .iter()
        .map(|(_txns, peer)| *peer)
        .collect();
    assert!(peers.contains(&fn_1));
    assert!(peers.contains(&v_0));
}

// primary_peers < k, fallback = 0 (primary_peers + fallbacks < k)
#[test]
fn test_k_policy_broadcast_not_enough_fallbacks() {
    // all nodes
    let v_0 = PeerId::random();
    let fn_0 = PeerId::random();
    let fn_0_fallback_network_id = PeerId::random();

    let v0_config = NodeConfig::default();
    // fn_0 has v0 as primary upstream peer and v1 as fallback upstream peer
    let mut fn_0_config = NodeConfig::default();
    fn_0_config.mempool.shared_mempool_batch_size = 1;
    fn_0_config
        .mempool
        .shared_mempool_min_broadcast_recipient_count = Some(2);
    fn_0_config
        .upstream
        .upstream_peers
        .insert(PeerNetworkId(fn_0, v_0));
    fn_0_config
        .upstream
        .fallback_networks
        .push(fn_0_fallback_network_id);

    let mut smp = SharedMempoolNetwork::default();
    init_single_shared_mempool(&mut smp, v_0, v0_config);
    init_smp_multiple_networks(&mut smp, vec![fn_0, fn_0_fallback_network_id], fn_0_config);

    // fn_0 discovers primary peer but no fallback peers available
    smp.send_connection_event(
        &fn_0,
        ConnectionStatusNotification::NewPeer(v_0, NetworkAddress::mock()),
    );

    // add txn to fn_0
    smp.add_txns(&fn_0, vec![TestTransaction::new(1, 0, 1)]);

    // make sure it delivers txn to primary peer
    let peers = vec![smp.deliver_message(&fn_0).1];
    assert_eq!(peers[0], v_0);
    // check that no messages have been sent to fallback upstream peer
    smp.assert_no_message_sent(&fn_0_fallback_network_id);
}

// primary_peers < k, primary_peers + fallback_peers > k
#[test]
fn test_k_policy_broadcast_excess_fallbacks() {
    // all nodes
    let v_0 = PeerId::random();
    let fn_0 = PeerId::random();
    let fn_0_fallback_network_id = PeerId::random();
    let fn_1 = PeerId::random();
    let fn_2 = PeerId::random();

    // v0 config
    let v0_config = NodeConfig::default();
    let fn_2_config = NodeConfig::default();

    // fn_0 has v0 as primary upstream peer and v1 as fallback upstream peer
    let mut fn_0_config = NodeConfig::default();
    fn_0_config.mempool.shared_mempool_batch_size = 1;
    fn_0_config
        .mempool
        .shared_mempool_min_broadcast_recipient_count = Some(2);
    fn_0_config
        .upstream
        .upstream_peers
        .insert(PeerNetworkId(fn_0, v_0));
    fn_0_config
        .upstream
        .fallback_networks
        .push(fn_0_fallback_network_id);

    let fn_1_config = NodeConfig::default();

    let mut smp = SharedMempoolNetwork::default();
    init_single_shared_mempool(&mut smp, v_0, v0_config);
    init_single_shared_mempool(&mut smp, fn_1, fn_1_config);
    init_single_shared_mempool(&mut smp, fn_2, fn_2_config);
    init_smp_multiple_networks(&mut smp, vec![fn_0, fn_0_fallback_network_id], fn_0_config);

    // fn_0 discovers primary and fallback upstream peers
    smp.send_connection_event(
        &fn_0,
        ConnectionStatusNotification::NewPeer(v_0, NetworkAddress::mock()),
    );
    smp.send_connection_event(
        &fn_0_fallback_network_id,
        ConnectionStatusNotification::NewPeer(fn_1, NetworkAddress::mock()),
    );
    smp.send_connection_event(
        &fn_0_fallback_network_id,
        ConnectionStatusNotification::NewPeer(fn_2, NetworkAddress::mock()),
    );

    // add txn to fn_0
    smp.add_txns(&fn_0, vec![TestTransaction::new(1, 0, 1)]);

    // make sure it delivers txn to both nodes
    let peers: Vec<PeerId> = smp
        .deliver_multi_network_message(vec![(fn_0, 1), (fn_0_fallback_network_id, 1)])
        .iter()
        .map(|(_txns, peer)| *peer)
        .collect();
    assert!(peers.contains(&v_0));
    assert!(peers.contains(&fn_1) || peers.contains(&fn_2));
}

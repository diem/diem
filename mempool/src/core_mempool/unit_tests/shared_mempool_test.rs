// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    core_mempool::{
        unit_tests::common::{batch_add_signed_txn, TestTransaction},
        CoreMempool, TimelineState,
    },
    mocks::MockSharedMempool,
    shared_mempool::{
        start_shared_mempool, ConsensusRequest, SharedMempoolNotification, SyncEvent,
    },
    CommitNotification, CommittedTransaction,
};
use channel::{self, libra_channel, message_queues::QueueStyle};
use futures::{
    channel::{
        mpsc::{self, unbounded, UnboundedReceiver, UnboundedSender},
        oneshot,
    },
    executor::block_on,
    sink::SinkExt,
    StreamExt,
};
use libra_config::config::{NetworkConfig, NodeConfig};
use libra_types::{transaction::SignedTransaction, PeerId};
use network::{
    peer_manager::{
        conn_status_channel, ConnectionStatusNotification, PeerManagerNotification,
        PeerManagerRequest,
    },
    proto::MempoolSyncMsg,
    validator_network::{MempoolNetworkEvents, MempoolNetworkSender},
    DisconnectReason, ProtocolId,
};
use parity_multiaddr::Multiaddr;
use prost::Message;
use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
    num::NonZeroUsize,
    sync::{Arc, Mutex},
    time::Duration,
};
use storage_service::mocks::mock_storage_client::MockStorageReadClient;
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
}

impl SharedMempoolNetwork {
    fn bootstrap_validator_network(validator_nodes_count: u32) -> (Self, Vec<PeerId>) {
        let mut smp = Self::default();
        let mut peers = vec![];

        for _ in 0..validator_nodes_count {
            let peer_id = PeerId::random();
            let mut validator_network_config = NetworkConfig::default();
            validator_network_config.peer_id = peer_id;
            let mut config = NodeConfig::random();
            config.validator_network = Some(validator_network_config);
            config.mempool.shared_mempool_batch_size = 1;

            let mempool = Arc::new(Mutex::new(CoreMempool::new(&config)));
            let (network_reqs_tx, network_reqs_rx) =
                libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
            let (network_notifs_tx, network_notifs_rx) =
                libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
            let (conn_status_tx, conn_status_rx) = conn_status_channel::new();
            let network_sender = MempoolNetworkSender::new(network_reqs_tx);
            let network_events = MempoolNetworkEvents::new(network_notifs_rx, conn_status_rx);
            let (sender, subscriber) = unbounded();
            let (timer_sender, timer_receiver) = unbounded();
            let (_ac_endpoint_sender, ac_endpoint_receiver) = mpsc::channel(1_024);
            let network_handles = vec![(peer_id, network_sender, network_events)];
            let (_consensus_sender, consensus_events) = mpsc::channel(1_024);
            let (_state_sync_sender, state_sync_events) = mpsc::channel(1_024);

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
                Arc::new(MockStorageReadClient),
                Arc::new(MockVMValidator),
                vec![sender],
                Some(timer_receiver.map(|_| SyncEvent).boxed()),
            );

            peers.push(peer_id);
            smp.mempools.insert(peer_id, mempool);
            smp.network_reqs_rxs.insert(peer_id, network_reqs_rx);
            smp.network_notifs_txs.insert(peer_id, network_notifs_tx);
            smp.network_conn_event_notifs_txs
                .insert(peer_id, conn_status_tx);
            smp.subscribers.insert(peer_id, subscriber);
            smp.timers.insert(peer_id, timer_sender);
            smp.runtimes.insert(peer_id, runtime);
        }
        (smp, peers)
    }

    fn add_txns(&mut self, peer_id: &PeerId, txns: Vec<TestTransaction>) {
        let mut mempool = self.mempools.get(peer_id).unwrap().lock().unwrap();
        for txn in txns {
            let transaction = txn.make_signed_transaction_with_max_gas_amount(5);
            mempool.add_txn(transaction, 0, 0, 10, TimelineState::NotReady);
        }
    }

    fn send_connection_event(&mut self, peer: &PeerId, notif: ConnectionStatusNotification) {
        let conn_notifs_tx = self.network_conn_event_notifs_txs.get_mut(peer).unwrap();
        conn_notifs_tx.push(*peer, notif).unwrap();
        self.wait_for_event(peer, SharedMempoolNotification::PeerStateChange);
    }

    fn wait_for_event(&mut self, peer_id: &PeerId, event: SharedMempoolNotification) {
        let subscriber = self.subscribers.get_mut(peer_id).unwrap();
        while block_on(subscriber.next()).unwrap() != event {
            continue;
        }
    }

    /// deliveres next message from given node to it's peer
    fn deliver_message(&mut self, peer: &PeerId) -> (SignedTransaction, PeerId) {
        // emulate timer tick
        self.timers
            .get(peer)
            .unwrap()
            .unbounded_send(SyncEvent)
            .unwrap();

        // await next message from node
        let network_reqs_rx = self.network_reqs_rxs.get_mut(peer).unwrap();
        let network_req = block_on(network_reqs_rx.next()).unwrap();

        match network_req {
            PeerManagerRequest::SendMessage(peer_id, msg) => {
                let mut sync_msg = MempoolSyncMsg::decode(msg.mdata.as_ref()).unwrap();
                let transaction =
                    SignedTransaction::try_from(sync_msg.transactions.pop().unwrap()).unwrap();
                // send it to peer
                let receiver_network_notif_tx = self.network_notifs_txs.get_mut(&peer_id).unwrap();
                receiver_network_notif_tx
                    .push(
                        (
                            *peer,
                            ProtocolId::from_static(
                                network::validator_network::MEMPOOL_DIRECT_SEND_PROTOCOL,
                            ),
                        ),
                        PeerManagerNotification::RecvMessage(*peer, msg),
                    )
                    .unwrap();

                // await message delivery
                self.wait_for_event(&peer_id, SharedMempoolNotification::NewTransactions);

                // verify transaction was inserted into Mempool
                let mempool = self.mempools.get(&peer_id).unwrap();
                let block = mempool.lock().unwrap().get_block(100, HashSet::new());
                assert!(block.iter().any(|t| t == &transaction));
                (transaction, peer_id)
            }
            _ => panic!("peer {:?} didn't broadcast transaction", peer),
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
    let (mut smp, peers) = SharedMempoolNetwork::bootstrap_validator_network(2);
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
        ConnectionStatusNotification::NewPeer(*peer_b, Multiaddr::empty()),
    );

    for seq in 0..3 {
        // A attempts to send message
        let transaction = smp.deliver_message(&peer_a).0;
        assert_eq!(transaction.sequence_number(), seq);
    }
}

#[test]
fn test_metric_cache_ignore_shared_txns() {
    let (mut smp, peers) = SharedMempoolNetwork::bootstrap_validator_network(2);
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
        ConnectionStatusNotification::NewPeer(*peer_b, Multiaddr::empty()),
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
    let (mut smp, peers) = SharedMempoolNetwork::bootstrap_validator_network(3);
    let (peer_a, peer_b, peer_c) = (
        peers.get(0).unwrap(),
        peers.get(1).unwrap(),
        peers.get(2).unwrap(),
    );
    smp.add_txns(&peer_a, vec![TestTransaction::new(1, 0, 1)]);

    // A discovers 2 peers
    smp.send_connection_event(
        &peer_a,
        ConnectionStatusNotification::NewPeer(*peer_b, Multiaddr::empty()),
    );
    smp.send_connection_event(
        &peer_a,
        ConnectionStatusNotification::NewPeer(*peer_c, Multiaddr::empty()),
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
            Multiaddr::empty(),
            DisconnectReason::ConnectionLost,
        ),
    );

    // only C receives following transactions
    smp.add_txns(&peer_a, vec![TestTransaction::new(1, 1, 1)]);
    let (txn, peer_id) = smp.deliver_message(&peer_a);
    assert_eq!(peer_id, *peer_c);
    assert_eq!(txn.sequence_number(), 1);

    smp.add_txns(&peer_a, vec![TestTransaction::new(1, 2, 1)]);
    let (txn, peer_id) = smp.deliver_message(&peer_a);
    assert_eq!(peer_id, *peer_c);
    assert_eq!(txn.sequence_number(), 2);

    // A reconnects to B
    smp.send_connection_event(
        &peer_a,
        ConnectionStatusNotification::NewPeer(*peer_b, Multiaddr::empty()),
    );

    // B should receive transaction 2
    let (txn, peer_id) = smp.deliver_message(&peer_a);
    assert_eq!(peer_id, *peer_b);
    assert_eq!(txn.sequence_number(), 1);
}

#[test]
fn test_ready_transactions() {
    let (mut smp, peers) = SharedMempoolNetwork::bootstrap_validator_network(2);
    let (peer_a, peer_b) = (peers.get(0).unwrap(), peers.get(1).unwrap());
    smp.add_txns(
        &peer_a,
        vec![TestTransaction::new(1, 0, 1), TestTransaction::new(1, 2, 1)],
    );
    // first message delivery
    smp.send_connection_event(
        &peer_a,
        ConnectionStatusNotification::NewPeer(*peer_b, Multiaddr::empty()),
    );
    smp.deliver_message(&peer_a);

    // add txn1 to Mempool
    smp.add_txns(&peer_a, vec![TestTransaction::new(1, 1, 1)]);
    // txn1 unlocked txn2. Now all transactions can go through in correct order
    let txn = &smp.deliver_message(&peer_a).0;
    assert_eq!(txn.sequence_number(), 1);
    let txn = &smp.deliver_message(&peer_a).0;
    assert_eq!(txn.sequence_number(), 2);
}

#[test]
fn test_broadcast_self_transactions() {
    let (mut smp, peers) = SharedMempoolNetwork::bootstrap_validator_network(2);
    let (peer_a, peer_b) = (peers.get(0).unwrap(), peers.get(1).unwrap());
    smp.add_txns(&peer_a, vec![TestTransaction::new(0, 0, 1)]);

    // A and B discover each other
    smp.send_connection_event(
        &peer_a,
        ConnectionStatusNotification::NewPeer(*peer_b, Multiaddr::empty()),
    );
    smp.send_connection_event(
        &peer_b,
        ConnectionStatusNotification::NewPeer(*peer_a, Multiaddr::empty()),
    );

    // A sends txn to B
    smp.deliver_message(&peer_a);

    // add new txn to B
    smp.add_txns(&peer_b, vec![TestTransaction::new(1, 0, 1)]);

    // verify that A will receive only second transaction from B
    let (txn, _) = smp.deliver_message(&peer_b);
    assert_eq!(txn.sender(), TestTransaction::get_address(1));
}

#[test]
fn test_broadcast_dependencies() {
    let (mut smp, peers) = SharedMempoolNetwork::bootstrap_validator_network(2);
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
        ConnectionStatusNotification::NewPeer(*peer_b, Multiaddr::empty()),
    );
    smp.send_connection_event(
        &peer_b,
        ConnectionStatusNotification::NewPeer(*peer_a, Multiaddr::empty()),
    );

    // B receives 0
    smp.deliver_message(&peer_a);
    // now B can broadcast 1
    let txn = smp.deliver_message(&peer_b).0;
    assert_eq!(txn.sequence_number(), 1);
    // now A can broadcast 2
    let txn = smp.deliver_message(&peer_a).0;
    assert_eq!(txn.sequence_number(), 2);
}

#[test]
fn test_broadcast_updated_transaction() {
    let (mut smp, peers) = SharedMempoolNetwork::bootstrap_validator_network(2);
    let (peer_a, peer_b) = (peers.get(0).unwrap(), peers.get(1).unwrap());

    // Peer A has a transaction with sequence number 0 and gas price 1
    smp.add_txns(&peer_a, vec![TestTransaction::new(0, 0, 1)]);

    // A and B discover each other
    smp.send_connection_event(
        &peer_a,
        ConnectionStatusNotification::NewPeer(*peer_b, Multiaddr::empty()),
    );
    smp.send_connection_event(
        &peer_b,
        ConnectionStatusNotification::NewPeer(*peer_a, Multiaddr::empty()),
    );

    // B receives 0
    let txn = smp.deliver_message(&peer_a).0;
    assert_eq!(txn.sequence_number(), 0);
    assert_eq!(txn.gas_unit_price(), 1);

    // Update the gas price of the transaction with sequence 0 after B has already received 0
    smp.add_txns(&peer_a, vec![TestTransaction::new(0, 0, 5)]);

    // trigger send from A to B and check B has updated gas price for sequence 0
    let txn = smp.deliver_message(&peer_a).0;
    assert_eq!(txn.sequence_number(), 0);
    assert_eq!(txn.gas_unit_price(), 5);
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

// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    core_mempool::{unit_tests::common::TestTransaction, CoreMempool, TimelineState},
    shared_mempool::{start_shared_mempool, SharedMempoolNotification, SyncEvent},
};
use channel;
use futures::{
    channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender},
    executor::block_on,
    SinkExt, StreamExt,
};
use libra_config::config::NodeConfig;
use libra_types::{transaction::SignedTransaction, PeerId};
use network::{
    interface::{NetworkNotification, NetworkRequest},
    proto::MempoolSyncMsg,
    validator_network::{MempoolNetworkEvents, MempoolNetworkSender},
};
use prost::Message;
use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
    sync::{Arc, Mutex},
};
use storage_service::mocks::mock_storage_client::MockStorageReadClient;
use tokio::runtime::Runtime;
use vm_validator::mocks::mock_vm_validator::MockVMValidator;

#[derive(Default)]
struct SharedMempoolNetwork {
    mempools: HashMap<PeerId, Arc<Mutex<CoreMempool>>>,
    network_reqs_rxs: HashMap<PeerId, channel::Receiver<NetworkRequest>>,
    network_notifs_txs: HashMap<PeerId, channel::Sender<NetworkNotification>>,
    runtimes: HashMap<PeerId, Runtime>,
    subscribers: HashMap<PeerId, UnboundedReceiver<SharedMempoolNotification>>,
    timers: HashMap<PeerId, UnboundedSender<SyncEvent>>,
}

impl SharedMempoolNetwork {
    fn bootstrap_with_config(peers: Vec<PeerId>, mut config: NodeConfig) -> Self {
        let mut smp = Self::default();
        config.mempool.shared_mempool_batch_size = 1;

        for peer in peers {
            let mempool = Arc::new(Mutex::new(CoreMempool::new(&config)));
            let (network_reqs_tx, network_reqs_rx) = channel::new_test(8);
            let (network_notifs_tx, network_notifs_rx) = channel::new_test(8);
            let network_sender = MempoolNetworkSender::new(network_reqs_tx);
            let network_events = MempoolNetworkEvents::new(network_notifs_rx);
            let (sender, subscriber) = unbounded();
            let (timer_sender, timer_receiver) = unbounded();

            let runtime = start_shared_mempool(
                &config,
                Arc::clone(&mempool),
                network_sender,
                network_events,
                Arc::new(MockStorageReadClient),
                Arc::new(MockVMValidator),
                vec![sender],
                Some(timer_receiver.map(|_| SyncEvent).boxed()),
            );

            smp.mempools.insert(peer, mempool);
            smp.network_reqs_rxs.insert(peer, network_reqs_rx);
            smp.network_notifs_txs.insert(peer, network_notifs_tx);
            smp.subscribers.insert(peer, subscriber);
            smp.timers.insert(peer, timer_sender);
            smp.runtimes.insert(peer, runtime);
        }
        smp
    }

    fn bootstrap(peers: Vec<PeerId>) -> Self {
        Self::bootstrap_with_config(peers, NodeConfig::random())
    }

    fn add_txns(&mut self, peer_id: &PeerId, txns: Vec<TestTransaction>) {
        let mut mempool = self.mempools.get(peer_id).unwrap().lock().unwrap();
        for txn in txns {
            let transaction = txn.make_signed_transaction_with_max_gas_amount(5);
            mempool.add_txn(transaction, 0, 0, 10, TimelineState::NotReady);
        }
    }

    fn send_event(&mut self, peer: &PeerId, notif: NetworkNotification) {
        let network_notifs_tx = self.network_notifs_txs.get_mut(peer).unwrap();
        block_on(network_notifs_tx.send(notif)).unwrap();
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
            NetworkRequest::SendMessage(peer_id, msg) => {
                let mut sync_msg = MempoolSyncMsg::decode(msg.mdata.as_ref()).unwrap();
                let transaction =
                    SignedTransaction::try_from(sync_msg.transactions.pop().unwrap()).unwrap();
                // send it to peer
                let receiver_network_notif_tx = self.network_notifs_txs.get_mut(&peer_id).unwrap();
                block_on(
                    receiver_network_notif_tx.send(NetworkNotification::RecvMessage(*peer, msg)),
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
    let (peer_a, peer_b) = (PeerId::random(), PeerId::random());

    let mut smp = SharedMempoolNetwork::bootstrap(vec![peer_a, peer_b]);
    smp.add_txns(
        &peer_a,
        vec![
            TestTransaction::new(1, 0, 1),
            TestTransaction::new(1, 1, 1),
            TestTransaction::new(1, 2, 1),
        ],
    );

    // A discovers new peer B
    smp.send_event(&peer_a, NetworkNotification::NewPeer(peer_b));

    for seq in 0..3 {
        // A attempts to send message
        let transaction = smp.deliver_message(&peer_a).0;
        assert_eq!(transaction.sequence_number(), seq);
    }
}

#[test]
fn test_metric_cache_ignore_shared_txns() {
    let (peer_a, peer_b) = (PeerId::random(), PeerId::random());

    let mut smp = SharedMempoolNetwork::bootstrap(vec![peer_a, peer_b]);
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
    smp.send_event(&peer_a, NetworkNotification::NewPeer(peer_b));
    for txn in txns.iter().take(3) {
        // Let peer_a share txns with peer_b
        let (_transaction, rx_peer) = smp.deliver_message(&peer_a);
        // Check if txns's creation timestamp exist in peer_b's metrics_cache.
        assert_eq!(smp.exist_in_metrics_cache(&rx_peer, txn), false);
    }
}

#[test]
fn test_interruption_in_sync() {
    let (peer_a, peer_b, peer_c) = (PeerId::random(), PeerId::random(), PeerId::random());
    let mut smp = SharedMempoolNetwork::bootstrap(vec![peer_a, peer_b, peer_c]);
    smp.add_txns(&peer_a, vec![TestTransaction::new(1, 0, 1)]);

    // A discovers 2 peers
    smp.send_event(&peer_a, NetworkNotification::NewPeer(peer_b));
    smp.send_event(&peer_a, NetworkNotification::NewPeer(peer_c));

    // make sure it delivered first transaction to both nodes
    let mut peers = vec![
        smp.deliver_message(&peer_a).1,
        smp.deliver_message(&peer_a).1,
    ];
    peers.sort();
    let mut expected_peers = vec![peer_b, peer_c];
    expected_peers.sort();
    assert_eq!(peers, expected_peers);

    // A loses connection to B
    smp.send_event(&peer_a, NetworkNotification::LostPeer(peer_b));

    // only C receives following transactions
    smp.add_txns(&peer_a, vec![TestTransaction::new(1, 1, 1)]);
    let (txn, peer_id) = smp.deliver_message(&peer_a);
    assert_eq!(peer_id, peer_c);
    assert_eq!(txn.sequence_number(), 1);

    smp.add_txns(&peer_a, vec![TestTransaction::new(1, 2, 1)]);
    let (txn, peer_id) = smp.deliver_message(&peer_a);
    assert_eq!(peer_id, peer_c);
    assert_eq!(txn.sequence_number(), 2);

    // A reconnects to B
    smp.send_event(&peer_a, NetworkNotification::NewPeer(peer_b));

    // B should receive transaction 2
    let (txn, peer_id) = smp.deliver_message(&peer_a);
    assert_eq!(peer_id, peer_b);
    assert_eq!(txn.sequence_number(), 1);
}

#[test]
fn test_ready_transactions() {
    let (peer_a, peer_b) = (PeerId::random(), PeerId::random());
    let mut smp = SharedMempoolNetwork::bootstrap(vec![peer_a, peer_b]);
    smp.add_txns(
        &peer_a,
        vec![TestTransaction::new(1, 0, 1), TestTransaction::new(1, 2, 1)],
    );
    // first message delivery
    smp.send_event(&peer_a, NetworkNotification::NewPeer(peer_b));
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
    let (peer_a, peer_b) = (PeerId::random(), PeerId::random());
    let mut smp = SharedMempoolNetwork::bootstrap(vec![peer_a, peer_b]);
    smp.add_txns(&peer_a, vec![TestTransaction::new(0, 0, 1)]);

    // A and B discover each other
    smp.send_event(&peer_a, NetworkNotification::NewPeer(peer_b));
    smp.send_event(&peer_b, NetworkNotification::NewPeer(peer_a));

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
    let (peer_a, peer_b) = (PeerId::random(), PeerId::random());
    let mut smp = SharedMempoolNetwork::bootstrap(vec![peer_a, peer_b]);
    // Peer A has transactions with sequence numbers 0 and 2
    smp.add_txns(
        &peer_a,
        vec![TestTransaction::new(0, 0, 1), TestTransaction::new(0, 2, 1)],
    );
    // Peer B has txn1
    smp.add_txns(&peer_b, vec![TestTransaction::new(0, 1, 1)]);

    // A and B discover each other
    smp.send_event(&peer_a, NetworkNotification::NewPeer(peer_b));
    smp.send_event(&peer_b, NetworkNotification::NewPeer(peer_a));

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
    let (peer_a, peer_b) = (PeerId::random(), PeerId::random());
    let mut smp = SharedMempoolNetwork::bootstrap(vec![peer_a, peer_b]);

    // Peer A has a transaction with sequence number 0 and gas price 1
    smp.add_txns(&peer_a, vec![TestTransaction::new(0, 0, 1)]);

    // A and B discover each other
    smp.send_event(&peer_a, NetworkNotification::NewPeer(peer_b));
    smp.send_event(&peer_b, NetworkNotification::NewPeer(peer_a));

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

// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::core_mempool::{unit_tests::common::TestTransaction, CoreMempool, TimelineState};
use crate::shared_mempool::{start_shared_mempool, SharedMempoolNotification};
use assert_matches;
use channel;
use futures::{
    channel::{
        mpsc::{unbounded, UnboundedReceiver},
        oneshot,
    },
    executor::block_on,
    SinkExt, StreamExt,
};
use libra_config::config::{NetworkConfig, NodeConfig};
use libra_prost_ext::MessageExt;
use libra_types::{transaction::SignedTransaction, PeerId};
use network::{
    interface::{NetworkNotification, NetworkRequest},
    proto::{BroadcastTransactionsResponse, MempoolSyncMsg, MempoolSyncMsg_oneof},
    validator_network::{MempoolNetworkEvents, MempoolNetworkSender},
};
use network::{
    protocols::rpc::InboundRpcRequest, validator_network::MEMPOOL_RPC_PROTOCOL, ProtocolId,
};
use prost::Message as _;
use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
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
}

impl SharedMempoolNetwork {
    fn bootstrap_validator_network_smp(validator_nodes_count: u32) -> (Self, Vec<PeerId>) {
        let mut config = NodeConfig::random();
        let mut smp = Self::default();
        config.mempool.shared_mempool_batch_size = 1;

        // add all peers in this vector as validator peers
        let validator_network_peers =
            NetworkConfig::default_validator_network_with_peers(validator_nodes_count);

        let mut peers: Vec<PeerId> = Vec::new();
        for peer in validator_network_peers.keys() {
            peers.push(peer.clone());
        }
        for (peer, validator_network_config) in validator_network_peers.into_iter() {
            let mempool = Arc::new(Mutex::new(CoreMempool::new(&config)));
            let (network_reqs_tx, network_reqs_rx) = channel::new_test(8);
            let (network_notifs_tx, network_notifs_rx) = channel::new_test(8);
            let network_sender = MempoolNetworkSender::new(network_reqs_tx);
            let network_events = MempoolNetworkEvents::new(network_notifs_rx);
            let (sender, subscriber) = unbounded();

            config.validator_network = Some(validator_network_config);
            let runtime = start_shared_mempool(
                &config,
                Arc::clone(&mempool),
                network_sender,
                network_events,
                Arc::new(MockStorageReadClient),
                Arc::new(MockVMValidator),
                vec![sender],
            );

            smp.mempools.insert(peer, mempool);
            smp.network_reqs_rxs.insert(peer, network_reqs_rx);
            smp.network_notifs_txs.insert(peer, network_notifs_tx);
            smp.subscribers.insert(peer, subscriber);
            smp.runtimes.insert(peer, runtime);
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

    /// wait for a set of SharedMempoolNotification events in no particular order
    fn wait_for_sync_events(
        &mut self,
        peer_id: &PeerId,
        mut events: HashSet<SharedMempoolNotification>,
    ) {
        let subscriber = self.subscribers.get_mut(peer_id).unwrap();
        loop {
            let event = block_on(subscriber.next()).unwrap();
            events.remove(&event);
            if events.is_empty() {
                break;
            }
        }
    }

    fn wait_for_worker_update(&mut self, peer_id: &PeerId, expected_worker_peer: &PeerId) {
        self.wait_for_worker_updates(peer_id, vec![*expected_worker_peer]);
    }

    /// wait for SharedMempoolNotification::SyncUpdate from worker processes for `expected_worker_peers`
    /// in SMP of `peer_id`. There is no expected ordering of events received from `expected_worker_peers`
    fn wait_for_worker_updates(&mut self, peer_id: &PeerId, expected_worker_peers: Vec<PeerId>) {
        let expected_events = expected_worker_peers
            .into_iter()
            .map(|worker_peer| SharedMempoolNotification::SyncUpdate { worker_peer })
            .collect();

        self.wait_for_sync_events(peer_id, expected_events);
    }

    /// delivers next single message from given node to its peer
    fn deliver_message(&mut self, peer: &PeerId) -> (Vec<SignedTransaction>, PeerId) {
        thread::sleep(Duration::from_millis(50)); // TODO this is hardcoded from default config - change this!

        let res_msg = BroadcastTransactionsResponse::default();
        let res_msg_enum = MempoolSyncMsg {
            message: Some(MempoolSyncMsg_oneof::BroadcastTransactionsResponse(
                res_msg.clone(),
            )),
        };
        let res_data = res_msg_enum.to_bytes().unwrap();

        // wait for requests sent
        let network_reqs_rx = self.network_reqs_rxs.get_mut(peer).unwrap();
        let network_req = block_on(network_reqs_rx.next()).unwrap();

        match network_req {
            NetworkRequest::SendRpc(peer_id, msg) => {
                // parse msg
                let sync_msg = MempoolSyncMsg::decode(msg.data.as_ref()).unwrap();
                let req = assert_matches::assert_matches!(
                    sync_msg.message.clone(),
                    Some(MempoolSyncMsg_oneof::BroadcastTransactionsRequest(req)) => req,
                    "expected to receive BroadcastTransactionRequest"
                );

                // get transactions
                let transactions: Vec<_> = req
                    .transactions
                    .clone()
                    .into_iter()
                    .filter_map(|txn| match SignedTransaction::try_from(txn.clone()) {
                        Ok(t) => Some(t),
                        Err(_e) => None,
                    })
                    .collect();
                let protocol = ProtocolId::from_static(MEMPOOL_RPC_PROTOCOL);
                let req_msg_enum = MempoolSyncMsg {
                    message: sync_msg.message,
                };
                let data = req_msg_enum.to_bytes().unwrap();

                // callback from receiving node to (this) sending node
                let (res_tx, res_rx) = oneshot::channel();
                let inbound_rpc_req = InboundRpcRequest {
                    protocol,
                    data,
                    res_tx,
                };
                let receiver_network_notif_tx = self.network_notifs_txs.get_mut(&peer_id).unwrap();
                block_on(
                    receiver_network_notif_tx
                        .send(NetworkNotification::RecvRpc(*peer, inbound_rpc_req)),
                )
                .unwrap();
                block_on(res_rx).unwrap().ok();

                // remote replies with some response message
                block_on(async {
                    msg.res_tx.send(Ok(res_data)).unwrap();
                });

                // await message delivery
                self.wait_for_event(&peer_id, SharedMempoolNotification::NewTransactions);

                // verify txns were inserted into mempool
                let mempool = self.mempools.get(&peer_id).unwrap();

                let mut mempool_lock = mempool
                    .lock()
                    .expect("[shared mempool test] failed to get mempool lock");
                let block = mempool_lock.get_block(100, HashSet::new());
                for transaction in transactions.iter() {
                    assert!(block.contains(&transaction));
                }
                (transactions, peer_id)
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

// TODO: add test for txns sent within same validator network
#[test]
fn test_basic_flow() {
    let (mut smp, peers) = SharedMempoolNetwork::bootstrap_validator_network_smp(2);
    let (peer_a, peer_b) = (peers.get(0).unwrap(), peers.get(1).unwrap());

    smp.add_txns(
        peer_a,
        vec![
            TestTransaction::new(1, 0, 1),
            TestTransaction::new(1, 1, 1),
            TestTransaction::new(1, 2, 1),
        ],
    );

    // A discovers new peer B
    smp.send_event(&peer_a, NetworkNotification::NewPeer(*peer_b));

    for seq in 0..3 {
        // A attempts to send message
        let (transactions, _) = smp.deliver_message(&peer_a);
        assert_eq!(transactions.len(), 1); // TODO use test config batch size
        let txn = assert_matches::assert_matches!(
            transactions.get(0),
            Some(txn) => txn,
            "expect first SignedTransaction to exist",
        );
        assert_eq!(txn.sequence_number(), seq);
        smp.wait_for_worker_update(peer_a, peer_b);
    }
}

#[test]
fn test_metric_cache_ignore_shared_txns() {
    let (mut smp, peers) = SharedMempoolNetwork::bootstrap_validator_network_smp(2);
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
    smp.send_event(&peer_a, NetworkNotification::NewPeer(*peer_b));
    for txn in txns.iter().take(3) {
        // Let peer_a share txns with peer_b
        let (_transactions, rx_peer) = smp.deliver_message(&peer_a);
        assert_eq!(smp.exist_in_metrics_cache(&rx_peer, txn), false);
    }
}

#[test]
fn test_interruption_in_sync() {
    let (mut smp, peers) = SharedMempoolNetwork::bootstrap_validator_network_smp(3);
    let (peer_a, peer_b, peer_c) = (
        peers.get(0).unwrap(),
        peers.get(1).unwrap(),
        peers.get(2).unwrap(),
    );

    smp.add_txns(&peer_a, vec![TestTransaction::new(1, 0, 1)]);

    // A discovers 2 peers
    smp.send_event(&peer_a, NetworkNotification::NewPeer(*peer_b));
    smp.send_event(&peer_a, NetworkNotification::NewPeer(*peer_c));

    // make sure it delivered first transaction to both nodes
    let mut peers = vec![
        smp.deliver_message(&peer_a).1,
        smp.deliver_message(&peer_a).1,
    ];
    smp.wait_for_worker_updates(&peer_a, vec![*peer_b, *peer_c]);
    peers.sort();
    let mut expected_peers = vec![*peer_b, *peer_c];
    expected_peers.sort();
    assert_eq!(peers, expected_peers);

    // A loses connection to B
    smp.send_event(&peer_a, NetworkNotification::LostPeer(*peer_b));

    let mut results = Vec::new();
    smp.add_txns(&peer_a, vec![TestTransaction::new(1, 1, 1)]);
    results.push(smp.deliver_message(&peer_a));

    smp.add_txns(&peer_a, vec![TestTransaction::new(1, 2, 1)]);
    results.push(smp.deliver_message(&peer_a));
    smp.send_event(&peer_a, NetworkNotification::NewPeer(*peer_b));
    for _ in 0..2 {
        results.push(smp.deliver_message(&peer_a));
    }

    // check for the following possible ordering of transactions in message delivery:
    // C1, C2, B1, B2
    // C1, B2, C2, B2
    // B1, C1, C2, B2
    //
    // there are multiple possible orderings because the worker process for peer B may not be paused
    // immediately (the `PAUSE` message from master may reach worker for peer B between the time the worker
    // checked the channel from master and the time it broadcasts transactions

    let expected_values = vec![
        vec![(peer_c, 1), (peer_c, 2), (peer_b, 1), (peer_b, 2)],
        vec![(peer_c, 1), (peer_b, 1), (peer_c, 2), (peer_b, 2)],
        vec![(peer_b, 1), (peer_c, 1), (peer_b, 2), (peer_c, 2)],
    ];
    let mut matched = false;

    for expected in &expected_values {
        let mut actual_matches_expected = true;
        for i in 0..4 {
            let expected_event = expected[i];
            let actual_event = &results[i];

            actual_matches_expected = (actual_event.1 == *expected_event.0)
                && (actual_event.0.get(0).unwrap().sequence_number() == expected_event.1);
        }
        if actual_matches_expected {
            matched = true;
            break;
        }
    }
    assert!(matched);
}

#[test]
fn test_ready_transactions() {
    let (mut smp, peers) = SharedMempoolNetwork::bootstrap_validator_network_smp(2);
    let (peer_a, peer_b) = (peers.get(0).unwrap(), peers.get(1).unwrap());

    smp.add_txns(
        &peer_a,
        vec![TestTransaction::new(1, 0, 1), TestTransaction::new(1, 2, 1)],
    );
    // first message delivery
    smp.send_event(&peer_a, NetworkNotification::NewPeer(*peer_b));
    smp.deliver_message(&peer_a);
    smp.wait_for_worker_update(peer_a, peer_b);

    // add txn1 to Mempool
    smp.add_txns(&peer_a, vec![TestTransaction::new(1, 1, 1)]);
    // txn1 unlocked txn2. Now all transactions can go through in correct order
    let txns = &smp.deliver_message(&peer_a).0;
    smp.wait_for_worker_update(peer_a, peer_b);
    assert_eq!(txns.len(), 1);
    let txn = assert_matches::assert_matches!(
        txns.get(0),
        Some(txn) => txn,
        "expect first SignedTransaction to exist",
    );
    assert_eq!(txn.sequence_number(), 1);
    let txns = &smp.deliver_message(&peer_a).0;
    smp.wait_for_worker_update(peer_a, peer_b);
    assert_eq!(txns.len(), 1);
    let txn = assert_matches::assert_matches!(
        txns.get(0),
        Some(txn) => txn,
        "expect first SignedTransaction to exist",
    );
    assert_eq!(txn.sequence_number(), 2);
}

#[test]
fn test_broadcast_self_transactions() {
    let (mut smp, peers) = SharedMempoolNetwork::bootstrap_validator_network_smp(2);
    let (peer_a, peer_b) = (peers.get(0).unwrap(), peers.get(1).unwrap());

    smp.add_txns(&peer_a, vec![TestTransaction::new(0, 0, 1)]);

    // A and B discover each other
    smp.send_event(&peer_a, NetworkNotification::NewPeer(*peer_b));
    smp.send_event(&peer_b, NetworkNotification::NewPeer(*peer_a));

    // A sends txn to B
    smp.deliver_message(&peer_a);
    smp.wait_for_worker_update(peer_a, peer_b);

    // add new txn to B
    smp.add_txns(&peer_b, vec![TestTransaction::new(1, 0, 1)]);

    // verify that A will receive only second transaction from B
    let (txns, _) = smp.deliver_message(&peer_b);
    smp.wait_for_worker_update(peer_b, peer_a);
    assert_eq!(txns.len(), 1);
    let txn = assert_matches::assert_matches!(
        txns.get(0),
        Some(txn) => txn,
        "expect first SignedTransaction to exist",
    );
    assert_eq!(txn.sender(), TestTransaction::get_address(1));
}

#[test]
fn test_broadcast_dependencies() {
    let (mut smp, peers) = SharedMempoolNetwork::bootstrap_validator_network_smp(2);
    let (peer_a, peer_b) = (peers.get(0).unwrap(), peers.get(1).unwrap());

    // Peer A has transactions with sequence numbers 0 and 2
    smp.add_txns(
        &peer_a,
        vec![TestTransaction::new(0, 0, 1), TestTransaction::new(0, 2, 1)],
    );
    // Peer B has txn1
    smp.add_txns(&peer_b, vec![TestTransaction::new(0, 1, 1)]);

    // A and B discover each other
    smp.send_event(&peer_a, NetworkNotification::NewPeer(*peer_b));
    smp.send_event(&peer_b, NetworkNotification::NewPeer(*peer_a));

    // B receives 0
    smp.deliver_message(&peer_a);
    smp.wait_for_worker_update(peer_a, peer_b);
    // now B can broadcast 1
    let txns = smp.deliver_message(&peer_b).0;
    smp.wait_for_worker_update(peer_b, peer_a);
    assert_eq!(txns.len(), 1);
    let txn = assert_matches::assert_matches!(
        txns.get(0),
        Some(txn) => txn,
        "expect first SignedTransaction to exist",
    );
    assert_eq!(txn.sequence_number(), 1);

    // now A can broadcast 2
    let txns = smp.deliver_message(&peer_a).0;
    smp.wait_for_worker_update(peer_a, peer_b);
    assert_eq!(txns.len(), 1);
    let txn = assert_matches::assert_matches!(
        txns.get(0),
        Some(txn) => txn,
        "expect first SignedTransaction to exist",
    );
    assert_eq!(txn.sequence_number(), 2);
}

#[test]
fn test_broadcast_updated_transaction() {
    let (mut smp, peers) = SharedMempoolNetwork::bootstrap_validator_network_smp(2);
    let (peer_a, peer_b) = (peers.get(0).unwrap(), peers.get(1).unwrap());

    // Peer A has a transaction with sequence number 0 and gas price 1
    smp.add_txns(&peer_a, vec![TestTransaction::new(0, 0, 1)]);

    // A and B discover each other
    smp.send_event(&peer_a, NetworkNotification::NewPeer(*peer_b));
    smp.send_event(&peer_b, NetworkNotification::NewPeer(*peer_a));

    // B receives 0
    let txns = smp.deliver_message(&peer_a).0;
    smp.wait_for_worker_update(peer_a, peer_b);
    assert_eq!(txns.len(), 1);
    let txn = assert_matches::assert_matches!(
        txns.get(0),
        Some(txn) => txn,
        "expect first SignedTransaction to exist",
    );
    assert_eq!(txn.sequence_number(), 0);
    assert_eq!(txn.gas_unit_price(), 1);

    // Update the gas price of the transaction with sequence 0 after B has already received 0
    smp.add_txns(&peer_a, vec![TestTransaction::new(0, 0, 5)]);

    // trigger send from A to B and check B has updated gas price for sequence 0
    let txns = smp.deliver_message(&peer_a).0;
    smp.wait_for_worker_update(peer_a, peer_b);
    assert_eq!(txns.len(), 1);
    let txn = assert_matches::assert_matches!(
        txns.get(0),
        Some(txn) => txn,
        "expect first SignedTransaction to exist",
    );
    assert_eq!(txn.sequence_number(), 0);
    assert_eq!(txn.gas_unit_price(), 5);
}

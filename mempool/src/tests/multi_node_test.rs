// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    network::MempoolSyncMsg,
    shared_mempool::types::SharedMempoolNotification,
    tests::{
        common::TestTransaction,
        node::{
            public_full_node_config, validator_config, vfn_config, Node, NodeId, NodeInfo,
            NodeInfoTrait, NodeType,
        },
    },
};
use diem_config::config::PeerRole;
use diem_types::{transaction::SignedTransaction, PeerId};
use netcore::transport::ConnectionOrigin;
use network::{
    peer_manager::{PeerManagerNotification, PeerManagerRequest},
    ProtocolId,
};
use rand::{rngs::StdRng, SeedableRng};
use std::collections::{HashMap, HashSet};

/// A test harness representing a combined network of Nodes and the mempool interactions between them
#[derive(Default)]
struct TestHarness {
    nodes: HashMap<NodeId, Node>,
    /// A mapping of `PeerId` to `NodeId`.  Since network messages use `PeerId` this mapping is needed
    peer_to_node_id: HashMap<PeerId, NodeId>,
}

impl TestHarness {
    /// Builds a validator only network for testing the SharedMempool interactions
    fn bootstrap_validator_network(
        validator_nodes_count: u32,
        broadcast_batch_size: usize,
        mempool_size: Option<usize>,
        max_broadcasts_per_peer: Option<usize>,
        max_ack_timeout: bool,
    ) -> (TestHarness, Vec<NodeId>) {
        let (harness, mut peers) = Self::bootstrap_network(
            validator_nodes_count,
            false,
            0,
            broadcast_batch_size,
            mempool_size,
            max_broadcasts_per_peer,
            max_ack_timeout,
        );
        let validators = peers.remove(&PeerRole::Validator).unwrap();
        (harness, validators)
    }

    /// Builds a fully functional network with Validators, attached VFNs, and full nodes
    /// Note: None of these nodes are told about each other, and must manually be done afterwards
    fn bootstrap_network(
        validator_nodes_count: u32,
        vfns_attached: bool,
        other_full_nodes_count: u32,
        broadcast_batch_size: usize,
        mempool_size: Option<usize>,
        max_broadcasts_per_peer: Option<usize>,
        max_ack_timeout: bool,
    ) -> (Self, HashMap<PeerRole, Vec<NodeId>>) {
        let mut harness = Self::default();
        let mut rng = StdRng::from_seed([0u8; 32]);
        let mut peers = HashMap::<PeerRole, Vec<NodeId>>::new();

        // Build up validators
        for idx in 0..validator_nodes_count {
            let (validator, mut v_config) = validator_config(&mut rng, idx);
            v_config.mempool.shared_mempool_batch_size = broadcast_batch_size;
            // Set the ack timeout duration to 0 to avoid sleeping to test rebroadcast scenario (broadcast must timeout for this).
            v_config.mempool.shared_mempool_ack_timeout_ms =
                if max_ack_timeout { u64::MAX } else { 0 };
            v_config.mempool.capacity = mempool_size.unwrap_or(v_config.mempool.capacity);
            v_config.mempool.max_broadcasts_per_peer =
                max_broadcasts_per_peer.unwrap_or(v_config.mempool.max_broadcasts_per_peer);

            let node_id = NodeId::new(NodeType::Validator, idx);
            harness
                .peer_to_node_id
                .insert(validator.primary_peer_id(), node_id);
            let validator_node = NodeInfo::Validator(validator);
            harness
                .nodes
                .insert(node_id, Node::new(validator_node, v_config));
            peers.entry(PeerRole::Validator).or_default().push(node_id);

            // Build up VFNs if we've determined we want those too
            if vfns_attached {
                let (vfn, mut vfn_config) = vfn_config(&mut rng, idx);
                vfn_config.mempool.shared_mempool_batch_size = broadcast_batch_size;
                vfn_config.mempool.shared_mempool_backoff_interval_ms = 50;
                // Set the ack timeout duration to 0 to avoid sleeping to test rebroadcast scenario (broadcast must timeout for this).
                vfn_config.mempool.shared_mempool_ack_timeout_ms =
                    if max_ack_timeout { u64::MAX } else { 0 };
                vfn_config.mempool.max_broadcasts_per_peer =
                    max_broadcasts_per_peer.unwrap_or(vfn_config.mempool.max_broadcasts_per_peer);
                let node_id = NodeId::new(NodeType::ValidatorFullNode, idx);
                harness
                    .peer_to_node_id
                    .insert(vfn.primary_peer_id(), node_id);
                harness
                    .peer_to_node_id
                    .insert(vfn.secondary_peer_id().unwrap(), node_id);
                let vfn_node = NodeInfo::ValidatorFull(vfn);
                harness
                    .nodes
                    .insert(node_id, Node::new(vfn_node, vfn_config));
                peers
                    .entry(PeerRole::ValidatorFullNode)
                    .or_default()
                    .push(node_id);
            }
        }

        // Create any additional full nodes
        for idx in validator_nodes_count
            ..other_full_nodes_count
                .checked_add(validator_nodes_count)
                .unwrap()
        {
            let (full_node, mut fn_config) =
                public_full_node_config(&mut rng, idx, PeerRole::Unknown);
            fn_config.mempool.shared_mempool_batch_size = broadcast_batch_size;
            // Set the ack timeout duration to 0 to avoid sleeping to test rebroadcast scenario (broadcast must timeout for this).
            fn_config.mempool.shared_mempool_ack_timeout_ms =
                if max_ack_timeout { u64::MAX } else { 0 };
            fn_config.mempool.capacity = mempool_size.unwrap_or(fn_config.mempool.capacity);
            fn_config.mempool.max_broadcasts_per_peer =
                max_broadcasts_per_peer.unwrap_or(fn_config.mempool.max_broadcasts_per_peer);

            let node_id = NodeId::new(NodeType::FullNode, idx);
            harness
                .peer_to_node_id
                .insert(full_node.primary_peer_id(), node_id);
            let full_node = NodeInfo::Full(full_node);
            harness
                .nodes
                .insert(node_id, Node::new(full_node, fn_config));
            peers.entry(PeerRole::Unknown).or_default().push(node_id)
        }

        (harness, peers)
    }

    fn node(&self, node_id: &NodeId) -> &Node {
        self.nodes.get(node_id).unwrap()
    }

    fn mut_node(&mut self, node_id: &NodeId) -> &mut Node {
        self.nodes.get_mut(node_id).unwrap()
    }

    /// Queues transactions for sending on a node.  Must use `broadcast_txns` to send to other nodes
    fn add_txns(&self, node_id: &NodeId, txns: Vec<TestTransaction>) {
        self.node(node_id).add_txns(txns)
    }

    /// Commits transactions and removes them from the local mempool, stops them from being broadcasted later
    fn commit_txns(&self, node_id: &NodeId, txns: Vec<TestTransaction>) {
        self.node(node_id).commit_txns(txns)
    }

    /// Connect two nodes, A -> B, direction is important
    fn connect_a_to_b(&mut self, node_a: &NodeId, node_b: &NodeId) {
        self.connect_a_to_b_with_networks(node_a, true, node_b, true);
    }

    /// Connect two nodes on specific interfaces
    fn connect_a_to_b_with_networks(
        &mut self,
        node_a_id: &NodeId,
        is_primary_a: bool,
        node_b_id: &NodeId,
        is_primary_b: bool,
    ) {
        // Tell B about A
        let node_a = self.node(node_a_id);
        let id_a = node_a.peer_id(is_primary_a);
        let role_a = node_a.peer_role();
        let node_b = self.mut_node(node_b_id);

        node_b.send_new_peer_event(is_primary_b, id_a, role_a, ConnectionOrigin::Inbound);

        // Tell A about B
        let node_b = self.node(node_b_id);
        let id_b = node_b.peer_id(is_primary_b);
        let role_b = node_b.peer_role();
        let node_a = self.mut_node(node_a_id);
        node_a.send_new_peer_event(is_primary_a, id_b, role_b, ConnectionOrigin::Outbound);
    }

    /// Disconnect two nodes
    fn disconnect(
        &mut self,
        node_a_id: &NodeId,
        is_primary_a: bool,
        node_b_id: &NodeId,
        is_primary_b: bool,
    ) {
        // Tell B about A
        let node_a = self.node(node_a_id);
        let id_a = node_a.peer_id(is_primary_a);
        let node_b = self.mut_node(node_b_id);
        node_b.send_lost_peer_event(is_primary_b, id_a);

        // Tell A about B
        let node_b = self.node(node_b_id);
        let id_b = node_b.peer_id(is_primary_b);
        let node_a = self.mut_node(node_a_id);
        node_a.send_lost_peer_event(is_primary_a, id_b);
    }

    /// Blocks, expecting the next event to be the type provided
    fn wait_for_event(&mut self, node_id: &NodeId, expected: SharedMempoolNotification) {
        self.mut_node(node_id).wait_for_event(expected);
    }

    fn check_no_events(&mut self, node_id: &NodeId) {
        self.mut_node(node_id).check_no_subscriber_events();
    }

    /// Checks that a node has no pending messages to send.
    fn assert_no_message_sent(&mut self, node_id: &NodeId, is_primary: bool) {
        self.check_no_events(node_id);
        self.mut_node(node_id)
            .check_no_network_messages_sent(is_primary);
    }

    /// Convenience function to get rid of the string of true falses
    fn broadcast_txns_successfully(
        &mut self,
        sender: &NodeId,
        is_primary: bool,
        num_messages: usize,
    ) -> (Vec<SignedTransaction>, PeerId) {
        self.broadcast_txns(sender, is_primary, num_messages, true, true, false)
    }

    /// Broadcast Transactions queued up in the local mempool of the sender
    fn broadcast_txns(
        &mut self,
        sender_id: &NodeId,
        sender_is_primary: bool,
        num_messages: usize,
        check_txns_in_mempool: bool, // Check whether all txns in this broadcast are accepted into recipient's mempool
        execute_send: bool, // If true, actually delivers msg to remote peer; else, drop the message (useful for testing unreliable msg delivery)
        drop_ack: bool,     // If true, drop ack from remote peer to this peer
    ) -> (Vec<SignedTransaction>, PeerId) {
        // Await broadcast notification
        // Note: If there are other messages you're looking for, this could throw them away
        // Wait for the number of messages to be broadcasted on this node
        for _ in 0..num_messages {
            self.wait_for_event(sender_id, SharedMempoolNotification::Broadcast);
        }

        // Get the outgoing network request on the sender
        let sender = self.mut_node(sender_id);
        let sender_peer_id = sender.peer_id(sender_is_primary);
        let network_req = sender.get_next_network_req(sender_is_primary);

        // Handle outgoing message
        match network_req {
            PeerManagerRequest::SendDirectSend(remote_peer_id, msg) => {
                let decoded_msg = bcs::from_bytes(&msg.mdata).unwrap();
                match decoded_msg {
                    MempoolSyncMsg::BroadcastTransactionsRequest {
                        transactions,
                        request_id: _request_id,
                    } => {
                        // If we don't want to forward the request, let's just drop it
                        if !execute_send {
                            return (transactions, remote_peer_id);
                        }

                        // Otherwise, let's forward it
                        let receiver_id = *self.peer_to_node_id.get(&remote_peer_id).unwrap();

                        let receiver = self.mut_node(&receiver_id);
                        let is_primary = receiver.primary_peer_id() == remote_peer_id;

                        receiver.send_network_req(
                            is_primary,
                            ProtocolId::MempoolDirectSend,
                            PeerManagerNotification::RecvMessage(sender_peer_id, msg),
                        );
                        receiver.wait_for_event(SharedMempoolNotification::NewTransactions);

                        // Verify transaction was inserted into Mempool
                        if check_txns_in_mempool {
                            let block = self
                                .node(sender_id)
                                .mempool()
                                .get_block(100, HashSet::new());
                            for txn in transactions.iter() {
                                assert!(block.contains(txn));
                            }
                        }

                        // Sends an ACK response
                        if !drop_ack {
                            self.deliver_response(&receiver_id, is_primary);
                        }
                        (transactions, remote_peer_id)
                    }
                    req => {
                        panic!("Unexpected broadcast transactions response {:?}", req)
                    }
                }
            }
            req => {
                panic!(
                    "Unexpected peer manager request, didn't receive broadcast {:?}",
                    req
                )
            }
        }
    }

    /// Convenience function, broadcasts transactions, and makes sure they got to the right place,
    /// and received the right sequence number
    fn broadcast_txns_and_validate(
        &mut self,
        sender_id: &NodeId,
        receiver_id: &NodeId,
        seq_num: u64,
    ) {
        self.broadcast_txns_and_validate_with_networks(sender_id, true, receiver_id, true, seq_num)
    }

    fn broadcast_txns_and_validate_with_networks(
        &mut self,
        sender_id: &NodeId,
        is_primary_sender: bool,
        receiver_id: &NodeId,
        is_primary_receiver: bool,
        seq_num: u64,
    ) {
        let (txns, rx_peer) = self.broadcast_txns_successfully(&sender_id, is_primary_sender, 1);
        assert_eq!(1, txns.len(), "Expected only one transaction");
        let actual_seq_num = txns.get(0).unwrap().sequence_number();
        assert_eq!(
            seq_num, actual_seq_num,
            "Expected seq_num {}, got {}",
            seq_num, actual_seq_num
        );
        let receiver = self.node(receiver_id);
        let expected_peer_id = receiver.peer_id(is_primary_receiver);
        assert_eq!(
            expected_peer_id, rx_peer,
            "Expected peer {} to receive message, but {} got it instead",
            expected_peer_id, rx_peer
        );
    }

    /// Delivers broadcast ACK from `peer`.
    fn deliver_response(&mut self, sender_id: &NodeId, sender_is_primary: bool) {
        // Wait for an ACK to come in on the events
        self.wait_for_event(sender_id, SharedMempoolNotification::ACK);
        let sender = self.mut_node(sender_id);
        let sender_peer_id = sender.peer_id(sender_is_primary);
        let network_req = sender.get_next_network_req(sender_is_primary);

        match network_req {
            PeerManagerRequest::SendDirectSend(remote_peer_id, msg) => {
                let decoded_msg = bcs::from_bytes(&msg.mdata).unwrap();
                match decoded_msg {
                    MempoolSyncMsg::BroadcastTransactionsResponse { .. } => {
                        // send it to peer
                        let receiver_id = *self.peer_to_node_id.get(&remote_peer_id).unwrap();

                        let receiver = self.mut_node(&receiver_id);
                        let is_primary = receiver.primary_peer_id() == remote_peer_id;
                        receiver.send_network_req(
                            is_primary,
                            ProtocolId::MempoolDirectSend,
                            PeerManagerNotification::RecvMessage(sender_peer_id, msg),
                        );
                    }
                    request => panic!(
                        "did not receive expected broadcast ACK, instead got {:?}",
                        request
                    ),
                }
            }
            request => panic!("Node did not ACK broadcast, instead got {:?}", request),
        }
    }

    /// Check if a transaction made it into the metrics cache
    fn exist_in_metrics_cache(&self, node_id: &NodeId, txn: &TestTransaction) -> bool {
        self.node(node_id)
            .mempool()
            .metrics_cache
            .get(&(
                TestTransaction::get_address(txn.address),
                txn.sequence_number,
            ))
            .is_some()
    }
}

fn test_transactions(start: u64, num: u64) -> Vec<TestTransaction> {
    let mut txns = vec![];
    for seq_num in start..start.checked_add(num).unwrap() {
        txns.push(test_transaction(seq_num))
    }
    txns
}

fn test_transaction(seq_num: u64) -> TestTransaction {
    TestTransaction::new(1, seq_num, 1)
}

#[test]
fn test_basic_flow() {
    let (mut harness, validators) =
        TestHarness::bootstrap_validator_network(2, 1, None, None, false);
    let (v_a, v_b) = (validators.get(0).unwrap(), validators.get(1).unwrap());

    // Add transactions to send
    harness.add_txns(&v_a, test_transactions(0, 3));

    // A discovers new peer B
    harness.connect_a_to_b(&v_b, &v_a);

    // A sends messages, which are received by B
    for seq_num in 0..3 {
        harness.broadcast_txns_and_validate(v_a, v_b, seq_num);
    }
}

#[test]
fn test_metric_cache_ignore_shared_txns() {
    let (mut harness, validators) =
        TestHarness::bootstrap_validator_network(2, 1, None, None, false);
    let (v_a, v_b) = (validators.get(0).unwrap(), validators.get(1).unwrap());

    let txns = test_transactions(0, 3);
    harness.add_txns(v_a, test_transactions(0, 3));
    // Check if txns's creation timestamp exist in peer_a's metrics_cache.
    assert_eq!(
        harness.exist_in_metrics_cache(v_a, &test_transaction(0)),
        true
    );
    assert_eq!(
        harness.exist_in_metrics_cache(v_a, &test_transaction(1)),
        true
    );
    assert_eq!(
        harness.exist_in_metrics_cache(v_a, &test_transaction(2)),
        true
    );

    // Connect B to A incoming
    harness.connect_a_to_b(&v_b, &v_a);

    // TODO: Why not use the information that comes back from the broadcast?
    for txn in txns.iter().take(3) {
        // Let peer_a share txns with peer_b
        let _ = harness.broadcast_txns_successfully(v_a, true, 1);
        // Check if txns's creation timestamp exist in peer_b's metrics_cache.
        assert_eq!(harness.exist_in_metrics_cache(v_b, txn), false);
    }
}

#[test]
fn test_interruption_in_sync() {
    let (mut harness, validators) =
        TestHarness::bootstrap_validator_network(3, 1, None, None, false);
    let (v_a, v_b, v_c) = (
        validators.get(0).unwrap(),
        validators.get(1).unwrap(),
        validators.get(2).unwrap(),
    );

    harness.add_txns(v_a, vec![test_transaction(0)]);

    // A discovers first peer
    harness.connect_a_to_b(&v_b, &v_a);

    // Make sure first txn delivered to first peer
    harness.broadcast_txns_and_validate(v_a, v_b, 0);

    // A discovers second peer
    harness.connect_a_to_b(&v_c, &v_a);

    // Make sure first txn delivered to second peer
    harness.broadcast_txns_and_validate(v_a, v_c, 0);

    // A loses connection to B
    harness.disconnect(&v_a, true, &v_b, true);

    // Only C receives the following transactions
    for seq_num in 1..3 {
        harness.add_txns(v_a, vec![test_transaction(seq_num)]);
        harness.broadcast_txns_and_validate(v_a, v_c, seq_num);
    }

    // B reconnects to A
    harness.connect_a_to_b(&v_b, &v_a);

    // B should receive the remaining txns
    for seq_num in 1..3 {
        harness.broadcast_txns_and_validate(v_a, v_b, seq_num);
    }
}

#[test]
fn test_ready_transactions() {
    let (mut harness, validators) =
        TestHarness::bootstrap_validator_network(2, 1, None, None, false);
    let (v_a, v_b) = (validators.get(0).unwrap(), validators.get(1).unwrap());

    harness.add_txns(v_a, vec![test_transaction(0), test_transaction(2)]);

    // First message delivery
    harness.connect_a_to_b(v_b, v_a);
    harness.broadcast_txns_and_validate(v_a, v_b, 0);

    // Add txn1 to mempool
    harness.add_txns(v_a, vec![test_transaction(1)]);
    // txn1 unlocked txn2. Now all transactions can go through in correct order
    harness.broadcast_txns_and_validate(v_a, v_b, 1);
    harness.broadcast_txns_and_validate(v_a, v_b, 2);
}

#[test]
fn test_broadcast_self_transactions() {
    let (mut harness, validators) =
        TestHarness::bootstrap_validator_network(2, 1, None, None, false);
    let (v_a, v_b) = (validators.get(0).unwrap(), validators.get(1).unwrap());
    harness.add_txns(&v_a, vec![test_transaction(0)]);

    // A and B discover each other
    harness.connect_a_to_b(v_b, v_a);

    // A sends txn to B
    harness.broadcast_txns_successfully(v_a, true, 1);

    // Add new txn to B
    harness.add_txns(v_b, vec![TestTransaction::new(2, 0, 1)]);

    // Verify that A will receive only second transaction from B
    let (txn, _) = harness.broadcast_txns_successfully(v_b, true, 1);
    assert_eq!(
        txn.get(0).unwrap().sender(),
        TestTransaction::get_address(2)
    );
}

#[test]
fn test_broadcast_dependencies() {
    let (mut harness, validators) =
        TestHarness::bootstrap_validator_network(2, 1, None, None, false);
    let (v_a, v_b) = (validators.get(0).unwrap(), validators.get(1).unwrap());

    // Peer A has transactions with sequence numbers 0 and 2
    harness.add_txns(v_a, vec![test_transaction(0), test_transaction(2)]);

    // Peer B has txn1
    harness.add_txns(v_b, vec![test_transaction(1)]);

    // A and B discover each other
    harness.connect_a_to_b(v_b, v_a);

    // B receives 0
    harness.broadcast_txns_and_validate(v_a, v_b, 0);
    // Now B can broadcast 1
    harness.broadcast_txns_and_validate(v_b, v_a, 1);
    // Now A can broadcast 2
    harness.broadcast_txns_and_validate(v_a, v_b, 2);
}

#[test]
fn test_broadcast_updated_transaction() {
    let (mut harness, validators) =
        TestHarness::bootstrap_validator_network(2, 1, None, None, false);
    let (v_a, v_b) = (validators.get(0).unwrap(), validators.get(1).unwrap());

    // Peer A has a transaction with sequence number 0 and gas price 1
    harness.add_txns(v_a, vec![test_transaction(0)]);

    // A and B discover each other
    harness.connect_a_to_b(v_b, v_a);

    // B receives 0
    let (txn, _) = harness.broadcast_txns_successfully(&v_a, true, 1);
    assert_eq!(txn.get(0).unwrap().sequence_number(), 0);
    assert_eq!(txn.get(0).unwrap().gas_unit_price(), 1);

    // Update the gas price of the transaction with sequence 0 after B has already received 0
    harness.add_txns(v_a, vec![TestTransaction::new(1, 0, 5)]);

    // Trigger send from A to B and check B has updated gas price for sequence 0
    let (txn, _) = harness.broadcast_txns_successfully(v_a, true, 1);
    assert_eq!(txn.get(0).unwrap().sequence_number(), 0);
    assert_eq!(txn.get(0).unwrap().gas_unit_price(), 5);
}

// Tests VFN properly identifying upstream peers in a network with both upstream and downstream peers
#[test]
fn test_vfn_multi_network() {
    let (mut harness, peers) = TestHarness::bootstrap_network(2, true, 1, 1, None, None, false);
    let validators = peers.get(&PeerRole::Validator).unwrap();
    let v_a = validators.get(0).unwrap();

    let vfns = peers.get(&PeerRole::ValidatorFullNode).unwrap();
    let (vfn_a, vfn_b) = (vfns.get(0).unwrap(), vfns.get(1).unwrap());

    let pfn = peers.get(&PeerRole::Unknown).unwrap().get(0).unwrap();

    // Make a Chain PFN -> VFN A -> VFN B (-> upstream)
    // VFN A discovers pfn as Inbound
    harness.connect_a_to_b_with_networks(pfn, true, vfn_a, false);
    // VFN B discovers VFN A as Inbound
    harness.connect_a_to_b_with_networks(vfn_a, false, vfn_b, false);

    // Also add Validator chain PFN -> VFN A -> V A
    harness.connect_a_to_b(vfn_a, v_a);

    // Add txn to VFN A
    harness.add_txns(vfn_a, vec![test_transaction(0)]);

    // VFN A should broadcast to upstream
    harness.broadcast_txns_and_validate(vfn_a, v_a, 0);

    // VFN A should additionally broadcast to failover upstream vfn in public network
    harness.broadcast_txns_and_validate_with_networks(vfn_a, false, vfn_b, false, 0);

    // Check no other mesages sent
    harness.assert_no_message_sent(&vfn_a, true);
    harness.assert_no_message_sent(&vfn_a, false);
}

/// The purpose of this test is to set up a network with 1 Validators, 1 VFNs, and 1 PFN.
/// VFN1 is the subject of fallback testing, and should fallback to the PFN.
#[test]
fn test_failover() {
    let (mut harness, peers) = TestHarness::bootstrap_network(1, true, 1, 1, None, None, false);
    let validators = peers.get(&PeerRole::Validator).unwrap();
    let v = validators.get(0).unwrap();

    let vfns = peers.get(&PeerRole::ValidatorFullNode).unwrap();
    let vfn = vfns.get(0).unwrap();
    let pfn = peers.get(&PeerRole::Unknown).unwrap().get(0).unwrap();

    // VFNs discover primary and fallback upstream peers
    harness.connect_a_to_b(vfn, v);
    harness.connect_a_to_b_with_networks(vfn, false, pfn, true);

    // Send txn to VFN1
    harness.add_txns(vfn, vec![test_transaction(0)]);

    // Make sure it delivers txn to primary peer and fallbacks
    harness.broadcast_txns_and_validate(vfn, v, 0);
    harness.broadcast_txns_and_validate_with_networks(vfn, false, pfn, true, 0);

    // Bring validator down
    harness.disconnect(vfn, true, v, true);

    // Send another txn to vfn1 now that the primary is down
    harness.add_txns(vfn, vec![test_transaction(1)]);

    // Messages still go to fallback, but don't make it to the primary
    // Note: these two statements MUST be in this order, due to complications of the simulator
    // TODO: This test will continue to be flaky here until some changes are made in mempool
    // Basically, the original transaction continues to be sent out, because it's not ack'd by the
    // primary upstream.
    harness.broadcast_txns_and_validate_with_networks(vfn, false, pfn, true, 1);
    harness.assert_no_message_sent(&vfn, true);
}

#[test]
fn test_rebroadcast_mempool_is_full() {
    let (mut harness, peers) = TestHarness::bootstrap_network(1, true, 0, 3, Some(5), None, false);
    let val = peers.get(&PeerRole::Validator).unwrap().get(0).unwrap();
    let vfn = peers
        .get(&PeerRole::ValidatorFullNode)
        .unwrap()
        .get(0)
        .unwrap();
    let all_txns = test_transactions(0, 8);

    harness.add_txns(vfn, all_txns.clone());

    // VFN connects to Val
    harness.connect_a_to_b(vfn, val);

    // We should get all three txns in a batch
    let (txns, _) = harness.broadcast_txns_successfully(vfn, true, 1);
    let seq_nums = txns
        .iter()
        .map(|txn| txn.sequence_number())
        .collect::<Vec<_>>();
    assert_eq!(vec![0, 1, 2], seq_nums);

    // We continue getting broadcasts because we haven't committed the txns
    for _ in 0..2 {
        let (txns, _) = harness.broadcast_txns(vfn, true, 1, false, true, false);
        let seq_nums = txns
            .iter()
            .map(|txn| txn.sequence_number())
            .collect::<Vec<_>>();
        assert_eq!(vec![3, 4, 5], seq_nums);
    }

    // Test getting out of rebroadcasting mode: checking we can move past rebroadcasting after receiving non-retry ACK.
    // Remove some txns, which should free space for more
    harness.commit_txns(val, all_txns[..1].to_vec());

    // Send retry batch again, this time it should be processed
    harness.broadcast_txns_successfully(vfn, true, 1);

    // Retry batch sent above should be processed successfully, and FN should move on to broadcasting later txns
    let (txns, _) = harness.broadcast_txns(vfn, true, 1, false, true, false);
    let seq_nums = txns
        .iter()
        .map(|txn| txn.sequence_number())
        .collect::<Vec<_>>();
    assert_eq!(vec![6, 7], seq_nums);
}

#[test]
fn test_rebroadcast_missing_ack() {
    let (mut harness, validators) =
        TestHarness::bootstrap_validator_network(2, 1, None, None, false);
    let (v_a, v_b) = (validators.get(0).unwrap(), validators.get(1).unwrap());
    let pool_txns = test_transactions(0, 3);
    harness.add_txns(v_a, pool_txns);

    // A and B discover each other
    harness.connect_a_to_b(v_b, v_a);

    // Test that txn broadcasts that don't receive an ACK, A rebroadcasts the unACK'ed batch of txns
    for _ in 0..3 {
        let (txns, _) = harness.broadcast_txns(v_a, true, 1, true, false, false);
        assert_eq!(0, txns.get(0).unwrap().sequence_number());
    }

    // Getting out of rebroadcasting mode scenario 1: B sends back ACK eventually
    let (txns, _) = harness.broadcast_txns(v_a, true, 1, true, true, false);
    assert_eq!(0, txns.get(0).unwrap().sequence_number());

    for _ in 0..3 {
        let (txns, _) = harness.broadcast_txns(v_a, true, 1, true, false, false);
        assert_eq!(1, txns.get(0).unwrap().sequence_number());
    }

    // Getting out of rebroadcasting mode scenario 2: txns in unACK'ed batch gets committed
    harness.commit_txns(v_a, vec![test_transaction(1)]);

    let (txns, _) = harness.broadcast_txns(v_a, true, 1, true, false, false);
    assert_eq!(2, txns.get(0).unwrap().sequence_number());
}

#[test]
fn test_max_broadcast_limit() {
    let (mut harness, validators) =
        TestHarness::bootstrap_validator_network(2, 1, None, Some(3), true);
    let (v_a, v_b) = (validators.get(0).unwrap(), validators.get(1).unwrap());

    let pool_txns = test_transactions(0, 6);
    harness.add_txns(v_a, pool_txns);

    // A and B discover each other
    harness.connect_a_to_b(v_b, v_a);

    // Test that for mempool broadcasts txns up till max broadcast, even if they are not ACK'ed
    let (txns, _) = harness.broadcast_txns(v_a, true, 1, true, true, true);
    assert_eq!(0, txns.get(0).unwrap().sequence_number());

    for seq_num in 1..3 {
        let (txns, _) = harness.broadcast_txns(v_a, true, 1, true, false, false);
        assert_eq!(seq_num, txns.get(0).unwrap().sequence_number());
    }

    // Check that mempool doesn't broadcast more than max_broadcasts_per_peer, even
    // if there are more txns in mempool.
    for _ in 0..10 {
        harness.assert_no_message_sent(&v_a, true);
    }

    // Deliver ACK from B to A.
    // This should unblock A to send more broadcasts.
    harness.deliver_response(&v_b, true);
    let (txns, _) = harness.broadcast_txns(&v_a, true, 1, false, true, true);
    assert_eq!(3, txns.get(0).unwrap().sequence_number());

    // Check that mempool doesn't broadcast more than max_broadcasts_per_peer, even
    // if there are more txns in mempool.
    for _ in 0..10 {
        harness.assert_no_message_sent(&v_a, true);
    }
}

// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        common::Author,
        consensus_types::{
            block::Block, proposal_info::ProposalInfo, quorum_cert::QuorumCert, sync_info::SyncInfo,
        },
        network::{BlockRetrievalResponse, ConsensusNetworkImpl, NetworkReceivers},
        safety::vote_msg::VoteMsg,
        test_utils::{consensus_runtime, placeholder_ledger_info},
    },
    state_replication::ExecutedState,
};
use channel;
use crypto::{signing::generate_keypair, HashValue};
use futures::{channel::mpsc, executor::block_on, FutureExt, SinkExt, StreamExt, TryFutureExt};
use network::{
    interface::{NetworkNotification, NetworkRequest},
    proto::{BlockRetrievalStatus, ConsensusMsg, QuorumCert as ProtoQuorumCert, RequestChunk},
    protocols::rpc::InboundRpcRequest,
    validator_network::{ConsensusNetworkEvents, ConsensusNetworkSender},
};
use nextgen_crypto::ed25519::*;
use proto_conv::FromProto;
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex, RwLock},
    time::Duration,
};
use tokio::runtime::TaskExecutor;
use types::{
    account_address::AccountAddress,
    proto::ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    test_helpers::transaction_test_helpers::get_test_signed_txn,
    transaction::{SignedTransaction, TransactionInfo, TransactionListWithProof},
    validator_signer::ValidatorSigner,
    validator_verifier::ValidatorVerifier,
};

/// `NetworkPlayground` mocks the network implementation and provides convenience
/// methods for testing. Test clients can use `wait_for_messages` or
/// `deliver_messages` to inspect the direct-send messages sent between peers.
/// They can also configure network messages to be dropped between specific peers.
///
/// Currently, RPC messages are delivered immediately and are not controlled by
/// `wait_for_messages` or `deliver_messages` for delivery. They are also not
/// currently dropped according to the `NetworkPlayground`'s drop config.
pub struct NetworkPlayground {
    /// Maps each Author to a Sender of their inbound network notifications.
    /// These events will usually be handled by the event loop spawned in
    /// `ConsensusNetworkImpl`.
    node_consensus_txs: Arc<Mutex<HashMap<Author, channel::Sender<NetworkNotification>>>>,
    /// Nodes' outbound handlers forward their outbound non-rpc messages to this
    /// queue.
    outbound_msgs_tx: mpsc::Sender<(Author, NetworkRequest)>,
    /// NetworkPlayground reads all nodes' outbound messages through this queue.
    outbound_msgs_rx: mpsc::Receiver<(Author, NetworkRequest)>,
    /// Allow test code to drop direct-send messages between peers.
    drop_config: Arc<RwLock<DropConfig>>,
    /// An executor for spawning node outbound network event handlers
    executor: TaskExecutor,
}

impl NetworkPlayground {
    pub fn new(executor: TaskExecutor) -> Self {
        let (outbound_msgs_tx, outbound_msgs_rx) = mpsc::channel(1_024);

        NetworkPlayground {
            node_consensus_txs: Arc::new(Mutex::new(HashMap::new())),
            outbound_msgs_tx,
            outbound_msgs_rx,
            drop_config: Arc::new(RwLock::new(DropConfig(HashMap::new()))),
            executor,
        }
    }

    /// Create a new async task that handles outbound messages sent by a node.
    ///
    /// All non-rpc messages are forwarded to the NetworkPlayground's
    /// `outbound_msgs_rx` queue, which controls delivery through the
    /// `deliver_messages` and `wait_for_messages` API's.
    ///
    /// Rpc messages are immediately sent to the destination for handling, so
    /// they don't block.
    async fn start_node_outbound_handler(
        drop_config: Arc<RwLock<DropConfig>>,
        src: Author,
        mut network_reqs_rx: channel::Receiver<NetworkRequest>,
        mut outbound_msgs_tx: mpsc::Sender<(Author, NetworkRequest)>,
        node_consensus_txs: Arc<Mutex<HashMap<Author, channel::Sender<NetworkNotification>>>>,
    ) {
        while let Some(net_req) = network_reqs_rx.next().await {
            let drop_rpc = drop_config
                .read()
                .unwrap()
                .is_message_dropped(&src, &net_req);
            match net_req {
                // Immediately forward rpc requests for handling. Unfortunately,
                // we can't handle rpc requests in `deliver_messages` due to
                // blocking issues, e.g., I want to write:
                // ```
                // let block = sender.request_block(peer_id, block_id).await.unwrap();
                // playground.wait_for_messages(1).await;
                // ```
                // but because the rpc call blocks and depends on the message
                // delivery, we'd have to spawn the sending behaviour on a
                // separate task, which is inconvenient.
                NetworkRequest::SendRpc(dst, outbound_req) => {
                    if drop_rpc {
                        continue;
                    }
                    let mut node_consensus_tx = node_consensus_txs
                        .lock()
                        .unwrap()
                        .get(&dst.into())
                        .unwrap()
                        .clone();

                    let inbound_req = InboundRpcRequest {
                        protocol: outbound_req.protocol,
                        data: outbound_req.data,
                        res_tx: outbound_req.res_tx,
                    };

                    node_consensus_tx
                        .send(NetworkNotification::RecvRpc(src.into(), inbound_req))
                        .await
                        .unwrap();
                }
                // Other NetworkRequest get buffered for `deliver_messages` to
                // synchronously drain.
                net_req => {
                    let _ = outbound_msgs_tx.send((src, net_req)).await;
                }
            }
        }
    }

    /// Add a new node to the NetworkPlayground.
    pub fn add_node(
        &mut self,
        author: Author,
        // The `Sender` of inbound network events. The `Receiver` end of this
        // queue is usually wrapped in a `ConsensusNetworkEvents` adapter.
        consensus_tx: channel::Sender<NetworkNotification>,
        // The `Receiver` of outbound network events this node sends. The
        // `Sender` side of this queue is usually wrapped in a
        // `ConsensusNetworkSender` adapter.
        network_reqs_rx: channel::Receiver<NetworkRequest>,
    ) {
        self.node_consensus_txs
            .lock()
            .unwrap()
            .insert(author, consensus_tx);
        self.drop_config.write().unwrap().add_node(author);

        let fut = NetworkPlayground::start_node_outbound_handler(
            Arc::clone(&self.drop_config),
            author,
            network_reqs_rx,
            self.outbound_msgs_tx.clone(),
            self.node_consensus_txs.clone(),
        );
        self.executor.spawn(fut.boxed().unit_error().compat());
    }

    /// Deliver a `NetworkRequest` from peer `src` to the destination peer.
    /// Returns a copy of the delivered message and the sending peer id.
    async fn deliver_message(
        &mut self,
        src: Author,
        msg: NetworkRequest,
    ) -> (Author, ConsensusMsg) {
        // extract destination peer
        let dst = match &msg {
            NetworkRequest::SendMessage(dst, _) => *dst,
            msg => panic!("[network playground] Unexpected NetworkRequest: {:?}", msg),
        };

        // get his sender
        let mut node_consensus_tx = self
            .node_consensus_txs
            .lock()
            .unwrap()
            .get(&dst.into())
            .unwrap()
            .clone();

        // convert NetworkRequest to corresponding NetworkNotification
        let msg_notif = match msg {
            NetworkRequest::SendMessage(_dst, msg) => {
                NetworkNotification::RecvMessage(src.into(), msg)
            }
            msg => panic!("[network playground] Unexpected NetworkRequest: {:?}", msg),
        };

        // copy message data
        let msg_copy = match &msg_notif {
            NetworkNotification::RecvMessage(src, msg) => {
                let msg: ConsensusMsg = ::protobuf::parse_from_bytes(msg.mdata.as_ref()).unwrap();
                ((*src).into(), msg)
            }
            msg_notif => panic!(
                "[network playground] Unexpected NetworkNotification: {:?}",
                msg_notif
            ),
        };

        node_consensus_tx.send(msg_notif).await.unwrap();
        msg_copy
    }

    /// Wait for exactly `num_messages` to be enqueued and delivered. Return a
    /// copy of all messages for verification.
    /// While all the sent messages are delivered, only the messages that satisfy the given
    /// msg inspector are counted.
    pub async fn wait_for_messages<F>(
        &mut self,
        num_messages: usize,
        msg_inspector: F,
    ) -> Vec<(Author, ConsensusMsg)>
    where
        F: Fn(&(Author, ConsensusMsg)) -> bool,
    {
        let mut msg_copies = vec![];
        while msg_copies.len() < num_messages {
            // Take the next queued message
            let (src, net_req) = self.outbound_msgs_rx.next().await
                .expect("[network playground] waiting for messages, but message queue has shutdown unexpectedly");

            // Deliver and copy message it if it's not dropped
            if !self.is_message_dropped(&src, &net_req) {
                let msg_copy = self.deliver_message(src, net_req).await;
                if msg_inspector(&msg_copy) {
                    msg_copies.push(msg_copy);
                }
            }
        }
        assert_eq!(msg_copies.len(), num_messages);
        msg_copies
    }

    /// Returns true for any message
    pub fn take_all(_msg_copy: &(Author, ConsensusMsg)) -> bool {
        true
    }

    /// Returns true for any message other than timeout
    pub fn exclude_timeout_msg(msg_copy: &(Author, ConsensusMsg)) -> bool {
        !msg_copy.1.has_timeout_msg()
    }

    /// Returns true for proposal messages only.
    pub fn proposals_only(msg_copy: &(Author, ConsensusMsg)) -> bool {
        msg_copy.1.has_proposal()
    }

    /// Returns true for vote messages only.
    pub fn votes_only(msg_copy: &(Author, ConsensusMsg)) -> bool {
        msg_copy.1.has_vote()
    }

    /// Returns true for timeout messages only.
    pub fn timeout_msg_only(msg_copy: &(Author, ConsensusMsg)) -> bool {
        msg_copy.1.has_timeout_msg()
    }

    /// Returns true for sync info messages only.
    pub fn sync_info_only(msg_copy: &(Author, ConsensusMsg)) -> bool {
        msg_copy.1.has_sync_info()
    }

    fn is_message_dropped(&self, src: &Author, net_req: &NetworkRequest) -> bool {
        self.drop_config
            .read()
            .unwrap()
            .is_message_dropped(src, net_req)
    }

    pub fn drop_message_for(&mut self, src: &Author, dst: Author) -> bool {
        self.drop_config.write().unwrap().drop_message_for(src, dst)
    }

    pub fn stop_drop_message_for(&mut self, src: &Author, dst: &Author) -> bool {
        self.drop_config
            .write()
            .unwrap()
            .stop_drop_message_for(src, dst)
    }
}

struct DropConfig(HashMap<Author, HashSet<Author>>);

impl DropConfig {
    pub fn is_message_dropped(&self, src: &Author, net_req: &NetworkRequest) -> bool {
        match net_req {
            NetworkRequest::SendMessage(dst, _) => self
                .0
                .get(src.into())
                .unwrap()
                .contains(&Author::from(*dst)),
            NetworkRequest::SendRpc(dst, _) => self
                .0
                .get(src.into())
                .unwrap()
                .contains(&Author::from(*dst)),
            _ => true,
        }
    }

    pub fn drop_message_for(&mut self, src: &Author, dst: Author) -> bool {
        self.0.get_mut(src).unwrap().insert(dst)
    }

    pub fn stop_drop_message_for(&mut self, src: &Author, dst: &Author) -> bool {
        self.0.get_mut(src).unwrap().remove(dst)
    }

    fn add_node(&mut self, src: Author) {
        self.0.insert(src, HashSet::new());
    }
}

#[test]
fn test_network_api() {
    let runtime = consensus_runtime();
    let num_nodes = 5;
    let mut peers = Vec::new();
    let mut receivers: Vec<NetworkReceivers<u64, Author>> = Vec::new();
    let mut playground = NetworkPlayground::new(runtime.executor());
    let mut nodes = Vec::new();
    let mut author_to_public_keys = HashMap::new();
    let mut signers = Vec::new();
    for i in 0..num_nodes {
        let random_validator_signer = ValidatorSigner::random([i as u8; 32]);
        author_to_public_keys.insert(
            random_validator_signer.author(),
            random_validator_signer.public_key(),
        );
        peers.push(random_validator_signer.author());
        signers.push(random_validator_signer);
    }
    let validator = Arc::new(ValidatorVerifier::new(author_to_public_keys));
    for i in 0..num_nodes {
        let (network_reqs_tx, network_reqs_rx) = channel::new_test(8);
        let (consensus_tx, consensus_rx) = channel::new_test(8);
        let network_sender = ConsensusNetworkSender::new(network_reqs_tx);
        let network_events = ConsensusNetworkEvents::new(consensus_rx);

        playground.add_node(peers[i], consensus_tx, network_reqs_rx);
        let mut node = ConsensusNetworkImpl::new(
            peers[i],
            network_sender,
            network_events,
            Arc::new(peers.clone()),
            Arc::clone(&validator),
        );
        receivers.push(node.start(&runtime.executor()));
        nodes.push(node);
    }
    let vote = VoteMsg::new(
        HashValue::random(),
        ExecutedState::state_for_genesis(),
        1,
        HashValue::random(),
        0,
        HashValue::random(),
        0,
        peers[0],
        placeholder_ledger_info(),
        &signers[0],
    );
    let previous_block = Block::make_genesis_block();
    let previous_qc = QuorumCert::certificate_for_genesis();
    let proposal = ProposalInfo {
        proposal: Block::make_block(&previous_block, 0, 1, 0, previous_qc.clone(), &signers[0]),
        proposer_info: signers[0].author(),
        sync_info: SyncInfo::new(previous_qc.clone(), previous_qc.clone(), None),
    };
    block_on(async move {
        nodes[0].send_vote(vote.clone(), peers[2..5].to_vec()).await;
        playground
            .wait_for_messages(3, NetworkPlayground::take_all)
            .await;
        for r in receivers.iter_mut().take(5).skip(2) {
            let v = r.votes.next().await.unwrap();
            assert_eq!(v, vote);
        }
        nodes[4].broadcast_proposal(proposal.clone()).await;
        playground
            .wait_for_messages(4, NetworkPlayground::take_all)
            .await;
        for r in receivers.iter_mut().take(num_nodes - 1) {
            let p = r.proposals.next().await.unwrap();
            assert_eq!(p, proposal);
        }
    });
}

#[test]
fn test_rpc() {
    let runtime = consensus_runtime();
    let num_nodes = 2;
    let mut peers = Arc::new(Vec::new());
    let mut senders = Vec::new();
    let mut receivers: Vec<NetworkReceivers<u64, Author>> = Vec::new();
    let mut playground = NetworkPlayground::new(runtime.executor());
    let mut nodes = Vec::new();
    let mut author_to_public_keys = HashMap::new();
    for i in 0..num_nodes {
        let random_validator_signer = ValidatorSigner::<Ed25519PrivateKey>::random([i as u8; 32]);
        author_to_public_keys.insert(
            random_validator_signer.author(),
            random_validator_signer.public_key(),
        );
        Arc::get_mut(&mut peers)
            .unwrap()
            .push(random_validator_signer.author());
    }
    let validator = Arc::new(ValidatorVerifier::new(author_to_public_keys));
    for i in 0..num_nodes {
        let (network_reqs_tx, network_reqs_rx) = channel::new_test(8);
        let (consensus_tx, consensus_rx) = channel::new_test(8);
        let network_sender = ConsensusNetworkSender::new(network_reqs_tx);
        let network_events = ConsensusNetworkEvents::new(consensus_rx);

        playground.add_node(peers[i], consensus_tx, network_reqs_rx);
        let mut node = ConsensusNetworkImpl::new(
            peers[i],
            network_sender.clone(),
            network_events,
            Arc::clone(&peers),
            Arc::clone(&validator),
        );
        senders.push(network_sender);
        receivers.push(node.start(&runtime.executor()));
        nodes.push(node);
    }
    let receiver_1 = receivers.remove(1);
    let genesis = Arc::new(Block::<u64>::make_genesis_block());
    let genesis_clone = Arc::clone(&genesis);

    // verify request block rpc
    let mut block_retrieval = receiver_1.block_retrieval;
    let on_request_block = async move {
        while let Some(request) = block_retrieval.next().await {
            request
                .response_sender
                .send(BlockRetrievalResponse {
                    status: BlockRetrievalStatus::SUCCEEDED,
                    blocks: vec![Block::clone(genesis_clone.as_ref())],
                })
                .unwrap();
        }
    };
    runtime
        .executor()
        .spawn(on_request_block.boxed().unit_error().compat());
    let peer = peers[1];
    block_on(async move {
        let response = nodes[0]
            .request_block(genesis.id(), 1, peer, Duration::from_secs(5))
            .await
            .unwrap();
        assert_eq!(response.blocks[0], *genesis);
    });

    // verify request chunk rpc
    let mut chunk_retrieval = receiver_1.chunk_retrieval;
    let on_request_chunk = async move {
        while let Some(request) = chunk_retrieval.next().await {
            let keypair = generate_keypair();
            let proto_txn =
                get_test_signed_txn(AccountAddress::random(), 0, keypair.0, keypair.1, None);
            let txn = SignedTransaction::from_proto(proto_txn).unwrap();
            let info =
                TransactionInfo::new(HashValue::zero(), HashValue::zero(), HashValue::zero(), 0);
            request
                .response_sender
                .send(Ok(TransactionListWithProof::new(
                    vec![(txn, info)],
                    None,
                    None,
                    None,
                    None,
                )))
                .unwrap();
        }
    };
    runtime
        .executor()
        .spawn(on_request_chunk.boxed().unit_error().compat());

    block_on(async move {
        let mut ledger_info = LedgerInfo::new();
        ledger_info.set_transaction_accumulator_hash(HashValue::zero().to_vec());
        ledger_info.set_consensus_block_id(HashValue::zero().to_vec());
        ledger_info.set_consensus_data_hash(
            VoteMsg::vote_digest(
                HashValue::zero(),
                ExecutedState {
                    state_id: HashValue::zero(),
                    version: 0,
                },
                0,
                HashValue::zero(),
                0,
                HashValue::zero(),
                0,
            )
            .to_vec(),
        );
        let mut ledger_info_with_sigs = LedgerInfoWithSignatures::new();
        ledger_info_with_sigs.set_ledger_info(ledger_info);
        let mut target = ProtoQuorumCert::new();
        target.set_block_id(HashValue::zero().into());
        target.set_state_id(HashValue::zero().into());
        target.set_round(0);
        target.set_signed_ledger_info(ledger_info_with_sigs);
        target.set_parent_block_id(HashValue::zero().into());
        target.set_parent_block_round(0);
        target.set_grandparent_block_id(HashValue::zero().into());
        target.set_grandparent_block_round(0);
        let mut req = RequestChunk::new();
        req.set_start_version(0);
        req.set_batch_size(1);
        req.set_target(target);
        let chunk = senders[0]
            .request_chunk(peers[1], req, Duration::from_secs(5))
            .await
            .unwrap();
        assert_eq!(chunk.get_txn_list_with_proof().get_transactions().len(), 1);
    });
}

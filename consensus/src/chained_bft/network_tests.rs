// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::{
    network::{NetworkReceivers, NetworkSender},
    test_utils::{self, consensus_runtime, placeholder_ledger_info},
};
use channel;
use consensus_types::{
    block::block_test_utils::certificate_for_genesis, block::Block, common::Author,
    proposal_msg::ProposalMsg, sync_info::SyncInfo, vote::Vote, vote_data::VoteData,
    vote_msg::VoteMsg,
};
use futures::{channel::mpsc, executor::block_on, SinkExt, StreamExt};
use libra_prost_ext::MessageExt;
use libra_types::block_info::BlockInfo;
use network::{
    interface::{NetworkNotification, NetworkRequest},
    proto::{ConsensusMsg, ConsensusMsg_oneof},
    protocols::rpc::InboundRpcRequest,
    validator_network::{ConsensusNetworkEvents, ConsensusNetworkSender},
};
use prost::Message;
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex, RwLock},
    time::Duration,
};
use tokio::runtime::Handle;

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
    executor: Handle,
}

impl NetworkPlayground {
    pub fn new(executor: Handle) -> Self {
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
                        .get(&dst)
                        .unwrap()
                        .clone();

                    let inbound_req = InboundRpcRequest {
                        protocol: outbound_req.protocol,
                        data: outbound_req.data,
                        res_tx: outbound_req.res_tx,
                    };

                    node_consensus_tx
                        .send(NetworkNotification::RecvRpc(src, inbound_req))
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
        self.executor.spawn(fut);
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
            .get(&dst)
            .unwrap()
            .clone();

        // convert NetworkRequest to corresponding NetworkNotification
        let msg_notif = match msg {
            NetworkRequest::SendMessage(_dst, msg) => NetworkNotification::RecvMessage(src, msg),
            msg => panic!("[network playground] Unexpected NetworkRequest: {:?}", msg),
        };

        // copy message data
        let msg_copy = match &msg_notif {
            NetworkNotification::RecvMessage(src, msg) => {
                let msg = ConsensusMsg::decode(msg.mdata.as_ref()).unwrap();
                (*src, msg)
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
        !Self::timeout_votes_only(msg_copy)
    }

    /// Returns true for proposal messages only.
    pub fn proposals_only(msg_copy: &(Author, ConsensusMsg)) -> bool {
        if let Some(ConsensusMsg_oneof::Proposal(_)) = msg_copy.1.message {
            true
        } else {
            false
        }
    }

    /// Returns true for vote messages only.
    pub fn votes_only(msg_copy: &(Author, ConsensusMsg)) -> bool {
        if let Some(ConsensusMsg_oneof::VoteMsg(_)) = msg_copy.1.message {
            true
        } else {
            false
        }
    }

    /// Returns true for vote messages that carry round signatures only.
    pub fn timeout_votes_only(msg_copy: &(Author, ConsensusMsg)) -> bool {
        // Timeout votes carry non-empty round signatures.
        if let Some(ConsensusMsg_oneof::VoteMsg(vote_msg)) = &msg_copy.1.message {
            let vote_msg = VoteMsg::try_from(vote_msg.clone()).unwrap();
            vote_msg.vote().timeout_signature().is_some()
        } else {
            false
        }
    }

    /// Returns true for sync info messages only.
    pub fn sync_info_only(msg_copy: &(Author, ConsensusMsg)) -> bool {
        if let Some(ConsensusMsg_oneof::SyncInfo(_)) = msg_copy.1.message {
            true
        } else {
            false
        }
    }

    pub fn epoch_change_only(msg_copy: &(Author, ConsensusMsg)) -> bool {
        if let Some(ConsensusMsg_oneof::EpochChange(_)) = msg_copy.1.message {
            true
        } else {
            false
        }
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
            NetworkRequest::SendMessage(dst, _) => self.0.get(src).unwrap().contains(&dst),
            NetworkRequest::SendRpc(dst, _) => self.0.get(src).unwrap().contains(&dst),
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

use crate::chained_bft::network::NetworkTask;
use crate::chained_bft::test_utils::TestPayload;
use consensus_types::block_retrieval::{
    BlockRetrievalRequest, BlockRetrievalResponse, BlockRetrievalStatus,
};
use libra_crypto::HashValue;
#[cfg(test)]
use libra_types::crypto_proxies::random_validator_verifier;
use libra_types::crypto_proxies::EpochInfo;
use std::convert::{TryFrom, TryInto};

#[test]
fn test_network_api() {
    let runtime = consensus_runtime();
    let num_nodes = 5;
    let mut receivers: Vec<NetworkReceivers<u64>> = Vec::new();
    let mut playground = NetworkPlayground::new(runtime.handle().clone());
    let mut nodes = Vec::new();
    let (signers, validator_verifier) = random_validator_verifier(num_nodes, None, false);
    let peers: Vec<_> = signers.iter().map(|signer| signer.author()).collect();
    let validators = Arc::new(validator_verifier);
    for peer in &peers {
        let (network_reqs_tx, network_reqs_rx) = channel::new_test(8);
        let (consensus_tx, consensus_rx) = channel::new_test(8);
        let network_sender = ConsensusNetworkSender::new(network_reqs_tx);
        let network_events = ConsensusNetworkEvents::new(consensus_rx);

        playground.add_node(*peer, consensus_tx, network_reqs_rx);
        let (self_sender, self_receiver) = channel::new_test(8);
        let node = NetworkSender::new(*peer, network_sender, self_sender, Arc::clone(&validators));
        let (task, receiver) = NetworkTask::new(
            Arc::new(RwLock::new(EpochInfo {
                epoch: 1,
                verifier: Arc::clone(&validators),
            })),
            network_events,
            self_receiver,
        );
        receivers.push(receiver);
        runtime.handle().spawn(task.start());
        nodes.push(node);
    }
    let vote_msg = VoteMsg::new(
        Vote::new(
            VoteData::new(BlockInfo::random(1), BlockInfo::random(0)),
            peers[0],
            placeholder_ledger_info(),
            &signers[0],
        ),
        test_utils::placeholder_sync_info(),
    );
    let previous_qc = certificate_for_genesis();
    let proposal = ProposalMsg::new(
        Block::new_proposal(0, 1, 1, previous_qc.clone(), &signers[0]),
        SyncInfo::new(previous_qc.clone(), previous_qc.clone(), None),
    );
    block_on(async move {
        nodes[0]
            .send_vote(vote_msg.clone(), peers[2..5].to_vec())
            .await;
        playground
            .wait_for_messages(3, NetworkPlayground::take_all)
            .await;
        for r in receivers.iter_mut().take(5).skip(2) {
            let v = r.votes.next().await.unwrap();
            assert_eq!(v, vote_msg);
        }
        nodes[0].broadcast_proposal(proposal.clone()).await;
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
    let mut senders = Vec::new();
    let mut receivers: Vec<NetworkReceivers<u64>> = Vec::new();
    let mut playground = NetworkPlayground::new(runtime.handle().clone());
    let mut nodes = Vec::new();
    let (signers, validator_verifier) = random_validator_verifier(num_nodes, None, false);
    let validators = Arc::new(validator_verifier);
    let peers: Vec<_> = signers.iter().map(|signer| signer.author()).collect();
    for peer in peers.iter() {
        let (network_reqs_tx, network_reqs_rx) = channel::new_test(8);
        let (consensus_tx, consensus_rx) = channel::new_test(1);
        let network_sender = ConsensusNetworkSender::new(network_reqs_tx);
        let network_events = ConsensusNetworkEvents::new(consensus_rx);

        playground.add_node(*peer, consensus_tx, network_reqs_rx);
        let (self_sender, self_receiver) = channel::new_test(8);
        let node = NetworkSender::new(
            *peer,
            network_sender.clone(),
            self_sender,
            Arc::clone(&validators),
        );
        let (task, receiver) = NetworkTask::new(
            Arc::new(RwLock::new(EpochInfo {
                epoch: 1,
                verifier: Arc::clone(&validators),
            })),
            network_events,
            self_receiver,
        );
        senders.push(network_sender);
        receivers.push(receiver);
        runtime.handle().spawn(task.start());
        nodes.push(node);
    }
    let receiver_1 = receivers.remove(1);
    let node0 = nodes[0].clone();
    let peer1 = peers[1];
    let vote_msg = VoteMsg::new(
        Vote::new(
            VoteData::new(BlockInfo::random(1), BlockInfo::random(0)),
            peers[0],
            placeholder_ledger_info(),
            &signers[0],
        ),
        test_utils::placeholder_sync_info(),
    );

    // verify request block rpc
    let mut block_retrieval = receiver_1.block_retrieval;
    let on_request_block = async move {
        while let Some(request) = block_retrieval.next().await {
            // make sure the network task is not blocked during RPC
            // we limit the network notification queue size to 1 so if it's blocked,
            // we can not process 2 votes and the test will timeout
            node0.send_vote(vote_msg.clone(), vec![peer1]).await;
            node0.send_vote(vote_msg.clone(), vec![peer1]).await;
            playground
                .wait_for_messages(2, NetworkPlayground::votes_only)
                .await;
            let response = BlockRetrievalResponse::<TestPayload>::new(
                BlockRetrievalStatus::IdNotFound,
                vec![],
            );
            let bytes = ConsensusMsg {
                message: Some(ConsensusMsg_oneof::RespondBlock(
                    response.try_into().unwrap(),
                )),
            }
            .to_bytes()
            .unwrap();
            request.response_sender.send(Ok(bytes)).unwrap();
        }
    };
    runtime.handle().spawn(on_request_block);
    let peer = peers[1];
    block_on(async move {
        let response = nodes[0]
            .request_block::<TestPayload>(
                BlockRetrievalRequest::new(HashValue::zero(), 1),
                peer,
                Duration::from_secs(5),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), BlockRetrievalStatus::IdNotFound);
    });
}

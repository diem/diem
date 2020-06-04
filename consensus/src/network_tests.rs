// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    network::{NetworkReceivers, NetworkSender},
    network_interface::{ConsensusMsg, ConsensusNetworkEvents, ConsensusNetworkSender},
    test_utils::{self, consensus_runtime, placeholder_ledger_info, timed_block_on},
};
use channel::{self, libra_channel, message_queues::QueueStyle};
use consensus_types::{
    block::{block_test_utils::certificate_for_genesis, Block},
    common::Author,
    proposal_msg::ProposalMsg,
    sync_info::SyncInfo,
    vote::Vote,
    vote_data::VoteData,
    vote_msg::VoteMsg,
};
use futures::{channel::mpsc, SinkExt, StreamExt};
use libra_types::{block_info::BlockInfo, PeerId};
use network::{
    peer_manager::{
        conn_notifs_channel, ConnectionRequestSender, PeerManagerNotification, PeerManagerRequest,
        PeerManagerRequestSender,
    },
    protocols::rpc::InboundRpcRequest,
    ProtocolId,
};
use std::{
    collections::{HashMap, HashSet},
    num::NonZeroUsize,
    sync::{Arc, Mutex, RwLock},
    time::Duration,
};
use tokio::runtime::Handle;

/// `TwinId` is used by the NetworkPlayground to uniquely identify
/// nodes, even if they have the same `AccountAddress` (e.g. for Twins)
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct TwinId {
    /// Node's ID
    pub id: usize,
    /// Author (AccountAddress)
    pub author: Author,
}

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
    ///
    node_consensus_txs: Arc<
        Mutex<
            HashMap<TwinId, libra_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>>,
        >,
    >,
    /// Nodes' outbound handlers forward their outbound non-rpc messages to this
    /// queue.
    outbound_msgs_tx: mpsc::Sender<(TwinId, PeerManagerRequest)>,
    /// NetworkPlayground reads all nodes' outbound messages through this queue.
    outbound_msgs_rx: mpsc::Receiver<(TwinId, PeerManagerRequest)>,
    /// Allow test code to drop direct-send messages between peers.
    drop_config: Arc<RwLock<DropConfig>>,
    /// An executor for spawning node outbound network event handlers
    executor: Handle,
    // Maps authors to twins IDs
    // An author may have multiple twin IDs for Twins
    author_to_twin_ids: Arc<RwLock<AuthorToTwinIds>>,
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
            author_to_twin_ids: Arc::new(RwLock::new(AuthorToTwinIds(HashMap::new()))),
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
        src_twin_id: TwinId,
        mut network_reqs_rx: libra_channel::Receiver<(PeerId, ProtocolId), PeerManagerRequest>,
        mut outbound_msgs_tx: mpsc::Sender<(TwinId, PeerManagerRequest)>,
        node_consensus_txs: Arc<
            Mutex<
                HashMap<
                    TwinId,
                    libra_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>,
                >,
            >,
        >,
        author_to_twin_ids: Arc<RwLock<AuthorToTwinIds>>,
    ) {
        while let Some(net_req) = network_reqs_rx.next().await {
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
                PeerManagerRequest::SendRpc(dst, outbound_req) => {
                    let dst_twin_ids = author_to_twin_ids.read().unwrap().get_twin_ids(dst);

                    let dst_twin_id = match dst_twin_ids.iter().find(|dst_twin_id| {
                        !drop_config
                            .read()
                            .unwrap()
                            .is_message_dropped(&src_twin_id, dst_twin_id)
                    }) {
                        Some(id) => id,
                        None => continue, // drop rpc
                    };

                    let mut node_consensus_tx = node_consensus_txs
                        .lock()
                        .unwrap()
                        .get(&dst_twin_id)
                        .unwrap()
                        .clone();

                    let inbound_req = InboundRpcRequest {
                        protocol: outbound_req.protocol,
                        data: outbound_req.data,
                        res_tx: outbound_req.res_tx,
                    };

                    node_consensus_tx
                        .push(
                            (src_twin_id.author, ProtocolId::ConsensusRpc),
                            PeerManagerNotification::RecvRpc(src_twin_id.author, inbound_req),
                        )
                        .unwrap();
                }
                // Other PeerManagerRequest get buffered for `deliver_messages` to
                // synchronously drain.
                net_req => {
                    let _ = outbound_msgs_tx.send((src_twin_id, net_req)).await;
                }
            }
        }
    }

    /// Add a new node to the NetworkPlayground.
    pub fn add_node(
        &mut self,
        twin_id: TwinId,
        // The `Sender` of inbound network events. The `Receiver` end of this
        // queue is usually wrapped in a `ConsensusNetworkEvents` adapter.
        consensus_tx: libra_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>,
        // The `Receiver` of outbound network events this node sends. The
        // `Sender` side of this queue is usually wrapped in a
        // `ConsensusNetworkSender` adapter.
        network_reqs_rx: libra_channel::Receiver<(PeerId, ProtocolId), PeerManagerRequest>,
        conn_mgr_reqs_rx: channel::Receiver<network::ConnectivityRequest>,
    ) {
        self.node_consensus_txs
            .lock()
            .unwrap()
            .insert(twin_id.clone(), consensus_tx);
        self.drop_config.write().unwrap().add_node(twin_id);

        self.extend_author_to_twin_ids(twin_id.author, twin_id);

        let fut1 = NetworkPlayground::start_node_outbound_handler(
            Arc::clone(&self.drop_config),
            twin_id,
            network_reqs_rx,
            self.outbound_msgs_tx.clone(),
            self.node_consensus_txs.clone(),
            self.author_to_twin_ids.clone(),
        );
        let fut2 = conn_mgr_reqs_rx.map(Ok).forward(::futures::sink::drain());
        self.executor.spawn(futures::future::join(fut1, fut2));
    }

    /// Deliver a `PeerManagerRequest` from peer `src` to the destination peer.
    /// Returns a copy of the delivered message and the sending peer id.
    async fn deliver_message(
        &mut self,
        src_twin_id: TwinId,
        dst_twin_id: TwinId,
        msg_notif: PeerManagerNotification,
    ) -> (Author, ConsensusMsg) {
        let mut node_consensus_tx = self
            .node_consensus_txs
            .lock()
            .unwrap()
            .get(&dst_twin_id)
            .unwrap()
            .clone();

        // copy message data
        let msg_copy = match &msg_notif {
            PeerManagerNotification::RecvMessage(src, msg) => {
                let msg: ConsensusMsg = lcs::from_bytes(&msg.mdata).unwrap();
                (*src, msg)
            }
            msg_notif => panic!(
                "[network playground] Unexpected PeerManagerNotification: {:?}",
                msg_notif
            ),
        };

        node_consensus_tx
            .push(
                (src_twin_id.author, ProtocolId::ConsensusDirectSend),
                msg_notif,
            )
            .unwrap();
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
            let (src_twin_id, net_req) = self.outbound_msgs_rx.next().await
                .expect("[network playground] waiting for messages, but message queue has shutdown unexpectedly");

            // Convert PeerManagerRequest to corresponding PeerManagerNotification,
            // and extract destination peer
            let (dst, msg) = match &net_req {
                PeerManagerRequest::SendMessage(dst_inner, msg_inner) => {
                    (*dst_inner, msg_inner.clone())
                }
                msg_inner => panic!(
                    "[network playground] Unexpected PeerManagerRequest: {:?}",
                    msg_inner
                ),
            };

            let dst_twin_ids = self.get_twin_ids(dst);

            for (idx, dst_twin_id) in dst_twin_ids.iter().enumerate() {
                let src_twin_id_copy = src_twin_id;
                let dst_twin_id_copy = *dst_twin_id;

                let msg_notif =
                    PeerManagerNotification::RecvMessage(src_twin_id.author, msg.clone());

                // Deliver and copy message it if it's not dropped
                if !self.is_message_dropped(&src_twin_id_copy, &dst_twin_id_copy) {
                    let msg_copy = self
                        .deliver_message(src_twin_id_copy, dst_twin_id_copy, msg_notif)
                        .await;
                    // Only insert msg_copy once for twins
                    if idx == 0 && msg_inspector(&msg_copy) {
                        msg_copies.push(msg_copy);
                    }
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
    pub fn proposals_only(msg: &(Author, ConsensusMsg)) -> bool {
        matches!(&msg.1, ConsensusMsg::ProposalMsg(_))
    }

    /// Returns true for vote messages only.
    pub fn votes_only(msg: &(Author, ConsensusMsg)) -> bool {
        matches!(&msg.1, ConsensusMsg::VoteMsg(_))
    }

    /// Returns true for vote messages that carry round signatures only.
    pub fn timeout_votes_only(msg: &(Author, ConsensusMsg)) -> bool {
        matches!(
            &msg.1,
            // Timeout votes carry non-empty round signatures.
            ConsensusMsg::VoteMsg(vote_msg) if vote_msg.vote().timeout_signature().is_some()
        )
    }

    /// Returns true for sync info messages only.
    pub fn sync_info_only(msg: &(Author, ConsensusMsg)) -> bool {
        matches!(&msg.1, ConsensusMsg::SyncInfo(_))
    }

    pub fn epoch_change_only(msg: &(Author, ConsensusMsg)) -> bool {
        matches!(&msg.1, ConsensusMsg::EpochChangeProof(_))
    }

    pub fn extend_author_to_twin_ids(&mut self, author: Author, twin_id: TwinId) {
        self.author_to_twin_ids
            .write()
            .unwrap()
            .extend_author_to_twin_ids(author, twin_id);
    }

    pub fn get_twin_ids(&self, author: Author) -> Vec<TwinId> {
        self.author_to_twin_ids.read().unwrap().get_twin_ids(author)
    }

    fn is_message_dropped(&self, src_twin_id: &TwinId, dst_twin_id: &TwinId) -> bool {
        self.drop_config
            .read()
            .unwrap()
            .is_message_dropped(src_twin_id, dst_twin_id)
    }

    pub fn drop_message_for(&mut self, src: &TwinId, dst: TwinId) -> bool {
        self.drop_config.write().unwrap().drop_message_for(src, dst)
    }

    pub fn stop_drop_message_for(&mut self, src: &TwinId, dst: &TwinId) -> bool {
        self.drop_config
            .write()
            .unwrap()
            .stop_drop_message_for(src, dst)
    }

    pub async fn start(mut self) {
        // Take the next queued message
        while let Some((src_twin_id, net_req)) = self.outbound_msgs_rx.next().await {
            // Convert PeerManagerRequest to corresponding PeerManagerNotification,
            // and extract destination peer
            let (dst, msg) = match &net_req {
                PeerManagerRequest::SendMessage(dst_inner, msg_inner) => {
                    (*dst_inner, msg_inner.clone())
                }
                msg_inner => panic!(
                    "[network playground] Unexpected PeerManagerRequest: {:?}",
                    msg_inner
                ),
            };

            let dst_twin_ids = self.get_twin_ids(dst);

            for dst_twin_id in dst_twin_ids.iter() {
                let msg_notif =
                    PeerManagerNotification::RecvMessage(src_twin_id.author, msg.clone());

                // Deliver and copy message it if it's not dropped
                if !self.is_message_dropped(&src_twin_id, &dst_twin_id) {
                    self.deliver_message(src_twin_id.clone(), dst_twin_id.clone(), msg_notif)
                        .await;
                }
            }
        }
    }
}

struct AuthorToTwinIds(HashMap<Author, Vec<TwinId>>);

impl AuthorToTwinIds {
    pub fn extend_author_to_twin_ids(&mut self, author: Author, twin_id: TwinId) {
        self.0.entry(author).or_insert_with(|| vec![]);

        self.0.get_mut(&author).unwrap().push(twin_id)
    }

    pub fn get_twin_ids(&self, author: Author) -> Vec<TwinId> {
        self.0.get(&author).unwrap().clone()
    }

    pub fn get_author_to_twin_ids(&self) -> HashMap<Author, Vec<TwinId>> {
        self.0.clone()
    }
}

struct DropConfig(HashMap<TwinId, HashSet<TwinId>>);

impl DropConfig {
    pub fn is_message_dropped(&self, src: &TwinId, dst: &TwinId) -> bool {
        self.0.get(src).unwrap().contains(&dst)
    }

    pub fn drop_message_for(&mut self, src: &TwinId, dst: TwinId) -> bool {
        self.0.get_mut(src).unwrap().insert(dst)
    }

    pub fn stop_drop_message_for(&mut self, src: &TwinId, dst: &TwinId) -> bool {
        self.0.get_mut(src).unwrap().remove(dst)
    }

    fn add_node(&mut self, src: TwinId) {
        self.0.insert(src, HashSet::new());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::NetworkTask;
    use consensus_types::block_retrieval::{
        BlockRetrievalRequest, BlockRetrievalResponse, BlockRetrievalStatus,
    };
    use libra_crypto::HashValue;
    use libra_types::validator_verifier::random_validator_verifier;

    #[test]
    fn test_network_api() {
        let mut runtime = consensus_runtime();
        let num_nodes = 5;
        let mut receivers: Vec<NetworkReceivers> = Vec::new();
        let mut playground = NetworkPlayground::new(runtime.handle().clone());
        let mut nodes = Vec::new();
        let (signers, validator_verifier) = random_validator_verifier(num_nodes, None, false);
        let peers: Vec<_> = signers.iter().map(|signer| signer.author()).collect();

        for (peer_id, peer) in peers.iter().enumerate() {
            let (network_reqs_tx, network_reqs_rx) =
                libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
            let (connection_reqs_tx, _) =
                libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
            let (consensus_tx, consensus_rx) =
                libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
            let (_conn_mgr_reqs_tx, conn_mgr_reqs_rx) = channel::new_test(8);
            let (_, conn_status_rx) = conn_notifs_channel::new();
            let network_sender = ConsensusNetworkSender::new(
                PeerManagerRequestSender::new(network_reqs_tx),
                ConnectionRequestSender::new(connection_reqs_tx),
            );
            let network_events = ConsensusNetworkEvents::new(consensus_rx, conn_status_rx);

            let twin_id = TwinId {
                id: peer_id,
                author: *peer,
            };

            playground.add_node(twin_id, consensus_tx, network_reqs_rx, conn_mgr_reqs_rx);

            let (self_sender, self_receiver) = channel::new_test(8);
            let node = NetworkSender::new(
                *peer,
                network_sender,
                self_sender,
                validator_verifier.clone(),
            );
            let (task, receiver) = NetworkTask::new(network_events, self_receiver);
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
            Block::new_proposal(vec![], 1, 1, previous_qc.clone(), &signers[0]),
            SyncInfo::new(previous_qc.clone(), previous_qc, None),
        );
        timed_block_on(&mut runtime, async {
            nodes[0]
                .send_vote(vote_msg.clone(), peers[2..5].to_vec())
                .await;
            playground
                .wait_for_messages(3, NetworkPlayground::take_all)
                .await;
            for r in receivers.iter_mut().take(5).skip(2) {
                let (_, msg) = r.consensus_messages.next().await.unwrap();
                match msg {
                    ConsensusMsg::VoteMsg(v) => assert_eq!(*v, vote_msg),
                    _ => panic!("unexpected messages"),
                }
            }
            nodes[0].broadcast_proposal(proposal.clone()).await;
            playground
                .wait_for_messages(4, NetworkPlayground::take_all)
                .await;
            for r in receivers.iter_mut().take(num_nodes - 1) {
                let (_, msg) = r.consensus_messages.next().await.unwrap();
                match msg {
                    ConsensusMsg::ProposalMsg(p) => assert_eq!(*p, proposal),
                    _ => panic!("unexpected messages"),
                }
            }
        });
    }

    #[test]
    fn test_rpc() {
        let mut runtime = consensus_runtime();
        let num_nodes = 2;
        let mut senders = Vec::new();
        let mut receivers: Vec<NetworkReceivers> = Vec::new();
        let mut playground = NetworkPlayground::new(runtime.handle().clone());
        let mut nodes = Vec::new();
        let (signers, validator_verifier) = random_validator_verifier(num_nodes, None, false);
        let peers: Vec<_> = signers.iter().map(|signer| signer.author()).collect();

        for (peer_id, peer) in peers.iter().enumerate() {
            let (network_reqs_tx, network_reqs_rx) =
                libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
            let (connection_reqs_tx, _) =
                libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
            let (consensus_tx, consensus_rx) =
                libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
            let (_conn_mgr_reqs_tx, conn_mgr_reqs_rx) = channel::new_test(8);
            let (_, conn_status_rx) = conn_notifs_channel::new();
            let network_sender = ConsensusNetworkSender::new(
                PeerManagerRequestSender::new(network_reqs_tx),
                ConnectionRequestSender::new(connection_reqs_tx),
            );
            let network_events = ConsensusNetworkEvents::new(consensus_rx, conn_status_rx);

            let twin_id = TwinId {
                id: peer_id,
                author: *peer,
            };

            playground.add_node(twin_id, consensus_tx, network_reqs_rx, conn_mgr_reqs_rx);

            let (self_sender, self_receiver) = channel::new_test(8);
            let node = NetworkSender::new(
                *peer,
                network_sender.clone(),
                self_sender,
                validator_verifier.clone(),
            );
            let (task, receiver) = NetworkTask::new(network_events, self_receiver);
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
                let response =
                    BlockRetrievalResponse::new(BlockRetrievalStatus::IdNotFound, vec![]);
                let response = ConsensusMsg::BlockRetrievalResponse(Box::new(response));
                let bytes = lcs::to_bytes(&response).unwrap();
                request.response_sender.send(Ok(bytes.into())).unwrap();
            }
        };
        runtime.handle().spawn(on_request_block);
        let peer = peers[1];
        timed_block_on(&mut runtime, async {
            let response = nodes[0]
                .request_block(
                    BlockRetrievalRequest::new(HashValue::zero(), 1),
                    peer,
                    Duration::from_secs(5),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), BlockRetrievalStatus::IdNotFound);
        });
    }
}

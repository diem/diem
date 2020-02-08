// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::counters;
use anyhow::ensure;
use bytes::Bytes;
use channel;
use consensus_types::block_retrieval::{BlockRetrievalRequest, BlockRetrievalResponse};
use consensus_types::epoch_retrieval::EpochRetrievalRequest;
use consensus_types::{
    common::{Author, Payload},
    proposal_msg::{ProposalMsg, ProposalUncheckedSignatures},
    sync_info::SyncInfo,
    vote_msg::VoteMsg,
};
use futures::{
    channel::{mpsc, oneshot},
    select, SinkExt, Stream, StreamExt,
};
use libra_logger::prelude::*;
use libra_types::account_address::AccountAddress;
use libra_types::crypto_proxies::ValidatorChangeProof;
use libra_types::crypto_proxies::ValidatorVerifier;
use libra_types::proto::types::ValidatorChangeProof as ValidatorChangeProofProto;
use network::{
    proto::{
        ConsensusMsg, ConsensusMsg_oneof, Proposal, RequestBlock, RequestEpoch,
        SyncInfo as SyncInfoProto, VoteMsg as VoteMsgProto,
    },
    validator_network::{ConsensusNetworkSender, Event, RpcError},
};
use smallvec::SmallVec;
use std::{
    collections::{HashMap, VecDeque},
    convert::{TryFrom, TryInto},
    sync::Arc,
    time::{Duration, Instant},
};

/// The block retrieval request is used internally for implementing RPC: the callback is executed
/// for carrying the response
#[derive(Debug)]
pub struct IncomingBlockRetrievalRequest {
    pub req: BlockRetrievalRequest,
    pub response_sender: oneshot::Sender<Result<Bytes, RpcError>>,
}

/// Implements the actual networking support for all consensus messaging.
#[derive(Clone)]
pub struct NetworkSender {
    author: Author,
    network_sender: ConsensusNetworkSender,
    // Self sender and self receivers provide a shortcut for sending the messages to itself.
    // (self sending is not supported by the networking API).
    // Note that we do not support self rpc requests as it might cause infinite recursive calls.
    self_sender: channel::Sender<anyhow::Result<Event<ConsensusMsg>>>,
    validators: Arc<ValidatorVerifier>,
}

impl NetworkSender {
    pub fn new(
        author: Author,
        network_sender: ConsensusNetworkSender,
        self_sender: channel::Sender<anyhow::Result<Event<ConsensusMsg>>>,
        validators: Arc<ValidatorVerifier>,
    ) -> Self {
        NetworkSender {
            author,
            network_sender,
            self_sender,
            validators,
        }
    }

    /// Tries to retrieve num of blocks backwards starting from id from the given peer: the function
    /// returns a future that is fulfilled with BlockRetrievalResponse.
    pub async fn request_block<T: Payload>(
        &mut self,
        retrieval_request: BlockRetrievalRequest,
        from: Author,
        timeout: Duration,
    ) -> anyhow::Result<BlockRetrievalResponse<T>> {
        ensure!(from != self.author, "Retrieve block from self");
        counters::BLOCK_RETRIEVAL_COUNT.inc_by(retrieval_request.num_blocks() as i64);
        let pre_retrieval_instant = Instant::now();
        let req_msg = RequestBlock::try_from(retrieval_request.clone())?;
        let response_msg = self
            .network_sender
            .request_block(from, req_msg, timeout)
            .await?;
        counters::BLOCK_RETRIEVAL_DURATION_S.observe_duration(pre_retrieval_instant.elapsed());
        let response = BlockRetrievalResponse::<T>::try_from(response_msg)?;
        response
            .verify(
                retrieval_request.block_id(),
                retrieval_request.num_blocks(),
                &self.validators,
            )
            .map_err(|e| {
                security_log(SecurityEvent::InvalidRetrievedBlock)
                    .error(&e)
                    .data(&response)
                    .log();
                e
            })?;

        Ok(response)
    }

    /// Tries to send the given proposal (block and proposer metadata) to all the participants.
    /// A validator on the receiving end is going to be notified about a new proposal in the
    /// proposal queue.
    ///
    /// The future is fulfilled as soon as the message put into the mpsc channel to network
    /// internal(to provide back pressure), it does not indicate the message is delivered or sent
    /// out. It does not give indication about when the message is delivered to the recipients,
    /// as well as there is no indication about the network failures.
    pub async fn broadcast_proposal<T: Payload>(&mut self, proposal: ProposalMsg<T>) {
        let proposal: Proposal = match proposal.try_into() {
            Ok(bytes) => bytes,
            Err(e) => {
                warn!("Fail to serialize VoteMsg: {:?}", e);
                return;
            }
        };
        counters::UNWRAPPED_PROPOSAL_SIZE_BYTES.observe(proposal.bytes.len() as f64);
        let msg = ConsensusMsg {
            message: Some(ConsensusMsg_oneof::Proposal(proposal)),
        };
        self.broadcast(msg).await
    }

    async fn broadcast(&mut self, msg: ConsensusMsg) {
        // Directly send the message to ourself without going through network.
        let self_msg = Event::Message((self.author, msg.clone()));
        if let Err(err) = self.self_sender.send(Ok(self_msg)).await {
            error!("Error broadcasting to self: {:?}", err);
        }

        // Get the list of validators excluding our own account address. Note the
        // ordering is not important in this case.
        let self_author = self.author;
        let other_validators = self
            .validators
            .get_ordered_account_addresses_iter()
            .filter(|author| author != &self_author);

        // Broadcast message over direct-send to all other validators.
        if let Err(err) = self
            .network_sender
            .send_to_many(other_validators, msg)
            .await
        {
            error!("Error broadcasting message: {:?}", err);
        }
    }

    /// Sends the vote to the chosen recipients (typically that would be the recipients that
    /// we believe could serve as proposers in the next round). The recipients on the receiving
    /// end are going to be notified about a new vote in the vote queue.
    ///
    /// The future is fulfilled as soon as the message put into the mpsc channel to network
    /// internal(to provide back pressure), it does not indicate the message is delivered or sent
    /// out. It does not give indication about when the message is delivered to the recipients,
    /// as well as there is no indication about the network failures.
    pub async fn send_vote(&self, vote_msg: VoteMsg, recipients: Vec<Author>) {
        let mut network_sender = self.network_sender.clone();
        let mut self_sender = self.self_sender.clone();
        let vote_msg = match vote_msg.try_into() {
            Ok(bytes) => bytes,
            Err(e) => {
                warn!("Fail to serialize VoteMsg: {:?}", e);
                return;
            }
        };
        let msg = ConsensusMsg {
            message: Some(ConsensusMsg_oneof::VoteMsg(vote_msg)),
        };
        for peer in recipients {
            if self.author == peer {
                let self_msg = Event::Message((self.author, msg.clone()));
                if let Err(err) = self_sender.send(Ok(self_msg)).await {
                    error!("Error delivering a self vote: {:?}", err);
                }
                continue;
            }
            if let Err(e) = network_sender.send_to(peer, msg.clone()).await {
                error!("Failed to send a vote to peer {:?}: {:?}", peer, e);
            }
        }
    }

    /// Broadcasts vote message to all validators
    pub async fn broadcast_vote(&mut self, vote_msg: VoteMsg) {
        let vote_msg = match vote_msg.try_into() {
            Ok(bytes) => bytes,
            Err(e) => {
                warn!("Fail to serialize VoteMsg: {:?}", e);
                return;
            }
        };
        let msg = ConsensusMsg {
            message: Some(ConsensusMsg_oneof::VoteMsg(vote_msg)),
        };
        self.broadcast(msg).await
    }

    /// Sends the given sync info to the given author.
    /// The future is fulfilled as soon as the message is added to the internal network channel
    /// (does not indicate whether the message is delivered or sent out).
    pub async fn send_sync_info(&self, sync_info: SyncInfo, recipient: Author) {
        if recipient == self.author {
            error!("An attempt to deliver sync info msg to itself: ignore.");
            return;
        }
        let sync_info = match sync_info.try_into() {
            Ok(bytes) => bytes,
            Err(e) => {
                warn!("Fail to serialize SyncInfo: {:?}", e);
                return;
            }
        };
        let msg = ConsensusMsg {
            message: Some(ConsensusMsg_oneof::SyncInfo(sync_info)),
        };
        let mut network_sender = self.network_sender.clone();
        if let Err(e) = network_sender.send_to(recipient, msg).await {
            warn!(
                "Failed to send a sync info msg to peer {:?}: {:?}",
                recipient, e
            );
        }
    }

    /// Broadcast about epoch changes with proof to the current validator set (including self)
    /// when we commit the reconfiguration block
    pub async fn broadcast_epoch_change(&mut self, proof: ValidatorChangeProof) {
        let msg = ConsensusMsg {
            message: Some(ConsensusMsg_oneof::EpochChange(proof.into())),
        };
        self.broadcast(msg).await
    }

    pub async fn notify_epoch_change(&mut self, proof: ValidatorChangeProof) {
        let msg = ConsensusMsg {
            message: Some(ConsensusMsg_oneof::EpochChange(proof.into())),
        };
        let self_msg = Event::Message((self.author, msg.clone()));
        if let Err(e) = self.self_sender.send(Ok(self_msg)).await {
            warn!("Failed to notify to self an epoch change {:?}", e);
        }
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
enum MessageType {
    Proposal,
    RequestBlock,
    RequestEpoch,
    EpochChange,
    Sync,
    Vote,
}

pub enum FromNetworkMsg<T> {
    Proposal(ProposalUncheckedSignatures<T>),
    RequestBlock(IncomingBlockRetrievalRequest),
    RequestEpoch(EpochRetrievalRequest),
    EpochChange(ValidatorChangeProof),
    Sync(Box<SyncInfo>),
    Vote(Box<VoteMsg>),
}

pub enum ConsensusDataRequest {
    NewData,
    ClearEpochData,
}

pub struct NetworkTask<T> {
    incoming: HashMap<AccountAddress, HashMap<MessageType, SmallVec<[FromNetworkMsg<T>; 2]>>>,
    round_robin_queue: VecDeque<(AccountAddress, MessageType)>,
    network_data_request_receiver: mpsc::UnboundedReceiver<ConsensusDataRequest>,
    network_data_sender: mpsc::UnboundedSender<(AccountAddress, FromNetworkMsg<T>)>,
}

impl<T: Payload> NetworkTask<T> {
    /// Establishes the initial connections with the peers and returns the receivers.
    pub fn new(
        network_data_request_receiver: mpsc::UnboundedReceiver<ConsensusDataRequest>,
        network_data_sender: mpsc::UnboundedSender<(AccountAddress, FromNetworkMsg<T>)>,
    ) -> NetworkTask<T> {
        NetworkTask {
            incoming: Default::default(),
            round_robin_queue: Default::default(),
            network_data_request_receiver,
            network_data_sender,
        }
    }

    pub async fn start(
        mut self,
        all_events: Box<dyn Stream<Item = anyhow::Result<Event<ConsensusMsg>>> + Send + Unpin>,
    ) {
        // If consensus asked for data, and we had none to give,
        // this flag is set,
        // so that when new data comes in over the network,
        // it is immediately sent over to consensus without first requiring another data request.
        let mut consensus_needs_data = false;

        let mut all_events = all_events.fuse();
        loop {
            select! {
                message = all_events.select_next_some() => {
                    match message {
                        Ok(msg) => self.handle_msg(msg).await,
                        Err(e) => {
                            warn!("Received an error instead of a consensus msg: {:?}", e);

                            // Note: does it make sense to wait for another message?
                            continue;
                        },
                    }

                    if consensus_needs_data {
                        // If consensus is waiting for data,
                        // we can immediately send the new stuff that just came in.
                        if let Some(data) = self.deque_msg() {
                            let res = self.network_data_sender.unbounded_send(data);
                            consensus_needs_data = res.is_err();
                        }
                    }
                },
                data_request = self.network_data_request_receiver.select_next_some() => {
                    match data_request {
                        ConsensusDataRequest::NewData => {
                            // Consensus wants more data to process.
                            if let Some(data) = self.deque_msg() {
                                let res = self.network_data_sender.unbounded_send(data);
                                consensus_needs_data = res.is_err();
                            } else {
                                consensus_needs_data = true;
                            }
                        },
                        ConsensusDataRequest::ClearEpochData => {
                            // Clear all buffered epoch-specific messages.
                            self.clear_data();
                        }
                    }
                },
                complete => break,
            }
        }
    }

    async fn handle_msg(&mut self, event: Event<ConsensusMsg>) {
        use ConsensusMsg_oneof::*;

        match event {
            Event::Message((peer_id, msg)) => {
                let msg = match msg.message {
                    Some(msg) => msg,
                    None => {
                        warn!("Unexpected msg from {}: {:?}", peer_id, msg);
                        return;
                    }
                };

                let r = match msg {
                    Proposal(proposal) => self.process_proposal(peer_id, proposal).await,
                    VoteMsg(vote_msg) => self.process_vote(peer_id, vote_msg).await,
                    SyncInfo(sync_info) => self.process_sync_info(sync_info, peer_id).await,
                    EpochChange(proof) => self.process_epoch_change(peer_id, proof).await,
                    RequestEpoch(request) => self.process_epoch_request(peer_id, request).await,
                    _ => {
                        warn!("Unexpected msg from {}: {:?}", peer_id, msg);
                        Ok(())
                    }
                };
                if let Err(e) = r {
                    warn!("Failed to process msg {}", e)
                }
            }
            Event::RpcRequest((peer_id, msg, callback)) => {
                let r = match msg.message {
                    Some(RequestBlock(request)) => {
                        self.process_request_block(peer_id, request, callback).await
                    }
                    _ => {
                        warn!("Unexpected RPC from {}: {:?}", peer_id, msg);
                        Ok(())
                    }
                };
                if let Err(e) = r {
                    warn!("Failed to process RPC {:?}", e)
                }
            }
            Event::NewPeer(peer_id) => {
                debug!("Peer {} connected", peer_id);
            }
            Event::LostPeer(peer_id) => {
                debug!("Peer {} dis-connected", peer_id);

                // Remove all the queues related to this peer.
                self.incoming.remove(&peer_id);
                self.round_robin_queue = self
                    .round_robin_queue
                    .clone()
                    .into_iter()
                    .filter_map(|(id, msg_type)| {
                        if peer_id == id {
                            return None;
                        }
                        Some((id, msg_type))
                    })
                    .collect();
            }
        }
    }

    fn clear_data(&mut self) {
        // Clear messages after epoch change.
        for per_peer_queue in self.incoming.values_mut() {
            for (msg_type, queue) in per_peer_queue.iter_mut() {
                match msg_type {
                    MessageType::Proposal
                    | MessageType::Vote
                    | MessageType::RequestBlock
                    | MessageType::Sync => {
                        queue.clear();
                    }
                    _ => {}
                }
            }
        }
    }

    fn deque_msg(&mut self) -> Option<(AccountAddress, FromNetworkMsg<T>)> {
        // Note: if we want/need to increase paralellism
        // between this task and the other consensus task,
        // we could use a batching strategy,
        // that would take into account the message-type
        // and whether the message could be relevant if the epoch changes.

        let mut count = 0;
        loop {
            count += 1;
            // If we've cycled through the entire round-robin,
            // we don't have any data to send.
            if count > self.round_robin_queue.len() {
                return None;
            }

            let (peer_id, msg_type) = match self.round_robin_queue.pop_front() {
                Some(v) => v,
                _ => {
                    // If the round-robin is empty,
                    // we also don't have any data.
                    return None;
                }
            };

            let per_peer_queue = match self.incoming.get_mut(&peer_id) {
                Some(queue) => queue,
                None => {
                    self.round_robin_queue.push_back((peer_id, msg_type));
                    continue;
                }
            };

            let msg = match per_peer_queue.get_mut(&msg_type) {
                Some(queue) => queue.pop(),
                None => {
                    self.round_robin_queue.push_back((peer_id, msg_type));
                    continue;
                }
            };

            let msg = match msg {
                Some(msg) => msg,
                None => {
                    self.round_robin_queue.push_back((peer_id, msg_type));
                    continue;
                }
            };

            self.round_robin_queue
                .push_back((peer_id, msg_type.clone()));

            match msg_type {
                MessageType::Proposal => {
                    counters::PROPOSAL_CHANNEL_MSGS
                        .with_label_values(&["dequeued"])
                        .inc();
                }
                MessageType::Vote => {
                    counters::VOTES_CHANNEL_MSGS
                        .with_label_values(&["dequeued"])
                        .inc();
                }
                MessageType::RequestBlock => {
                    counters::BLOCK_RETRIEVAL_CHANNEL_MSGS
                        .with_label_values(&["dequeued"])
                        .inc();
                }
                MessageType::Sync => {
                    counters::SYNC_INFO_CHANNEL_MSGS
                        .with_label_values(&["dequeued"])
                        .inc();
                }
                MessageType::EpochChange => {
                    counters::EPOCH_CHANGE_CHANNEL_MSGS
                        .with_label_values(&["dequeued"])
                        .inc();
                }
                _ => {}
            }

            return Some((peer_id, msg));
        }
    }

    fn queue_msg(
        &mut self,
        peer_id: AccountAddress,
        msg: FromNetworkMsg<T>,
        msg_type: MessageType,
    ) {
        let incoming = self
            .incoming
            .entry(peer_id.clone())
            .or_insert_with(Default::default);
        let msg_queue = incoming
            .entry(msg_type.clone())
            .or_insert_with(Default::default);

        let dropped = {
            if msg_queue.len() == 2 {
                msg_queue.remove(0);
                true
            } else {
                false
            }
        };

        msg_queue.push(msg);

        // Add the relevant key to the round-robin,
        // if it is not present already.
        let msg_type = {
            let key = (peer_id, msg_type);

            if !self.round_robin_queue.contains(&key) {
                self.round_robin_queue.push_back(key.clone());
            }

            key.1
        };

        match msg_type {
            MessageType::Proposal => {
                if dropped {
                    counters::PROPOSAL_CHANNEL_MSGS
                        .with_label_values(&["dropped"])
                        .inc();
                }
                counters::PROPOSAL_CHANNEL_MSGS
                    .with_label_values(&["enqueued"])
                    .inc();
            }
            MessageType::Vote => {
                if dropped {
                    counters::VOTES_CHANNEL_MSGS
                        .with_label_values(&["dropped"])
                        .inc();
                }
                counters::VOTES_CHANNEL_MSGS
                    .with_label_values(&["enqueued"])
                    .inc();
            }
            MessageType::RequestBlock => {
                if dropped {
                    counters::BLOCK_RETRIEVAL_CHANNEL_MSGS
                        .with_label_values(&["dropped"])
                        .inc();
                }
                counters::BLOCK_RETRIEVAL_CHANNEL_MSGS
                    .with_label_values(&["enqueued"])
                    .inc();
            }
            MessageType::Sync => {
                if dropped {
                    counters::SYNC_INFO_CHANNEL_MSGS
                        .with_label_values(&["dropped"])
                        .inc();
                }
                counters::SYNC_INFO_CHANNEL_MSGS
                    .with_label_values(&["enqueued"])
                    .inc();
            }
            MessageType::EpochChange => {
                if dropped {
                    counters::EPOCH_CHANGE_CHANNEL_MSGS
                        .with_label_values(&["dropped"])
                        .inc();
                }
                counters::EPOCH_CHANGE_CHANNEL_MSGS
                    .with_label_values(&["enqueued"])
                    .inc();
            }
            _ => {}
        }
    }

    async fn process_proposal(
        &mut self,
        peer_id: AccountAddress,
        proposal: Proposal,
    ) -> anyhow::Result<()> {
        let proposal = ProposalUncheckedSignatures::<T>::try_from(proposal)?;
        self.queue_msg(
            peer_id,
            FromNetworkMsg::Proposal(proposal),
            MessageType::Proposal,
        );
        Ok(())
    }

    async fn process_vote(
        &mut self,
        peer_id: AccountAddress,
        vote_msg: VoteMsgProto,
    ) -> anyhow::Result<()> {
        let vote_msg = VoteMsg::try_from(vote_msg)?;
        ensure!(
            vote_msg.vote().author() == peer_id,
            "vote received must be from the sending peer"
        );
        self.queue_msg(
            peer_id,
            FromNetworkMsg::Vote(Box::new(vote_msg)),
            MessageType::Vote,
        );
        Ok(())
    }

    async fn process_sync_info(
        &mut self,
        sync_info: SyncInfoProto,
        peer_id: AccountAddress,
    ) -> anyhow::Result<()> {
        let sync_info = SyncInfo::try_from(sync_info)?;
        self.queue_msg(
            peer_id,
            FromNetworkMsg::Sync(Box::new(sync_info)),
            MessageType::Sync,
        );
        Ok(())
    }

    async fn process_request_block(
        &mut self,
        peer_id: AccountAddress,
        request_msg: RequestBlock,
        callback: oneshot::Sender<Result<Bytes, RpcError>>,
    ) -> anyhow::Result<()> {
        let req = BlockRetrievalRequest::try_from(request_msg)?;
        debug!("Received block retrieval request {}", req);

        let req_with_callback = IncomingBlockRetrievalRequest {
            req,
            response_sender: callback,
        };
        self.queue_msg(
            peer_id,
            FromNetworkMsg::RequestBlock(req_with_callback),
            MessageType::RequestBlock,
        );
        Ok(())
    }

    async fn process_epoch_change(
        &mut self,
        peer_id: AccountAddress,
        proof: ValidatorChangeProofProto,
    ) -> anyhow::Result<()> {
        let proof = ValidatorChangeProof::try_from(proof)?;
        self.queue_msg(
            peer_id,
            FromNetworkMsg::EpochChange(proof),
            MessageType::EpochChange,
        );
        Ok(())
    }

    async fn process_epoch_request(
        &mut self,
        peer_id: AccountAddress,
        request: RequestEpoch,
    ) -> anyhow::Result<()> {
        let request = EpochRetrievalRequest::try_from(request)?;
        self.queue_msg(
            peer_id,
            FromNetworkMsg::RequestEpoch(request),
            MessageType::RequestEpoch,
        );
        Ok(())
    }
}

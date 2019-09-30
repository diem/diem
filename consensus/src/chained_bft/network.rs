// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        common::{Author, Payload},
        consensus_types::{
            block::Block,
            proposal_msg::{ProposalMsg, ProposalUncheckedSignatures},
            sync_info::SyncInfo,
            timeout_msg::TimeoutMsg,
            vote_msg::VoteMsg,
        },
        epoch_manager::EpochManager,
    },
    counters,
};
use bytes::Bytes;
use failure::{self, ResultExt};
use futures::{
    channel::oneshot, stream::select, FutureExt, SinkExt, Stream, StreamExt, TryFutureExt,
    TryStreamExt,
};
use libra_channel;
use libra_crypto::HashValue;
use libra_logger::prelude::*;
use libra_network::{
    proto::{BlockRetrievalStatus, ConsensusMsg, RequestBlock, RespondBlock},
    validator_network::{ConsensusNetworkEvents, ConsensusNetworkSender, Event, RpcError},
};
use libra_proto_conv::{FromProto, IntoProto};
use libra_types::account_address::AccountAddress;
use protobuf::Message;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::runtime::TaskExecutor;

/// The response sent back from EventProcessor for the BlockRetrievalRequest.
#[derive(Debug)]
pub struct BlockRetrievalResponse<T> {
    pub status: BlockRetrievalStatus,
    pub blocks: Vec<Block<T>>,
}

impl<T: Payload> BlockRetrievalResponse<T> {
    pub fn verify(&self, block_id: HashValue, num_blocks: u64) -> failure::Result<()> {
        ensure!(
            self.status != BlockRetrievalStatus::SUCCEEDED
                || self.blocks.len() as u64 == num_blocks,
            "not enough blocks returned, expect {}, get {}",
            num_blocks,
            self.blocks.len(),
        );
        self.blocks
            .iter()
            .try_fold(block_id, |expected_id, block| {
                ensure!(
                    block.id() == expected_id,
                    "blocks doesn't form a chain: expect {}, get {}",
                    expected_id,
                    block.id()
                );
                Ok(block.parent_id())
            })
            .map(|_| ())
    }
}

/// BlockRetrievalRequest carries a block id for the requested block as well as the
/// oneshot sender to deliver the response.
#[derive(Debug)]
pub struct BlockRetrievalRequest<T> {
    pub block_id: HashValue,
    pub num_blocks: u64,
    pub response_sender: oneshot::Sender<BlockRetrievalResponse<T>>,
}

/// Just a convenience struct to keep all the network proxy receiving queues in one place.
/// Will be returned by the networking trait upon startup.
pub struct NetworkReceivers<T> {
    pub proposals: libra_channel::Receiver<ProposalMsg<T>>,
    pub votes: libra_channel::Receiver<VoteMsg>,
    pub block_retrieval: libra_channel::Receiver<BlockRetrievalRequest<T>>,
    pub timeout_msgs: libra_channel::Receiver<TimeoutMsg>,
    pub sync_info_msgs: libra_channel::Receiver<(SyncInfo, AccountAddress)>,
}

/// Implements the actual networking support for all consensus messaging.
pub struct ConsensusNetworkImpl {
    author: Author,
    network_sender: ConsensusNetworkSender,
    network_events: Option<ConsensusNetworkEvents>,
    // Self sender and self receivers provide a shortcut for sending the messages to itself.
    // (self sending is not supported by the networking API).
    // Note that we do not support self rpc requests as it might cause infinite recursive calls.
    self_sender: libra_channel::Sender<failure::Result<Event<ConsensusMsg>>>,
    self_receiver: Option<libra_channel::Receiver<failure::Result<Event<ConsensusMsg>>>>,
    epoch_mgr: Arc<EpochManager>,
}

impl Clone for ConsensusNetworkImpl {
    fn clone(&self) -> Self {
        Self {
            author: self.author,
            network_sender: self.network_sender.clone(),
            network_events: None,
            self_sender: self.self_sender.clone(),
            self_receiver: None,
            epoch_mgr: Arc::clone(&self.epoch_mgr),
        }
    }
}

impl ConsensusNetworkImpl {
    pub fn new(
        author: Author,
        network_sender: ConsensusNetworkSender,
        network_events: ConsensusNetworkEvents,
        epoch_mgr: Arc<EpochManager>,
    ) -> Self {
        let (self_sender, self_receiver) =
            libra_channel::new(1_024, &counters::PENDING_SELF_MESSAGES);
        ConsensusNetworkImpl {
            author,
            network_sender,
            network_events: Some(network_events),
            self_sender,
            self_receiver: Some(self_receiver),
            epoch_mgr,
        }
    }

    /// Establishes the initial connections with the peers and returns the receivers.
    pub fn start<T: Payload>(&mut self, executor: &TaskExecutor) -> NetworkReceivers<T> {
        let (proposal_tx, proposal_rx) = libra_channel::new(1_024, &counters::PENDING_PROPOSAL);
        let (vote_tx, vote_rx) = libra_channel::new(1_024, &counters::PENDING_VOTES);
        let (block_request_tx, block_request_rx) =
            libra_channel::new(1_024, &counters::PENDING_BLOCK_REQUESTS);
        let (timeout_msg_tx, timeout_msg_rx) =
            libra_channel::new(1_024, &counters::PENDING_NEW_ROUND_MESSAGES);
        let (sync_info_tx, sync_info_rx) =
            libra_channel::new(1_024, &counters::PENDING_SYNC_INFO_MSGS);
        let network_events = self
            .network_events
            .take()
            .expect("[consensus] Failed to start; network_events stream is already taken")
            .map_err(Into::<failure::Error>::into);
        let own_msgs = self
            .self_receiver
            .take()
            .expect("[consensus]: self receiver is already taken");
        let all_events = select(network_events, own_msgs);
        executor.spawn(
            NetworkTask {
                proposal_tx,
                vote_tx,
                block_request_tx,
                timeout_msg_tx,
                sync_info_tx,
                all_events,
                epoch_mgr: Arc::clone(&self.epoch_mgr),
            }
            .run()
            .boxed()
            .unit_error()
            .compat(),
        );
        NetworkReceivers {
            proposals: proposal_rx,
            votes: vote_rx,
            block_retrieval: block_request_rx,
            timeout_msgs: timeout_msg_rx,
            sync_info_msgs: sync_info_rx,
        }
    }

    /// Tries to retrieve num of blocks backwards starting from id from the given peer: the function
    /// returns a future that is either fulfilled with BlockRetrievalResponse, or with a
    /// BlockRetrievalFailure.
    pub async fn request_block<T: Payload>(
        &mut self,
        block_id: HashValue,
        num_blocks: u64,
        from: Author,
        timeout: Duration,
    ) -> failure::Result<BlockRetrievalResponse<T>> {
        ensure!(from != self.author, "Retrieve block from self");
        let mut req_msg = RequestBlock::new();
        req_msg.set_block_id(block_id.into());
        req_msg.set_num_blocks(num_blocks);
        counters::BLOCK_RETRIEVAL_COUNT.inc_by(num_blocks as i64);
        let pre_retrieval_instant = Instant::now();

        let mut res_block = self
            .network_sender
            .request_block(from, req_msg, timeout)
            .await?;
        let mut blocks = vec![];
        for block in res_block.take_blocks().into_iter() {
            match Block::from_proto(block) {
                Ok(block) => {
                    block
                        .validate_signatures(self.epoch_mgr.validators().as_ref())
                        .and_then(|_| block.verify_well_formed())
                        .with_context(|e| format_err!("Invalid block because of {:?}", e))?;
                    blocks.push(block);
                }
                Err(e) => bail!("Failed to deserialize block because of {:?}", e),
            };
        }
        counters::BLOCK_RETRIEVAL_DURATION_S.observe_duration(pre_retrieval_instant.elapsed());
        let response = BlockRetrievalResponse {
            status: res_block.get_status(),
            blocks,
        };
        response.verify(block_id, num_blocks)?;
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
        let mut msg = ConsensusMsg::new();
        msg.set_proposal(proposal.into_proto());
        self.broadcast(msg).await
    }

    async fn broadcast(&mut self, msg: ConsensusMsg) {
        for peer in self.epoch_mgr.validators().get_ordered_account_addresses() {
            if self.author == peer {
                let self_msg = Event::Message((self.author, msg.clone()));
                if let Err(err) = self.self_sender.send(Ok(self_msg)).await {
                    error!("Error delivering a self proposal: {:?}", err);
                }
                continue;
            }
            if let Err(err) = self.network_sender.send_to(peer, msg.clone()).await {
                error!(
                    "Error broadcasting proposal to peer: {:?}, error: {:?}, msg: {:?}",
                    peer, err, msg
                );
            }
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
        let mut msg = ConsensusMsg::new();
        msg.set_vote(vote_msg.into_proto());
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

    /// Broadcasts timeout message to all validators
    pub async fn broadcast_timeout_msg(&mut self, timeout_msg: TimeoutMsg) {
        let mut msg = ConsensusMsg::new();
        msg.set_timeout_msg(timeout_msg.into_proto());
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
        let mut msg = ConsensusMsg::new();
        msg.set_sync_info(sync_info.into_proto());
        let mut network_sender = self.network_sender.clone();
        if let Err(e) = network_sender.send_to(recipient, msg).await {
            warn!(
                "Failed to send a sync info msg to peer {:?}: {:?}",
                recipient, e
            );
        }
    }
}

struct NetworkTask<T, S> {
    proposal_tx: libra_channel::Sender<ProposalMsg<T>>,
    vote_tx: libra_channel::Sender<VoteMsg>,
    block_request_tx: libra_channel::Sender<BlockRetrievalRequest<T>>,
    timeout_msg_tx: libra_channel::Sender<TimeoutMsg>,
    sync_info_tx: libra_channel::Sender<(SyncInfo, AccountAddress)>,
    all_events: S,
    epoch_mgr: Arc<EpochManager>,
}

impl<T, S> NetworkTask<T, S>
where
    S: Stream<Item = failure::Result<Event<ConsensusMsg>>> + Unpin,
    T: Payload,
{
    pub async fn run(mut self) {
        while let Some(Ok(message)) = self.all_events.next().await {
            match message {
                Event::Message((peer_id, mut msg)) => {
                    let r = if msg.has_proposal() {
                        self.process_proposal(&mut msg).await.map_err(|e| {
                            security_log(SecurityEvent::InvalidConsensusProposal)
                                .error(&e)
                                .data(&msg)
                                .log();
                            e
                        })
                    } else if msg.has_vote() {
                        self.process_vote(&mut msg).await
                    } else if msg.has_timeout_msg() {
                        self.process_timeout_msg(&mut msg).await
                    } else if msg.has_sync_info() {
                        self.process_sync_info(&mut msg, peer_id).await
                    } else {
                        warn!("Unexpected msg from {}: {:?}", peer_id, msg);
                        continue;
                    };
                    if let Err(e) = r {
                        warn!("Failed to process msg {:?}: {:?}", msg, e)
                    }
                }
                Event::RpcRequest((peer_id, mut msg, callback)) => {
                    let r = if msg.has_request_block() {
                        self.process_request_block(&mut msg, callback).await
                    } else {
                        warn!("Unexpected RPC from {}: {:?}", peer_id, msg);
                        continue;
                    };
                    if let Err(e) = r {
                        warn!("Failed to process RPC {:?}: {:?}", msg, e)
                    }
                }
                Event::NewPeer(peer_id) => {
                    debug!("Peer {} connected", peer_id);
                }
                Event::LostPeer(peer_id) => {
                    debug!("Peer {} disconnected", peer_id);
                }
            }
        }
    }

    async fn process_proposal<'a>(&'a mut self, msg: &'a mut ConsensusMsg) -> failure::Result<()> {
        let proposal = ProposalUncheckedSignatures::<T>::from_proto(msg.take_proposal())?;
        let proposal = proposal
            .validate_signatures(self.epoch_mgr.validators().as_ref())?
            .verify_well_formed()?;
        debug!("Received proposal {}", proposal);
        self.proposal_tx.send(proposal).await?;
        Ok(())
    }

    async fn process_vote<'a>(&'a mut self, msg: &'a mut ConsensusMsg) -> failure::Result<()> {
        let vote = VoteMsg::from_proto(msg.take_vote())?;
        debug!("Received {}", vote);
        vote.verify(self.epoch_mgr.validators().as_ref())
            .map_err(|e| {
                security_log(SecurityEvent::InvalidConsensusVote)
                    .error(&e)
                    .data(&vote)
                    .log();
                e
            })?;
        self.vote_tx.send(vote).await?;
        Ok(())
    }

    async fn process_timeout_msg<'a>(
        &'a mut self,
        msg: &'a mut ConsensusMsg,
    ) -> failure::Result<()> {
        let timeout_msg = TimeoutMsg::from_proto(msg.take_timeout_msg())?;
        timeout_msg
            .verify(self.epoch_mgr.validators().as_ref())
            .map_err(|e| {
                security_log(SecurityEvent::InvalidConsensusRound)
                    .error(&e)
                    .data(&timeout_msg)
                    .log();
                e
            })?;
        self.timeout_msg_tx.send(timeout_msg).await?;
        Ok(())
    }

    async fn process_sync_info<'a>(
        &'a mut self,
        msg: &'a mut ConsensusMsg,
        peer: AccountAddress,
    ) -> failure::Result<()> {
        let sync_info = SyncInfo::from_proto(msg.take_sync_info())?;
        sync_info
            .verify(self.epoch_mgr.validators().as_ref())
            .map_err(|e| {
                security_log(SecurityEvent::InvalidSyncInfoMsg)
                    .error(&e)
                    .data(&sync_info)
                    .log();
                e
            })?;
        self.sync_info_tx.send((sync_info, peer)).await?;
        Ok(())
    }

    async fn process_request_block<'a>(
        &'a mut self,
        msg: &'a mut ConsensusMsg,
        callback: oneshot::Sender<Result<Bytes, RpcError>>,
    ) -> failure::Result<()> {
        let block_id = HashValue::from_slice(msg.get_request_block().get_block_id())?;
        let num_blocks = msg.get_request_block().get_num_blocks();
        debug!(
            "Received request_block RPC for {} blocks from {:?}",
            num_blocks, block_id
        );
        let (tx, rx) = oneshot::channel();
        let request = BlockRetrievalRequest {
            block_id,
            num_blocks,
            response_sender: tx,
        };
        self.block_request_tx.send(request).await?;
        let BlockRetrievalResponse { status, blocks } = rx.await?;
        let mut response_msg = ConsensusMsg::new();
        let mut response = RespondBlock::new();
        response.set_status(status);
        response.set_blocks(blocks.into_iter().map(IntoProto::into_proto).collect());
        response_msg.set_respond_block(response);
        let response_data = Bytes::from(
            response_msg
                .write_to_bytes()
                .expect("fail to serialize proto"),
        );
        callback
            .send(Ok(response_data))
            .map_err(|_| format_err!("handling inbound rpc call timed out"))
    }
}

// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{chained_bft::epoch_manager::EpochManager, counters};
use bytes::Bytes;
use channel;
use consensus_types::{
    block::Block,
    common::{Author, Payload},
    proposal_msg::{ProposalMsg, ProposalUncheckedSignatures},
    sync_info::SyncInfo,
    timeout_msg::TimeoutMsg,
    vote_msg::VoteMsg,
};
use crypto::HashValue;
use failure::{self, ResultExt};
use futures::{channel::oneshot, stream::select, SinkExt, Stream, StreamExt, TryStreamExt};
use libra_types::account_address::AccountAddress;
use logger::prelude::*;
use network::{
    proto::{
        BlockRetrievalStatus, ConsensusMsg, ConsensusMsg_oneof, Proposal, RequestBlock,
        RespondBlock, SyncInfo as SyncInfoProto, TimeoutMsg as TimeoutMsgProto, Vote,
    },
    validator_network::{ConsensusNetworkEvents, ConsensusNetworkSender, Event, RpcError},
};
use prost_ext::MessageExt;
use std::{
    convert::TryFrom,
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
            self.status != BlockRetrievalStatus::Succeeded
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
    pub proposals: channel::Receiver<ProposalMsg<T>>,
    pub votes: channel::Receiver<VoteMsg>,
    pub block_retrieval: channel::Receiver<BlockRetrievalRequest<T>>,
    pub timeout_msgs: channel::Receiver<TimeoutMsg>,
    pub sync_info_msgs: channel::Receiver<(SyncInfo, AccountAddress)>,
}

/// Implements the actual networking support for all consensus messaging.
pub struct ConsensusNetworkImpl {
    author: Author,
    network_sender: ConsensusNetworkSender,
    network_events: Option<ConsensusNetworkEvents>,
    // Self sender and self receivers provide a shortcut for sending the messages to itself.
    // (self sending is not supported by the networking API).
    // Note that we do not support self rpc requests as it might cause infinite recursive calls.
    self_sender: channel::Sender<failure::Result<Event<ConsensusMsg>>>,
    self_receiver: Option<channel::Receiver<failure::Result<Event<ConsensusMsg>>>>,
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
        let (self_sender, self_receiver) = channel::new(1_024, &counters::PENDING_SELF_MESSAGES);
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
        let (proposal_tx, proposal_rx) = channel::new(1_024, &counters::PENDING_PROPOSAL);
        let (vote_tx, vote_rx) = channel::new(1_024, &counters::PENDING_VOTES);
        let (block_request_tx, block_request_rx) =
            channel::new(1_024, &counters::PENDING_BLOCK_REQUESTS);
        let (timeout_msg_tx, timeout_msg_rx) =
            channel::new(1_024, &counters::PENDING_NEW_ROUND_MESSAGES);
        let (sync_info_tx, sync_info_rx) = channel::new(1_024, &counters::PENDING_SYNC_INFO_MSGS);
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
            .run(),
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
        let mut req_msg = RequestBlock::default();
        req_msg.block_id = block_id.to_vec();
        req_msg.num_blocks = num_blocks;
        counters::BLOCK_RETRIEVAL_COUNT.inc_by(num_blocks as i64);
        let pre_retrieval_instant = Instant::now();

        let res_block = self
            .network_sender
            .request_block(from, req_msg, timeout)
            .await?;
        let mut blocks = vec![];
        let status = res_block.status();
        for block in res_block.blocks.into_iter() {
            match Block::try_from(block) {
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
        let response = BlockRetrievalResponse { status, blocks };
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
        let msg = ConsensusMsg {
            message: Some(ConsensusMsg_oneof::Proposal(proposal.into())),
        };
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
        let msg = ConsensusMsg {
            message: Some(ConsensusMsg_oneof::Vote(vote_msg.into())),
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

    /// Broadcasts timeout message to all validators
    pub async fn broadcast_timeout_msg(&mut self, timeout_msg: TimeoutMsg) {
        let msg = ConsensusMsg {
            message: Some(ConsensusMsg_oneof::TimeoutMsg(timeout_msg.into())),
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
        let msg = ConsensusMsg {
            message: Some(ConsensusMsg_oneof::SyncInfo(sync_info.into())),
        };
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
    proposal_tx: channel::Sender<ProposalMsg<T>>,
    vote_tx: channel::Sender<VoteMsg>,
    block_request_tx: channel::Sender<BlockRetrievalRequest<T>>,
    timeout_msg_tx: channel::Sender<TimeoutMsg>,
    sync_info_tx: channel::Sender<(SyncInfo, AccountAddress)>,
    all_events: S,
    epoch_mgr: Arc<EpochManager>,
}

impl<T, S> NetworkTask<T, S>
where
    S: Stream<Item = failure::Result<Event<ConsensusMsg>>> + Unpin,
    T: Payload,
{
    pub async fn run(mut self) {
        use ConsensusMsg_oneof::*;
        while let Some(Ok(message)) = self.all_events.next().await {
            match message {
                Event::Message((peer_id, msg)) => {
                    let msg = match msg.message {
                        Some(msg) => msg,
                        None => {
                            warn!("Unexpected msg from {}: {:?}", peer_id, msg);
                            continue;
                        }
                    };

                    let r = match msg.clone() {
                        Proposal(proposal) => self.process_proposal(proposal).await.map_err(|e| {
                            security_log(SecurityEvent::InvalidConsensusProposal)
                                .error(&e)
                                .data(&msg)
                                .log();
                            e
                        }),
                        Vote(vote) => self.process_vote(vote).await,
                        TimeoutMsg(timeout_msg) => self.process_timeout_msg(timeout_msg).await,
                        SyncInfo(sync_info) => self.process_sync_info(sync_info, peer_id).await,
                        _ => {
                            warn!("Unexpected msg from {}: {:?}", peer_id, msg);
                            continue;
                        }
                    };
                    if let Err(e) = r {
                        warn!("Failed to process msg {:?}", e)
                    }
                }
                Event::RpcRequest((peer_id, msg, callback)) => {
                    let r = match msg.message {
                        Some(RequestBlock(request)) => {
                            self.process_request_block(request, callback).await
                        }
                        _ => {
                            warn!("Unexpected RPC from {}: {:?}", peer_id, msg);
                            continue;
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
                    debug!("Peer {} disconnected", peer_id);
                }
            }
        }
    }

    async fn process_proposal(&mut self, proposal: Proposal) -> failure::Result<()> {
        let proposal = ProposalUncheckedSignatures::<T>::try_from(proposal)?;
        let proposal = proposal
            .validate_signatures(self.epoch_mgr.validators().as_ref())?
            .verify_well_formed()?;
        debug!("Received proposal {}", proposal);
        self.proposal_tx.try_send(proposal)?;
        Ok(())
    }

    async fn process_vote(&mut self, vote: Vote) -> failure::Result<()> {
        let vote = VoteMsg::try_from(vote)?;
        debug!("Received {}", vote);
        vote.verify(self.epoch_mgr.validators().as_ref())
            .map_err(|e| {
                security_log(SecurityEvent::InvalidConsensusVote)
                    .error(&e)
                    .data(&vote)
                    .log();
                e
            })?;
        self.vote_tx.try_send(vote)?;
        Ok(())
    }

    async fn process_timeout_msg(&mut self, timeout_msg: TimeoutMsgProto) -> failure::Result<()> {
        let timeout_msg = TimeoutMsg::try_from(timeout_msg)?;
        timeout_msg
            .verify(self.epoch_mgr.validators().as_ref())
            .map_err(|e| {
                security_log(SecurityEvent::InvalidConsensusRound)
                    .error(&e)
                    .data(&timeout_msg)
                    .log();
                e
            })?;
        self.timeout_msg_tx.try_send(timeout_msg)?;
        Ok(())
    }

    async fn process_sync_info(
        &mut self,
        sync_info: SyncInfoProto,
        peer: AccountAddress,
    ) -> failure::Result<()> {
        let sync_info = SyncInfo::try_from(sync_info)?;
        sync_info
            .verify(self.epoch_mgr.validators().as_ref())
            .map_err(|e| {
                security_log(SecurityEvent::InvalidSyncInfoMsg)
                    .error(&e)
                    .data(&sync_info)
                    .log();
                e
            })?;
        self.sync_info_tx.try_send((sync_info, peer))?;
        Ok(())
    }

    async fn process_request_block(
        &mut self,
        request: RequestBlock,
        callback: oneshot::Sender<Result<Bytes, RpcError>>,
    ) -> failure::Result<()> {
        let block_id = HashValue::from_slice(&request.block_id[..])?;
        let num_blocks = request.num_blocks;
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
        self.block_request_tx.try_send(request)?;
        let BlockRetrievalResponse { status, blocks } = rx.await?;
        let mut response = RespondBlock::default();
        response.set_status(status);
        response.blocks = blocks.into_iter().map(Into::into).collect();
        let response_msg = ConsensusMsg {
            message: Some(ConsensusMsg_oneof::RespondBlock(response)),
        };
        let response_data = response_msg.to_bytes()?;
        callback
            .send(Ok(response_data))
            .map_err(|_| format_err!("handling inbound rpc call timed out"))
    }
}

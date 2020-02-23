// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::network_interface::{ConsensusNetworkEvents, ConsensusNetworkSender},
    counters,
};
use anyhow::{anyhow, ensure, Error};
use bytes::Bytes;
use channel::{self, libra_channel, message_queues::QueueStyle};
use consensus_types::{
    block_retrieval::{BlockRetrievalRequest, BlockRetrievalResponse},
    common::{Author, Payload},
    epoch_retrieval::EpochRetrievalRequest,
    proposal_msg::{ProposalMsg, ProposalUncheckedSignatures},
    sync_info::SyncInfo,
    vote_msg::VoteMsg,
};
use futures::{channel::oneshot, stream::select, SinkExt, Stream, StreamExt, TryStreamExt};
use libra_logger::prelude::*;
use libra_types::{
    account_address::AccountAddress,
    crypto_proxies::{
        EpochInfo, LedgerInfoWithSignatures, ValidatorChangeProof, ValidatorVerifier,
    },
    validator_change::VerifierType,
};
use network::{
    proto::ConsensusMsg,
    validator_network::{Event, RpcError},
};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    convert::TryFrom,
    marker::PhantomData,
    num::NonZeroUsize,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

#[derive(Deserialize, Serialize)]
pub enum ConsensusTypes<T> {
    BlockRetrievalRequest(Box<BlockRetrievalRequest>),
    #[serde(bound = "T: Payload")]
    BlockRetrievalResponse(Box<BlockRetrievalResponse<T>>),
    EpochRetrievalRequest(Box<EpochRetrievalRequest>),
    #[serde(bound = "T: Payload")]
    ProposalMsg(Box<ProposalMsg<T>>),
    SyncInfo(Box<SyncInfo>),
    ValidatorChangeProof(Box<ValidatorChangeProof>),
    VoteMsg(Box<VoteMsg>),
}

impl<T: Payload> TryFrom<&ConsensusMsg> for ConsensusTypes<T> {
    type Error = Error;

    fn try_from(value: &ConsensusMsg) -> Result<Self, Self::Error> {
        Ok(lcs::from_bytes(&value.message)?)
    }
}

impl<T: Payload> TryFrom<ConsensusTypes<T>> for ConsensusMsg {
    type Error = Error;

    fn try_from(value: ConsensusTypes<T>) -> Result<Self, Self::Error> {
        Ok(Self {
            message: lcs::to_bytes(&value)?,
        })
    }
}

/// The block retrieval request is used internally for implementing RPC: the callback is executed
/// for carrying the response
#[derive(Debug)]
pub struct IncomingBlockRetrievalRequest {
    pub req: BlockRetrievalRequest,
    pub response_sender: oneshot::Sender<Result<Bytes, RpcError>>,
}

/// Just a convenience struct to keep all the network proxy receiving queues in one place.
/// Will be returned by the networking trait upon startup.
pub struct NetworkReceivers<T> {
    pub proposals: libra_channel::Receiver<AccountAddress, ProposalMsg<T>>,
    pub votes: libra_channel::Receiver<AccountAddress, VoteMsg>,
    pub block_retrieval: libra_channel::Receiver<AccountAddress, IncomingBlockRetrievalRequest>,
    pub sync_info_msgs: libra_channel::Receiver<AccountAddress, (SyncInfo, AccountAddress)>,
    pub epoch_change: libra_channel::Receiver<AccountAddress, LedgerInfoWithSignatures>,
    pub different_epoch: libra_channel::Receiver<AccountAddress, (u64, AccountAddress)>,
    pub epoch_retrieval:
        libra_channel::Receiver<AccountAddress, (EpochRetrievalRequest, AccountAddress)>,
}

impl<T> NetworkReceivers<T> {
    pub fn clear_prev_epoch_msgs(&mut self) {
        // clear all the channels that are relevant for the previous epoch event processor
        self.proposals.clear();
        self.votes.clear();
        self.block_retrieval.clear();
        self.sync_info_msgs.clear();
    }
}

/// Implements the actual networking support for all consensus messaging.
#[derive(Clone)]
pub struct NetworkSender<T> {
    author: Author,
    network_sender: ConsensusNetworkSender,
    // Self sender and self receivers provide a shortcut for sending the messages to itself.
    // (self sending is not supported by the networking API).
    // Note that we do not support self rpc requests as it might cause infinite recursive calls.
    self_sender: channel::Sender<anyhow::Result<Event<ConsensusMsg>>>,
    validators: Arc<ValidatorVerifier>,
    marker: PhantomData<T>,
}

impl<T: Payload> NetworkSender<T> {
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
            marker: PhantomData,
        }
    }

    /// Tries to retrieve num of blocks backwards starting from id from the given peer: the function
    /// returns a future that is fulfilled with BlockRetrievalResponse.
    pub async fn request_block(
        &mut self,
        retrieval_request: BlockRetrievalRequest,
        from: Author,
        timeout: Duration,
    ) -> anyhow::Result<BlockRetrievalResponse<T>> {
        ensure!(from != self.author, "Retrieve block from self");
        counters::BLOCK_RETRIEVAL_COUNT.inc_by(retrieval_request.num_blocks() as i64);
        let pre_retrieval_instant = Instant::now();
        let msg = ConsensusMsg::try_from(ConsensusTypes::BlockRetrievalRequest::<T>(Box::new(
            retrieval_request.clone(),
        )))?;
        let response_msg = self.network_sender.send_rpc(from, msg, timeout).await?;
        counters::BLOCK_RETRIEVAL_DURATION_S.observe_duration(pre_retrieval_instant.elapsed());
        let response = match ConsensusTypes::try_from(&response_msg) {
            Ok(ConsensusTypes::BlockRetrievalResponse(resp)) => *resp,
            _ => return Err(anyhow!("Invalid response to request")),
        };
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
    pub async fn broadcast_proposal(&mut self, proposal: ProposalMsg<T>) {
        let msg = match ConsensusMsg::try_from(ConsensusTypes::ProposalMsg(Box::new(proposal))) {
            Ok(msg) => msg,
            Err(e) => {
                warn!("Failed to serialize ProposalMsg: {:?}", e);
                return;
            }
        };
        counters::UNWRAPPED_PROPOSAL_SIZE_BYTES.observe(msg.message.len() as f64);
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
        if let Err(err) = self.network_sender.send_to_many(other_validators, msg) {
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
        let msg = match ConsensusMsg::try_from(ConsensusTypes::VoteMsg::<T>(Box::new(vote_msg))) {
            Ok(msg) => msg,
            Err(e) => {
                warn!("Failed to serialize VoteMsg: {:?}", e);
                return;
            }
        };
        for peer in recipients {
            if self.author == peer {
                let self_msg = Event::Message((self.author, msg.clone()));
                if let Err(err) = self_sender.send(Ok(self_msg)).await {
                    error!("Error delivering a self vote: {:?}", err);
                }
                continue;
            }
            if let Err(e) = network_sender.send_to(peer, msg.clone()) {
                error!("Failed to send a vote to peer {:?}: {:?}", peer, e);
            }
        }
    }

    /// Broadcasts vote message to all validators
    pub async fn broadcast_vote(&mut self, vote_msg: VoteMsg) {
        let msg = match ConsensusMsg::try_from(ConsensusTypes::VoteMsg::<T>(Box::new(vote_msg))) {
            Ok(msg) => msg,
            Err(e) => {
                warn!("Failed to serialize VoteMsg: {:?}", e);
                return;
            }
        };
        self.broadcast(msg).await
    }

    /// Sends the given sync info to the given author.
    /// The future is fulfilled as soon as the message is added to the internal network channel
    /// (does not indicate whether the message is delivered or sent out).
    pub fn send_sync_info(&self, sync_info: SyncInfo, recipient: Author) {
        if recipient == self.author {
            error!("An attempt to deliver sync info msg to itself: ignore.");
            return;
        }
        let msg = match ConsensusMsg::try_from(ConsensusTypes::SyncInfo::<T>(Box::new(sync_info))) {
            Ok(msg) => msg,
            Err(e) => {
                warn!("Failed to serialize SyncInfo: {:?}", e);
                return;
            }
        };
        let mut network_sender = self.network_sender.clone();
        if let Err(e) = network_sender.send_to(recipient, msg) {
            warn!(
                "Failed to send a sync info msg to peer {:?}: {:?}",
                recipient, e
            );
        }
    }

    /// Broadcast about epoch changes with proof to the current validator set (including self)
    /// when we commit the reconfiguration block
    pub async fn broadcast_epoch_change(&mut self, proof: ValidatorChangeProof) {
        let msg = ConsensusTypes::ValidatorChangeProof::<T>(Box::new(proof));
        let msg = match ConsensusMsg::try_from(msg) {
            Ok(msg) => msg,
            Err(e) => {
                warn!("Failed to serialize ValidatorChangeProof: {:?}", e);
                return;
            }
        };
        self.broadcast(msg).await
    }

    pub async fn notify_epoch_change(&mut self, proof: ValidatorChangeProof) {
        let msg = ConsensusTypes::ValidatorChangeProof::<T>(Box::new(proof));
        let msg = match ConsensusMsg::try_from(msg) {
            Ok(msg) => msg,
            Err(e) => {
                warn!("Failed to serialize ValidatorChangeProof: {:?}", e);
                return;
            }
        };
        let self_msg = Event::Message((self.author, msg));
        if let Err(e) = self.self_sender.send(Ok(self_msg)).await {
            warn!("Failed to notify to self an epoch change {:?}", e);
        }
    }
}

pub struct NetworkTask<T> {
    epoch_info: Arc<RwLock<EpochInfo>>,
    proposal_tx: libra_channel::Sender<AccountAddress, ProposalMsg<T>>,
    vote_tx: libra_channel::Sender<AccountAddress, VoteMsg>,
    block_request_tx: libra_channel::Sender<AccountAddress, IncomingBlockRetrievalRequest>,
    sync_info_tx: libra_channel::Sender<AccountAddress, (SyncInfo, AccountAddress)>,
    epoch_change_tx: libra_channel::Sender<AccountAddress, LedgerInfoWithSignatures>,
    different_epoch_tx: libra_channel::Sender<AccountAddress, (u64, AccountAddress)>,
    epoch_retrieval_tx:
        libra_channel::Sender<AccountAddress, (EpochRetrievalRequest, AccountAddress)>,
    all_events: Box<dyn Stream<Item = anyhow::Result<Event<ConsensusMsg>>> + Send + Unpin>,
}

impl<T: Payload> NetworkTask<T> {
    /// Establishes the initial connections with the peers and returns the receivers.
    pub fn new(
        epoch_info: Arc<RwLock<EpochInfo>>,
        network_events: ConsensusNetworkEvents,
        self_receiver: channel::Receiver<anyhow::Result<Event<ConsensusMsg>>>,
    ) -> (NetworkTask<T>, NetworkReceivers<T>) {
        let (proposal_tx, proposal_rx) = libra_channel::new(
            QueueStyle::LIFO,
            NonZeroUsize::new(1).unwrap(),
            Some(&counters::PROPOSAL_CHANNEL_MSGS),
        );
        let (vote_tx, vote_rx) = libra_channel::new(
            QueueStyle::LIFO,
            NonZeroUsize::new(1).unwrap(),
            Some(&counters::VOTES_CHANNEL_MSGS),
        );
        let (block_request_tx, block_request_rx) = libra_channel::new(
            QueueStyle::LIFO,
            NonZeroUsize::new(1).unwrap(),
            Some(&counters::BLOCK_RETRIEVAL_CHANNEL_MSGS),
        );
        let (sync_info_tx, sync_info_rx) = libra_channel::new(
            QueueStyle::LIFO,
            NonZeroUsize::new(1).unwrap(),
            Some(&counters::SYNC_INFO_CHANNEL_MSGS),
        );
        let (epoch_change_tx, epoch_change_rx) = libra_channel::new(
            QueueStyle::LIFO,
            NonZeroUsize::new(1).unwrap(),
            Some(&counters::EPOCH_CHANGE_CHANNEL_MSGS),
        );
        let (different_epoch_tx, different_epoch_rx) =
            libra_channel::new(QueueStyle::LIFO, NonZeroUsize::new(1).unwrap(), None);
        let (epoch_retrieval_tx, epoch_retrieval_rx) =
            libra_channel::new(QueueStyle::LIFO, NonZeroUsize::new(1).unwrap(), None);
        let network_events = network_events.map_err(Into::<anyhow::Error>::into);
        let all_events = Box::new(select(network_events, self_receiver));
        (
            NetworkTask {
                epoch_info,
                proposal_tx,
                vote_tx,
                block_request_tx,
                sync_info_tx,
                epoch_change_tx,
                different_epoch_tx,
                epoch_retrieval_tx,
                all_events,
            },
            NetworkReceivers {
                proposals: proposal_rx,
                votes: vote_rx,
                block_retrieval: block_request_rx,
                sync_info_msgs: sync_info_rx,
                epoch_change: epoch_change_rx,
                different_epoch: different_epoch_rx,
                epoch_retrieval: epoch_retrieval_rx,
            },
        )
    }

    fn epoch(&self) -> u64 {
        self.epoch_info.read().unwrap().epoch
    }

    pub async fn start(mut self) {
        while let Some(Ok(message)) = self.all_events.next().await {
            match message {
                Event::Message((peer_id, msg)) => {
                    let input = match ConsensusTypes::<T>::try_from(&msg) {
                        Ok(input) => input,
                        Err(_) => {
                            warn!("Unexpected msg from {}: {:?}", peer_id, msg);
                            continue;
                        }
                    };

                    let r = match input {
                        ConsensusTypes::ProposalMsg(proposal) => self
                            .process_proposal(peer_id, *proposal)
                            .await
                            .map_err(|e| {
                                security_log(SecurityEvent::InvalidConsensusProposal)
                                    .error(&e)
                                    .data(&msg)
                                    .log();
                                e
                            }),
                        ConsensusTypes::SyncInfo(sync_info) => {
                            self.process_sync_info(*sync_info, peer_id).await
                        }
                        ConsensusTypes::VoteMsg(vote_msg) => {
                            self.process_vote(peer_id, *vote_msg).await
                        }
                        ConsensusTypes::ValidatorChangeProof(proof) => {
                            self.process_epoch_change(peer_id, *proof).await
                        }
                        ConsensusTypes::EpochRetrievalRequest(request) => {
                            self.process_epoch_request(peer_id, *request).await
                        }
                        _ => {
                            warn!("Unexpected msg from {}: {:?}", peer_id, msg);
                            continue;
                        }
                    };
                    if let Err(e) = r {
                        warn!("Failed to process msg {}", e)
                    }
                }
                Event::RpcRequest((peer_id, msg, callback)) => {
                    let input = match lcs::from_bytes(&msg.message) {
                        Ok(input) => input,
                        Err(_) => {
                            warn!("Unexpected msg from {}: {:?}", peer_id, msg);
                            continue;
                        }
                    };

                    let r = match input {
                        ConsensusTypes::BlockRetrievalRequest::<T>(request) => {
                            self.process_request_block(peer_id, *request, callback)
                                .await
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

    async fn process_proposal(
        &mut self,
        peer_id: AccountAddress,
        proposal: ProposalMsg<T>,
    ) -> anyhow::Result<()> {
        let proposal = ProposalUncheckedSignatures::new(proposal);
        if proposal.epoch() != self.epoch() {
            return self
                .different_epoch_tx
                .push(peer_id, (proposal.epoch(), peer_id));
        }
        let proposal = proposal
            .validate_signatures(&self.epoch_info.read().unwrap().verifier)?
            .verify_well_formed()?;
        ensure!(
            proposal.proposal().author() == Some(peer_id),
            "proposal received must be from the sending peer"
        );
        debug!("Received proposal {}", proposal);
        self.proposal_tx.push(peer_id, proposal)
    }

    async fn process_vote(
        &mut self,
        peer_id: AccountAddress,
        vote_msg: VoteMsg,
    ) -> anyhow::Result<()> {
        ensure!(
            vote_msg.vote().author() == peer_id,
            "vote received must be from the sending peer"
        );

        if vote_msg.epoch() != self.epoch() {
            return self
                .different_epoch_tx
                .push(peer_id, (vote_msg.epoch(), peer_id));
        }

        debug!("Received {}", vote_msg);
        vote_msg
            .verify(&self.epoch_info.read().unwrap().verifier)
            .map_err(|e| {
                security_log(SecurityEvent::InvalidConsensusVote)
                    .error(&e)
                    .data(&vote_msg)
                    .log();
                e
            })?;
        self.vote_tx.push(peer_id, vote_msg)
    }

    async fn process_sync_info(
        &mut self,
        sync_info: SyncInfo,
        peer_id: AccountAddress,
    ) -> anyhow::Result<()> {
        match sync_info.epoch().cmp(&self.epoch()) {
            Ordering::Equal => {
                // SyncInfo verification is postponed to the moment it's actually used.
                self.sync_info_tx.push(peer_id, (sync_info, peer_id))
            }
            Ordering::Less | Ordering::Greater => self
                .different_epoch_tx
                .push(peer_id, (sync_info.epoch(), peer_id)),
        }
    }

    async fn process_request_block(
        &mut self,
        peer_id: AccountAddress,
        req: BlockRetrievalRequest,
        callback: oneshot::Sender<Result<Bytes, RpcError>>,
    ) -> anyhow::Result<()> {
        debug!("Received block retrieval request {}", req);
        let req_with_callback = IncomingBlockRetrievalRequest {
            req,
            response_sender: callback,
        };
        self.block_request_tx.push(peer_id, req_with_callback)
    }

    async fn process_epoch_change(
        &mut self,
        peer_id: AccountAddress,
        proof: ValidatorChangeProof,
    ) -> anyhow::Result<()> {
        let msg_epoch = proof.epoch()?;
        match msg_epoch.cmp(&self.epoch()) {
            Ordering::Equal => {
                let verifier =
                    VerifierType::TrustedVerifier(self.epoch_info.read().unwrap().clone());
                let target_ledger_info = proof.verify(&verifier)?;
                debug!(
                    "Received epoch change to {}",
                    target_ledger_info.ledger_info().epoch() + 1
                );
                self.epoch_change_tx.push(peer_id, target_ledger_info)
            }
            Ordering::Less | Ordering::Greater => {
                self.different_epoch_tx.push(peer_id, (msg_epoch, peer_id))
            }
        }
    }

    async fn process_epoch_request(
        &mut self,
        peer_id: AccountAddress,
        request: EpochRetrievalRequest,
    ) -> anyhow::Result<()> {
        debug!(
            "Received epoch retrieval from peer {}, start epoch {}, end epoch {}",
            peer_id, request.start_epoch, request.end_epoch
        );
        match request.end_epoch.cmp(&self.epoch()) {
            Ordering::Less | Ordering::Equal => {
                self.epoch_retrieval_tx.push(peer_id, (request, peer_id))
            }
            Ordering::Greater => {
                warn!("Received EpochRetrievalRequest beyond what we have locally");
                Ok(())
            }
        }
    }
}

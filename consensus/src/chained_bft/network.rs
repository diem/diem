// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::counters;
use bytes::Bytes;
use channel::{
    self, libra_channel,
    message_queues::{self, QueueStyle},
};
use consensus_types::block_retrieval::{BlockRetrievalRequest, BlockRetrievalResponse};
use consensus_types::{
    common::{Author, Payload},
    proposal_msg::{ProposalMsg, ProposalUncheckedSignatures},
    sync_info::SyncInfo,
    vote_msg::VoteMsg,
};
use failure::{self};
use futures::{channel::oneshot, stream::select, SinkExt, Stream, StreamExt, TryStreamExt};
use libra_logger::prelude::*;
use libra_prost_ext::MessageExt;
use libra_types::account_address::AccountAddress;
use libra_types::crypto_proxies::{LedgerInfoWithSignatures, ValidatorVerifier};
use libra_types::proto::types::LedgerInfoWithSignatures as LedgerInfoWithSignaturesProto;
use network::{
    proto::{
        ConsensusMsg, ConsensusMsg_oneof, Proposal, RequestBlock, RespondBlock,
        SyncInfo as SyncInfoProto, VoteMsg as VoteMsgProto,
    },
    validator_network::{ConsensusNetworkEvents, ConsensusNetworkSender, Event, RpcError},
};
use std::{
    convert::{TryFrom, TryInto},
    sync::Arc,
    time::{Duration, Instant},
};

/// The block retrieval request is used internally for implementing RPC: the callback is executed
/// for carrying the response
#[derive(Debug)]
pub struct IncomingBlockRetrievalRequest<T> {
    pub req: BlockRetrievalRequest,
    pub response_sender: oneshot::Sender<BlockRetrievalResponse<T>>,
}

/// Just a convenience struct to keep all the network proxy receiving queues in one place.
/// Will be returned by the networking trait upon startup.
pub struct NetworkReceivers<T> {
    pub proposals: libra_channel::Receiver<AccountAddress, ProposalMsg<T>>,
    pub votes: libra_channel::Receiver<AccountAddress, VoteMsg>,
    pub block_retrieval: libra_channel::Receiver<AccountAddress, IncomingBlockRetrievalRequest<T>>,
    pub sync_info_msgs: libra_channel::Receiver<AccountAddress, (SyncInfo, AccountAddress)>,
    pub epoch_change: libra_channel::Receiver<AccountAddress, LedgerInfoWithSignatures>,
}

/// Implements the actual networking support for all consensus messaging.
#[derive(Clone)]
pub struct NetworkSender {
    author: Author,
    network_sender: ConsensusNetworkSender,
    // Self sender and self receivers provide a shortcut for sending the messages to itself.
    // (self sending is not supported by the networking API).
    // Note that we do not support self rpc requests as it might cause infinite recursive calls.
    self_sender: channel::Sender<failure::Result<Event<ConsensusMsg>>>,
    validators: Arc<ValidatorVerifier>,
}

impl NetworkSender {
    pub fn new(
        author: Author,
        network_sender: ConsensusNetworkSender,
        self_sender: channel::Sender<failure::Result<Event<ConsensusMsg>>>,
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
    ) -> failure::Result<BlockRetrievalResponse<T>> {
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
        response.verify(
            retrieval_request.block_id(),
            retrieval_request.num_blocks(),
            self.validators.as_ref(),
        )?;
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
        let proposal = match proposal.try_into() {
            Ok(bytes) => bytes,
            Err(e) => {
                warn!("Fail to serialize VoteMsg: {:?}", e);
                return;
            }
        };
        let msg = ConsensusMsg {
            message: Some(ConsensusMsg_oneof::Proposal(proposal)),
        };
        self.broadcast(msg).await
    }

    async fn broadcast(&mut self, msg: ConsensusMsg) {
        let msg_raw = msg.to_bytes().unwrap();
        for peer in self.validators.get_ordered_account_addresses() {
            if self.author == peer {
                let self_msg = Event::Message((self.author, msg.clone()));
                if let Err(err) = self.self_sender.send(Ok(self_msg)).await {
                    error!("Error delivering a self proposal: {:?}", err);
                }
                continue;
            }
            if let Err(err) = self.network_sender.send_bytes(peer, msg_raw.clone()).await {
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
    #[allow(dead_code)]
    pub async fn send_epoch_change(&mut self, ledger_info: LedgerInfoWithSignatures) {
        let msg = ConsensusMsg {
            message: Some(ConsensusMsg_oneof::LedgerInfo(ledger_info.into())),
        };
        self.broadcast(msg).await
    }
}

pub struct NetworkTask<T> {
    proposal_tx: libra_channel::Sender<AccountAddress, ProposalMsg<T>>,
    vote_tx: libra_channel::Sender<AccountAddress, VoteMsg>,
    block_request_tx: libra_channel::Sender<AccountAddress, IncomingBlockRetrievalRequest<T>>,
    sync_info_tx: libra_channel::Sender<AccountAddress, (SyncInfo, AccountAddress)>,
    epoch_change_tx: libra_channel::Sender<AccountAddress, LedgerInfoWithSignatures>,
    all_events: Box<dyn Stream<Item = failure::Result<Event<ConsensusMsg>>> + Send + Unpin>,
    validators: Arc<ValidatorVerifier>,
}

impl<T: Payload> NetworkTask<T> {
    /// Establishes the initial connections with the peers and returns the receivers.
    pub fn new(
        network_events: ConsensusNetworkEvents,
        self_receiver: channel::Receiver<failure::Result<Event<ConsensusMsg>>>,
        validators: Arc<ValidatorVerifier>,
    ) -> (NetworkTask<T>, NetworkReceivers<T>) {
        let (proposal_tx, proposal_rx) = libra_channel::new(
            QueueStyle::LIFO,
            1,
            Some(message_queues::Counters {
                dropped_msgs_counter: &counters::PROPOSAL_DROPPED_MSGS,
                enqueued_msgs_counter: &counters::PROPOSAL_ENQUEUED_MSGS,
                dequeued_msgs_counter: &counters::PROPOSAL_DEQUEUED_MSGS,
            }),
        );
        let (vote_tx, vote_rx) = libra_channel::new(
            QueueStyle::LIFO,
            1,
            Some(message_queues::Counters {
                dropped_msgs_counter: &counters::VOTES_DROPPED_MSGS,
                enqueued_msgs_counter: &counters::VOTES_ENQUEUED_MSGS,
                dequeued_msgs_counter: &counters::VOTES_DEQUEUED_MSGS,
            }),
        );
        let (block_request_tx, block_request_rx) = libra_channel::new(
            QueueStyle::LIFO,
            1,
            Some(message_queues::Counters {
                dropped_msgs_counter: &counters::BLOCK_RETRIEVAL_DROPPED_MSGS,
                enqueued_msgs_counter: &counters::BLOCK_RETRIEVAL_ENQUEUED_MSGS,
                dequeued_msgs_counter: &counters::BLOCK_RETRIEVAL_DEQUEUED_MSGS,
            }),
        );
        let (sync_info_tx, sync_info_rx) = libra_channel::new(
            QueueStyle::LIFO,
            1,
            Some(message_queues::Counters {
                dropped_msgs_counter: &counters::SYNC_INFO_DROPPED_MSGS,
                enqueued_msgs_counter: &counters::SYNC_INFO_ENQUEUED_MSGS,
                dequeued_msgs_counter: &counters::SYNC_INFO_DEQUEUED_MSGS,
            }),
        );
        let (epoch_change_tx, epoch_change_rx) = libra_channel::new(
            QueueStyle::LIFO,
            1,
            Some(message_queues::Counters {
                dropped_msgs_counter: &counters::EPOCH_CHANGE_DROPPED_MSGS,
                enqueued_msgs_counter: &counters::EPOCH_CHANGE_ENQUEUED_MSGS,
                dequeued_msgs_counter: &counters::EPOCH_CHANGE_DEQUEUED_MSGS,
            }),
        );
        let network_events = network_events.map_err(Into::<failure::Error>::into);
        let all_events = Box::new(select(network_events, self_receiver));
        (
            NetworkTask {
                proposal_tx,
                vote_tx,
                block_request_tx,
                sync_info_tx,
                epoch_change_tx,
                all_events,
                validators,
            },
            NetworkReceivers {
                proposals: proposal_rx,
                votes: vote_rx,
                block_retrieval: block_request_rx,
                sync_info_msgs: sync_info_rx,
                epoch_change: epoch_change_rx,
            },
        )
    }

    pub async fn start(mut self) {
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
                        Proposal(proposal) => {
                            self.process_proposal(peer_id, proposal).await.map_err(|e| {
                                security_log(SecurityEvent::InvalidConsensusProposal)
                                    .error(&e)
                                    .data(&msg)
                                    .log();
                                e
                            })
                        }
                        VoteMsg(vote_msg) => self.process_vote(peer_id, vote_msg).await,
                        SyncInfo(sync_info) => self.process_sync_info(sync_info, peer_id).await,
                        LedgerInfo(ledger_info) => {
                            self.process_epoch_change(peer_id, ledger_info).await
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
                    let r = match msg.message {
                        Some(RequestBlock(request)) => {
                            self.process_request_block(peer_id, request, callback).await
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
        proposal: Proposal,
    ) -> failure::Result<()> {
        let proposal = ProposalUncheckedSignatures::<T>::try_from(proposal)?;
        let proposal = proposal
            .validate_signatures(self.validators.as_ref())?
            .verify_well_formed()?;
        debug!("Received proposal {}", proposal);
        Ok(self.proposal_tx.push(peer_id, proposal)?)
    }

    async fn process_vote(
        &mut self,
        peer_id: AccountAddress,
        vote_msg: VoteMsgProto,
    ) -> failure::Result<()> {
        let vote_msg = VoteMsg::try_from(vote_msg)?;
        debug!("Received {}", vote_msg);
        vote_msg
            .vote()
            .verify(self.validators.as_ref())
            .map_err(|e| {
                security_log(SecurityEvent::InvalidConsensusVote)
                    .error(&e)
                    .data(&vote_msg)
                    .log();
                e
            })?;
        Ok(self.vote_tx.push(peer_id, vote_msg)?)
    }

    async fn process_sync_info(
        &mut self,
        sync_info: SyncInfoProto,
        peer: AccountAddress,
    ) -> failure::Result<()> {
        let sync_info = SyncInfo::try_from(sync_info)?;
        sync_info.verify(self.validators.as_ref()).map_err(|e| {
            security_log(SecurityEvent::InvalidSyncInfoMsg)
                .error(&e)
                .data(&sync_info)
                .log();
            e
        })?;
        Ok(self.sync_info_tx.push(peer, (sync_info, peer))?)
    }

    async fn process_request_block(
        &mut self,
        peer_id: AccountAddress,
        request_msg: RequestBlock,
        callback: oneshot::Sender<Result<Bytes, RpcError>>,
    ) -> failure::Result<()> {
        let req = BlockRetrievalRequest::try_from(request_msg)?;
        debug!("Received block retrieval request {}", req);
        let (tx, rx) = oneshot::channel();
        let req_with_callback = IncomingBlockRetrievalRequest {
            req,
            response_sender: tx,
        };
        self.block_request_tx.push(peer_id, req_with_callback)?;
        let response = rx.await?;
        let response_serialized = RespondBlock::try_from(response)?;
        let response_msg = ConsensusMsg {
            message: Some(ConsensusMsg_oneof::RespondBlock(response_serialized)),
        };
        let response_data = response_msg.to_bytes()?;
        callback
            .send(Ok(response_data))
            .map_err(|_| format_err!("handling inbound rpc call timed out"))
    }

    async fn process_epoch_change(
        &mut self,
        peer_id: AccountAddress,
        ledger_info: LedgerInfoWithSignaturesProto,
    ) -> failure::Result<()> {
        let ledger_info = LedgerInfoWithSignatures::try_from(ledger_info)?;
        ledger_info.verify(&self.validators)?;
        let validators = match ledger_info.ledger_info().next_validator_set() {
            Some(v) => v.into(),
            None => bail!("Epoch change doesn't carry next validator set"),
        };
        self.validators = Arc::new(validators);
        self.epoch_change_tx
            .push(peer_id, ledger_info)
            .map_err(|e| format_err!("Failed to push to epoch_change channel: {}", e))
    }
}

// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    counters,
    network_interface::{ConsensusMsg, ConsensusNetworkEvents, ConsensusNetworkSender},
};
use anyhow::{anyhow, ensure};
use bytes::Bytes;
use channel::{self, libra_channel, message_queues::QueueStyle};
use consensus_types::{
    block_retrieval::{BlockRetrievalRequest, BlockRetrievalResponse, MAX_BLOCKS_PER_REQUEST},
    common::Author,
    sync_info::SyncInfo,
    vote_msg::VoteMsg,
};
use futures::{channel::oneshot, stream::select, SinkExt, Stream, StreamExt, TryStreamExt};
use libra_logger::prelude::*;
use libra_metrics::monitor;
use libra_types::{
    account_address::AccountAddress, epoch_change::EpochChangeProof,
    validator_verifier::ValidatorVerifier,
};
use network::protocols::{network::Event, rpc::error::RpcError};
use std::{
    mem::{discriminant, Discriminant},
    num::NonZeroUsize,
    time::Duration,
};

/// The block retrieval request is used internally for implementing RPC: the callback is executed
/// for carrying the response
#[derive(Debug)]
pub struct IncomingBlockRetrievalRequest {
    pub req: BlockRetrievalRequest,
    pub response_sender: oneshot::Sender<Result<Bytes, RpcError>>,
}

/// Just a convenience struct to keep all the network proxy receiving queues in one place.
/// Will be returned by the NetworkTask upon startup.
pub struct NetworkReceivers {
    /// Provide a LIFO buffer for each (Author, MessageType) key
    pub consensus_messages: libra_channel::Receiver<
        (AccountAddress, Discriminant<ConsensusMsg>),
        (AccountAddress, ConsensusMsg),
    >,
    pub block_retrieval: libra_channel::Receiver<AccountAddress, IncomingBlockRetrievalRequest>,
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
    validators: ValidatorVerifier,
}

impl NetworkSender {
    pub fn new(
        author: Author,
        network_sender: ConsensusNetworkSender,
        self_sender: channel::Sender<anyhow::Result<Event<ConsensusMsg>>>,
        validators: ValidatorVerifier,
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
    pub async fn request_block(
        &mut self,
        retrieval_request: BlockRetrievalRequest,
        from: Author,
        timeout: Duration,
    ) -> anyhow::Result<BlockRetrievalResponse> {
        ensure!(from != self.author, "Retrieve block from self");
        let msg = ConsensusMsg::BlockRetrievalRequest(Box::new(retrieval_request.clone()));
        let response_msg = monitor!(
            "block_retrieval",
            self.network_sender.send_rpc(from, msg, timeout).await?
        );
        let response = match response_msg {
            ConsensusMsg::BlockRetrievalResponse(resp) => *resp,
            _ => return Err(anyhow!("Invalid response to request")),
        };
        response
            .verify(
                retrieval_request.block_id(),
                retrieval_request.num_blocks(),
                &self.validators,
            )
            .map_err(|e| {
                error!(
                    SecurityEvent::InvalidRetrievedBlock,
                    StructuredLogEntry::default()
                        .data("request_block_reponse", &response)
                        .data_display("error", &e)
                );
                e
            })?;

        Ok(response)
    }

    /// Tries to send the given msg to all the participants.
    ///
    /// The future is fulfilled as soon as the message put into the mpsc channel to network
    /// internal(to provide back pressure), it does not indicate the message is delivered or sent
    /// out. It does not give indication about when the message is delivered to the recipients,
    /// as well as there is no indication about the network failures.
    pub async fn broadcast(&mut self, msg: ConsensusMsg) {
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
        let msg = ConsensusMsg::VoteMsg(Box::new(vote_msg));
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

    /// Sends the given sync info to the given author.
    /// The future is fulfilled as soon as the message is added to the internal network channel
    /// (does not indicate whether the message is delivered or sent out).
    pub fn send_sync_info(&self, sync_info: SyncInfo, recipient: Author) {
        let msg = ConsensusMsg::SyncInfo(Box::new(sync_info));
        let mut network_sender = self.network_sender.clone();
        if let Err(e) = network_sender.send_to(recipient, msg) {
            warn!(
                "Failed to send a sync info msg to peer {:?}: {:?}",
                recipient, e
            );
        }
    }

    pub async fn notify_epoch_change(&mut self, proof: EpochChangeProof) {
        let msg = ConsensusMsg::EpochChangeProof(Box::new(proof));
        let self_msg = Event::Message((self.author, msg));
        if let Err(e) = self.self_sender.send(Ok(self_msg)).await {
            warn!("Failed to notify to self an epoch change {:?}", e);
        }
    }
}

pub struct NetworkTask {
    consensus_messages_tx: libra_channel::Sender<
        (AccountAddress, Discriminant<ConsensusMsg>),
        (AccountAddress, ConsensusMsg),
    >,
    block_retrieval_tx: libra_channel::Sender<AccountAddress, IncomingBlockRetrievalRequest>,
    all_events: Box<dyn Stream<Item = anyhow::Result<Event<ConsensusMsg>>> + Send + Unpin>,
}

impl NetworkTask {
    /// Establishes the initial connections with the peers and returns the receivers.
    pub fn new(
        network_events: ConsensusNetworkEvents,
        self_receiver: channel::Receiver<anyhow::Result<Event<ConsensusMsg>>>,
    ) -> (NetworkTask, NetworkReceivers) {
        let (consensus_messages_tx, consensus_messages) = libra_channel::new(
            QueueStyle::LIFO,
            NonZeroUsize::new(1).unwrap(),
            Some(&counters::CONSENSUS_CHANNEL_MSGS),
        );
        let (block_retrieval_tx, block_retrieval) = libra_channel::new(
            QueueStyle::LIFO,
            NonZeroUsize::new(1).unwrap(),
            Some(&counters::BLOCK_RETRIEVAL_CHANNEL_MSGS),
        );
        let network_events = network_events.map_err(Into::<anyhow::Error>::into);
        let all_events = Box::new(select(network_events, self_receiver));
        (
            NetworkTask {
                consensus_messages_tx,
                block_retrieval_tx,
                all_events,
            },
            NetworkReceivers {
                consensus_messages,
                block_retrieval,
            },
        )
    }

    pub async fn start(mut self) {
        while let Some(Ok(message)) = self.all_events.next().await {
            match message {
                Event::Message((peer_id, msg)) => {
                    if let Err(e) = self
                        .consensus_messages_tx
                        .push((peer_id, discriminant(&msg)), (peer_id, msg))
                    {
                        warn!(
                            "Error pushing consensus msg from {}, error: {:?}",
                            peer_id, e
                        );
                    }
                }
                Event::RpcRequest((peer_id, msg, callback)) => match msg {
                    ConsensusMsg::BlockRetrievalRequest(request) => {
                        debug!("Received block retrieval request {}", request);
                        if request.num_blocks() > MAX_BLOCKS_PER_REQUEST {
                            warn!(
                                "Ignore block retrieval with too many blocks: {}",
                                request.num_blocks()
                            );
                            continue;
                        }
                        let req_with_callback = IncomingBlockRetrievalRequest {
                            req: *request,
                            response_sender: callback,
                        };
                        if let Err(e) = self.block_retrieval_tx.push(peer_id, req_with_callback) {
                            warn!("libra channel closed: {:?}", e);
                        }
                    }
                    _ => {
                        warn!("Unexpected msg from {}: {:?}", peer_id, msg);
                        continue;
                    }
                },
                Event::NewPeer(peer_id, _origin) => {
                    debug!("Peer {} connected", peer_id);
                }
                Event::LostPeer(peer_id, _origin) => {
                    debug!("Peer {} disconnected", peer_id);
                }
            }
        }
    }
}

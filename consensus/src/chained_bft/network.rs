// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::network_interface::{
        ConsensusMsg, ConsensusNetworkEvents, ConsensusNetworkSender,
    },
    counters,
};
use anyhow::{anyhow, ensure};
use bytes::Bytes;
use channel::{self, libra_channel, message_queues::QueueStyle};
use consensus_types::{
    block_retrieval::{BlockRetrievalRequest, BlockRetrievalResponse},
    common::{Author, Payload},
    proposal_msg::ProposalMsg,
    sync_info::SyncInfo,
    vote_msg::VoteMsg,
};
use futures::{
    channel::oneshot, executor::block_on, stream::select, SinkExt, Stream, StreamExt, TryStreamExt,
};
use libra_logger::prelude::*;
use libra_security_logger::{security_log, SecurityEvent};
use libra_types::{
    account_address::AccountAddress, validator_change::ValidatorChangeProof,
    validator_verifier::ValidatorVerifier,
};
use network::protocols::{network::Event, rpc::error::RpcError};
use std::{
    marker::PhantomData,
    mem::{discriminant, Discriminant},
    num::NonZeroUsize,
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

/// Just a convenience struct to keep all the network proxy receiving queues in one place.
/// Will be returned by the NetworkTask upon startup.
pub struct NetworkReceivers<T> {
    /// Provide a LIFO buffer for each (Author, MessageType) key
    pub consensus_messages: libra_channel::Receiver<
        (AccountAddress, Discriminant<ConsensusMsg<T>>),
        (AccountAddress, ConsensusMsg<T>),
    >,
    pub block_retrieval: libra_channel::Receiver<AccountAddress, IncomingBlockRetrievalRequest>,
}

/// Implements the actual networking support for all consensus messaging.
#[derive(Clone)]
pub struct NetworkSender<T> {
    author: Author,
    network_sender: ConsensusNetworkSender<T>,
    // Self sender and self receivers provide a shortcut for sending the messages to itself.
    // (self sending is not supported by the networking API).
    // Note that we do not support self rpc requests as it might cause infinite recursive calls.
    self_sender: channel::Sender<anyhow::Result<Event<ConsensusMsg<T>>>>,
    validators: Arc<ValidatorVerifier>,
    marker: PhantomData<T>,
}

impl<T: Payload> NetworkSender<T> {
    pub fn new(
        author: Author,
        network_sender: ConsensusNetworkSender<T>,
        self_sender: channel::Sender<anyhow::Result<Event<ConsensusMsg<T>>>>,
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
    pub fn request_block(
        &mut self,
        retrieval_request: BlockRetrievalRequest,
        from: Author,
        timeout: Duration,
    ) -> anyhow::Result<BlockRetrievalResponse<T>> {
        ensure!(from != self.author, "Retrieve block from self");
        counters::BLOCK_RETRIEVAL_COUNT.inc_by(retrieval_request.num_blocks() as i64);
        let pre_retrieval_instant = Instant::now();
        let msg = ConsensusMsg::BlockRetrievalRequest::<T>(Box::new(retrieval_request.clone()));
        let response_msg = block_on(self.network_sender.send_rpc(from, msg, timeout))?;
        counters::BLOCK_RETRIEVAL_DURATION_S.observe_duration(pre_retrieval_instant.elapsed());
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
    pub fn broadcast_proposal(&mut self, proposal: ProposalMsg<T>) {
        let msg = ConsensusMsg::ProposalMsg(Box::new(proposal));
        // counters::UNWRAPPED_PROPOSAL_SIZE_BYTES.observe(msg.message.len() as f64);
        self.broadcast(msg)
    }

    fn broadcast(&mut self, msg: ConsensusMsg<T>) {
        // Directly send the message to ourself without going through network.
        let self_msg = Event::Message((self.author, msg.clone()));
        if let Err(err) = block_on(self.self_sender.send(Ok(self_msg))) {
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
    pub fn send_vote(&self, vote_msg: VoteMsg, recipients: Vec<Author>) {
        let mut network_sender = self.network_sender.clone();
        let mut self_sender = self.self_sender.clone();
        let msg = ConsensusMsg::VoteMsg::<T>(Box::new(vote_msg));
        for peer in recipients {
            if self.author == peer {
                let self_msg = Event::Message((self.author, msg.clone()));
                if let Err(err) = block_on(self_sender.send(Ok(self_msg))) {
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
    pub fn broadcast_vote(&mut self, vote_msg: VoteMsg) {
        let msg = ConsensusMsg::VoteMsg::<T>(Box::new(vote_msg));
        self.broadcast(msg)
    }

    /// Sends the given sync info to the given author.
    /// The future is fulfilled as soon as the message is added to the internal network channel
    /// (does not indicate whether the message is delivered or sent out).
    pub fn send_sync_info(&self, sync_info: SyncInfo, recipient: Author) {
        if recipient == self.author {
            error!("An attempt to deliver sync info msg to itself: ignore.");
            return;
        }
        let msg = ConsensusMsg::SyncInfo::<T>(Box::new(sync_info));
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
    pub fn broadcast_epoch_change(&mut self, proof: ValidatorChangeProof) {
        let msg = ConsensusMsg::ValidatorChangeProof::<T>(Box::new(proof));
        self.broadcast(msg)
    }

    pub fn notify_epoch_change(&mut self, proof: ValidatorChangeProof) {
        let msg = ConsensusMsg::ValidatorChangeProof::<T>(Box::new(proof));
        let self_msg = Event::Message((self.author, msg));
        if let Err(e) = block_on(self.self_sender.send(Ok(self_msg))) {
            warn!("Failed to notify to self an epoch change {:?}", e);
        }
    }
}

pub struct NetworkTask<T> {
    consensus_messages_tx: libra_channel::Sender<
        (AccountAddress, Discriminant<ConsensusMsg<T>>),
        (AccountAddress, ConsensusMsg<T>),
    >,
    block_retrieval_tx: libra_channel::Sender<AccountAddress, IncomingBlockRetrievalRequest>,
    all_events: Box<dyn Stream<Item = anyhow::Result<Event<ConsensusMsg<T>>>> + Send + Unpin>,
}

impl<T: Payload> NetworkTask<T> {
    /// Establishes the initial connections with the peers and returns the receivers.
    pub fn new(
        network_events: ConsensusNetworkEvents<T>,
        self_receiver: channel::Receiver<anyhow::Result<Event<ConsensusMsg<T>>>>,
    ) -> (NetworkTask<T>, NetworkReceivers<T>) {
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
                Event::NewPeer(peer_id) => {
                    debug!("Peer {} connected", peer_id);
                }
                Event::LostPeer(peer_id) => {
                    debug!("Peer {} disconnected", peer_id);
                }
            }
        }
    }
}

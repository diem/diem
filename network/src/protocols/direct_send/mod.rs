// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Protocol for fire-and-forget style message delivery to a peer
//!
//! DirectSend protocol takes advantage of [muxers] and [substream negotiation] to build a simple
//! best effort message delivery protocol. Concretely,
//!
//! 1. Every message runs in its own ephemeral substream. The substream is directional in the way
//!    that only the dialer sends a message to the listener, but no messages or acknowledgements
//!    sending back on the other direction. So the message delivery is best effort and not
//!    guaranteed. Because the substreams are independent, there is no guarantee on the ordering
//!    of the message delivery either.
//! 2. An DirectSend call negotiates which protocol to speak using [`protocol-select`].  This
//!    allows simple versioning of message delivery and negotiation of which message types are
//!    supported. In the future, we can potentially support multiple backwards-incompatible
//!    versions of any messages.
//! 3. The actual structure of the wire messages is left for higher layers to specify. The
//!    DirectSend protocol is only concerned with shipping around opaque blobs. Current libra
//!    DirectSend clients (consensus, mempool) mostly send protobuf enums around over a single
//!    DirectSend protocol, e.g., `/libra/direct_send/0.1.0/consensus/0.1.0`.
//!
//! ## Wire Protocol (dialer):
//!
//! To send a message to a remote peer, the dialer
//!
//! 1. Requests a new outbound substream from the muxer.
//! 2. Negotiates the substream using [`protocol-select`] to the protocol they
//!    wish to speak, e.g., `/libra/direct_send/0.1.0/mempool/0.1.0`.
//! 3. Sends the serialized message on the newly negotiated substream.
//! 4. Drops the substream.
//!
//! ## Wire Protocol (listener):
//!
//! To receive a message from remote peers, the listener
//!
//! 1. Polls for new inbound substreams on the muxer.
//! 2. Negotiates inbound substreams using [`protocol-select`]. The negotiation
//!    must only succeed if the requested protocol is actually supported.
//! 3. Awaits the serialized message on the newly negotiated substream.
//! 4. Drops the substream.
//!
//! [muxers]: ../../../netcore/multiplexing/index.html
//! [substream negotiation]: ../../../netcore/negotiate/index.html
//! [`protocol-select`]: ../../../netcore/negotiate/index.html
use crate::{
    counters,
    error::NetworkError,
    peer_manager::{PeerManagerNotification, PeerManagerRequestSender},
    ProtocolId,
};
use bytes::Bytes;
use channel;
use futures::{
    io::{AsyncRead, AsyncWrite},
    sink::SinkExt,
    stream::StreamExt,
};
use libra_logger::prelude::*;
use libra_types::PeerId;
use netcore::compat::IoCompat;
use std::{
    collections::{hash_map::Entry, HashMap},
    fmt::Debug,
};
use tokio::runtime::Handle;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

#[cfg(test)]
mod test;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DirectSendRequest {
    /// A request to send out a message.
    SendMessage(PeerId, Message),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DirectSendNotification {
    /// A notification that a DirectSend message is received.
    RecvMessage(PeerId, Message),
}

#[derive(Clone, Eq, PartialEq)]
pub struct Message {
    /// Message type.
    pub protocol: ProtocolId,
    /// Serialized message data.
    pub mdata: Bytes,
}

impl Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mdata_str = if self.mdata.len() <= 10 {
            format!("{:?}", self.mdata)
        } else {
            format!("{:?}...", self.mdata.slice(..10))
        };
        write!(
            f,
            "Message {{ protocol: {:?}, mdata: {} }}",
            self.protocol, mdata_str
        )
    }
}

/// The DirectSend actor.
pub struct DirectSend<TSubstream> {
    /// A handle to a tokio executor.
    executor: Handle,
    /// Channel to receive requests from other upstream actors.
    ds_requests_rx: channel::Receiver<DirectSendRequest>,
    /// Channels to send notifictions to upstream actors.
    ds_notifs_tx: channel::Sender<DirectSendNotification>,
    /// Channel to receive notifications from PeerManager.
    peer_mgr_notifs_rx: channel::Receiver<PeerManagerNotification<TSubstream>>,
    /// Channel to send requests to PeerManager.
    peer_mgr_reqs_tx: PeerManagerRequestSender<TSubstream>,
    /// Outbound message queues for each (PeerId, ProtocolId) pair.
    message_queues: HashMap<(PeerId, ProtocolId), channel::Sender<Bytes>>,
}

impl<TSubstream> DirectSend<TSubstream>
where
    TSubstream: AsyncRead + AsyncWrite + Send + Unpin + Debug + 'static,
{
    pub fn new(
        executor: Handle,
        ds_requests_rx: channel::Receiver<DirectSendRequest>,
        ds_notifs_tx: channel::Sender<DirectSendNotification>,
        peer_mgr_notifs_rx: channel::Receiver<PeerManagerNotification<TSubstream>>,
        peer_mgr_reqs_tx: PeerManagerRequestSender<TSubstream>,
    ) -> Self {
        Self {
            executor,
            ds_requests_rx,
            ds_notifs_tx,
            peer_mgr_notifs_rx,
            peer_mgr_reqs_tx,
            message_queues: HashMap::new(),
        }
    }

    pub async fn start(mut self) {
        loop {
            futures::select! {
                req = self.ds_requests_rx.select_next_some() => {
                    self.handle_direct_send_request(req).await;
                }
                notif = self.peer_mgr_notifs_rx.select_next_some() => {
                    self.handle_peer_mgr_notification(notif);
                }
                complete => {
                    crit!("Direct send actor terminated");
                    break;
                }
            }
        }
    }

    // Handle PeerManagerNotification, which can only be NewInboundSubstream for now.
    fn handle_peer_mgr_notification(&self, notif: PeerManagerNotification<TSubstream>) {
        trace!("PeerManagerNotification::{:?}", notif);
        match notif {
            PeerManagerNotification::NewInboundSubstream(peer_id, substream) => {
                self.executor.spawn(Self::handle_inbound_substream(
                    peer_id,
                    substream.protocol,
                    substream.substream,
                    self.ds_notifs_tx.clone(),
                ));
            }
            _ => unreachable!("Unexpected PeerManagerNotification"),
        }
    }

    // Handle a new inbound substream. Keep forwarding the messages to the NetworkProvider.
    async fn handle_inbound_substream(
        peer_id: PeerId,
        protocol: ProtocolId,
        substream: TSubstream,
        mut ds_notifs_tx: channel::Sender<DirectSendNotification>,
    ) {
        let mut substream = Framed::new(IoCompat::new(substream), LengthDelimitedCodec::new());
        while let Some(item) = substream.next().await {
            match item {
                Ok(data) => {
                    let notif = DirectSendNotification::RecvMessage(
                        peer_id,
                        Message {
                            protocol: protocol.clone(),
                            mdata: data.freeze(),
                        },
                    );
                    ds_notifs_tx
                        .send(notif)
                        .await
                        .expect("DirectSendNotification send error");
                }
                Err(e) => {
                    warn!(
                        "DirectSend substream with peer {} receives error {}",
                        peer_id.short_str(),
                        e
                    );
                    break;
                }
            }
        }
        warn!(
            "DirectSend inbound substream with peer {} closed",
            peer_id.short_str()
        );
    }

    // Create a new message queue and spawn a task to forward the messages from the queue to the
    // corresponding substream.
    async fn start_message_queue_handler(
        executor: Handle,
        mut peer_mgr_reqs_tx: PeerManagerRequestSender<TSubstream>,
        peer_id: PeerId,
        protocol: ProtocolId,
    ) -> Result<channel::Sender<Bytes>, NetworkError> {
        // Create a channel for the (PeerId, ProtocolId) pair.
        let (msg_tx, msg_rx) = channel::new::<Bytes>(
            1024,
            &counters::OP_COUNTERS.peer_gauge(
                &counters::PENDING_DIRECT_SEND_OUTBOUND_MESSAGES,
                &peer_id.short_str(),
            ),
        );

        // Open a new substream for the (PeerId, ProtocolId) pair
        let raw_substream = peer_mgr_reqs_tx.open_substream(peer_id, protocol).await?;
        let substream = Framed::new(IoCompat::new(raw_substream), LengthDelimitedCodec::new());

        // Spawn a task to forward the messages from the queue to the substream.
        let f_substream = async move {
            if let Err(e) = msg_rx.map(Ok).forward(substream).await {
                warn!(
                    "Forward messages to peer {} error {:?}",
                    peer_id.short_str(),
                    e
                );
            }
            // The messages in queue will be dropped
            counters::LIBRA_NETWORK_DIRECT_SEND_MESSAGES
                .with_label_values(&["dropped"])
                .inc_by(
                    counters::OP_COUNTERS
                        .peer_gauge(
                            &counters::PENDING_DIRECT_SEND_OUTBOUND_MESSAGES,
                            &peer_id.short_str(),
                        )
                        .get(),
                );
        };
        executor.spawn(f_substream);

        Ok(msg_tx)
    }

    // Try to send a message to the message queue.
    async fn try_send_msg(
        &mut self,
        peer_id: PeerId,
        msg: Message,
        peer_mgr_reqs_tx: PeerManagerRequestSender<TSubstream>,
    ) -> Result<(), NetworkError> {
        let protocol = msg.protocol.clone();

        let substream_queue_tx = match self.message_queues.entry((peer_id, protocol.clone())) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let msg_tx = Self::start_message_queue_handler(
                    self.executor.clone(),
                    peer_mgr_reqs_tx,
                    peer_id,
                    protocol.clone(),
                )
                .await?;
                entry.insert(msg_tx)
            }
        };

        substream_queue_tx.try_send(msg.mdata).map_err(|e| {
            // If the channel is full, simply drop the message on the floor;
            // If the channel is disconnected, remove the message queue from the collection.
            if e.is_disconnected() {
                self.message_queues.remove(&(peer_id, protocol));
            }
            e.into()
        })
    }

    // Handle DirectSendRequest, which can only be SendMessage request for now.
    async fn handle_direct_send_request(&mut self, req: DirectSendRequest) {
        trace!("DirectSendRequest::{:?}", req);
        match req {
            DirectSendRequest::SendMessage(peer_id, msg) => {
                if let Err(e) = self
                    .try_send_msg(peer_id, msg.clone(), self.peer_mgr_reqs_tx.clone())
                    .await
                {
                    counters::LIBRA_NETWORK_DIRECT_SEND_MESSAGES
                        .with_label_values(&["dropped"])
                        .inc();
                    warn!("DirectSend to peer {} failed: {}", peer_id.short_str(), e);
                }
            }
        }
    }
}

// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of DirectSend as per Libra wire protocol v1.
//!
//! Design:
//! -------
//!
//! DirectSend exposes a very simple API to clients. It receives SendMessage requests from upstream
//! clients for outbound message while sending RecvMessage notifications on receipt of inbound
//! messages. The DirectSend actor runs in a single event loop where it handles notifications about
//! new inbound messages from the Peer actor and requests for outbound messages from upstream
//! clients. Both types of messages are processed inline instead of being spawned into separate
//! tasks, since they both simply entail forwarding of messages.
//!
//! Limits:
//! -------
//! Since DirectSend does not dedicate any resources to inbound messages (except forwarding them to
//! the upstream client), it does not try to limit them in any way. Rate-limiting of overall number
//! of messages received at the Peer actor should be sufficient for safe-guarding against malicious
//! actors.
use crate::{
    counters,
    peer::{PeerHandle, PeerNotification},
    protocols::wire::messaging::v1::{DirectSendMsg, NetworkMessage, Priority},
    ProtocolId,
};
use bytes::Bytes;
use futures::{sink::SinkExt, stream::StreamExt};
use libra_logger::prelude::*;
use std::fmt::Debug;

#[cfg(test)]
mod test;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DirectSendRequest {
    /// A request to send out a message.
    SendMessage(Message),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DirectSendNotification {
    /// A notification that a DirectSend message is received.
    RecvMessage(Message),
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
pub struct DirectSend {
    /// Channel to send requests to Peer.
    peer_handle: PeerHandle,
    /// Channel to receive requests from other upstream actors.
    ds_requests_rx: channel::Receiver<DirectSendRequest>,
    /// Channels to send notifictions to upstream actors.
    ds_notifs_tx: channel::Sender<DirectSendNotification>,
    /// Channel to receive notifications from Peer.
    peer_notifs_rx: channel::Receiver<PeerNotification>,
}

impl DirectSend {
    pub fn new(
        peer_handle: PeerHandle,
        ds_requests_rx: channel::Receiver<DirectSendRequest>,
        ds_notifs_tx: channel::Sender<DirectSendNotification>,
        peer_notifs_rx: channel::Receiver<PeerNotification>,
    ) -> Self {
        Self {
            peer_handle,
            ds_requests_rx,
            ds_notifs_tx,
            peer_notifs_rx,
        }
    }

    pub async fn start(mut self) {
        let peer_id = self.peer_handle.peer_id();
        info!(
            "Starting direct send actor for peer: {}",
            peer_id.short_str()
        );
        // Outbound message queues for various protocols.
        loop {
            ::futures::select! {
                // Handle requests and terminate when all request senders are dropped.
                maybe_req = self.ds_requests_rx.next() => {
                    if let Some(req) = maybe_req {
                        self.handle_direct_send_request(
                            req,
                        )
                        .await;
                    } else {
                        break;
                    }
                },
                // Handle inbound direct-send messages.
                notif = self.peer_notifs_rx.select_next_some() => {
                    self.handle_peer_notification(notif).await;
                }
            }
        }
        info!(
            "Terminating direct send actor for peer: {}",
            peer_id.short_str()
        );
    }

    // Handle PeerNotification, which can only be NewMessage for now.
    async fn handle_peer_notification(&mut self, notif: PeerNotification) {
        trace!("PeerNotification::{:?}", notif);
        match notif {
            PeerNotification::NewMessage(message) => {
                let peer_id = self.peer_handle.peer_id();
                if let NetworkMessage::DirectSendMsg(message) = message {
                    let protocol = message.protocol_id;
                    debug!(
                        "DirectSend: Received inbound message from peer {} for protocol {:?}",
                        peer_id.short_str(),
                        protocol
                    );
                    let data = message.raw_msg;
                    counters::LIBRA_NETWORK_DIRECT_SEND_MESSAGES
                        .with_label_values(&["received"])
                        .inc();
                    counters::LIBRA_NETWORK_DIRECT_SEND_BYTES
                        .with_label_values(&["received"])
                        .observe(data.len() as f64);
                    let notif = DirectSendNotification::RecvMessage(Message {
                        protocol,
                        mdata: data,
                    });
                    if let Err(err) = self.ds_notifs_tx.send(notif).await {
                        warn!(
                            "Failed to notify upstream actor about inbound DirectSend message. Error: {:?}",
                            err
                        );
                    }
                } else {
                    error!("Unexpected message from peer actor: {:?}", message);
                }
            }
            _ => unreachable!("Unexpected PeerNotification"),
        }
    }

    // Handle DirectSendRequest, which can only be SendMessage request for now.
    // Tries to synchronously send a message to the peer handle.
    async fn handle_direct_send_request(&mut self, req: DirectSendRequest) {
        trace!("DirectSendRequest::{:?}", req);
        match req {
            DirectSendRequest::SendMessage(msg) => {
                let protocol_id = msg.protocol;
                // If send to PeerHandle fails, simply drop the message on the floor;
                let msg_len = msg.mdata.len();
                let send_result = self
                    .peer_handle
                    .send_message(
                        NetworkMessage::DirectSendMsg(DirectSendMsg {
                            protocol_id,
                            // TODO: Use default priority for now. To be exposed via network API.
                            priority: Priority::default(),
                            raw_msg: msg.mdata,
                        }),
                        protocol_id,
                    )
                    .await;
                match send_result {
                    Ok(()) => {
                        counters::LIBRA_NETWORK_DIRECT_SEND_MESSAGES
                            .with_label_values(&["sent"])
                            .inc();
                        counters::LIBRA_NETWORK_DIRECT_SEND_BYTES
                            .with_label_values(&["sent"])
                            .observe(msg_len as f64);
                    }
                    Err(e) => {
                        warn!(
                            "Failed to send message for protocol: {:?} to peer: {}. Error: {:?}",
                            protocol_id,
                            self.peer_handle.peer_id().short_str(),
                            e
                        );
                        counters::LIBRA_NETWORK_DIRECT_SEND_MESSAGES
                            .with_label_values(&["failed"])
                            .inc();
                    }
                }
            }
        }
    }
}

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
    counters::{self, FAILED_LABEL, RECEIVED_LABEL, SENT_LABEL},
    logging::NetworkSchema,
    peer::{PeerHandle, PeerNotification},
    protocols::wire::messaging::v1::{DirectSendMsg, NetworkMessage, Priority},
    ProtocolId,
};
use bytes::Bytes;
use futures::{sink::SinkExt, stream::StreamExt};
use libra_config::network_id::NetworkContext;
use libra_logger::prelude::*;
use serde::Serialize;
use std::{fmt::Debug, sync::Arc};

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

#[derive(Clone, Eq, PartialEq, Serialize)]
pub struct Message {
    /// Message type.
    pub protocol_id: ProtocolId,
    /// Serialized message data.
    #[serde(skip)]
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
            self.protocol_id, mdata_str
        )
    }
}

/// The DirectSend actor.
pub struct DirectSend {
    /// The network instance this DirectSend actor is running under.
    network_context: Arc<NetworkContext>,
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
        network_context: Arc<NetworkContext>,
        peer_handle: PeerHandle,
        ds_requests_rx: channel::Receiver<DirectSendRequest>,
        ds_notifs_tx: channel::Sender<DirectSendNotification>,
        peer_notifs_rx: channel::Receiver<PeerNotification>,
    ) -> Self {
        Self {
            network_context,
            peer_handle,
            ds_requests_rx,
            ds_notifs_tx,
            peer_notifs_rx,
        }
    }

    pub async fn start(mut self) {
        let peer_id = self.peer_handle.peer_id();
        info!(
            NetworkSchema::new(&self.network_context).remote_peer(&peer_id),
            "{} Starting direct send actor for peer: {}",
            self.network_context,
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
            NetworkSchema::new(&self.network_context).remote_peer(&peer_id),
            "{} Direct send actor for '{}' terminated",
            self.network_context,
            peer_id.short_str()
        );
    }

    // Handle PeerNotification, which can only be NewMessage for now.
    async fn handle_peer_notification(&mut self, notif: PeerNotification) {
        trace!(
            NetworkSchema::new(&self.network_context),
            "{} PeerNotification::{:?}",
            self.network_context,
            notif
        );
        match notif {
            PeerNotification::NewMessage(message) => {
                let peer_id = self.peer_handle.peer_id();
                if let NetworkMessage::DirectSendMsg(message) = message {
                    let protocol_id = message.protocol_id;
                    trace!(
                        NetworkSchema::new(&self.network_context).remote_peer(&peer_id),
                        protocol_id = protocol_id,
                        "{} DirectSend: Received inbound message from peer {} for protocol {:?}",
                        self.network_context,
                        peer_id.short_str(),
                        protocol_id
                    );
                    let data = message.raw_msg;
                    counters::direct_send_messages(&self.network_context, RECEIVED_LABEL).inc();
                    counters::direct_send_bytes(&self.network_context, RECEIVED_LABEL)
                        .inc_by(data.len() as i64);
                    let notif = DirectSendNotification::RecvMessage(Message {
                        protocol_id,
                        mdata: Bytes::from(data),
                    });
                    if let Err(err) = self.ds_notifs_tx.send(notif).await {
                        warn!(
                            NetworkSchema::new(&self.network_context)
                                .debug_error(&err),
                            "{} Failed to notify upstream actor about inbound DirectSend message. Error: {:?}",
                            self.network_context,
                            err
                        );
                    }
                } else {
                    error!(
                        NetworkSchema::new(&self.network_context),
                        message = message,
                        "{} Unexpected message from peer actor: {:?}",
                        self.network_context,
                        message
                    );
                }
            }
            _ => warn!(
                NetworkSchema::new(&self.network_context),
                "{} Unexpected PeerNotification: {:?}", self.network_context, notif
            ),
        }
    }

    // Handle DirectSendRequest, which can only be SendMessage request for now.
    // Tries to synchronously send a message to the peer handle.
    async fn handle_direct_send_request(&mut self, req: DirectSendRequest) {
        trace!(
            NetworkSchema::new(&self.network_context),
            "{} DirectSendRequest::{:?}",
            self.network_context,
            req
        );
        match req {
            DirectSendRequest::SendMessage(msg) => {
                let protocol_id = msg.protocol_id;
                // If send to PeerHandle fails, simply drop the message on the floor;
                let msg_len = msg.mdata.len();
                let send_result = self
                    .peer_handle
                    .send_message(
                        NetworkMessage::DirectSendMsg(DirectSendMsg {
                            protocol_id,
                            // TODO: Use default priority for now. To be exposed via network API.
                            priority: Priority::default(),
                            raw_msg: Vec::from(msg.mdata.as_ref()),
                        }),
                        protocol_id,
                    )
                    .await;
                match send_result {
                    Ok(()) => {
                        counters::direct_send_messages(&self.network_context, SENT_LABEL).inc();
                        counters::direct_send_bytes(&self.network_context, SENT_LABEL)
                            .inc_by(msg_len as i64);
                    }
                    Err(e) => {
                        let peer_id = self.peer_handle.peer_id();
                        warn!(
                            NetworkSchema::new(&self.network_context).remote_peer(&peer_id),
                            protocol_id = protocol_id,
                            "Failed to send message for protocol: {} to peer: {}. Error: {:?}",
                            protocol_id,
                            peer_id.short_str(),
                            e
                        );
                        counters::direct_send_messages(&self.network_context, FAILED_LABEL).inc();
                    }
                }
            }
        }
    }
}

// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! The Peer actor owns the underlying connection and is responsible for listening for
//! and opening substreams as well as negotiating particular protocols on those substreams.
use crate::{
    counters,
    peer_manager::PeerManagerError,
    protocols::wire::messaging::v1::NetworkMessage,
    transport,
    transport::{Connection, ConnectionMetadata},
    ProtocolId,
};
use bytes::BytesMut;
use futures::{
    self,
    channel::oneshot,
    io::{AsyncRead, AsyncWrite},
    stream::StreamExt,
    FutureExt, SinkExt, TryFutureExt,
};
use libra_logger::prelude::*;
use libra_types::PeerId;
use netcore::compat::IoCompat;
use std::{fmt::Debug, io, time::Duration};
use stream_ratelimiter::*;
use tokio::runtime::Handle;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

// Rate-limit configuration for inbound messages. Allows 100 messages for every 10ms window.
pub const MESSAGE_RATE_LIMIT_WINDOW: Duration = Duration::from_millis(10);
pub const MESSAGE_RATE_LIMIT_COUNT: usize = 100;

#[cfg(test)]
mod test;

#[derive(Debug)]
pub enum PeerRequest {
    SendMessage(
        NetworkMessage,
        ProtocolId,
        oneshot::Sender<Result<(), PeerManagerError>>,
    ),
    CloseConnection,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisconnectReason {
    Requested,
    ConnectionLost,
}

#[derive(Debug)]
pub enum PeerNotification {
    NewMessage(NetworkMessage),
    PeerDisconnected(ConnectionMetadata, DisconnectReason),
}

enum State {
    Connected,
    ShuttingDown(DisconnectReason),
}

pub struct Peer<TSocket> {
    /// A handle to a tokio executor.
    executor: Handle,
    /// Connection specific information.
    connection_metadata: ConnectionMetadata,
    /// Underlying connection.
    connection: Option<TSocket>,
    /// Channel to receive requests for opening new outbound substreams.
    requests_rx: channel::Receiver<PeerRequest>,
    /// Channel to send peer notifications to PeerManager.
    peer_notifs_tx: channel::Sender<PeerNotification>,
    /// Channel to notify about new inbound RPC substreams.
    rpc_notifs_tx: channel::Sender<PeerNotification>,
    /// Channel to notify about new inbound DirectSend substreams.
    direct_send_notifs_tx: channel::Sender<PeerNotification>,
    /// Flag to indicate if the actor is being shut down.
    state: State,
}

impl<TSocket> Peer<TSocket>
where
    TSocket: AsyncRead + AsyncWrite + Send + 'static,
{
    pub fn new(
        executor: Handle,
        connection: Connection<TSocket>,
        requests_rx: channel::Receiver<PeerRequest>,
        peer_notifs_tx: channel::Sender<PeerNotification>,
        rpc_notifs_tx: channel::Sender<PeerNotification>,
        direct_send_notifs_tx: channel::Sender<PeerNotification>,
    ) -> Self {
        let Connection {
            metadata: connection_metadata,
            socket,
        } = connection;
        Self {
            executor,
            connection_metadata,
            connection: Some(socket),
            requests_rx,
            peer_notifs_tx,
            rpc_notifs_tx,
            direct_send_notifs_tx,
            state: State::Connected,
        }
    }

    fn peer_id(&self) -> PeerId {
        self.connection_metadata.peer_id()
    }

    pub async fn start(mut self) {
        let self_peer_id = self.peer_id();
        info!(
            "Starting Peer actor for peer: {:?}",
            self_peer_id.short_str()
        );
        // Split the connection into a ReadHalf and a WriteHalf.
        let (reader, writer) = tokio::io::split(IoCompat::new(self.connection.take().unwrap()));
        // Convert ReadHalf to Stream of length-delimited messages.
        let reader = FramedRead::new(reader, LengthDelimitedCodec::new()).fuse();
        // Create a rate-limited stream of inbound messages.
        let mut reader = reader
            .ratelimit(MESSAGE_RATE_LIMIT_WINDOW, MESSAGE_RATE_LIMIT_COUNT)
            .fuse();
        // Convert WriteHalf to Sink of length-delimited messages.
        let writer = FramedWrite::new(writer, LengthDelimitedCodec::new());
        // Start writer "process" as a separate task. We receive two handles to communicate with
        // the task:
        // `write_reqs_tx`: Instruction to send a NetworkMessage on the wire.
        // `close_tx`: Instruction to close the underlying connection.
        let (write_reqs_tx, close_tx) =
            Self::start_writer_task(&self.executor, self_peer_id, writer);
        // Start main Peer event loop.
        loop {
            match self.state {
                State::Connected => {
                    futures::select! {
                        maybe_req = self.requests_rx.next() => {
                            if let Some(request) = maybe_req {
                                self.handle_request(request, write_reqs_tx.clone()).await;
                            } else {
                                // This branch will only be taken if all PeerRequest senders for this Peer
                                // get dropped.
                                break;
                            }
                        },
                        maybe_message = reader.next() => {
                            match maybe_message {
                                Some(Ok(message)) =>  {
                                    if let Err(err) = self.handle_inbound_message(message, write_reqs_tx.clone()).await {
                                        warn!("Error in handling inbound message from peer: {:?}. Error: {:?}",
                                            self_peer_id.short_str(), err);
                                    }
                                },
                                Some(Err(err)) => {
                                    warn!("Failure in reading messages from socket from peer: {:?}. Error: {:?}",
                                        self_peer_id.short_str(), err);
                                    self.close_connection( DisconnectReason::ConnectionLost).await;
                                }
                                None => {
                                    warn!("Received connection closed event for peer: {:?}",
                                        self_peer_id.short_str());
                                    self.close_connection(DisconnectReason::ConnectionLost).await;
                                }
                            }
                        },
                    }
                }
                State::ShuttingDown(reason) => {
                    // Send a close instruction to the writer task. On receipt of this instruction, the writer
                    // task drops all pending outbound messages and closes the connection.
                    if let Err(e) = close_tx.send(()) {
                        info!(
                            "Failed to send close intruction to writer task. It must already be terminating/terminated. Error: {:?}",
                            e
                        );
                    }
                    // Send a PeerDisconnected event to upstream.
                    if let Err(e) = self
                        .peer_notifs_tx
                        .send(PeerNotification::PeerDisconnected(
                            self.connection_metadata.clone(),
                            reason,
                        ))
                        .await
                    {
                        warn!(
                            "Failed to notify upstream about disconnection of peer: {}; error: {:?}",
                            self.peer_id().short_str(),
                            e
                        );
                    }
                    debug!("Peer actor '{}' shutdown", self.peer_id().short_str(),);
                    break;
                }
            }
        }
    }

    // Start a new task on the given executor which is responsible for writing outbound messages on
    // the wire. The function returns two channels which can be used to send intructions to the
    // task:
    // 1. The first channel is used to send outbound NetworkMessages to the task
    // 2. The second channel is used to instruct the task to close the connection and terminate.
    // If outbound messages are queued when the task receives a close instruction, it discards
    // them and immediately closes the connection.
    fn start_writer_task<T: tokio::io::AsyncWrite + Send + Unpin + 'static>(
        executor: &Handle,
        self_peer_id: PeerId,
        mut writer: FramedWrite<T, LengthDelimitedCodec>,
    ) -> (
        channel::Sender<(
            NetworkMessage,
            oneshot::Sender<Result<(), PeerManagerError>>,
        )>,
        oneshot::Sender<()>,
    ) {
        let (write_reqs_tx, mut write_reqs_rx): (
            channel::Sender<(
                NetworkMessage,
                oneshot::Sender<Result<(), PeerManagerError>>,
            )>,
            _,
        ) = channel::new(1024, &counters::PENDING_WIRE_MESSAGES);
        let (close_tx, close_rx) = oneshot::channel();
        let writer_task = async move {
            let mut close_rx = close_rx.into_stream();
            loop {
                futures::select! {
                    (message, ack_ch) = write_reqs_rx.select_next_some() => {
                        if let Err(e) = writer
                            .send(
                                lcs::to_bytes(&message)
                                    .expect("Outboung message failed to serialize")
                                    .into(),
                            )
                            .map_ok(|_| ack_ch.send(Ok(())))
                            .await
                        {
                            warn!(
                                "Error in sending message to peer: {:?}. Error: {:?}",
                                self_peer_id.short_str(),
                                e
                            );
                            break;
                        }
                    },
                    _ = close_rx.select_next_some() => {
                        break;
                    }
                }
            }
            info!("Closing connection to peer: {:?}", self_peer_id.short_str());
            let flush_and_close = async move {
                writer.flush().await?;
                writer.close().await?;
                Ok(()) as io::Result<()>
            };
            match tokio::time::timeout(transport::TRANSPORT_TIMEOUT, flush_and_close).await {
                Err(_) => {
                    info!(
                        "Timeout in flush/close of connection to peer: {:?}",
                        self_peer_id.short_str()
                    );
                }
                Ok(Err(e)) => {
                    info!(
                        "Failure in flush/close of connection to peer: {:?}. Error: {:?}",
                        self_peer_id.short_str(),
                        e
                    );
                }
                Ok(Ok(())) => {
                    info!("Closed connection to peer: {:?}", self_peer_id.short_str());
                }
            }
        };
        executor.spawn(writer_task);
        (write_reqs_tx, close_tx)
    }

    async fn handle_inbound_message(
        &mut self,
        message: BytesMut,
        mut write_reqs_tx: channel::Sender<(
            NetworkMessage,
            oneshot::Sender<Result<(), PeerManagerError>>,
        )>,
    ) -> Result<(), PeerManagerError> {
        trace!("Received message from Peer {}", self.peer_id().short_str(),);
        // Read inbound message from stream.
        let message = message.freeze();
        let message: NetworkMessage = lcs::from_bytes(&message)?;
        match message {
            NetworkMessage::RpcRequest(_) | NetworkMessage::RpcResponse(_) => {
                let notif = PeerNotification::NewMessage(message);
                self.rpc_notifs_tx.send(notif).await.map_err(|err| {
                    warn!("Failed to send notification to RPC actor. Error: {:?}", err);
                    err
                })?;
                Ok(())
            }
            NetworkMessage::DirectSendMsg(_) => {
                let notif = PeerNotification::NewMessage(message);
                self.direct_send_notifs_tx
                    .send(notif)
                    .await
                    .map_err(|err| {
                        warn!(
                            "Failed to send notification to DirectSend actor. Error: {:?}",
                            err
                        );
                        err
                    })?;
                Ok(())
            }
            NetworkMessage::Ping(nonce) => {
                let pong = NetworkMessage::Pong(nonce);
                let (ack_tx, _) = oneshot::channel();
                // Resond to a ping right away.
                write_reqs_tx.send((pong, ack_tx)).await?;
                Ok(())
            }
            _ => unreachable!("Unhandled"),
        }
    }

    async fn handle_request<'a>(
        &'a mut self,
        request: PeerRequest,
        mut write_reqs_tx: channel::Sender<(
            NetworkMessage,
            oneshot::Sender<Result<(), PeerManagerError>>,
        )>,
    ) {
        trace!(
            "Peer {} PeerRequest::{:?}",
            self.peer_id().short_str(),
            request
        );
        match request {
            PeerRequest::SendMessage(message, protocol, channel) => {
                if let Err(e) = write_reqs_tx.send((message, channel)).await {
                    error!(
                        "Failed to send message for protocol {:?} to peer: {:?}. Error: {:?}",
                        protocol,
                        self.peer_id().short_str(),
                        e
                    );
                }
            }
            PeerRequest::CloseConnection => {
                self.close_connection(DisconnectReason::Requested).await;
            }
        }
    }

    async fn close_connection(&mut self, reason: DisconnectReason) {
        // Set the state of the actor to `State::ShuttingDown` to true ensures that the peer actor
        // will terminate and close the connection.
        self.state = State::ShuttingDown(reason);
    }
}

pub struct PeerHandle {
    peer_id: PeerId,
    sender: channel::Sender<PeerRequest>,
}

impl Clone for PeerHandle {
    fn clone(&self) -> Self {
        Self {
            peer_id: self.peer_id,
            sender: self.sender.clone(),
        }
    }
}

impl PeerHandle {
    pub fn new(peer_id: PeerId, sender: channel::Sender<PeerRequest>) -> Self {
        Self { peer_id, sender }
    }

    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    pub async fn send_message(
        &mut self,
        message: NetworkMessage,
        protocol: ProtocolId,
    ) -> Result<(), PeerManagerError> {
        // If we fail to send the request to the Peer, then it must have already been shutdown.
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        if self
            .sender
            .send(PeerRequest::SendMessage(message, protocol, oneshot_tx))
            .await
            .is_err()
        {
            error!(
                "Sending message to Peer {} \
                 failed because it has already been shutdown.",
                self.peer_id.short_str()
            );
        }
        oneshot_rx
            .await
            // The send_message request can get dropped/canceled if the peer
            // connection is in the process of shutting down.
            .map_err(|_| PeerManagerError::NotConnected(self.peer_id))?
    }

    pub async fn disconnect(&mut self) {
        // If we fail to send the request to the Peer, then it must have already been shutdown.
        if self
            .sender
            .send(PeerRequest::CloseConnection)
            .await
            .is_err()
        {
            error!(
                "Sending CloseConnection request to Peer {} \
                 failed because it has already been shutdown.",
                self.peer_id.short_str()
            );
        }
    }
}

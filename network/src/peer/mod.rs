// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! The Peer actor owns the underlying connection and is responsible for listening for
//! and opening substreams as well as negotiating particular protocols on those substreams.
use crate::{
    counters,
    counters::inc_by_with_context,
    logging::NetworkSchema,
    peer_manager::PeerManagerError,
    protocols::wire::messaging::v1::{
        ErrorCode, NetworkMessage, NetworkMessageSink, NetworkMessageStream, ReadError, WriteError,
    },
    transport,
    transport::{Connection, ConnectionMetadata},
    ProtocolId,
};
use diem_config::network_id::NetworkContext;
use diem_logger::prelude::*;
use diem_types::PeerId;
use futures::{
    self,
    channel::oneshot,
    io::{AsyncRead, AsyncWrite},
    stream::StreamExt,
    FutureExt, SinkExt, TryFutureExt,
};
use netcore::compat::IoCompat;
use serde::{export::Formatter, Serialize};
use std::{fmt::Debug, sync::Arc};
use tokio::runtime::Handle;

#[cfg(test)]
mod test;

#[cfg(any(test, feature = "fuzzing"))]
pub mod fuzzing;

#[derive(Debug)]
pub enum PeerRequest {
    SendMessage(
        NetworkMessage,
        ProtocolId,
        oneshot::Sender<Result<(), PeerManagerError>>,
    ),
    CloseConnection,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum DisconnectReason {
    Requested,
    ConnectionLost,
}

impl std::fmt::Display for DisconnectReason {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DisconnectReason::Requested => "Requested",
                DisconnectReason::ConnectionLost => "ConnectionLost",
            }
        )
    }
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
    /// The network instance this Peer actor is running under.
    network_context: Arc<NetworkContext>,
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
    /// The maximum size of an inbound or outbound request frame
    /// Currently, requests are only a single frame
    max_frame_size: usize,
}

impl<TSocket> Peer<TSocket>
where
    TSocket: AsyncRead + AsyncWrite + Send + 'static,
{
    pub fn new(
        network_context: Arc<NetworkContext>,
        executor: Handle,
        connection: Connection<TSocket>,
        requests_rx: channel::Receiver<PeerRequest>,
        peer_notifs_tx: channel::Sender<PeerNotification>,
        rpc_notifs_tx: channel::Sender<PeerNotification>,
        direct_send_notifs_tx: channel::Sender<PeerNotification>,
        max_frame_size: usize,
    ) -> Self {
        let Connection {
            metadata: connection_metadata,
            socket,
        } = connection;
        Self {
            network_context,
            executor,
            connection_metadata,
            connection: Some(socket),
            requests_rx,
            peer_notifs_tx,
            rpc_notifs_tx,
            direct_send_notifs_tx,
            state: State::Connected,
            max_frame_size,
        }
    }

    fn remote_peer_id(&self) -> PeerId {
        self.connection_metadata.remote_peer_id
    }

    pub async fn start(mut self) {
        let remote_peer_id = self.remote_peer_id();
        trace!(
            NetworkSchema::new(&self.network_context)
                .connection_metadata(&self.connection_metadata),
            "{} Starting Peer actor for peer: {}",
            self.network_context,
            remote_peer_id.short_str()
        );

        // Split the connection into a ReadHalf and a WriteHalf.
        let (read_socket, write_socket) =
            tokio::io::split(IoCompat::new(self.connection.take().unwrap()));

        let mut reader =
            NetworkMessageStream::new(IoCompat::new(read_socket), self.max_frame_size).fuse();
        let writer = NetworkMessageSink::new(IoCompat::new(write_socket), self.max_frame_size);

        // Start writer "process" as a separate task. We receive two handles to communicate with
        // the task:
        // `write_reqs_tx`: Instruction to send a NetworkMessage on the wire.
        // `close_tx`: Instruction to close the underlying connection.
        let (mut write_reqs_tx, close_tx) = Self::start_writer_task(
            &self.executor,
            self.connection_metadata.clone(),
            self.network_context.clone(),
            writer,
        );
        // Start main Peer event loop.
        loop {
            match self.state {
                State::Connected => {
                    futures::select! {
                        maybe_req = self.requests_rx.next() => {
                            if let Some(request) = maybe_req {
                                self.handle_request(request, &mut write_reqs_tx).await;
                            } else {
                                // This branch will only be taken if all PeerRequest senders for this Peer
                                // get dropped.
                                break;
                            }
                        },
                        maybe_message = reader.next() => {
                            match maybe_message {
                                Some(message) =>  {
                                    if let Err(err) = self.handle_inbound_message(message, &mut write_reqs_tx).await {
                                        warn!(
                                            NetworkSchema::new(&self.network_context)
                                                .connection_metadata(&self.connection_metadata),
                                            error = %err,
                                            "{} Error in handling inbound message from peer: {}, error: {}",
                                            self.network_context,
                                            remote_peer_id.short_str(),
                                            err
                                        );
                                    }
                                },
                                None => {
                                    info!(
                                        NetworkSchema::new(&self.network_context)
                                            .connection_metadata(&self.connection_metadata),
                                        "{} Received connection closed event for peer: {}",
                                        self.network_context,
                                        remote_peer_id.short_str()
                                    );
                                    self.close_connection(DisconnectReason::ConnectionLost);
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
                            NetworkSchema::new(&self.network_context)
                                .connection_metadata(&self.connection_metadata),
                            error = ?e,
                            "{} Failed to send close instruction to writer task. It must already be terminating/terminated. Error: {:?}",
                            self.network_context,
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
                            NetworkSchema::new(&self.network_context)
                                .connection_metadata(&self.connection_metadata),
                            error = ?e,
                            "{} Failed to notify upstream about disconnection of peer: {}; error: {:?}",
                            self.network_context,
                            remote_peer_id.short_str(),
                            e
                        );
                    }
                    break;
                }
            }
        }

        trace!(
            NetworkSchema::new(&self.network_context)
                .connection_metadata(&self.connection_metadata),
            "{} Peer actor for '{}' terminated",
            self.network_context,
            remote_peer_id.short_str()
        );
    }

    // Start a new task on the given executor which is responsible for writing outbound messages on
    // the wire. The function returns two channels which can be used to send intructions to the
    // task:
    // 1. The first channel is used to send outbound NetworkMessages to the task
    // 2. The second channel is used to instruct the task to close the connection and terminate.
    // If outbound messages are queued when the task receives a close instruction, it discards
    // them and immediately closes the connection.
    fn start_writer_task(
        executor: &Handle,
        connection_metadata: ConnectionMetadata,
        network_context: Arc<NetworkContext>,
        mut writer: NetworkMessageSink<impl AsyncWrite + Unpin + Send + 'static>,
    ) -> (
        channel::Sender<(
            NetworkMessage,
            oneshot::Sender<Result<(), PeerManagerError>>,
        )>,
        oneshot::Sender<()>,
    ) {
        let remote_peer_id = connection_metadata.remote_peer_id;
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
                        if let Err(err) = writer
                            .send(&message)
                            .map_ok(|_| ack_ch.send(Ok(())))
                            .await
                        {
                            warn!(
                                NetworkSchema::new(&network_context)
                                    .connection_metadata(&connection_metadata),
                                error = %err,
                                "{} Error in sending message to peer: {}, error: {}",
                                network_context,
                                remote_peer_id.short_str(),
                                err
                            );
                            break;
                        }
                    },
                    _ = close_rx.select_next_some() => {
                        break;
                    }
                }
            }
            info!(
                NetworkSchema::new(&network_context).connection_metadata(&connection_metadata),
                "{} Closing connection to peer: {}",
                network_context,
                remote_peer_id.short_str()
            );
            let flush_and_close = async {
                writer.flush().await?;
                writer.close().await?;
                Ok(()) as Result<(), WriteError>
            };
            match tokio::time::timeout(transport::TRANSPORT_TIMEOUT, flush_and_close).await {
                Err(_) => {
                    info!(
                        NetworkSchema::new(&network_context)
                            .connection_metadata(&connection_metadata),
                        "{} Timeout in flush/close of connection to peer: {}",
                        network_context,
                        remote_peer_id.short_str()
                    );
                }
                Ok(Err(err)) => {
                    info!(
                        NetworkSchema::new(&network_context)
                            .connection_metadata(&connection_metadata),
                        error = %err,
                        "{} Failure in flush/close of connection to peer: {}, error: {}",
                        network_context,
                        remote_peer_id.short_str(),
                        err
                    );
                }
                Ok(Ok(())) => {
                    info!(
                        NetworkSchema::new(&network_context)
                            .connection_metadata(&connection_metadata),
                        "{} Closed connection to peer: {}",
                        network_context,
                        remote_peer_id.short_str()
                    );
                }
            }
        };
        executor.spawn(writer_task);
        (write_reqs_tx, close_tx)
    }

    async fn handle_inbound_message(
        &mut self,
        message: Result<NetworkMessage, ReadError>,
        write_reqs_tx: &mut channel::Sender<(
            NetworkMessage,
            oneshot::Sender<Result<(), PeerManagerError>>,
        )>,
    ) -> Result<(), PeerManagerError> {
        trace!(
            NetworkSchema::new(&self.network_context)
                .connection_metadata(&self.connection_metadata),
            "{} Received message from peer {}",
            self.network_context,
            self.remote_peer_id().short_str()
        );

        let message = match message {
            Ok(message) => message,
            Err(err) => match err {
                ReadError::DeserializeError(_, _, ref frame_prefix) => {
                    // DeserializeError's are recoverable so we'll let the other
                    // peer know about the error and log the issue, but we won't
                    // close the connection.
                    let message_type = frame_prefix.as_ref().get(0).unwrap_or(&0);
                    let protocol_id = frame_prefix.as_ref().get(1).unwrap_or(&0);
                    let error_code = ErrorCode::parsing_error(*message_type, *protocol_id);
                    let message = NetworkMessage::Error(error_code);

                    let (ack_tx, _) = oneshot::channel();
                    write_reqs_tx.send((message, ack_tx)).await?;
                    return Err(err.into());
                }
                ReadError::IoError(_) => {
                    // IoErrors are mostly unrecoverable so just close the connection.
                    self.close_connection(DisconnectReason::ConnectionLost);
                    return Err(err.into());
                }
            },
        };

        match message {
            NetworkMessage::DirectSendMsg(_) => {
                let notif = PeerNotification::NewMessage(message);
                self.direct_send_notifs_tx
                    .send(notif)
                    .await
                    .map_err(|err| {
                        warn!(
                            NetworkSchema::new(&self.network_context)
                                .connection_metadata(&self.connection_metadata),
                            error = ?err,
                            "{} Failed to send notification to DirectSend actor. Error: {:?}",
                            self.network_context,
                            err
                        );
                        err
                    })?;
            }
            NetworkMessage::Error(error_msg) => {
                warn!(
                    NetworkSchema::new(&self.network_context)
                        .connection_metadata(&self.connection_metadata),
                    error_msg = ?error_msg,
                    "{} Peer {} sent an error message: {:?}",
                    self.network_context,
                    self.remote_peer_id().short_str(),
                    error_msg,
                );
            }
            NetworkMessage::RpcRequest(_) | NetworkMessage::RpcResponse(_) => {
                let notif = PeerNotification::NewMessage(message);
                self.rpc_notifs_tx.send(notif).await.map_err(|err| {
                    warn!(
                        NetworkSchema::new(&self.network_context)
                            .connection_metadata(&self.connection_metadata),
                        error = ?err,
                        "{} Failed to send notification to RPC actor. Error: {:?}",
                        self.network_context,
                        err
                    );
                    err
                })?;
            }
        };
        Ok(())
    }

    async fn handle_request<'a>(
        &'a mut self,
        request: PeerRequest,
        write_reqs_tx: &mut channel::Sender<(
            NetworkMessage,
            oneshot::Sender<Result<(), PeerManagerError>>,
        )>,
    ) {
        trace!(
            "Peer {} PeerRequest::{:?}",
            self.remote_peer_id().short_str(),
            request
        );
        match request {
            PeerRequest::SendMessage(message, protocol, channel) => {
                if let Err(e) = write_reqs_tx.send((message, channel)).await {
                    inc_by_with_context(
                        &counters::PEER_SEND_FAILURES,
                        &self.network_context,
                        protocol.as_str(),
                        1,
                    );
                    error!(
                        NetworkSchema::new(&self.network_context)
                            .connection_metadata(&self.connection_metadata),
                        error = ?e,
                        "Failed to send message for protocol {} to peer: {}. Error: {:?}",
                        protocol,
                        self.remote_peer_id().short_str(),
                        e
                    );
                }
            }
            PeerRequest::CloseConnection => self.close_connection(DisconnectReason::Requested),
        }
    }

    fn close_connection(&mut self, reason: DisconnectReason) {
        // Set the state of the actor to `State::ShuttingDown` to true ensures that the peer actor
        // will terminate and close the connection.
        self.state = State::ShuttingDown(reason);
    }
}

#[derive(Clone)]
pub struct PeerHandle {
    connection_metadata: ConnectionMetadata,
    network_context: Arc<NetworkContext>,
    sender: channel::Sender<PeerRequest>,
}

impl PeerHandle {
    pub fn new(
        network_context: Arc<NetworkContext>,
        connection_metadata: ConnectionMetadata,
        sender: channel::Sender<PeerRequest>,
    ) -> Self {
        Self {
            network_context,
            connection_metadata,
            sender,
        }
    }

    pub fn peer_id(&self) -> PeerId {
        self.connection_metadata.remote_peer_id
    }

    pub async fn send_message(
        &mut self,
        message: NetworkMessage,
        protocol: ProtocolId,
    ) -> Result<(), PeerManagerError> {
        // If we fail to send the request to the Peer, then it must have already been shutdown.
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        if let Err(e) = self
            .sender
            .send(PeerRequest::SendMessage(message, protocol, oneshot_tx))
            .await
        {
            warn!(
                NetworkSchema::new(&self.network_context)
                    .connection_metadata(&self.connection_metadata),
                error = ?e,
                "Sending message to Peer {} \
                 failed because it has already been shutdown.",
                self.peer_id().short_str()
            );
        }
        oneshot_rx
            .await
            // The send_message request can get dropped/canceled if the peer
            // connection is in the process of shutting down.
            .map_err(|_| PeerManagerError::NotConnected(self.peer_id()))?
    }

    pub async fn disconnect(&mut self) {
        // If we fail to send the request to the Peer, then it must have already been shutdown.
        if let Err(e) = self.sender.send(PeerRequest::CloseConnection).await {
            info!(
                NetworkSchema::new(&self.network_context)
                    .connection_metadata(&self.connection_metadata),
                error = ?e,
                "Sending CloseConnection request to Peer {} \
                 failed because it has already been shutdown.",
                self.peer_id().short_str()
            );
        }
    }
}

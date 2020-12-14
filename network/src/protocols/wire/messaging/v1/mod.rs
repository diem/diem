// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines the DiemNet v1 message types, how they are
//! serialized/deserialized, and provides a `Sink` and `Stream` implementation
//! for sending `NetworkMessage`s over an abstract IO object (presumably a socket).
//!
//! The [DiemNet specification](https://github.com/diem/diem/blob/master/specifications/network/messaging-v1.md)
//! describes in greater detail how these messages are sent and received
//! over-the-wire.

use crate::protocols::wire::handshake::v1::ProtocolId;
use bytes::Bytes;
use futures::{
    io::{AsyncRead, AsyncWrite},
    sink::Sink,
    stream::Stream,
};
use netcore::compat::IoCompat;
use pin_project::pin_project;
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};
use thiserror::Error;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

#[cfg(test)]
mod test;

/// Most primitive message type set on the network.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub enum NetworkMessage {
    Error(ErrorCode),
    RpcRequest(RpcRequest),
    RpcResponse(RpcResponse),
    DirectSendMsg(DirectSendMsg),
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub enum ErrorCode {
    /// Failed to parse NetworkMessage when interpreting according to provided protocol version.
    ParsingError(ParsingErrorType),
    /// A message was received for a protocol that is not supported over this connection.
    NotSupported(NotSupportedType),
}

impl ErrorCode {
    pub fn parsing_error(message: u8, protocol: u8) -> Self {
        ErrorCode::ParsingError(ParsingErrorType { message, protocol })
    }
}

/// Flags an invalid network message with as much header information as possible. This is a message
/// that this peer cannot even parse its header information.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct ParsingErrorType {
    message: u8,
    protocol: u8,
}

/// Flags an unsupported network message.  This is a message that a peer can parse its header
/// information but does not have a handler.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub enum NotSupportedType {
    RpcRequest(ProtocolId),
    DirectSendMsg(ProtocolId),
}

/// Create alias RequestId for u32.
pub type RequestId = u32;

/// Create alias Priority for u8.
pub type Priority = u8;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct RpcRequest {
    /// `protocol_id` is a variant of the ProtocolId enum.
    pub protocol_id: ProtocolId,
    /// RequestId for the RPC Request.
    pub request_id: RequestId,
    /// Request priority in the range 0..=255.
    pub priority: Priority,
    /// Request payload. This will be parsed by the application-level handler.
    #[serde(with = "serde_bytes")]
    pub raw_request: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct RpcResponse {
    /// RequestId for corresponding request. This is copied as is from the RpcRequest.
    pub request_id: RequestId,
    /// Response priority in the range 0..=255. This will likely be same as the priority of
    /// corresponding request.
    pub priority: Priority,
    /// Response payload.
    #[serde(with = "serde_bytes")]
    pub raw_response: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct DirectSendMsg {
    /// `protocol_id` is a variant of the ProtocolId enum.
    pub protocol_id: ProtocolId,
    /// Message priority in the range 0..=255.
    pub priority: Priority,
    /// Message payload.
    #[serde(with = "serde_bytes")]
    pub raw_msg: Vec<u8>,
}

/// Errors from reading and deserializing network messages off the wire.
#[derive(Debug, Error)]
pub enum ReadError {
    #[error("network message stream: failed to deserialize network message frame: {0}, frame length: {1}, frame prefix: {2:?}")]
    DeserializeError(#[source] bcs::Error, usize, Bytes),

    #[error("network message stream: IO error while reading message: {0}")]
    IoError(#[from] io::Error),
}

/// Errors from serializing and sending network messages on the wire.
#[derive(Debug, Error)]
pub enum WriteError {
    #[error("network message sink: failed to serialize network message: {0}")]
    SerializeError(#[source] bcs::Error),

    #[error("network message sink: IO error while sending message: {0}")]
    IoError(#[from] io::Error),
}

/// Returns a fully configured length-delimited codec for writing/reading
/// serialized [`NetworkMessage`] frames to/from a socket.
pub fn network_message_frame_codec(max_frame_size: usize) -> LengthDelimitedCodec {
    LengthDelimitedCodec::builder()
        .max_frame_length(max_frame_size)
        .length_field_length(4)
        .big_endian()
        .new_codec()
}

/// A `Stream` of inbound `NetworkMessage`s read and deserialized from an
/// underlying socket.
#[pin_project]
pub struct NetworkMessageStream<TReadSocket: AsyncRead> {
    #[pin]
    framed_read: FramedRead<IoCompat<TReadSocket>, LengthDelimitedCodec>,
}

impl<TReadSocket: AsyncRead> NetworkMessageStream<TReadSocket> {
    pub fn new(socket: TReadSocket, max_frame_size: usize) -> Self {
        let frame_codec = network_message_frame_codec(max_frame_size);
        let compat_socket = IoCompat::new(socket);
        let framed_read = FramedRead::new(compat_socket, frame_codec);
        Self { framed_read }
    }
}

impl<TReadSocket: AsyncRead> Stream for NetworkMessageStream<TReadSocket> {
    type Item = Result<NetworkMessage, ReadError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.project().framed_read.poll_next(cx) {
            Poll::Ready(Some(Ok(frame))) => {
                let frame = frame.freeze();

                match bcs::from_bytes(&frame) {
                    Ok(message) => Poll::Ready(Some(Ok(message))),
                    // Failed to deserialize the NetworkMessage
                    Err(err) => {
                        let mut frame = frame;
                        let frame_len = frame.len();
                        // Keep a few bytes from the frame for debugging
                        frame.truncate(8);
                        let err = ReadError::DeserializeError(err, frame_len, frame);
                        Poll::Ready(Some(Err(err)))
                    }
                }
            }
            Poll::Ready(Some(Err(err))) => Poll::Ready(Some(Err(ReadError::IoError(err)))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// A `Sink` of outbound `NetworkMessage`s that will be serialized and sent over
/// an underlying socket.
#[pin_project]
pub struct NetworkMessageSink<TWriteSocket: AsyncWrite> {
    #[pin]
    framed_write: FramedWrite<IoCompat<TWriteSocket>, LengthDelimitedCodec>,
}

impl<TWriteSocket: AsyncWrite> NetworkMessageSink<TWriteSocket> {
    pub fn new(socket: TWriteSocket, max_frame_size: usize) -> Self {
        let frame_codec = network_message_frame_codec(max_frame_size);
        let compat_socket = IoCompat::new(socket);
        let framed_write = FramedWrite::new(compat_socket, frame_codec);
        Self { framed_write }
    }
}

#[cfg(test)]
impl<TWriteSocket: AsyncWrite + Unpin> NetworkMessageSink<TWriteSocket> {
    pub async fn send_raw_frame(&mut self, frame: Bytes) -> Result<(), WriteError> {
        use futures::sink::SinkExt;
        self.framed_write
            .send(frame)
            .await
            .map_err(WriteError::IoError)
    }
}

impl<TWriteSocket: AsyncWrite> Sink<&NetworkMessage> for NetworkMessageSink<TWriteSocket> {
    type Error = WriteError;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project()
            .framed_write
            .poll_ready(cx)
            .map_err(WriteError::IoError)
    }

    fn start_send(self: Pin<&mut Self>, message: &NetworkMessage) -> Result<(), Self::Error> {
        let frame = bcs::to_bytes(message).map_err(WriteError::SerializeError)?;
        let frame = Bytes::from(frame);

        self.project()
            .framed_write
            .start_send(frame)
            .map_err(WriteError::IoError)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project()
            .framed_write
            .poll_flush(cx)
            .map_err(WriteError::IoError)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project()
            .framed_write
            .poll_close(cx)
            .map_err(WriteError::IoError)
    }
}

// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines the structs transported during the network messaging protocol v1.
//! These should serialize as per [link](TODO: Add ref).

use crate::protocols::wire::handshake::v1::MessagingProtocolVersion;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[cfg(test)]
mod test;

/// Message variants that are sent on the wire.
/// New variants cannot be added without bumping up the MessagingProtocolVersion.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum NetworkMessage {
    Error(ErrorCode),
    Ping(Nonce),
    Pong(Nonce),
    RpcRequest(RpcRequest),
    RpcResponse(RpcResponse),
    DirectSendMsg(DirectSendMsg),
}

/// Unique identifier associated with each application protocol.
/// New application protocols can be added without bumping up the MessagingProtocolVersion.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Deserialize_repr, Serialize_repr)]
pub enum ProtocolId {
    ConsensusRpc = 0,
    ConsensusDirectSend = 1,
    MempoolDirectSend = 2,
    StateSynchronizerDirectSend = 3,
    DiscoveryDirectSend = 4,
    HealthCheckerRpc = 5,
    IdentityDirectSend = 6,
}

/// Enum representing various error codes that can be embedded in NetworkMessage.
/// New variants cannot be added without bumping up the MessagingProtocolVersion.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum ErrorCode {
    /// Failed to parse NetworkMessage when interpreting according to provided protocol version.
    ParsingError(MessagingProtocolVersion, Box<NetworkMessage>),
    /// Ping timed out.
    TimedOut,
}

/// Nonces used by Ping and Pong message types.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Nonce(pub u32);

/// Create alias RequestId for u32.
pub type RequestId = u32;

/// Create alias Priority for u8.
pub type Priority = u8;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct RpcRequest {
    /// RequestId for the RPC Request.
    pub request_id: RequestId,
    /// `protocol_id` is a variant of the ProtocolId enum.
    pub protocol_id: ProtocolId,
    /// Request priority in the range 0..=255.
    pub priority: Priority,
    /// Request payload. This will be parsed by the application-level handler.
    #[serde(with = "serde_bytes")]
    pub raw_request: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
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
pub struct DirectSendMsg {
    /// `protocol_id` is a variant of the ProtocolId enum.
    pub protocol_id: ProtocolId,
    /// Message priority in the range 0..=255.
    pub priority: Priority,
    /// Message payload.
    #[serde(with = "serde_bytes")]
    pub raw_msg: Vec<u8>,
}

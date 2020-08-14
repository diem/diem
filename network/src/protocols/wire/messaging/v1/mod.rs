// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines the structs transported during the network messaging protocol v1.
//! These should serialize as per the [specification](https://github.com/libra/libra/blob/master/specifications/network/messaging-v1.md)

use crate::protocols::wire::handshake::v1::ProtocolId;
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod test;

/// Most primitive message type set on the network.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum NetworkMessage {
    Error(ErrorCode),
    RpcRequest(RpcRequest),
    RpcResponse(RpcResponse),
    DirectSendMsg(DirectSendMsg),
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
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
pub struct ParsingErrorType {
    message: u8,
    protocol: u8,
}

/// Flags an unsupported network message.  This is a message that a peer can parse its header
/// information but does not have a handler.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum NotSupportedType {
    RpcRequest(ProtocolId),
    DirectSendMsg(ProtocolId),
}

/// Create alias RequestId for u32.
pub type RequestId = u32;

/// Create alias Priority for u8.
pub type Priority = u8;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
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

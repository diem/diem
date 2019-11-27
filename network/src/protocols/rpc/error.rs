// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Rpc protocol errors

use crate::peer_manager::PeerManagerError;
use failure::Fail;
use futures::channel::{mpsc, oneshot};
use libra_types::PeerId;
use std::io;

#[derive(Debug, Fail)]
pub enum RpcError {
    #[fail(display = "IO error: {}", _0)]
    IoError(#[fail(cause)] io::Error),

    #[fail(display = "Failed to open substream, not connected with peer: {}", _0)]
    NotConnected(PeerId),

    #[fail(display = "Error writing protobuf message: {:?}", _0)]
    ProstEncodeError(#[fail(cause)] prost::EncodeError),

    #[fail(display = "Error parsing protobuf message: {:?}", _0)]
    ProstDecodeError(#[fail(cause)] prost::DecodeError),

    #[fail(display = "Received invalid rpc response message")]
    InvalidRpcResponse,

    #[fail(display = "Received unexpected rpc response message; expected remote to half-close.")]
    UnexpectedRpcResponse,

    #[fail(display = "Received unexpected rpc request message; expected remote to half-close.")]
    UnexpectedRpcRequest,

    #[fail(display = "Application layer unexpectedly dropped response channel")]
    UnexpectedResponseChannelCancel,

    #[fail(display = "Error in application layer handling rpc request: {:?}", _0)]
    ApplicationError(#[fail(cause)] failure::Error),

    #[fail(display = "Error sending on mpsc channel: {:?}", _0)]
    MpscSendError(#[fail(cause)] mpsc::SendError),

    #[fail(display = "Rpc timed out")]
    TimedOut,
}

impl From<io::Error> for RpcError {
    fn from(err: io::Error) -> Self {
        RpcError::IoError(err)
    }
}

impl From<PeerManagerError> for RpcError {
    fn from(err: PeerManagerError) -> Self {
        match err {
            PeerManagerError::NotConnected(peer_id) => RpcError::NotConnected(peer_id),
            PeerManagerError::IoError(err) => RpcError::IoError(err),
            _ => unreachable!("open_substream only returns NotConnected or IoError"),
        }
    }
}

impl From<prost::EncodeError> for RpcError {
    fn from(err: prost::EncodeError) -> RpcError {
        RpcError::ProstEncodeError(err)
    }
}

impl From<prost::DecodeError> for RpcError {
    fn from(err: prost::DecodeError) -> RpcError {
        RpcError::ProstDecodeError(err)
    }
}

impl From<oneshot::Canceled> for RpcError {
    fn from(_: oneshot::Canceled) -> Self {
        RpcError::UnexpectedResponseChannelCancel
    }
}

impl From<mpsc::SendError> for RpcError {
    fn from(err: mpsc::SendError) -> RpcError {
        RpcError::MpscSendError(err)
    }
}

impl From<tokio::time::Elapsed> for RpcError {
    fn from(_err: tokio::time::Elapsed) -> RpcError {
        RpcError::TimedOut
    }
}

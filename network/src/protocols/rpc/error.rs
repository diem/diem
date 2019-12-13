// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Rpc protocol errors

use crate::peer_manager::PeerManagerError;
use futures::channel::{mpsc, oneshot};
use libra_types::PeerId;
use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RpcError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("Failed to open substream, not connected with peer: {0}")]
    NotConnected(PeerId),

    #[error("Error writing protobuf message: {0:?}")]
    ProstEncodeError(#[from] prost::EncodeError),

    #[error("Error parsing protobuf message: {0:?}")]
    ProstDecodeError(#[from] prost::DecodeError),

    #[error("Received invalid rpc response message")]
    InvalidRpcResponse,

    #[error("Received unexpected rpc response message; expected remote to half-close.")]
    UnexpectedRpcResponse,

    #[error("Received unexpected rpc request message; expected remote to half-close.")]
    UnexpectedRpcRequest,

    #[error("Application layer unexpectedly dropped response channel")]
    UnexpectedResponseChannelCancel,

    #[error("Error in application layer handling rpc request: {0:?}")]
    ApplicationError(anyhow::Error),

    #[error("Error sending on mpsc channel: {0:?}")]
    MpscSendError(#[from] mpsc::SendError),

    #[error("Rpc timed out")]
    TimedOut,
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
impl From<oneshot::Canceled> for RpcError {
    fn from(_: oneshot::Canceled) -> Self {
        RpcError::UnexpectedResponseChannelCancel
    }
}

impl From<tokio::time::Elapsed> for RpcError {
    fn from(_err: tokio::time::Elapsed) -> RpcError {
        RpcError::TimedOut
    }
}

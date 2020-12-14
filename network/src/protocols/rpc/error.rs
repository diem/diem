// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Rpc protocol errors

use crate::peer_manager::PeerManagerError;
use anyhow::anyhow;
use diem_types::PeerId;
use futures::channel::{mpsc, oneshot};
use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RpcError {
    #[error("Error: {0:?}")]
    Error(#[from] anyhow::Error),

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("Bcs error: {0:?}")]
    BcsError(#[from] bcs::Error),

    #[error("Failed to open substream, not connected with peer: {0}")]
    NotConnected(PeerId),

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

    #[error("Too many pending RPCs: {0}")]
    TooManyPending(u32),

    #[error("Rpc timed out")]
    TimedOut,
}

impl From<PeerManagerError> for RpcError {
    fn from(err: PeerManagerError) -> Self {
        match err {
            PeerManagerError::NotConnected(peer_id) => RpcError::NotConnected(peer_id),
            PeerManagerError::IoError(err) => RpcError::IoError(err),
            err => RpcError::Error(anyhow!(err)),
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

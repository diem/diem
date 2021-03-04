// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Errors that originate from the PeerManager module

use crate::protocols::wire::messaging::v1 as wire;
use diem_types::{network_address::NetworkAddress, PeerId};
use futures::channel::{mpsc, oneshot};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PeerManagerError {
    #[error("Error: {0:?}")]
    Error(#[from] anyhow::Error),

    #[error("IO error: {0}")]
    IoError(#[from] ::std::io::Error),

    #[error("Transport error: {0}")]
    TransportError(::anyhow::Error),

    #[error("Shutting down Peer")]
    ShuttingDownPeer,

    #[error("Not connected with Peer {0}")]
    NotConnected(PeerId),

    #[error("Already connected at {0}")]
    AlreadyConnected(NetworkAddress),

    #[error("Sending end of oneshot dropped")]
    OneshotSenderDropped,

    #[error("Failed to send on mpsc: {0}")]
    MpscSendError(mpsc::SendError),

    #[error("Serialization error {0}")]
    BcsError(bcs::Error),

    #[error("Error reading off wire: {0}")]
    WireReadError(#[from] wire::ReadError),

    #[error("Error writing to wire: {0}")]
    WireWriteError(#[from] wire::WriteError),
}

impl PeerManagerError {
    pub fn from_transport_error<E: Into<::anyhow::Error>>(error: E) -> Self {
        PeerManagerError::TransportError(error.into())
    }
}

impl From<oneshot::Canceled> for PeerManagerError {
    fn from(_: oneshot::Canceled) -> Self {
        PeerManagerError::OneshotSenderDropped
    }
}

impl From<bcs::Error> for PeerManagerError {
    fn from(e: bcs::Error) -> Self {
        PeerManagerError::BcsError(e)
    }
}

impl From<mpsc::SendError> for PeerManagerError {
    fn from(e: mpsc::SendError) -> Self {
        PeerManagerError::MpscSendError(e)
    }
}

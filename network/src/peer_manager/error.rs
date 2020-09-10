// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Errors that originate from the PeerManager module

use futures::channel::{mpsc, oneshot};
use libra_network_address::NetworkAddress;
use libra_types::PeerId;
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
    LcsError(lcs::Error),
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

impl From<lcs::Error> for PeerManagerError {
    fn from(e: lcs::Error) -> Self {
        PeerManagerError::LcsError(e)
    }
}

impl From<mpsc::SendError> for PeerManagerError {
    fn from(e: mpsc::SendError) -> Self {
        PeerManagerError::MpscSendError(e)
    }
}

// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Errors that originate from the PeerManager module

use failure::Fail;
use futures::channel::oneshot;
use parity_multiaddr::Multiaddr;
use types::PeerId;

#[derive(Debug, Fail)]
pub enum PeerManagerError {
    #[fail(display = "IO error: {}", _0)]
    IoError(#[fail(cause)] ::std::io::Error),

    #[fail(display = "Transport error: {}", _0)]
    TransportError(#[fail(cause)] ::failure::Error),

    #[fail(display = "Shutting down Peer")]
    ShuttingDownPeer,

    #[fail(display = "Not connected with Peer {}", _0)]
    NotConnected(PeerId),

    #[fail(display = "Already connected at {}", _0)]
    AlreadyConnected(Multiaddr),

    #[fail(display = "Sending end of oneshot dropped")]
    OneshotSenderDropped,
}

impl PeerManagerError {
    pub fn from_transport_error<E: Into<::failure::Error>>(error: E) -> Self {
        PeerManagerError::TransportError(error.into())
    }
}

impl From<::std::io::Error> for PeerManagerError {
    fn from(error: ::std::io::Error) -> Self {
        PeerManagerError::IoError(error)
    }
}

impl From<oneshot::Canceled> for PeerManagerError {
    fn from(_: oneshot::Canceled) -> Self {
        PeerManagerError::OneshotSenderDropped
    }
}

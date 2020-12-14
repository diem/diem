// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::peer_manager::PeerManagerError;
use std::io;
use thiserror::Error;

/// Errors propagated from the network module.
#[derive(Debug, Error)]
#[error(transparent)]
pub struct NetworkError(anyhow::Error);

#[derive(Copy, Clone, Eq, PartialEq, Debug, Error)]
pub enum NetworkErrorKind {
    #[error("IO error")]
    IoError,

    #[error("Bcs error")]
    BcsError,

    #[error("PeerManager error")]
    PeerManagerError,

    #[error("Peer not connected")]
    NotConnected,
}

impl From<NetworkErrorKind> for NetworkError {
    fn from(kind: NetworkErrorKind) -> NetworkError {
        NetworkError(anyhow::Error::new(kind))
    }
}

impl From<anyhow::Error> for NetworkError {
    fn from(err: anyhow::Error) -> NetworkError {
        NetworkError(err)
    }
}

impl From<io::Error> for NetworkError {
    fn from(err: io::Error) -> NetworkError {
        anyhow::Error::new(err)
            .context(NetworkErrorKind::IoError)
            .into()
    }
}

impl From<bcs::Error> for NetworkError {
    fn from(err: bcs::Error) -> NetworkError {
        anyhow::Error::new(err)
            .context(NetworkErrorKind::BcsError)
            .into()
    }
}

impl From<PeerManagerError> for NetworkError {
    fn from(err: PeerManagerError) -> NetworkError {
        match err {
            PeerManagerError::IoError(_) => anyhow::Error::new(err)
                .context(NetworkErrorKind::IoError)
                .into(),
            PeerManagerError::NotConnected(_) => anyhow::Error::new(err)
                .context(NetworkErrorKind::NotConnected)
                .into(),
            err => anyhow::Error::new(err)
                .context(NetworkErrorKind::PeerManagerError)
                .into(),
        }
    }
}

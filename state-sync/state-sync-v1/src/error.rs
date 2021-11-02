// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_types::transaction::Version;
use futures::channel::{mpsc::SendError, oneshot::Canceled};
use mempool_notifications;
use network::error::NetworkError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Deserialize, Error, PartialEq, Serialize)]
pub enum Error {
    #[error("Failed to send callback: {0}")]
    CallbackSendFailed(String),
    #[error("Consensus is executing. There is no need for state sync to drive synchronization.")]
    ConsensusIsExecuting,
    #[error("A sync request was sent to a full node, but this isn't supported.")]
    FullNodeSyncRequest,
    #[error("An integer overflow has occurred: {0}")]
    IntegerOverflow(String),
    #[error("Received an invalid chunk request: {0}")]
    InvalidChunkRequest(String),
    #[error(
        "Unable to add peer as they are not a valid state sync peer: {0}. Connection origin: {1}"
    )]
    InvalidStateSyncPeer(String, String),
    #[error("Encountered a network error: {0}")]
    NetworkError(String),
    #[error("No peers are currently available: {0}")]
    NoAvailablePeers(String),
    #[error("No sync request was issued by consensus: {0}")]
    NoSyncRequestFound(String),
    #[error("No transactions were committed, but received a commit notification!")]
    NoTransactionsCommitted,
    #[error("Received an old sync request for version {0}, but our known version is: {1}")]
    OldSyncRequestVersion(Version, Version),
    #[error("Processed an invalid chunk! Failed to apply the chunk: {0}")]
    ProcessInvalidChunk(String),
    #[error("State sync is operating in read-only mode! Error: {0}")]
    ReadOnlyMode(String),
    #[error(
        "Received a chunk for an outdated request from peer {0}. Known version: {1}, received: {2}"
    )]
    ReceivedChunkForOutdatedRequest(String, String, String),
    #[error("Received a chunk response from a downstream peer: {0}")]
    ReceivedChunkFromDownstream(String),
    #[error("Received an empty chunk response from a peer: {0}")]
    ReceivedEmptyChunk(String),
    #[error("Receivd a non-sequential chunk from {0}. Known version: {1}, received: {2}")]
    ReceivedNonSequentialChunk(String, String, String),
    #[error("Received an unexpected chunk type: {0}")]
    ReceivedWrongChunkType(String),
    #[error("Received a oneshot::canceled event as the sender of a channel was dropped: {0}")]
    SenderDroppedError(String),
    #[error("Synced beyond the target version. Synced version: {0}, target version: {1}")]
    SyncedBeyondTarget(Version, Version),
    #[error("State sync is uninitialized! Error: {0}")]
    UninitializedError(String),
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}

impl From<NetworkError> for Error {
    fn from(error: NetworkError) -> Self {
        Error::NetworkError(error.to_string())
    }
}

impl From<SendError> for Error {
    fn from(error: SendError) -> Self {
        Error::UnexpectedError(error.to_string())
    }
}

impl From<Canceled> for Error {
    fn from(canceled: Canceled) -> Self {
        Error::SenderDroppedError(canceled.to_string())
    }
}

impl From<mempool_notifications::Error> for Error {
    fn from(error: mempool_notifications::Error) -> Self {
        Error::UnexpectedError(error.to_string())
    }
}

impl From<event_notifications::Error> for Error {
    fn from(error: event_notifications::Error) -> Self {
        Error::UnexpectedError(error.to_string())
    }
}

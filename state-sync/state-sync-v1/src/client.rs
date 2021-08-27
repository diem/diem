// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{error::Error, shared_components::SyncState};
use futures::{
    channel::{mpsc, oneshot},
    future::Future,
    SinkExt,
};

/// Messages used by the StateSyncClient for communication with the StateSyncCoordinator.
pub enum CoordinatorMessage {
    GetSyncState(oneshot::Sender<SyncState>), // Return the local sync state.
    WaitForInitialization(oneshot::Sender<Result<(), Error>>), // Wait until state sync is initialized to the waypoint.
}

/// A client used for communicating with a StateSyncCoordinator.
pub struct StateSyncClient {
    coordinator_sender: mpsc::UnboundedSender<CoordinatorMessage>,
}

impl StateSyncClient {
    pub fn new(coordinator_sender: mpsc::UnboundedSender<CoordinatorMessage>) -> Self {
        Self { coordinator_sender }
    }

    /// Returns information about the state sync internal state. This should only
    /// be used by tests.
    // TODO(joshlind): remove this once unit tests are added!
    pub fn get_state(&self) -> impl Future<Output = Result<SyncState, Error>> {
        let mut sender = self.coordinator_sender.clone();
        let (cb_sender, cb_receiver) = oneshot::channel();

        async move {
            sender
                .send(CoordinatorMessage::GetSyncState(cb_sender))
                .await?;
            cb_receiver.await.map_err(|error| error.into())
        }
    }

    /// Waits until state sync is caught up with the waypoint specified in the local config.
    pub fn wait_until_initialized(&self) -> impl Future<Output = Result<(), Error>> {
        let mut sender = self.coordinator_sender.clone();
        let (cb_sender, cb_receiver) = oneshot::channel();

        async move {
            sender
                .send(CoordinatorMessage::WaitForInitialization(cb_sender))
                .await?;
            cb_receiver.await?
        }
    }
}

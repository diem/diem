// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Interface between State Sync and Network layers.

use crate::{
    chunk_request::GetChunkRequest, chunk_response::GetChunkResponse, counters, error::Error,
};
use channel::message_queues::QueueStyle;
use diem_metrics::IntCounterVec;
use diem_types::PeerId;
use network::{
    peer_manager::{ConnectionRequestSender, PeerManagerRequestSender},
    protocols::network::{NetworkEvents, NetworkSender, NewNetworkSender},
    ProtocolId,
};
use serde::{Deserialize, Serialize};

const STATE_SYNC_MAX_BUFFER_SIZE: usize = 1;

/// State sync network messages
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum StateSyncMessage {
    GetChunkRequest(Box<GetChunkRequest>),
    GetChunkResponse(Box<GetChunkResponse>),
}

/// The interface from Network to StateSync layer.
///
/// `StateSyncEvents` is a `Stream` of `PeerManagerNotification` where the
/// raw `Bytes` direct-send messages are deserialized into `StateSyncMessage`
/// types. `StateSyncEvents` is a thin wrapper around a
/// `channel::Receiver<PeerManagerNotification>`.
pub type StateSyncEvents = NetworkEvents<StateSyncMessage>;

/// The interface from StateSync to Networking layer.
///
/// This is a thin wrapper around a `NetworkSender<StateSyncMessage>`, so it
/// is easy to clone and send off to a separate task. For example, the rpc
/// requests return Futures that encapsulate the whole flow, from sending the
/// request to remote, to finally receiving the response and deserializing. It
/// therefore makes the most sense to make the rpc call on a separate async task,
/// which requires the `StateSyncSender` to be `Clone` and `Send`.
#[derive(Clone)]
pub struct StateSyncSender {
    inner: NetworkSender<StateSyncMessage>,
}

impl NewNetworkSender for StateSyncSender {
    fn new(
        peer_mgr_reqs_tx: PeerManagerRequestSender,
        connection_reqs_tx: ConnectionRequestSender,
    ) -> Self {
        Self {
            inner: NetworkSender::new(peer_mgr_reqs_tx, connection_reqs_tx),
        }
    }
}

impl StateSyncSender {
    pub fn send_to(&mut self, recipient: PeerId, message: StateSyncMessage) -> Result<(), Error> {
        let protocol = ProtocolId::StateSyncDirectSend;
        Ok(self.inner.send_to(recipient, protocol, message)?)
    }
}

/// Configuration for the network endpoints to support state sync.
pub fn network_endpoint_config() -> (
    Vec<ProtocolId>,
    Vec<ProtocolId>,
    QueueStyle,
    usize,
    Option<&'static IntCounterVec>,
) {
    (
        vec![],
        vec![ProtocolId::StateSyncDirectSend],
        QueueStyle::LIFO,
        STATE_SYNC_MAX_BUFFER_SIZE,
        Some(&counters::PENDING_STATE_SYNC_NETWORK_EVENTS),
    )
}

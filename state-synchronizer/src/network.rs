// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Interface between StateSynchronizer and Network layers.

use crate::{chunk_request::GetChunkRequest, chunk_response::GetChunkResponse, counters};
use channel::message_queues::QueueStyle;
use libra_metrics::IntCounterVec;
use libra_types::PeerId;
use network::{
    error::NetworkError,
    peer_manager::{ConnectionRequestSender, PeerManagerRequestSender},
    protocols::network::{NetworkEvents, NetworkSender, NewNetworkSender},
    ProtocolId,
};
use serde::{Deserialize, Serialize};

/// StateSynchronizer network messages
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum StateSynchronizerMsg {
    GetChunkRequest(Box<GetChunkRequest>),
    GetChunkResponse(Box<GetChunkResponse>),
}

/// The interface from Network to StateSynchronizer layer.
///
/// `StateSynchronizerEvents` is a `Stream` of `PeerManagerNotification` where the
/// raw `Bytes` direct-send messages are deserialized into `StateSynchronizerMsg`
/// types. `StateSynchronizerEvents` is a thin wrapper around an
/// `channel::Receiver<PeerManagerNotification>`.
pub type StateSynchronizerEvents = NetworkEvents<StateSynchronizerMsg>;

/// The interface from StateSynchronizer to Networking layer.
///
/// This is a thin wrapper around a `NetworkSender<StateSynchronizerMsg>`, so it
/// is easy to clone and send off to a separate task. For example, the rpc
/// requests return Futures that encapsulate the whole flow, from sending the
/// request to remote, to finally receiving the response and deserializing. It
/// therefore makes the most sense to make the rpc call on a separate async task,
/// which requires the `StateSynchronizerSender` to be `Clone` and `Send`.
#[derive(Clone)]
pub struct StateSynchronizerSender {
    inner: NetworkSender<StateSynchronizerMsg>,
}

/// Configuration for the network endpoints to support StateSynchronizer.
pub fn network_endpoint_config() -> (
    Vec<ProtocolId>,
    Vec<ProtocolId>,
    QueueStyle,
    usize,
    Option<&'static IntCounterVec>,
) {
    (
        vec![],
        vec![ProtocolId::StateSynchronizerDirectSend],
        QueueStyle::LIFO,
        // TODO:  Name this as a constant.
        1,
        Some(&counters::PENDING_STATE_SYNCHRONIZER_NETWORK_EVENTS),
    )
}

impl NewNetworkSender for StateSynchronizerSender {
    fn new(
        peer_mgr_reqs_tx: PeerManagerRequestSender,
        connection_reqs_tx: ConnectionRequestSender,
    ) -> Self {
        Self {
            inner: NetworkSender::new(peer_mgr_reqs_tx, connection_reqs_tx),
        }
    }
}

impl StateSynchronizerSender {
    pub fn send_to(
        &mut self,
        recipient: PeerId,
        message: StateSynchronizerMsg,
    ) -> Result<(), NetworkError> {
        let protocol = ProtocolId::StateSynchronizerDirectSend;
        self.inner.send_to(recipient, protocol, message)
    }
}

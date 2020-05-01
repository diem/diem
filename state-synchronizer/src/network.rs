// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Interface between StateSynchronizer and Network layers.

use crate::{chunk_request::GetChunkRequest, chunk_response::GetChunkResponse, counters};
use channel::message_queues::QueueStyle;
use libra_types::PeerId;
use network::{
    error::NetworkError,
    peer_manager::{ConnectionRequestSender, PeerManagerRequestSender},
    protocols::network::{NetworkEvents, NetworkSender},
    validator_network::network_builder::{NetworkBuilder, NETWORK_CHANNEL_SIZE},
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

pub fn add_to_network(
    network: &mut NetworkBuilder,
) -> (StateSynchronizerSender, StateSynchronizerEvents) {
    let (sender, receiver, connection_reqs_tx, connection_notifs_rx) = network
        .add_protocol_handler(
            vec![],
            vec![ProtocolId::StateSynchronizerDirectSend],
            QueueStyle::FIFO,
            NETWORK_CHANNEL_SIZE,
            Some(&counters::PENDING_STATE_SYNCHRONIZER_NETWORK_EVENTS),
        );
    (
        StateSynchronizerSender::new(sender, connection_reqs_tx),
        StateSynchronizerEvents::new(receiver, connection_notifs_rx),
    )
}

impl StateSynchronizerSender {
    pub fn new(
        peer_mgr_reqs_tx: PeerManagerRequestSender,
        connection_reqs_tx: ConnectionRequestSender,
    ) -> Self {
        Self {
            inner: NetworkSender::new(peer_mgr_reqs_tx, connection_reqs_tx),
        }
    }

    pub fn send_to(
        &mut self,
        recipient: PeerId,
        message: StateSynchronizerMsg,
    ) -> Result<(), NetworkError> {
        let protocol = ProtocolId::StateSynchronizerDirectSend;
        self.inner.send_to(recipient, protocol, message)
    }
}

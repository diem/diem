// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Interface between StateSynchronizer and Network layers.

use crate::counters;
use channel::{libra_channel, message_queues::QueueStyle};
use libra_types::PeerId;
use network::{
    error::NetworkError,
    peer_manager::{PeerManagerRequest, PeerManagerRequestSender},
    proto::StateSynchronizerMsg,
    validator_network::{network_builder::NetworkBuilder, NetworkEvents, NetworkSender},
    ProtocolId,
};

/// Protocol id for state-synchronizer direct-send calls
pub const STATE_SYNCHRONIZER_DIRECT_SEND_PROTOCOL: &[u8] =
    b"/libra/direct-send/0.1.0/state-synchronizer/0.1.0";

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
    let (sender, receiver, control_notifs_rx) = network.add_protocol_handler(
        vec![],
        vec![ProtocolId::from_static(
            STATE_SYNCHRONIZER_DIRECT_SEND_PROTOCOL,
        )],
        QueueStyle::FIFO,
        Some(&counters::PENDING_STATE_SYNCHRONIZER_NETWORK_EVENTS),
    );
    (
        StateSynchronizerSender::new(sender),
        StateSynchronizerEvents::new(receiver, control_notifs_rx),
    )
}

impl StateSynchronizerSender {
    pub fn new(inner: libra_channel::Sender<(PeerId, ProtocolId), PeerManagerRequest>) -> Self {
        Self {
            inner: NetworkSender::new(PeerManagerRequestSender::new(inner)),
        }
    }

    pub fn send_to(
        &mut self,
        recipient: PeerId,
        message: StateSynchronizerMsg,
    ) -> Result<(), NetworkError> {
        let protocol = ProtocolId::from_static(STATE_SYNCHRONIZER_DIRECT_SEND_PROTOCOL);
        self.inner.send_to(recipient, protocol, message)
    }
}

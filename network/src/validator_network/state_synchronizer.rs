// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Interface between StateSynchronizer and Network layers.

use crate::{
    counters,
    error::NetworkError,
    peer_manager::{PeerManagerRequest, PeerManagerRequestSender},
    proto::StateSynchronizerMsg,
    validator_network::network_builder::NetworkBuilder,
    validator_network::{NetworkEvents, NetworkSender},
    ProtocolId,
};
use channel::{libra_channel, message_queues::QueueStyle};
use libra_types::PeerId;

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
pub const STATE_SYNCHRONIZER_INBOUND_MSG_TIMEOUT_MS: u64 = 10 * 1000; // 10 seconds

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        peer_manager::{conn_status_channel, PeerManagerNotification, PeerManagerRequest},
        proto::{GetChunkRequest, GetChunkResponse, StateSynchronizerMsg_oneof},
        protocols::direct_send::Message,
        utils::MessageExt,
        validator_network::Event,
    };
    use channel::libra_channel;
    use futures::{executor::block_on, stream::StreamExt};
    use prost::Message as _;
    use std::num::NonZeroUsize;

    // `StateSynchronizerSender` should serialize outbound messages
    #[test]
    fn test_outbound_msg() {
        let (network_reqs_tx, mut network_reqs_rx) =
            libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
        let mut sender = StateSynchronizerSender::new(network_reqs_tx);
        let peer_id = PeerId::random();

        // Create GetChunkRequest and embed in StateSynchronizerMsg.
        let chunk_request = GetChunkRequest::default();
        let mut send_msg = StateSynchronizerMsg::default();
        send_msg.message = Some(StateSynchronizerMsg_oneof::ChunkRequest(chunk_request));

        // Send msg to network layer.
        sender.send_to(peer_id, send_msg.clone()).unwrap();

        // Wait for msg at network layer.
        let event = block_on(network_reqs_rx.next()).unwrap();
        match event {
            PeerManagerRequest::SendMessage(recv_peer_id, msg) => {
                assert_eq!(recv_peer_id, peer_id);
                assert_eq!(
                    msg.protocol.as_ref(),
                    STATE_SYNCHRONIZER_DIRECT_SEND_PROTOCOL
                );
                // check request deserializes
                let recv_msg = StateSynchronizerMsg::decode(msg.mdata.as_ref()).unwrap();
                assert_eq!(recv_msg, send_msg);
            }
            event => panic!("Unexpected event: {:?}", event),
        }
    }

    // Direct send messages should get deserialized through the `StateSynchronizerEvents` stream.
    #[test]
    fn test_inbound_msg() {
        let (_, control_notifs_rx) = conn_status_channel::new();
        let (mut state_sync_tx, state_sync_rx) =
            libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
        let mut stream = StateSynchronizerEvents::new(state_sync_rx, control_notifs_rx);
        let peer_id = PeerId::random();

        // Create GetChunkResponse and embed in StateSynchronizerMsg.
        let chunk_response = GetChunkResponse::default();
        let mut state_sync_msg = StateSynchronizerMsg::default();
        state_sync_msg.message = Some(StateSynchronizerMsg_oneof::ChunkResponse(chunk_response));

        // mock receiving request.
        let event = PeerManagerNotification::RecvMessage(
            peer_id,
            Message {
                protocol: ProtocolId::from_static(STATE_SYNCHRONIZER_DIRECT_SEND_PROTOCOL),
                mdata: state_sync_msg.clone().to_bytes().unwrap(),
            },
        );
        state_sync_tx
            .push(
                (
                    peer_id,
                    ProtocolId::from_static(STATE_SYNCHRONIZER_DIRECT_SEND_PROTOCOL),
                ),
                event,
            )
            .unwrap();

        // request should be properly deserialized
        let expected_event = Event::Message((peer_id, state_sync_msg));
        let event = block_on(stream.next()).unwrap().unwrap();
        assert_eq!(event, expected_event);
    }
}

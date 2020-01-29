// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Interface between Mempool and Network layers.

use crate::{
    counters,
    error::NetworkError,
    peer_manager::{PeerManagerRequest, PeerManagerRequestSender},
    proto::MempoolSyncMsg,
    validator_network::network_builder::NetworkBuilder,
    validator_network::{NetworkEvents, NetworkSender},
    ProtocolId,
};
use channel::{libra_channel, message_queues::QueueStyle};
use libra_types::PeerId;

/// Protocol id for mempool direct-send calls
pub const MEMPOOL_DIRECT_SEND_PROTOCOL: &[u8] = b"/libra/direct-send/0.1.0/mempool/0.1.0";

/// The interface from Network to Mempool layer.
///
/// `MempoolNetworkEvents` is a `Stream` of `PeerManagerNotification` where the
/// raw `Bytes` direct-send and rpc messages are deserialized into
/// `MempoolMessage` types. `MempoolNetworkEvents` is a thin wrapper around an
/// `channel::Receiver<PeerManagerNotification>`.
pub type MempoolNetworkEvents = NetworkEvents<MempoolSyncMsg>;

/// The interface from Mempool to Networking layer.
///
/// This is a thin wrapper around a `NetworkSender<MempoolSyncMsg>`, so it is
/// easy to clone and send off to a separate task. For example, the rpc requests
/// return Futures that encapsulate the whole flow, from sending the request to
/// remote, to finally receiving the response and deserializing. It therefore
/// makes the most sense to make the rpc call on a separate async task, which
/// requires the `MempoolNetworkSender` to be `Clone` and `Send`.
#[derive(Clone)]
pub struct MempoolNetworkSender {
    inner: NetworkSender<MempoolSyncMsg>,
}

pub fn add_to_network(
    network: &mut NetworkBuilder,
) -> (MempoolNetworkSender, MempoolNetworkEvents) {
    let (sender, receiver, control_notifs_rx) = network.add_protocol_handler(
        vec![],
        vec![ProtocolId::from_static(MEMPOOL_DIRECT_SEND_PROTOCOL)],
        QueueStyle::LIFO,
        Some(&counters::PENDING_MEMPOOL_NETWORK_EVENTS),
    );
    (
        MempoolNetworkSender::new(sender),
        MempoolNetworkEvents::new(receiver, control_notifs_rx),
    )
}

impl MempoolNetworkSender {
    pub fn new(inner: libra_channel::Sender<(PeerId, ProtocolId), PeerManagerRequest>) -> Self {
        Self {
            inner: NetworkSender::new(PeerManagerRequestSender::new(inner)),
        }
    }

    pub fn send_to(
        &mut self,
        recipient: PeerId,
        message: MempoolSyncMsg,
    ) -> Result<(), NetworkError> {
        let protocol = ProtocolId::from_static(MEMPOOL_DIRECT_SEND_PROTOCOL);
        self.inner.send_to(recipient, protocol, message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        peer_manager::{self, conn_status_channel, PeerManagerNotification, PeerManagerRequest},
        protocols::direct_send::Message,
        utils::MessageExt,
        validator_network::Event,
    };
    use channel::libra_channel;
    use futures::{executor::block_on, stream::StreamExt};
    use parity_multiaddr::Multiaddr;
    use std::num::NonZeroUsize;

    fn new_test_sync_msg(peer_id: PeerId) -> MempoolSyncMsg {
        let mut mempool_msg = MempoolSyncMsg::default();
        mempool_msg.peer_id = peer_id.into();
        mempool_msg
    }

    // Direct send messages should get deserialized through the
    // `MempoolNetworkEvents` stream.
    #[test]
    fn test_mempool_network_events() {
        let (mut control_notifs_tx, control_notifs_rx) = conn_status_channel::new();
        let (mut mempool_tx, mempool_rx) =
            libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
        let mut stream = MempoolNetworkEvents::new(mempool_rx, control_notifs_rx);

        let peer_id = PeerId::random();
        let mempool_msg = new_test_sync_msg(peer_id);
        let network_msg = Message {
            protocol: ProtocolId::from_static(MEMPOOL_DIRECT_SEND_PROTOCOL),
            mdata: mempool_msg.clone().to_bytes().unwrap(),
        };

        mempool_tx
            .push(
                (
                    peer_id,
                    ProtocolId::from_static(MEMPOOL_DIRECT_SEND_PROTOCOL),
                ),
                PeerManagerNotification::RecvMessage(peer_id, network_msg),
            )
            .unwrap();
        let event = block_on(stream.next()).unwrap().unwrap();
        assert_eq!(event, Event::Message((peer_id, mempool_msg)));

        let f = async {
            // Network notifies mempool about new peer.
            control_notifs_tx
                .push(
                    peer_id,
                    peer_manager::ConnectionStatusNotification::NewPeer(
                        peer_id,
                        Multiaddr::empty(),
                    ),
                )
                .unwrap();
            // Mempool should receive notification.
            let event = stream.next().await.unwrap().unwrap();
            assert_eq!(event, Event::NewPeer(peer_id));
        };
        block_on(f);
    }

    // `MempoolNetworkSender` should serialize outbound messages
    #[test]
    fn test_mempool_network_sender() {
        let (network_reqs_tx, mut network_reqs_rx) =
            libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
        let mut sender = MempoolNetworkSender::new(network_reqs_tx);

        let peer_id = PeerId::random();
        let mempool_msg = new_test_sync_msg(peer_id);
        let expected_network_msg = Message {
            protocol: ProtocolId::from_static(MEMPOOL_DIRECT_SEND_PROTOCOL),
            mdata: mempool_msg.clone().to_bytes().unwrap(),
        };

        // Send the message to network layer
        sender.send_to(peer_id, mempool_msg).unwrap();

        // Network layer should receive serialized message to send out
        let event = block_on(network_reqs_rx.next()).unwrap();
        match event {
            PeerManagerRequest::SendMessage(recv_peer_id, network_msg) => {
                assert_eq!(recv_peer_id, peer_id);
                assert_eq!(network_msg, expected_network_msg);
            }
            event => panic!("Unexpected event: {:?}", event),
        }
    }
}

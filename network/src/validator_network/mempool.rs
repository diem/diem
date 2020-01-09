// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Interface between Mempool and Network layers.

use crate::{
    error::NetworkError,
    interface::NetworkRequest,
    proto::MempoolSyncMsg,
    validator_network::{NetworkEvents, NetworkSender},
    ProtocolId,
};
use channel;
use libra_types::PeerId;

/// Protocol id for mempool direct-send calls
pub const MEMPOOL_DIRECT_SEND_PROTOCOL: &[u8] = b"/libra/direct-send/0.1.0/mempool/0.1.0";

/// The interface from Network to Mempool layer.
///
/// `MempoolNetworkEvents` is a `Stream` of `NetworkNotification` where the
/// raw `Bytes` direct-send and rpc messages are deserialized into
/// `MempoolMessage` types. `MempoolNetworkEvents` is a thin wrapper around an
/// `channel::Receiver<NetworkNotification>`.
pub type MempoolNetworkEvents = NetworkEvents<MempoolSyncMsg>;

/// The interface from Mempool to Networking layer.
///
/// This is a thin wrapper around a `NetworkSender<MempoolSyncMsg>`, which is in
/// turn a thin wrapper around a `channel::Sender<NetworkRequest>`, so it is
/// easy to clone and send off to a separate task. For example, the rpc requests
/// return Futures that encapsulate the whole flow, from sending the request to
/// remote, to finally receiving the response and deserializing. It therefore
/// makes the most sense to make the rpc call on a separate async task, which
/// requires the `MempoolNetworkSender` to be `Clone` and `Send`.
#[derive(Clone)]
pub struct MempoolNetworkSender {
    inner: NetworkSender<MempoolSyncMsg>,
}

impl MempoolNetworkSender {
    pub fn new(inner: channel::Sender<NetworkRequest>) -> Self {
        Self {
            inner: NetworkSender::new(inner),
        }
    }

    pub async fn send_to(
        &mut self,
        recipient: PeerId,
        message: MempoolSyncMsg,
    ) -> Result<(), NetworkError> {
        let protocol = ProtocolId::from_static(MEMPOOL_DIRECT_SEND_PROTOCOL);
        self.inner.send_to(recipient, protocol, message).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        interface::NetworkNotification,
        proto::{BroadcastTransactionsRequest, MempoolSyncMsg_oneof},
        protocols::direct_send::Message,
        utils::MessageExt,
        validator_network::Event,
    };
    use futures::{executor::block_on, sink::SinkExt, stream::StreamExt};

    fn new_test_sync_msg(peer_id: PeerId) -> MempoolSyncMsg {
        let mut req = BroadcastTransactionsRequest::default();
        req.peer_id = peer_id.into();
        MempoolSyncMsg {
            message: Some(MempoolSyncMsg_oneof::BroadcastTransactionsRequest(req)),
        }
    }

    // Direct send messages should get deserialized through the
    // `MempoolNetworkEvents` stream.
    #[test]
    fn test_mempool_network_events() {
        let (mut mempool_tx, mempool_rx) = channel::new_test(8);
        let mut stream = MempoolNetworkEvents::new(mempool_rx);

        let peer_id = PeerId::random();
        let mempool_msg = new_test_sync_msg(peer_id);
        let network_msg = Message {
            protocol: ProtocolId::from_static(MEMPOOL_DIRECT_SEND_PROTOCOL),
            mdata: mempool_msg.clone().to_bytes().unwrap(),
        };

        block_on(mempool_tx.send(NetworkNotification::RecvMessage(peer_id, network_msg))).unwrap();
        let event = block_on(stream.next()).unwrap().unwrap();
        assert_eq!(event, Event::Message((peer_id, mempool_msg)));

        block_on(mempool_tx.send(NetworkNotification::NewPeer(peer_id))).unwrap();
        let event = block_on(stream.next()).unwrap().unwrap();
        assert_eq!(event, Event::NewPeer(peer_id));
    }

    // `MempoolNetworkSender` should serialize outbound messages
    #[test]
    fn test_mempool_network_sender() {
        let (network_reqs_tx, mut network_reqs_rx) = channel::new_test(8);
        let mut sender = MempoolNetworkSender::new(network_reqs_tx);

        let peer_id = PeerId::random();
        let mempool_msg = new_test_sync_msg(peer_id);
        let expected_network_msg = Message {
            protocol: ProtocolId::from_static(MEMPOOL_DIRECT_SEND_PROTOCOL),
            mdata: mempool_msg.clone().to_bytes().unwrap(),
        };

        // Send the message to network layer
        block_on(sender.send_to(peer_id, mempool_msg)).unwrap();

        // Network layer should receive serialized message to send out
        let event = block_on(network_reqs_rx.next()).unwrap();
        match event {
            NetworkRequest::SendMessage(recv_peer_id, network_msg) => {
                assert_eq!(recv_peer_id, peer_id);
                assert_eq!(network_msg, expected_network_msg);
            }
            event => panic!("Unexpected event: {:?}", event),
        }
    }
}

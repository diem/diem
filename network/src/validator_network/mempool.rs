// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Interface between Mempool and Network layers.

use crate::{
    error::NetworkError,
    interface::{NetworkNotification, NetworkRequest},
    proto::MempoolSyncMsg,
    protocols::direct_send::Message,
    utils::MessageExt,
    validator_network::Event,
    ProtocolId,
};
use channel;
use futures::{
    stream::Map,
    task::{Context, Poll},
    SinkExt, Stream, StreamExt,
};
use libra_types::PeerId;
use pin_utils::unsafe_pinned;
use prost::Message as proto_msg;
use std::pin::Pin;

/// Protocol id for mempool direct-send calls
pub const MEMPOOL_DIRECT_SEND_PROTOCOL: &[u8] = b"/libra/mempool/direct-send/0.1.0";

/// The interface from Network to Mempool layer.
///
/// `MempoolNetworkEvents` is a `Stream` of `NetworkNotification` where the
/// raw `Bytes` direct-send and rpc messages are deserialized into
/// `MempoolMessage` types. `MempoolNetworkEvents` is a thin wrapper around an
/// `channel::Receiver<NetworkNotification>`.
pub struct MempoolNetworkEvents {
    // TODO(philiphayes): remove pub
    pub inner: Map<
        channel::Receiver<NetworkNotification>,
        fn(NetworkNotification) -> Result<Event<MempoolSyncMsg>, NetworkError>,
    >,
}

impl MempoolNetworkEvents {
    // This use of `unsafe_pinned` is safe because:
    //   1. This struct does not implement [`Drop`]
    //   2. This struct does not implement [`Unpin`]
    //   3. This struct is not `#[repr(packed)]`
    unsafe_pinned!(
        inner:
            Map<
                channel::Receiver<NetworkNotification>,
                fn(NetworkNotification) -> Result<Event<MempoolSyncMsg>, NetworkError>,
            >
    );

    pub fn new(receiver: channel::Receiver<NetworkNotification>) -> Self {
        let inner = receiver
            // TODO(philiphayes): filter_map might be better, so we can drop
            // messages that don't deserialize.
            .map::<_, fn(_) -> _>(|notification| match notification {
                NetworkNotification::NewPeer(peer_id) => Ok(Event::NewPeer(peer_id)),
                NetworkNotification::LostPeer(peer_id) => Ok(Event::LostPeer(peer_id)),
                NetworkNotification::RecvRpc(_, _) => {
                    unimplemented!("Mempool does not currently use RPC");
                }
                NetworkNotification::RecvMessage(peer_id, msg) => {
                    let msg = MempoolSyncMsg::decode(msg.mdata.as_ref())?;
                    Ok(Event::Message((peer_id, msg)))
                }
            });

        Self { inner }
    }
}

impl Stream for MempoolNetworkEvents {
    type Item = Result<Event<MempoolSyncMsg>, NetworkError>;

    fn poll_next(self: Pin<&mut Self>, context: &mut Context) -> Poll<Option<Self::Item>> {
        self.inner().poll_next(context)
    }
}

/// The interface from Mempool to Networking layer.
///
/// This is a thin wrapper around an `channel::Sender<NetworkRequest>`, so it is
/// easy to clone and send off to a separate task. For example, the rpc requests
/// return Futures that encapsulate the whole flow, from sending the request to
/// remote, to finally receiving the response and deserializing. It therefore
/// makes the most sense to make the rpc call on a separate async task, which
/// requires the `MempoolNetworkSender` to be `Clone` and `Send`.
#[derive(Clone)]
pub struct MempoolNetworkSender {
    // TODO(philiphayes): remove pub
    pub inner: channel::Sender<NetworkRequest>,
}

impl MempoolNetworkSender {
    pub fn new(inner: channel::Sender<NetworkRequest>) -> Self {
        Self { inner }
    }

    /// Send a fire-and-forget "direct-send" message to remote peer `recipient`.
    ///
    /// Currently, the returned Future simply resolves when the message has been
    /// enqueued on the network actor's event queue. It therefore makes no
    /// reliable delivery guarantees.
    pub async fn send_to(
        &mut self,
        recipient: PeerId,
        message: MempoolSyncMsg,
    ) -> Result<(), NetworkError> {
        self.inner
            .send(NetworkRequest::SendMessage(
                recipient,
                Message {
                    protocol: ProtocolId::from_static(MEMPOOL_DIRECT_SEND_PROTOCOL),
                    mdata: message.to_bytes().unwrap(),
                },
            ))
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;

    fn new_test_sync_msg(peer_id: PeerId) -> MempoolSyncMsg {
        let mut mempool_msg = MempoolSyncMsg::default();
        mempool_msg.peer_id = peer_id.into();
        mempool_msg
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

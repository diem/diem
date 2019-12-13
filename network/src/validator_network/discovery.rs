// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Protobuf based interface between Discovery and Network layers.
use crate::{
    error::NetworkError,
    interface::NetworkRequest,
    proto::DiscoveryMsg,
    validator_network::{NetworkEvents, NetworkSender},
    ProtocolId,
};
use libra_types::PeerId;

pub const DISCOVERY_DIRECT_SEND_PROTOCOL: &[u8] = b"/libra/direct-send/0.1.0/discovery/0.1.0";

/// The interface from Network to Discovery module.
///
/// `DiscoveryNetworkEvents` is a `Stream` of `NetworkNotification` where the
/// raw `Bytes` rpc messages are deserialized into
/// `DiscoveryMsg` types. `DiscoveryNetworkEvents` is a thin wrapper
/// around a `channel::Receiver<NetworkNotification>`.
pub type DiscoveryNetworkEvents = NetworkEvents<DiscoveryMsg>;

/// The interface from Discovery to Networking layer.
///
/// This is a thin wrapper around a `NetworkSender<Discoverymsg>`, which is
/// in turn a thin wrapper around a `channel::Sender<NetworkRequest>`, so it is
/// easy to clone and send off to a separate task. For example, the rpc requests
/// return Futures that encapsulate the whole flow, from sending the request to
/// remote, to finally receiving the response and deserializing. It therefore
/// makes the most sense to make the rpc call on a separate async task, which
/// requires the `DiscoveryNetworkSender` to be `Clone` and `Send`.
#[derive(Clone)]
pub struct DiscoveryNetworkSender {
    inner: NetworkSender<DiscoveryMsg>,
}

impl DiscoveryNetworkSender {
    pub fn new(inner: channel::Sender<NetworkRequest>) -> Self {
        Self {
            inner: NetworkSender::new(inner),
        }
    }

    /// Send a DiscoveryMsg to a peer.
    pub async fn send_to(&mut self, peer: PeerId, msg: DiscoveryMsg) -> Result<(), NetworkError> {
        self.inner
            .send_to(
                peer,
                ProtocolId::from_static(DISCOVERY_DIRECT_SEND_PROTOCOL),
                msg,
            )
            .await
    }
}

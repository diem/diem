// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Interface between Consensus and Network layers.

use crate::{
    error::NetworkError,
    interface::NetworkRequest,
    proto::ChainStateMsg,
    protocols::direct_send::Message,
    validator_network::{NetworkEvents, NetworkSender},
    ProtocolId,
};
use bytes05::Bytes;
use channel;
use futures::SinkExt;
use libra_types::PeerId;

/// Protocol id for chain_state direct-send calls
pub const CHAIN_STATE_DIRECT_SEND_PROTOCOL: &[u8] = b"/libra/direct-send/0.1.0/chain_state/0.1.0";

/// The interface from Network to Consensus layer.
pub type ChainStateNetworkEvents = NetworkEvents<ChainStateMsg>;

/// The interface from Consensus to Networking layer.
#[derive(Clone)]
pub struct ChainStateNetworkSender {
    inner: NetworkSender<ChainStateMsg>,
}

impl ChainStateNetworkSender {
    pub fn new(inner: channel::Sender<NetworkRequest>) -> Self {
        Self {
            inner: NetworkSender::new(inner),
        }
    }

    pub async fn send_to(
        &mut self,
        recipient: PeerId,
        message: ChainStateMsg,
    ) -> Result<(), NetworkError> {
        let protocol = ProtocolId::from_static(CHAIN_STATE_DIRECT_SEND_PROTOCOL);
        self.inner.send_to(recipient, protocol, message).await
    }

    pub async fn broadcast_bytes(
        &mut self,
        message_bytes: Bytes,
        ignore_peers: Vec<PeerId>,
    ) -> Result<(), NetworkError> {
        self.inner
            .get_mut()
            .send(NetworkRequest::BroadCastMessage(
                Message {
                    protocol: ProtocolId::from_static(CHAIN_STATE_DIRECT_SEND_PROTOCOL),
                    mdata: message_bytes,
                },
                ignore_peers,
            ))
            .await?;
        Ok(())
    }
}

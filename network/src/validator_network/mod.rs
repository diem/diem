// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Network API for [`Consensus`](/consensus/index.html) and [`Mempool`](/mempool/index.html)

pub use crate::protocols::rpc::error::RpcError;
use bytes::Bytes;
use futures::channel::oneshot;

pub mod network_builder;

mod consensus;
mod mempool;
#[cfg(test)]
mod test;

// Public re-exports
pub use consensus::{
    ConsensusNetworkEvents, ConsensusNetworkSender, CONSENSUS_DIRECT_SEND_PROTOCOL,
    CONSENSUS_RPC_PROTOCOL,
};
pub use mempool::{MempoolNetworkEvents, MempoolNetworkSender, MEMPOOL_DIRECT_SEND_PROTOCOL};
use types::PeerId;

/// Events received by network clients in a validator
///
/// An enumeration of the various types of messages that the network will be sending
/// to its clients. This differs from [`NetworkNotification`] since the contents are deserialized
/// into the type `TMessage` over which `Event` is generic. Note that we assume here that for every
/// consumer of this API there's a singleton message type, `TMessage`,  which encapsulates all the
/// messages and RPCs that are received by that consumer.
///
/// [`NetworkNotification`]: crate::interface::NetworkNotification
#[derive(Debug)]
pub enum Event<TMessage> {
    /// New inbound direct-send message from peer.
    Message((PeerId, TMessage)),
    /// New inbound rpc request. The request is fulfilled by sending the
    /// serialized response `Bytes` over the `onshot::Sender`, where the network
    /// layer will handle sending the response over-the-wire.
    RpcRequest((PeerId, TMessage, oneshot::Sender<Result<Bytes, RpcError>>)),
    /// Peer which we have a newly established connection with.
    NewPeer(PeerId),
    /// Peer with which we've lost our connection.
    LostPeer(PeerId),
}

/// impl PartialEq for simpler testing
impl<TMessage: PartialEq> PartialEq for Event<TMessage> {
    fn eq(&self, other: &Event<TMessage>) -> bool {
        use Event::*;
        match (self, other) {
            (Message((pid1, msg1)), Message((pid2, msg2))) => pid1 == pid2 && msg1 == msg2,
            // ignore oneshot::Sender in comparison
            (RpcRequest((pid1, msg1, _)), RpcRequest((pid2, msg2, _))) => {
                pid1 == pid2 && msg1 == msg2
            }
            (NewPeer(pid1), NewPeer(pid2)) => pid1 == pid2,
            (LostPeer(pid1), LostPeer(pid2)) => pid1 == pid2,
            _ => false,
        }
    }
}

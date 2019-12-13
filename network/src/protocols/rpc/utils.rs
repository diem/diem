// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    interface::NetworkRequest,
    protocols::rpc::{error::RpcError, OutboundRpcRequest},
    utils::MessageExt,
    ProtocolId,
};
use futures::{channel::oneshot, SinkExt};
use libra_types::PeerId;
use std::time::Duration;

/// Send a unary rpc request to remote peer `recipient`. Handles serialization and deserialization
/// of the message types, assuming that the request and response both have the same message type.
///
/// TODO: specify error cases
pub async fn unary_rpc<T: prost::Message + Default>(
    mut inner: channel::Sender<NetworkRequest>,
    recipient: PeerId,
    protocol: ProtocolId,
    req_msg: T,
    timeout: Duration,
) -> Result<T, RpcError> {
    // serialize request
    let req_data = req_msg.to_bytes()?;

    // ask network to fulfill rpc request
    let (res_tx, res_rx) = oneshot::channel();
    let req = OutboundRpcRequest {
        protocol,
        data: req_data,
        res_tx,
        timeout,
    };
    inner.send(NetworkRequest::SendRpc(recipient, req)).await?;
    // wait for response and deserialize
    let res_data = res_rx.await??;
    let res_msg = T::decode(res_data.as_ref())?;
    Ok(res_msg)
}

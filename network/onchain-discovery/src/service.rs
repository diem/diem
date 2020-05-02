// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    storage_query_discovery_set_async,
    types::{OnchainDiscoveryMsg, QueryDiscoverySetRequest},
};
use anyhow::{bail, ensure, format_err, Context as _};
use bounded_executor::BoundedExecutor;
use bytes::Bytes;
use channel::libra_channel;
use futures::{
    channel::oneshot,
    future::{self, FutureExt},
    stream::StreamExt,
};
use libra_logger::prelude::*;
use libra_types::PeerId;
use network::{peer_manager::PeerManagerNotification, protocols::rpc::error::RpcError, ProtocolId};
use std::{sync::Arc, task::Context};
use storage_interface::DbReader;
use tokio::runtime::Handle;

/// A LibraNet service for handling [`QueryDiscoverySetRequest`] rpc's.
///
/// Upon receiving a new rpc request, we query our local storage and respond with
/// the most recent discovery set and validator change proof (if needed).
pub struct OnchainDiscoveryService {
    /// A bounded executor for handling inbound discovery set queries.
    inbound_rpc_executor: BoundedExecutor,
    /// A channel to recevie notifications (rpc and direct send only) from the network.
    // TODO(philiphayes): refactor LibraNet interface to better support this kind
    // of use case.
    peer_mgr_notifs_rx: libra_channel::Receiver<(PeerId, ProtocolId), PeerManagerNotification>,
    /// handle to LibraDB storage
    libra_db: Arc<dyn DbReader>,
}

impl OnchainDiscoveryService {
    pub fn new(
        executor: Handle,
        peer_mgr_notifs_rx: libra_channel::Receiver<(PeerId, ProtocolId), PeerManagerNotification>,
        libra_db: Arc<dyn DbReader>,
        max_concurrent_inbound_queries: usize,
    ) -> Self {
        Self {
            inbound_rpc_executor: BoundedExecutor::new(max_concurrent_inbound_queries, executor),
            peer_mgr_notifs_rx,
            libra_db,
        }
    }

    pub async fn start(mut self) {
        debug!("starting onchain discovery service");

        while let Some(event) = self.peer_mgr_notifs_rx.next().await {
            match event {
                PeerManagerNotification::RecvRpc(peer_id, rpc_req) => {
                    let peer_id_short = peer_id.short_str();
                    trace!("received inbound rpc from peer: {}", peer_id_short);
                    if let Err(err) = self.handle_inbound_rpc(
                        peer_id,
                        rpc_req.protocol,
                        rpc_req.data,
                        rpc_req.res_tx,
                    ) {
                        warn!(
                            "error handling peer's inbound rpc request: peer: {}, err: {:?}",
                            peer_id_short, err
                        );
                    }
                }
                PeerManagerNotification::RecvMessage(peer_id, msg) => {
                    warn!(
                        "unexpected direct-send message from network: peer: {}, msg: {:?}",
                        peer_id.short_str(),
                        msg
                    );
                    debug_assert!(false);
                }
            }
        }
    }

    fn handle_inbound_rpc(
        &self,
        peer_id: PeerId,
        protocol: ProtocolId,
        data: Bytes,
        res_tx: oneshot::Sender<Result<Bytes, RpcError>>,
    ) -> anyhow::Result<()> {
        ensure!(
            protocol == ProtocolId::OnchainDiscoveryRpc,
            "unexpected protocol id: {:?}",
            protocol
        );

        let req_msg: OnchainDiscoveryMsg =
            lcs::from_bytes(data.as_ref()).context("failed to deserialize rpc")?;

        let req_msg = match req_msg {
            OnchainDiscoveryMsg::QueryDiscoverySetRequest(req_msg) => req_msg,
            OnchainDiscoveryMsg::QueryDiscoverySetResponse(_) => bail!("unexpected rpc from peer"),
        };

        debug!(
            "recevied query discovery set request: peer: {}, version: {}",
            peer_id.short_str(),
            req_msg.known_version,
        );

        self.inbound_rpc_executor
            .try_spawn(handle_query_discovery_set_request(
                Arc::clone(&self.libra_db),
                peer_id,
                req_msg,
                res_tx,
            ))
            .map(|_| ())
            .map_err(|_| {
                format_err!("inbound discovery set query executor at capcity; dropped rpc request")
            })
    }
}

async fn handle_query_discovery_set_request(
    libra_db: Arc<dyn DbReader>,
    peer_id: PeerId,
    req_msg: QueryDiscoverySetRequest,
    mut res_tx: oneshot::Sender<Result<Bytes, RpcError>>,
) {
    let mut f_query_storage = storage_query_discovery_set_async(libra_db, req_msg).fuse();
    let mut f_rpc_cancel = future::poll_fn(|cx: &mut Context<'_>| res_tx.poll_canceled(cx)).fuse();
    let peer_id_short = peer_id.short_str();

    futures::select! {
        res = f_query_storage => {
            let (_req_msg, res_msg) = match res {
                Ok(res) => res,
                Err(err) => {
                    warn!("error querying storage discovery set: peer: {}, err: {:?}", peer_id_short, err);
                    return;
                },
            };

            let res_msg = OnchainDiscoveryMsg::QueryDiscoverySetResponse(res_msg);
            let res_bytes = match lcs::to_bytes(&res_msg) {
                Ok(res_bytes) => res_bytes,
                Err(err) => {
                    error!("failed to serialize response message: err: {:?}, res_msg: {:?}", err, res_msg);
                    return;
                }
            };

            if res_tx.send(Ok(res_bytes.into())).is_err() {
                debug!("remote peer canceled discovery set query: peer: {}", peer_id_short);
            }
        },
        _ = f_rpc_cancel => {
            debug!("remote peer canceled discovery set query: peer: {}", peer_id_short);
        },
    }
}

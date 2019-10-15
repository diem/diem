// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::PeerId;
use admission_control_proto::proto::admission_control::{
    admission_control_msg::Message as AdmissionControlMsg_oneof, AdmissionControlClient,
    AdmissionControlMsg, SubmitTransactionRequest, SubmitTransactionResponse,
};
use bounded_executor::BoundedExecutor;
use bytes::Bytes;
use failure::format_err;
use futures::{
    channel::{mpsc, oneshot},
    stream::{select_all, StreamExt},
};
use libra_config::config::{AdmissionControlConfig, RoleType};
use libra_logger::prelude::*;
use libra_prost_ext::MessageExt;
use network::validator_network::{
    AdmissionControlNetworkEvents, AdmissionControlNetworkSender, Event, RpcError,
};
use rand::seq::SliceRandom;
use std::collections::HashMap;
use tokio::runtime::TaskExecutor;

/// UpstreamProxyData is the set of data needed for a full node to send transaction write
/// requests to their upstream validator.
/// UpstreamProxyData is instantiated in AC Runtime.
#[derive(Clone)]
pub struct UpstreamProxyData {
    ac_config: AdmissionControlConfig,
    network_sender: AdmissionControlNetworkSender,
    role: RoleType,
    /// AC Client
    pub client: AdmissionControlClient, // TODO remove client
}

impl UpstreamProxyData {
    pub fn new(
        ac_config: AdmissionControlConfig,
        network_sender: AdmissionControlNetworkSender,
        role: RoleType,
        client: AdmissionControlClient, // TODO remove client
    ) -> Self {
        Self {
            ac_config,
            network_sender,
            role,
            client,
        }
    }
}

/// Main routine for proxying write request. Starts a coordinator that listens for AdmissionControlMsg
pub async fn process_network_messages(
    upstream_proxy_data: UpstreamProxyData,
    network_events: Vec<AdmissionControlNetworkEvents>,
    mut peer_info: HashMap<PeerId, bool>,
    executor: TaskExecutor,
    mut client_events: mpsc::UnboundedReceiver<(
        SubmitTransactionRequest,
        oneshot::Sender<failure::Result<SubmitTransactionResponse>>,
    )>,
) {
    let mut events = select_all(network_events).fuse();
    let workers_available = upstream_proxy_data.ac_config.max_concurrent_inbound_syncs;
    let bounded_executor = BoundedExecutor::new(workers_available, executor);

    loop {
        ::futures::select! {
            (mut msg, callback) = client_events.select_next_some() => {
                let peer_id = pick_peer(&peer_info);
                bounded_executor
                    .spawn(start_submit_transaction_upstream(msg, upstream_proxy_data.clone(), peer_id, callback))
                    .await;
            },
            network_event = events.select_next_some() => {
                match network_event {
                    Ok(event) => {
                        match event {
                            Event::NewPeer(peer_id) => {
                                debug!("[admission control] new peer {}", peer_id);
                                new_peer(&mut peer_info, peer_id);
                            }
                            Event::LostPeer(peer_id) => {
                                debug!("[admission control] lost peer {}", peer_id);
                                lost_peer(&mut peer_info, peer_id);
                            }
                            Event::RpcRequest((peer_id, mut message, callback)) => {
                                if let Some(AdmissionControlMsg_oneof::SubmitTransactionRequest(request)) = message.message {
                                    let peer_id = pick_peer(&peer_info);
                                    bounded_executor
                                        .spawn(process_submit_transaction_request(upstream_proxy_data.clone(), peer_id, request, callback))
                                        .await;
                                }
                            }
                            _ => {},
                        }
                    },
                    Err(err) => { error!("[admission control] network error {:?}", err); },
                }
            }
        }
    }
}

/// new peer discovery handler
/// adds new entry to `peer_info`
fn new_peer(peer_info: &mut HashMap<PeerId, bool>, peer_id: PeerId) {
    if let Some(state) = peer_info.get_mut(&peer_id) {
        *state = true;
    }
}

/// lost peer handler. Marks connection as dead
fn lost_peer(peer_info: &mut HashMap<PeerId, bool>, peer_id: PeerId) {
    if let Some(state) = peer_info.get_mut(&peer_id) {
        *state = false;
    }
}

fn pick_peer(peer_info: &HashMap<PeerId, bool>) -> Option<PeerId> {
    let active_peer_ids: Vec<PeerId> = peer_info
        .iter()
        .filter(|(_, &is_alive)| is_alive)
        .map(|(&peer_id, _)| peer_id)
        .collect();
    if !active_peer_ids.is_empty() {
        return active_peer_ids.choose(&mut rand::thread_rng()).cloned();
    }
    None
}

async fn start_submit_transaction_upstream(
    request: SubmitTransactionRequest,
    upstream_proxy_data: UpstreamProxyData,
    peer_id: Option<PeerId>,
    callback: oneshot::Sender<failure::Result<SubmitTransactionResponse>>,
) {
    let mut response = None;
    if let Some(peer_id) = peer_id {
        let result = upstream_proxy_data
            .network_sender
            .send_transaction_upstream(
                peer_id.clone(),
                request,
                upstream_proxy_data.ac_config.upstream_proxy_timeout,
            )
            .await;
        match result {
            Ok(res) => {
                response = Some(Ok(res));
            }
            Err(e) => {
                response = Some(Err(format_err!(
                    "[admission-control] Sending transaction upstream returned an error: {:?}",
                    e
                )));
            }
        }
    }
    let res = response
        .unwrap_or_else(|| Err(format_err!("[admission-control] No active upstream peers")));
    if let Err(e) = callback.send(res) {
        error!(
            "[admission control] failed to send back transaction result with error: {:?}",
            e
        );
    };
}

async fn submit_transaction_upstream(
    request: SubmitTransactionRequest,
    upstream_proxy_data: &UpstreamProxyData,
    peer_id: Option<PeerId>,
) -> failure::Result<SubmitTransactionResponse> {
    if let Some(peer_id) = peer_id {
        let result = upstream_proxy_data
            .network_sender
            .send_transaction_upstream(
                peer_id.clone(),
                request,
                upstream_proxy_data.ac_config.upstream_proxy_timeout,
            )
            .await?;
        return Ok(result);
    }
    Err(format_err!("[admission-control] No active upstream peers"))
}

async fn process_submit_transaction_request(
    upstream_proxy_data: UpstreamProxyData,
    peer_id: Option<PeerId>,
    request: SubmitTransactionRequest,
    callback: oneshot::Sender<Result<Bytes, RpcError>>,
) {
    let mut response_msg = None;
    match upstream_proxy_data.role {
        RoleType::Validator => {
            if let Ok(response) = upstream_proxy_data.client.submit_transaction(&request) {
                let ac_control_msg = AdmissionControlMsg {
                    message: Some(AdmissionControlMsg_oneof::SubmitTransactionResponse(
                        response,
                    )),
                };
                response_msg = Some(ac_control_msg);
            }
        }
        RoleType::FullNode => {
            // node is not a validator, so send the transaction to upstream AC via networking stack
            if let Ok(response) =
                submit_transaction_upstream(request, &upstream_proxy_data, peer_id).await
            {
                let ac_control_msg = AdmissionControlMsg {
                    message: Some(AdmissionControlMsg_oneof::SubmitTransactionResponse(
                        response,
                    )),
                };
                response_msg = Some(ac_control_msg);
            }
        }
    };
    if let Some(response_msg) = response_msg {
        let response_data = response_msg.to_bytes().expect("fail to serialize proto");
        if let Err(err) = callback
            .send(Ok(response_data))
            .map_err(|_| format_err!("[admission-control] handling inbound rpc call timed out"))
        {
            error!(
                "[admission control] failed to process transaction request, error: {:?}",
                err
            );
        }
    } else {
        error!(
            "[admission control] Did not get a response msg back from submit transaction upstream request",
        );
    }
}

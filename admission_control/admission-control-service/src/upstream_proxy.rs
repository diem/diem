// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::PeerId;
use admission_control_proto::proto::admission_control::{
    admission_control_msg::Message as AdmissionControlMsg_oneof, AdmissionControlClient,
    AdmissionControlMsg, SubmitTransactionRequest, SubmitTransactionResponse,
};
use bytes::Bytes;
use config::config::{AdmissionControlConfig, NodeConfig, RoleType};
use failure::format_err;
use futures::{
    channel::{mpsc, oneshot},
    stream::{select_all, StreamExt},
};
use logger::prelude::*;
use network::validator_network::{
    AdmissionControlNetworkEvents, AdmissionControlNetworkSender, Event, RpcError,
};
use prost_ext::MessageExt;
use rand::seq::SliceRandom;
use std::collections::HashMap;

/// Struct that owns all dependencies required by upstream proxy routine
pub(crate) struct UpstreamProxy {
    ac_config: AdmissionControlConfig,
    network_sender: AdmissionControlNetworkSender,
    peer_info: HashMap<PeerId, bool>,
    /// used to process client requests
    client_events: mpsc::UnboundedReceiver<(SubmitTransactionRequest)>,
    role: RoleType,
    /// AC Client
    pub client: AdmissionControlClient,
}

impl UpstreamProxy {
    /// bootstrap of UpstreamProxy
    pub fn new(
        config: &NodeConfig,
        network_sender: AdmissionControlNetworkSender,
        client_events: mpsc::UnboundedReceiver<SubmitTransactionRequest>,
        client: AdmissionControlClient,
    ) -> Self {
        let upstream_peer_ids = config.get_upstream_peer_ids();
        let peer_info: HashMap<_, _> = upstream_peer_ids
            .iter()
            .map(|peer_id| (*peer_id, true))
            .collect();

        Self {
            ac_config: config.admission_control.clone(),
            network_sender,
            peer_info,
            client_events,
            role: config.get_role(),
            client,
        }
    }

    /// main routine. starts sync coordinator that listens for CoordinatorMsg
    pub async fn process_network_messages(
        mut self,
        network_events: Vec<AdmissionControlNetworkEvents>,
    ) {
        let mut events = select_all(network_events).fuse();

        loop {
            ::futures::select! {
            //                (mut msg, mut callback) = self.client_events.select_next_some() => {
                            mut msg = self.client_events.select_next_some() => {
                                self.submit_transaction_upstream(msg).await;
            //                    let result = match self.submit_transaction_upstream(msg).await {
            //                        Ok(res) => Ok(res),
            //                        Err(e) => Err(e),
            //                    };
            //                    callback.send(result).await;
                            },
                            network_event = events.select_next_some() => {
                                match network_event {
                                    Ok(event) => {
                                        match event {
                                            Event::NewPeer(peer_id) => {
                                                debug!("[admission control] new peer {}", peer_id);
                                                self.new_peer(peer_id);
                                            }
                                            Event::LostPeer(peer_id) => {
                                                debug!("[admission control] lost peer {}", peer_id);
                                                self.lost_peer(peer_id);
                                            }
                                            Event::RpcRequest((peer_id, mut message, callback)) => {
                                                if let Some(AdmissionControlMsg_oneof::SubmitTransactionRequest(request)) = message.message {
                                                    if let Err(err) = self.process_submit_transaction_request(request, callback).await {
                                                        error!("[admission control] failed to process transaction request, peer: {}, error: {:?}", peer_id, err);
                                                    }
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
    fn new_peer(&mut self, peer_id: PeerId) {
        if let Some(state) = self.peer_info.get_mut(&peer_id) {
            *state = true;
        }
    }

    /// lost peer handler. Marks connection as dead
    fn lost_peer(&mut self, peer_id: PeerId) {
        if let Some(state) = self.peer_info.get_mut(&peer_id) {
            *state = false;
        }
    }

    async fn submit_transaction_upstream(
        &mut self,
        request: SubmitTransactionRequest,
    ) -> Result<SubmitTransactionResponse, RpcError> {
        let active_peer_ids = self.get_active_upstream_peers();

        if !active_peer_ids.is_empty() {
            let peer_id = { active_peer_ids.choose(&mut rand::thread_rng()) };
            if let Some(peer_id) = peer_id {
                let sender = self
                    .network_sender
                    .send_transaction_upstream(
                        peer_id.clone(),
                        request,
                        self.ac_config.upstream_proxy_timeout,
                    )
                    .await;
            }
        }
        Err(RpcError::InvalidRpcResponse)
    }

    async fn process_submit_transaction_request(
        &mut self,
        request: SubmitTransactionRequest,
        callback: oneshot::Sender<Result<Bytes, RpcError>>,
    ) -> failure::Result<()> {
        let mut response_msg = None;
        if self.role == RoleType::Validator {
            let result = self.client.submit_transaction(&request);
            match result {
                Ok(response) => {
                    let ac_control_msg = AdmissionControlMsg {
                        message: Some(AdmissionControlMsg_oneof::SubmitTransactionResponse(
                            response,
                        )),
                    };
                    response_msg = Some(ac_control_msg);
                }
                Err(e) => {
                    return Err(format_err!("{:?}", e.to_string()));
                }
            }
        } else {
            // node is not a validator, so send the transaction to upstream AC via networking stack
            if let Ok(response) = self.submit_transaction_upstream(request).await {
                let ac_control_msg = AdmissionControlMsg {
                    message: Some(AdmissionControlMsg_oneof::SubmitTransactionResponse(
                        response,
                    )),
                };
                response_msg = Some(ac_control_msg);
            }
        };
        if let Some(response_msg) = response_msg {
            let response_data =
                Bytes::from(response_msg.to_bytes().expect("fail to serialize proto"));
            return callback
                .send(Ok(response_data))
                .map_err(|_| format_err!("handling inbound rpc call timed out"));
        };
        Err(format_err!("handling inbound rpc call timed out"))
    }

    fn get_active_upstream_peers(&self) -> Vec<PeerId> {
        self.peer_info
            .iter()
            .filter_map(|(peer_id, is_alive)| {
                if *is_alive {
                    return Some(*peer_id);
                }
                None
            })
            .collect()
    }
}

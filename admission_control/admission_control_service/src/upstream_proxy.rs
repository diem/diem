// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::PeerId;
use config::config::{AdmissionControlConfig, NodeConfig};
use failure::prelude::*;
use futures::{
    channel::mpsc,
    stream::{select_all, StreamExt},
};
use logger::prelude::*;
use network::{
    proto::{AdmissionControlMsg, SubmitTransactionRequest, SubmitTransactionResponse},
    validator_network::{AdmissionControlNetworkEvents, AdmissionControlNetworkSender, Event},
};
use rand::seq::SliceRandom;
use std::{collections::HashMap, str::FromStr};

#[derive(Clone)]
struct PeerSyncState {
    is_alive: bool,
}

type PeerInfo = HashMap<PeerId, PeerSyncState>;

/// Struct that owns all dependencies required by upstream proxy routine
pub(crate) struct UpstreamProxy {
    ac_config: AdmissionControlConfig,
    network_sender: AdmissionControlNetworkSender,
    peer_info: PeerInfo,
    // used to process client requests
    client_events: mpsc::UnboundedReceiver<AdmissionControlMsg>,
    is_validator: bool,
}

impl UpstreamProxy {
    /// bootstrap of UpstreamProxy
    pub fn new(
        config: &NodeConfig,
        network_sender: AdmissionControlNetworkSender,
        client_events: mpsc::UnboundedReceiver<AdmissionControlMsg>,
        is_validator: bool,
    ) -> Self {
        let upstream_peers: HashMap<_, _> = config
            .state_sync
            .upstream_peers
            .upstream_peers
            .iter()
            .map(|peer_id_str| {
                (
                    PeerId::from_str(peer_id_str).unwrap_or_else(|_| {
                        panic!("Failed to parse peer_id from string: {}", peer_id_str)
                    }),
                    PeerSyncState { is_alive: true },
                )
            })
            .collect();
        Self {
            ac_config: config.admission_control.clone(),
            network_sender,
            peer_info: upstream_peers,
            client_events,
            is_validator,
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
                mut msg = self.client_events.select_next_some() => {
                    if msg.has_submit_transaction_request() {
                        if let Err(err) = self.submit_transaction_upstream(msg.take_submit_transaction_request()).await {
                            error!("[upstream proxy] failed to send transaction upstream, error: {:?}", err);
                        }
                    };
                },
                network_event = events.select_next_some() => {
                    match network_event {
                        Ok(event) => {
                            match event {
                                Event::NewPeer(peer_id) => {
                                    debug!("[ac upstream proxy] new peer {}", peer_id);
                                    self.new_peer(peer_id);
                                }
                                Event::LostPeer(peer_id) => {
                                    debug!("[ac upstream proxy] lost peer {}", peer_id);
                                    self.lost_peer(peer_id);
                                }
                                Event::RpcRequest((peer_id, mut message, _)) => {
                                    if message.has_submit_transaction_request() {
                                        if let Err(err) = self.process_submit_transaction_request(message.take_submit_transaction_request()).await {
                                            error!("[upstream proxy] failed to process transaction request, peer: {}, error: {:?}", peer_id, err);
                                        }
                                    }
                                    if message.has_submit_transaction_response() {
                                       self.process_submit_transaction_response(&peer_id, message.take_submit_transaction_response()).await;
                                    }
                                }
                                _ => {},
                            }
                        },
                        Err(err) => { error!("[ac upstream proxy] network error {:?}", err); },
                    }
                },
            }
        }
    }

    /// new peer discovery handler
    /// adds new entry to `peer_info`
    fn new_peer(&mut self, peer_id: PeerId) {
        self.peer_info
            .entry(peer_id)
            .or_insert(PeerSyncState { is_alive: true })
            .is_alive = true;
    }

    /// lost peer handler. Marks connection as dead
    fn lost_peer(&mut self, peer_id: PeerId) {
        if let Some(state) = self.peer_info.get_mut(&peer_id) {
            state.is_alive = false;
        }
    }

    async fn submit_transaction_upstream(
        &mut self,
        request: SubmitTransactionRequest,
    ) -> Result<()> {
        let active_peer_ids = self.get_active_upstream_peers();
        if !active_peer_ids.is_empty() {
            let peer_id = { active_peer_ids.choose(&mut rand::thread_rng()) };
            if let Some(peer_id) = peer_id {
                self.network_sender
                    .send_transaction_upstream(
                        peer_id.clone(),
                        request,
                        self.ac_config.upstream_proxy_timeout,
                    )
                    .await?;
            }
        } else {
            error!("[upstream proxy] no active peers");
        }
        Ok(())
    }

    async fn process_submit_transaction_request(
        &mut self,
        mut request: SubmitTransactionRequest,
    ) -> Result<()> {
        if self.is_validator {
            // TODO send the transaction to AC client
            let _signed_transaction = request.take_signed_txn();
        } else {
            // node is not a validator, so send the transaction to upstream AC via networking stack
            self.submit_transaction_upstream(request).await?;
        }
        Ok(())
    }

    async fn process_submit_transaction_response(
        &self,
        _peer_id: &PeerId,
        mut response: SubmitTransactionResponse,
    ) {
        let _signed_transaction = response.take_ac_status();
        // TODO use the returned status somewhere
    }

    fn get_active_upstream_peers(&self) -> Vec<PeerId> {
        self.peer_info
            .iter()
            .filter_map(|(peer_id, peer_state)| {
                if peer_state.is_alive {
                    return Some(*peer_id);
                }
                None
            })
            .collect()
    }
}

// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{counters, PeerId};
use admission_control_proto::proto::admission_control::{
    admission_control_msg::Message as AdmissionControlMsg_oneof,
    submit_transaction_response::Status, AdmissionControlMsg, SubmitTransactionRequest,
    SubmitTransactionResponse,
};
use admission_control_proto::AdmissionControlStatus;
use anyhow::{format_err, Result};
use bounded_executor::BoundedExecutor;
use bytes::Bytes;
use futures::compat::Future01CompatExt;
use futures::{
    channel::{mpsc, oneshot},
    stream::{select_all, StreamExt},
};
use libra_config::config::{AdmissionControlConfig, RoleType};
use libra_logger::prelude::*;
use libra_mempool::proto::{
    mempool::{AddTransactionWithValidationRequest, HealthCheckRequest},
    mempool_client::MempoolClientTrait,
};
use libra_mempool_shared_proto::proto::mempool_status::{
    MempoolAddTransactionStatus,
    MempoolAddTransactionStatusCode::{self, MempoolIsFull},
};
use libra_prost_ext::MessageExt;
use libra_types::transaction::SignedTransaction;
use network::validator_network::{
    AdmissionControlNetworkEvents, AdmissionControlNetworkSender, Event, RpcError,
};
use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::Arc;
use std::time::Instant;
use storage_client::StorageRead;
use tokio::runtime::Handle;
use vm_validator::vm_validator::{get_account_state, TransactionValidation};

/// UpstreamProxyData is the set of data needed for a full node to send transaction write
/// requests to their upstream validator.
/// UpstreamProxyData is instantiated in AC Runtime.
#[derive(Clone)]
pub struct UpstreamProxyData<M, V> {
    ac_config: AdmissionControlConfig,
    network_sender: AdmissionControlNetworkSender,
    role: RoleType,
    /// gRPC client connecting Mempool.
    mempool_client: Option<M>,
    /// gRPC client to send read requests to Storage.
    storage_read_client: Arc<dyn StorageRead>,
    /// VM validator instance to validate transactions sent from wallets.
    vm_validator: Arc<V>,
    /// Flag indicating whether we need to check mempool before validation, drop txn if check
    /// fails.
    need_to_check_mempool_before_validation: bool,
}

impl<M: 'static, V> UpstreamProxyData<M, V>
where
    M: MempoolClientTrait,
    V: TransactionValidation,
{
    /// Render upstream proxy data
    pub fn new(
        ac_config: AdmissionControlConfig,
        network_sender: AdmissionControlNetworkSender,
        role: RoleType,
        mempool_client: Option<M>,
        storage_read_client: Arc<dyn StorageRead>,
        vm_validator: Arc<V>,
        need_to_check_mempool_before_validation: bool,
    ) -> Self {
        Self {
            ac_config,
            network_sender,
            role,
            mempool_client,
            storage_read_client,
            vm_validator,
            need_to_check_mempool_before_validation,
        }
    }
}

/// Main routine for proxying write request. Starts a coordinator that listens for AdmissionControlMsg
#[allow(clippy::implicit_hasher)]
pub async fn process_network_messages<M, V>(
    upstream_proxy_data: UpstreamProxyData<M, V>,
    network_events: Vec<AdmissionControlNetworkEvents>,
    mut peer_info: HashMap<PeerId, bool>,
    executor: Handle,
    mut client_events: mpsc::Receiver<(
        SubmitTransactionRequest,
        oneshot::Sender<Result<SubmitTransactionResponse>>,
    )>,
) where
    M: MempoolClientTrait + Clone + 'static,
    V: TransactionValidation + Clone + 'static,
{
    let mut events = select_all(network_events).fuse();
    let workers_available = upstream_proxy_data.ac_config.max_concurrent_inbound_syncs;
    let bounded_executor = BoundedExecutor::new(workers_available, executor);

    loop {
        ::futures::select! {
            (mut msg, callback) = client_events.select_next_some() => {
                let peer_id = pick_peer(&peer_info);
                bounded_executor
                    .spawn(submit_transaction(msg, upstream_proxy_data.clone(), peer_id, callback))
                    .await;
            },
            network_event = events.select_next_some() => {
                match network_event {
                    Ok(event) => {
                        match event {
                            Event::NewPeer(peer_id) => {
                                debug!("[admission control] new peer {}", peer_id);
                                new_peer(&mut peer_info, peer_id);
                                counters::UPSTREAM_PEERS.set(count_active_peers(&peer_info) as i64);
                            }
                            Event::LostPeer(peer_id) => {
                                debug!("[admission control] lost peer {}", peer_id);
                                lost_peer(&mut peer_info, peer_id);
                                counters::UPSTREAM_PEERS.set(count_active_peers(&peer_info) as i64);
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

fn count_active_peers(peer_info: &HashMap<PeerId, bool>) -> usize {
    peer_info.iter().filter(|(_, &is_alive)| is_alive).count()
}

async fn submit_transaction<M, V>(
    request: SubmitTransactionRequest,
    mut upstream_proxy_data: UpstreamProxyData<M, V>,
    peer_id: Option<PeerId>,
    callback: oneshot::Sender<Result<SubmitTransactionResponse>>,
) where
    M: MempoolClientTrait,
    V: TransactionValidation,
{
    let start_time = Instant::now();
    let mut response = None;
    let mut txn_result = "success";
    match upstream_proxy_data.role {
        RoleType::Validator => {
            response = Some(submit_transaction_to_mempool(upstream_proxy_data, request).await);
        }
        RoleType::FullNode => {
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
                        txn_result = "failure";
                        response = Some(Err(format_err!(
                            "[admission-control] Sending transaction upstream returned an error: {:?}",
                            e
                        )));
                    }
                }
            }
        }
    };
    let res = response.unwrap_or_else(|| {
        // timeout
        txn_result = "failure";
        counters::TIMEOUT
            .with_label_values(&["client", "timeout"])
            .inc();
        Err(format_err!(
            "[admission-control] Processing write request failed"
        ))
    });
    if let Err(e) = callback.send(res) {
        txn_result = "failure";
        counters::TIMEOUT
            .with_label_values(&["client", "callback_timeout"])
            .inc();
        error!(
            "[admission control] failed to send back transaction result with error: {:?}",
            e
        );
    };
    counters::TRANSACTION_LATENCY
        .with_label_values(&[txn_result])
        .observe(start_time.elapsed().as_secs() as f64);
    counters::TRANSACTION_PROXY
        .with_label_values(&["client", txn_result])
        .inc();
}

async fn submit_transaction_upstream<M, V>(
    request: SubmitTransactionRequest,
    upstream_proxy_data: &mut UpstreamProxyData<M, V>,
    peer_id: Option<PeerId>,
) -> Result<SubmitTransactionResponse> {
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

async fn process_submit_transaction_request<M, V>(
    mut upstream_proxy_data: UpstreamProxyData<M, V>,
    peer_id: Option<PeerId>,
    request: SubmitTransactionRequest,
    callback: oneshot::Sender<Result<Bytes, RpcError>>,
) where
    M: MempoolClientTrait,
    V: TransactionValidation,
{
    let start_time = Instant::now();
    let mut response_msg = None;
    let mut txn_result = "success";
    match upstream_proxy_data.role {
        RoleType::Validator => {
            if let Ok(response) = submit_transaction_to_mempool(upstream_proxy_data, request).await
            {
                let ac_control_msg = AdmissionControlMsg {
                    message: Some(AdmissionControlMsg_oneof::SubmitTransactionResponse(
                        response,
                    )),
                };
                response_msg = Some(ac_control_msg);
            } else {
                txn_result = "failure";
            }
        }
        RoleType::FullNode => {
            // node is not a validator, so send the transaction to upstream AC via networking stack
            if let Ok(response) =
                submit_transaction_upstream(request, &mut upstream_proxy_data, peer_id).await
            {
                let ac_control_msg = AdmissionControlMsg {
                    message: Some(AdmissionControlMsg_oneof::SubmitTransactionResponse(
                        response,
                    )),
                };
                response_msg = Some(ac_control_msg);
            } else {
                txn_result = "failure";
            }
        }
    };
    if let Some(response_msg) = response_msg {
        let response_data = response_msg.to_bytes().expect("fail to serialize proto");
        if let Err(err) = callback
            .send(Ok(response_data))
            .map_err(|_| format_err!("[admission-control] handling inbound rpc call timed out"))
        {
            txn_result = "failure";
            counters::TIMEOUT
                .with_label_values(&["full_node", "callback_timeout"])
                .inc();
            error!(
                "[admission control] failed to process transaction request, error: {:?}",
                err
            );
        }
    } else {
        txn_result = "failure";
        counters::TIMEOUT
            .with_label_values(&["full_node", "timeout"])
            .inc();
        error!(
            "[admission control] Did not get a response msg back from submit transaction upstream request",
        );
    }
    counters::TRANSACTION_LATENCY
        .with_label_values(&[txn_result])
        .observe(start_time.elapsed().as_secs() as f64);
    counters::TRANSACTION_PROXY
        .with_label_values(&["full_node", txn_result])
        .inc();
}

/// Validate transaction signature, then via VM, and add it to Mempool if it passes VM check.
pub(crate) async fn submit_transaction_to_mempool<M, V>(
    mut upstream_proxy_data: UpstreamProxyData<M, V>,
    req: SubmitTransactionRequest,
) -> Result<SubmitTransactionResponse>
where
    M: MempoolClientTrait,
    V: TransactionValidation,
{
    // Drop requests first if mempool is full (validator is lagging behind) so not to consume
    // unnecessary resources.
    if !can_send_txn_to_mempool(&mut upstream_proxy_data).await? {
        debug!("Mempool is full");
        counters::TRANSACTION_SUBMISSION
            .with_label_values(&["rejected", "mempool_full"])
            .inc();
        let mut response = SubmitTransactionResponse::default();
        let mut status = MempoolAddTransactionStatus::default();
        status.set_code(MempoolIsFull);
        status.message = "Mempool is full".to_string();
        response.status = Some(Status::MempoolStatus(status));
        return Ok(response);
    }

    let txn_proto = req.transaction.clone().unwrap_or_else(Default::default);

    let transaction = match SignedTransaction::try_from(txn_proto.clone()) {
        Ok(t) => t,
        Err(e) => {
            security_log(SecurityEvent::InvalidTransactionAC)
                .error(&e)
                .data(&txn_proto)
                .log();
            let mut response = SubmitTransactionResponse::default();
            response.status = Some(Status::AcStatus(
                AdmissionControlStatus::Rejected("submit txn rejected".to_string()).into(),
            ));
            counters::TRANSACTION_SUBMISSION
                .with_label_values(&["rejected", "invalid_txn"])
                .inc();
            return Ok(response);
        }
    };

    let gas_cost = transaction.max_gas_amount();
    let validation_status = upstream_proxy_data
        .vm_validator
        .validate_transaction(transaction.clone())
        .compat()
        .await
        .map_err(|e| {
            security_log(SecurityEvent::InvalidTransactionAC)
                .error(&e)
                .data(&transaction)
                .log();
            e
        })?;

    if let Some(validation_status) = validation_status {
        let mut response = SubmitTransactionResponse::default();
        counters::TRANSACTION_SUBMISSION
            .with_label_values(&["rejected", "vm_validation"])
            .inc();
        debug!(
            "txn failed in vm validation, status: {:?}, txn: {:?}",
            validation_status, transaction
        );
        response.status = Some(Status::VmStatus(validation_status.into()));
        return Ok(response);
    }
    let sender = transaction.sender();
    let account_state =
        get_account_state(upstream_proxy_data.storage_read_client.clone(), sender).await;
    let mut add_transaction_request = AddTransactionWithValidationRequest::default();
    add_transaction_request.transaction = req.transaction.clone();
    add_transaction_request.max_gas_cost = gas_cost;

    if let Ok((sequence_number, balance)) = account_state {
        add_transaction_request.account_balance = balance;
        add_transaction_request.latest_sequence_number = sequence_number;
    }

    add_txn_to_mempool(&mut upstream_proxy_data, add_transaction_request).await
}

async fn can_send_txn_to_mempool<M, V>(
    upstream_proxy_data: &mut UpstreamProxyData<M, V>,
) -> Result<bool>
where
    M: MempoolClientTrait,
{
    if upstream_proxy_data.need_to_check_mempool_before_validation {
        let req = HealthCheckRequest::default();
        let is_mempool_healthy = match upstream_proxy_data.mempool_client.as_mut() {
            Some(client) => client.health_check(req).await?.is_healthy,
            None => false,
        };
        return Ok(is_mempool_healthy);
    }
    Ok(true)
}

/// Add signed transaction to mempool once it passes vm check
async fn add_txn_to_mempool<M, V>(
    upstream_proxy_data: &mut UpstreamProxyData<M, V>,
    add_transaction_request: AddTransactionWithValidationRequest,
) -> Result<SubmitTransactionResponse>
where
    M: MempoolClientTrait,
{
    match upstream_proxy_data.mempool_client.as_mut() {
        Some(mempool_client) => {
            let mempool_result = mempool_client
                .add_transaction_with_validation(add_transaction_request.clone())
                .await?;

            debug!("[GRPC] Done with transaction submission request");
            let mut response = SubmitTransactionResponse::default();
            if let Some(status) = mempool_result.status {
                if status.code() == MempoolAddTransactionStatusCode::Valid {
                    counters::TRANSACTION_SUBMISSION
                        .with_label_values(&["accepted", ""])
                        .inc();
                    response.status =
                        Some(Status::AcStatus(AdmissionControlStatus::Accepted.into()));
                } else {
                    debug!(
                        "txn failed in mempool, status: {:?}, txn: {:?}",
                        status, add_transaction_request.transaction
                    );
                    counters::TRANSACTION_SUBMISSION
                        .with_label_values(&["rejected", "mempool"])
                        .inc();
                    response.status = Some(Status::MempoolStatus(status));
                }
            }
            Ok(response)
        }
        None => Err(format_err!("Mempool is not initialized")),
    }
}

// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    core_mempool::CoreMempool,
    network::{MempoolNetworkEvents, MempoolNetworkSender},
    shared_mempool::{
        coordinator::{coordinator, gc_coordinator, snapshot_job},
        peer_manager::PeerManager,
        types::{SharedMempool, SharedMempoolNotification},
    },
    ConsensusRequest, SubmissionStatus,
};
use anyhow::Result;
use channel::diem_channel;
use diem_config::{config::NodeConfig, network_id::NodeNetworkId};
use diem_infallible::{Mutex, RwLock};
use diem_types::{on_chain_config::OnChainConfigPayload, transaction::SignedTransaction};
use futures::channel::{
    mpsc::{self, Receiver, UnboundedSender},
    oneshot,
};
use mempool_notifications::MempoolNotificationListener;
use std::{collections::HashMap, sync::Arc};
use storage_interface::DbReader;
use tokio::runtime::{Builder, Handle, Runtime};
use vm_validator::vm_validator::{TransactionValidation, VMValidator};

/// Bootstrap of SharedMempool.
/// Creates a separate Tokio Runtime that runs the following routines:
///   - outbound_sync_task (task that periodically broadcasts transactions to peers).
///   - inbound_network_task (task that handles inbound mempool messages and network events).
///   - gc_task (task that performs GC of all expired transactions by SystemTTL).
pub(crate) fn start_shared_mempool<V>(
    executor: &Handle,
    config: &NodeConfig,
    mempool: Arc<Mutex<CoreMempool>>,
    // First element in tuple is the network ID.
    // See `NodeConfig::is_upstream_peer` for the definition of network ID.
    mempool_network_handles: Vec<(NodeNetworkId, MempoolNetworkSender, MempoolNetworkEvents)>,
    client_events: mpsc::Receiver<(SignedTransaction, oneshot::Sender<Result<SubmissionStatus>>)>,
    consensus_requests: mpsc::Receiver<ConsensusRequest>,
    mempool_listener: MempoolNotificationListener,
    mempool_reconfig_events: diem_channel::Receiver<(), OnChainConfigPayload>,
    db: Arc<dyn DbReader>,
    validator: Arc<RwLock<V>>,
    subscribers: Vec<UnboundedSender<SharedMempoolNotification>>,
) where
    V: TransactionValidation + 'static,
{
    let peer_manager = Arc::new(PeerManager::new(config.base.role, config.mempool.clone()));

    let mut all_network_events = vec![];
    let mut network_senders = HashMap::new();
    for (network_id, network_sender, network_events) in mempool_network_handles.into_iter() {
        all_network_events.push((network_id.clone(), network_events));
        network_senders.insert(network_id, network_sender);
    }

    let smp = SharedMempool {
        mempool: mempool.clone(),
        config: config.mempool.clone(),
        network_senders,
        db,
        validator,
        peer_manager,
        subscribers,
    };

    executor.spawn(coordinator(
        smp,
        executor.clone(),
        all_network_events,
        client_events,
        consensus_requests,
        mempool_listener,
        mempool_reconfig_events,
    ));

    executor.spawn(gc_coordinator(
        mempool.clone(),
        config.mempool.system_transaction_gc_interval_ms,
    ));

    executor.spawn(snapshot_job(
        mempool,
        config.mempool.mempool_snapshot_interval_secs,
    ));
}

pub fn bootstrap(
    config: &NodeConfig,
    db: Arc<dyn DbReader>,
    // The first element in the tuple is the ID of the network that this network is a handle to.
    // See `NodeConfig::is_upstream_peer` for the definition of network ID.
    mempool_network_handles: Vec<(NodeNetworkId, MempoolNetworkSender, MempoolNetworkEvents)>,
    client_events: Receiver<(SignedTransaction, oneshot::Sender<Result<SubmissionStatus>>)>,
    consensus_requests: Receiver<ConsensusRequest>,
    mempool_listener: MempoolNotificationListener,
    mempool_reconfig_events: diem_channel::Receiver<(), OnChainConfigPayload>,
) -> Runtime {
    let runtime = Builder::new_multi_thread()
        .thread_name("shared-mem")
        .enable_all()
        .build()
        .expect("[shared mempool] failed to create runtime");
    let mempool = Arc::new(Mutex::new(CoreMempool::new(config)));
    let vm_validator = Arc::new(RwLock::new(VMValidator::new(Arc::clone(&db))));
    start_shared_mempool(
        runtime.handle(),
        config,
        mempool,
        mempool_network_handles,
        client_events,
        consensus_requests,
        mempool_listener,
        mempool_reconfig_events,
        db,
        vm_validator,
        vec![],
    );
    runtime
}

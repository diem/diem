// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::{
    core_mempool::{CoreMempool, TimelineState},
    network::{MempoolNetworkEvents, MempoolNetworkSender},
    shared_mempool::{start_shared_mempool, SubmissionStatus},
    CommitNotification, ConsensusRequest,
};
use anyhow::{format_err, Result};
use channel::{self, libra_channel, message_queues::QueueStyle};
use futures::channel::{
    mpsc::{self, unbounded},
    oneshot,
};
use libra_config::config::{NetworkConfig, NodeConfig};
use libra_types::{mempool_status::MempoolStatusCode, transaction::SignedTransaction, PeerId};
use network::peer_manager::conn_status_channel;
use std::{
    num::NonZeroUsize,
    sync::{Arc, Mutex},
};
use storage_service::mocks::mock_storage_client::MockStorageReadClient;
use tokio::runtime::{Builder, Runtime};
use vm_validator::mocks::mock_vm_validator::MockVMValidator;

/// Mock of a running instance of shared mempool
pub struct MockSharedMempool {
    _runtime: Runtime,
    /// sender from admission control to shared mempool
    pub ac_client: mpsc::Sender<(SignedTransaction, oneshot::Sender<Result<SubmissionStatus>>)>,
    /// mempool
    pub mempool: Arc<Mutex<CoreMempool>>,
    /// sender from consensus to shared mempool
    pub consensus_sender: mpsc::Sender<ConsensusRequest>,
    /// sender from state sync to shared mempool
    pub state_sync_sender: Option<mpsc::Sender<CommitNotification>>,
}

impl MockSharedMempool {
    /// Creates a mock of a running instance of shared mempool
    /// Returns the runtime on which the shared mempool is running
    /// and the channel through which shared mempool receives client events
    pub fn new(state_sync: Option<mpsc::Receiver<CommitNotification>>) -> Self {
        let runtime = Builder::new()
            .thread_name("mock-shared-mem-")
            .threaded_scheduler()
            .enable_all()
            .build()
            .expect("[mock shared mempool] failed to create runtime");

        let peer_id = PeerId::random();
        let mut validator_network_config = NetworkConfig::default();
        validator_network_config.peer_id = peer_id;
        let mut config = NodeConfig::random();
        config.validator_network = Some(validator_network_config);

        let mempool = Arc::new(Mutex::new(CoreMempool::new(&config)));
        let (network_reqs_tx, _network_reqs_rx) =
            libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
        let (_network_notifs_tx, network_notifs_rx) =
            libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
        let (_, conn_notifs_rx) = conn_status_channel::new();
        let network_sender = MempoolNetworkSender::new(network_reqs_tx);
        let network_events = MempoolNetworkEvents::new(network_notifs_rx, conn_notifs_rx);
        let (sender, _subscriber) = unbounded();
        let (ac_client, client_events) = mpsc::channel(1_024);
        let (consensus_sender, consensus_events) = mpsc::channel(1_024);
        let (state_sync_sender, state_sync_events) = match state_sync {
            None => {
                let (sender, events) = mpsc::channel(1_024);
                (Some(sender), events)
            }
            Some(state_sync) => (None, state_sync),
        };
        let (_reconfig_event_publisher, reconfig_event_subscriber) =
            libra_channel::new(QueueStyle::LIFO, NonZeroUsize::new(1).unwrap(), None);
        let network_handles = vec![(peer_id, network_sender, network_events)];

        start_shared_mempool(
            runtime.handle(),
            &config,
            mempool.clone(),
            network_handles,
            client_events,
            consensus_events,
            state_sync_events,
            reconfig_event_subscriber,
            Arc::new(MockStorageReadClient),
            Arc::new(MockVMValidator),
            vec![sender],
            None,
        );

        Self {
            _runtime: runtime,
            ac_client,
            mempool,
            consensus_sender,
            state_sync_sender,
        }
    }

    /// add txns to mempool
    pub fn add_txns(&self, txns: Vec<SignedTransaction>) -> Result<()> {
        {
            let mut pool = self
                .mempool
                .lock()
                .expect("[mock shared mempool] failed to acquire mempool lock");
            for txn in txns {
                if pool.add_txn(txn, 0, 0, 1000, TimelineState::NotReady).code
                    != MempoolStatusCode::Accepted
                {
                    return Err(format_err!("failed to insert into mock mempool"));
                };
            }
        }
        Ok(())
    }

    /// true if all given txns are in mempool, else false
    pub fn read_timeline(&self, timeline_id: u64, count: usize) -> Vec<SignedTransaction> {
        let mut pool = self
            .mempool
            .lock()
            .expect("[mock shared mempool] failed to acquire mempool lock");
        pool.read_timeline(timeline_id, count).0
    }
}

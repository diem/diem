// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    core_mempool::{CoreMempool, TimelineState},
    network::{MempoolNetworkEvents, MempoolNetworkSender},
    shared_mempool::start_shared_mempool,
    ConsensusRequest, MempoolClientSender,
};
use anyhow::{format_err, Result};
use channel::{self, diem_channel, message_queues::QueueStyle};
use diem_config::{
    config::{NetworkConfig, NodeConfig},
    network_id::NetworkId,
};
use diem_infallible::{Mutex, RwLock};
use diem_types::{
    account_config::AccountSequenceInfo,
    mempool_status::MempoolStatusCode,
    on_chain_config::{
        DiemPaymentNetworkConfigRegistry, OnChainConfigRegistry, ON_CHAIN_CONFIG_REGISTRY,
    },
    protocol_spec::DpnProto,
    transaction::{GovernanceRole, SignedTransaction},
};
use event_notifications::EventSubscriptionService;
use futures::channel::mpsc;
use mempool_notifications::{self, MempoolNotifier};
use network::{
    application::storage::PeerMetadataStorage,
    peer_manager::{conn_notifs_channel, ConnectionRequestSender, PeerManagerRequestSender},
    protocols::network::{NewNetworkEvents, NewNetworkSender},
};
use std::{collections::HashSet, sync::Arc};
use storage_interface::{mock::MockDbReaderWriter, DbReaderWriter};
use tokio::runtime::{Builder, Handle, Runtime};
use vm_validator::{
    mocks::mock_vm_validator::MockVMValidator, vm_validator::TransactionValidation,
};

/// Mock of a running instance of shared mempool.
pub struct MockSharedMempool {
    _runtime: Option<Runtime>,
    _handle: Option<Handle>,
    pub ac_client: MempoolClientSender,
    pub mempool: Arc<Mutex<CoreMempool>>,
    pub consensus_sender: mpsc::Sender<ConsensusRequest>,
    pub mempool_notifier: MempoolNotifier,
}

impl MockSharedMempool {
    /// Creates a mock of a running instance of shared mempool.
    /// Returns the runtime on which the shared mempool is running
    /// and the channel through which shared mempool receives client events.
    pub fn new() -> Self {
        let runtime = Builder::new_multi_thread()
            .thread_name("mock-shared-mem")
            .enable_all()
            .build()
            .expect("[mock shared mempool] failed to create runtime");
        let (ac_client, mempool, consensus_sender, mempool_notifier) = Self::start(
            runtime.handle(),
            &DbReaderWriter::new(MockDbReaderWriter),
            MockVMValidator,
        );
        Self {
            _runtime: Some(runtime),
            _handle: None,
            ac_client,
            mempool,
            consensus_sender,
            mempool_notifier,
        }
    }

    /// Creates a mock of a running instance of shared mempool inside a tokio runtime;
    /// Holds a runtime handle instead.
    pub fn new_in_runtime<V: TransactionValidation + 'static>(
        db: &DbReaderWriter<DpnProto>,
        validator: V,
    ) -> Self {
        let handle = Handle::current();
        let (ac_client, mempool, consensus_sender, mempool_notifier) =
            Self::start(&handle, db, validator);
        Self {
            _runtime: None,
            _handle: Some(handle),
            ac_client,
            mempool,
            consensus_sender,
            mempool_notifier,
        }
    }

    pub fn start<V: TransactionValidation + 'static>(
        handle: &Handle,
        db: &DbReaderWriter<DpnProto>,
        validator: V,
    ) -> (
        MempoolClientSender,
        Arc<Mutex<CoreMempool>>,
        mpsc::Sender<ConsensusRequest>,
        MempoolNotifier,
    ) {
        let mut config = NodeConfig::random();
        config.validator_network = Some(NetworkConfig::network_with_id(NetworkId::Validator));

        let mempool = Arc::new(Mutex::new(CoreMempool::new(&config)));
        let (network_reqs_tx, _network_reqs_rx) = diem_channel::new(QueueStyle::FIFO, 8, None);
        let (connection_reqs_tx, _) = diem_channel::new(QueueStyle::FIFO, 8, None);
        let (_network_notifs_tx, network_notifs_rx) = diem_channel::new(QueueStyle::FIFO, 8, None);
        let (_, conn_notifs_rx) = conn_notifs_channel::new();
        let network_sender = MempoolNetworkSender::new(
            PeerManagerRequestSender::new(network_reqs_tx),
            ConnectionRequestSender::new(connection_reqs_tx),
        );
        let network_events = MempoolNetworkEvents::new(network_notifs_rx, conn_notifs_rx);
        let (ac_client, client_events) = mpsc::channel(1_024);
        let (consensus_sender, consensus_events) = mpsc::channel(1_024);
        let (mempool_notifier, mempool_listener) =
            mempool_notifications::new_mempool_notifier_listener_pair();
        let mut event_subscriber = EventSubscriptionService::new(
            ON_CHAIN_CONFIG_REGISTRY,
            Arc::new(RwLock::new(db.clone())),
        );
        let reconfig_event_subscriber = event_subscriber.subscribe_to_reconfigurations().unwrap();
        let network_handles = vec![(NetworkId::Validator, network_sender, network_events)];
        let peer_metadata_storage = PeerMetadataStorage::new(&[NetworkId::Validator]);

        start_shared_mempool(
            handle,
            &config,
            mempool.clone(),
            network_handles,
            client_events,
            consensus_events,
            mempool_listener,
            reconfig_event_subscriber,
            db.reader.clone(),
            Arc::new(RwLock::new(validator)),
            vec![],
            peer_metadata_storage,
        );

        (ac_client, mempool, consensus_sender, mempool_notifier)
    }

    pub fn add_txns(&self, txns: Vec<SignedTransaction>) -> Result<()> {
        {
            let mut pool = self.mempool.lock();
            for txn in txns {
                if pool
                    .add_txn(
                        txn.clone(),
                        0,
                        txn.gas_unit_price(),
                        AccountSequenceInfo::Sequential(0),
                        TimelineState::NotReady,
                        GovernanceRole::NonGovernanceRole,
                    )
                    .code
                    != MempoolStatusCode::Accepted
                {
                    return Err(format_err!("failed to insert into mock mempool"));
                };
            }
        }
        Ok(())
    }

    pub fn get_txns(&self, size: u64) -> Vec<SignedTransaction> {
        let pool = self.mempool.lock();
        pool.get_block(size, HashSet::new())
    }

    pub fn remove_txn(&self, txn: &SignedTransaction) {
        let mut pool = self.mempool.lock();
        pool.remove_transaction(&txn.sender(), txn.sequence_number(), false)
    }

    /// True if all the given txns are in mempool, else false.
    pub fn read_timeline(&self, timeline_id: u64, count: usize) -> Vec<SignedTransaction> {
        let pool = self.mempool.lock();
        pool.read_timeline(timeline_id, count)
            .0
            .into_iter()
            .collect()
    }
}

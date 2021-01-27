// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::logging::{LogEntry, LogSchema};
use anyhow::{format_err, Result};
use diem_logger::prelude::*;
use diem_types::{
    epoch_change::Verifier, epoch_state::EpochState, ledger_info::LedgerInfoWithSignatures,
    transaction::Version,
};
use executor_types::ExecutedTrees;
use std::{collections::BTreeMap, ops::Bound::Included};

// DS to help sync requester to keep track of ledger infos in the future
// if it is lagging far behind the upstream node
// Should only be modified upon local storage sync
pub(crate) struct PendingLedgerInfos {
    // In-memory store of ledger infos that are pending commits
    // (k, v) - (LI version, LI)
    pending_li_queue: BTreeMap<Version, LedgerInfoWithSignatures>,
    // max size limit on `pending_li_queue`, to prevent OOM
    max_pending_li_limit: usize,
    // target li
    target_li: Option<LedgerInfoWithSignatures>,
}

impl PendingLedgerInfos {
    pub(crate) fn new(max_pending_li_limit: usize) -> Self {
        Self {
            pending_li_queue: BTreeMap::new(),
            max_pending_li_limit,
            target_li: None,
        }
    }

    /// Adds `new_li` to the queue of pending LI's
    pub(crate) fn add_li(&mut self, new_li: LedgerInfoWithSignatures) {
        if self.pending_li_queue.len() >= self.max_pending_li_limit {
            warn!(
                LogSchema::new(LogEntry::ProcessChunkResponse),
                "pending LI store reached max capacity {}, failed to add LI {}",
                self.max_pending_li_limit,
                new_li
            );
            return;
        }

        // update pending_ledgers if new LI is ahead of target LI (in terms of version)
        let target_version = self
            .target_li
            .as_ref()
            .map_or(0, |li| li.ledger_info().version());
        if new_li.ledger_info().version() > target_version {
            self.pending_li_queue
                .insert(new_li.ledger_info().version(), new_li);
        }
    }

    pub(crate) fn update(&mut self, sync_state: &SyncState, chunk_limit: u64) -> Result<()> {
        let highest_committed_li = sync_state.committed_version();
        let highest_synced = sync_state.synced_version();

        // prune any pending LIs that are older than the latest local synced version
        let prune_version = highest_synced
            .checked_add(1)
            .ok_or_else(|| format_err!("Prune version has overflown!"))?;
        self.pending_li_queue = self.pending_li_queue.split_off(&prune_version);

        // pick target LI to use for sending ProgressiveTargetType requests.
        self.target_li = if highest_committed_li == highest_synced {
            // try to find LI with max version that will fit in a single chunk
            let highest_version = highest_synced
                .checked_add(chunk_limit)
                .ok_or_else(|| format_err!("Highest version has overflown!"))?;
            self.pending_li_queue
                .range((Included(0), Included(highest_version)))
                .rev()
                .next()
                .map(|(_version, ledger_info)| ledger_info.clone())
        } else {
            self.pending_li_queue
                .iter()
                .next()
                .map(|(_version, ledger_info)| ledger_info.clone())
        };
        Ok(())
    }

    pub(crate) fn target_li(&self) -> Option<LedgerInfoWithSignatures> {
        self.target_li.clone()
    }

    pub(crate) fn highest_version(&self) -> Option<Version> {
        self.pending_li_queue.keys().last().cloned()
    }
}

/// SyncState contains the following fields:
/// * `committed_ledger_info` holds the latest certified ledger info (committed to storage),
///    i.e., the ledger info for the highest version for which storage has all ledger state.
/// * `synced_trees` holds the latest transaction accumulator and state tree (which may
///    or may not be committed to storage), i.e., some ledger state for the next highest
///    ledger info version is missing.
/// * `trusted_epoch_state` corresponds to the current epoch if the highest committed
///    ledger info (`committed_ledger_info`) is in the middle of the epoch, otherwise, it
///    corresponds to the next epoch if the highest committed ledger info ends the epoch.
///
/// Note: `committed_ledger_info` is used for helping other Diem nodes synchronize (i.e.,
/// it corresponds to the highest version we have a proof for in storage). `synced_trees`
/// is used locally for retrieving missing chunks for the local storage.
#[derive(Clone, Debug)]
pub struct SyncState {
    committed_ledger_info: LedgerInfoWithSignatures,
    synced_trees: ExecutedTrees,
    trusted_epoch_state: EpochState,
}

impl SyncState {
    pub fn new(
        committed_ledger_info: LedgerInfoWithSignatures,
        synced_trees: ExecutedTrees,
        current_epoch_state: EpochState,
    ) -> Self {
        let trusted_epoch_state = committed_ledger_info
            .ledger_info()
            .next_epoch_state()
            .cloned()
            .unwrap_or(current_epoch_state);

        SyncState {
            committed_ledger_info,
            synced_trees,
            trusted_epoch_state,
        }
    }

    pub fn committed_epoch(&self) -> u64 {
        self.committed_ledger_info.ledger_info().epoch()
    }

    pub fn committed_ledger_info(&self) -> LedgerInfoWithSignatures {
        self.committed_ledger_info.clone()
    }

    pub fn committed_version(&self) -> u64 {
        self.committed_ledger_info.ledger_info().version()
    }

    /// Returns the highest available version in the local storage, even if it's not
    /// committed (i.e., covered by a ledger info).
    pub fn synced_version(&self) -> u64 {
        self.synced_trees.version().unwrap_or(0)
    }

    pub fn trusted_epoch(&self) -> u64 {
        self.trusted_epoch_state.epoch
    }

    pub fn verify_ledger_info(&self, ledger_info: &LedgerInfoWithSignatures) -> Result<()> {
        self.trusted_epoch_state.verify(ledger_info)
    }
}

#[cfg(any(feature = "fuzzing", test))]
pub(crate) mod test_utils {
    use crate::{
        coordinator::StateSyncCoordinator,
        executor_proxy::{ExecutorProxy, ExecutorProxyTrait},
        network::StateSyncSender,
    };
    use diem_types::waypoint::Waypoint;

    use channel::{diem_channel, message_queues::QueueStyle};
    use diem_config::{
        config::{RoleType, StateSyncConfig, UpstreamConfig},
        network_id::{NetworkId, NodeNetworkId},
    };
    use diem_types::transaction::{Transaction, WriteSetPayload};
    use diem_vm::DiemVM;
    use diemdb::DiemDB;
    use executor::Executor;
    use executor_test_helpers::bootstrap_genesis;
    use futures::channel::mpsc;
    use network::{
        peer_manager::{ConnectionRequestSender, PeerManagerRequestSender},
        protocols::network::NewNetworkSender,
    };
    use std::collections::HashMap;
    use storage_interface::DbReaderWriter;

    pub(crate) fn create_state_sync_coordinator_for_tests() -> StateSyncCoordinator<ExecutorProxy> {
        // Generate a genesis change set
        let (genesis, _) = vm_genesis::test_genesis_change_set_and_validators(Some(1));

        // Create test diem database
        let db_path = diem_temppath::TempPath::new();
        db_path.create_as_dir().unwrap();
        let (db, db_rw) = DbReaderWriter::wrap(DiemDB::new_for_test(db_path.path()));

        // Bootstrap the genesis transaction
        let genesis_txn = Transaction::GenesisTransaction(WriteSetPayload::Direct(genesis));
        bootstrap_genesis::<DiemVM>(&db_rw, &genesis_txn).unwrap();

        // Create executor proxy
        let chunk_executor = Box::new(Executor::<DiemVM>::new(db_rw));
        let executor_proxy = ExecutorProxy::new(db, chunk_executor, vec![]);

        // Get initial state
        let initial_state = executor_proxy.get_local_storage_state().unwrap();

        // Setup network senders
        let (network_reqs_tx, _network_reqs_rx) = diem_channel::new(QueueStyle::FIFO, 8, None);
        let (connection_reqs_tx, _) = diem_channel::new(QueueStyle::FIFO, 8, None);
        let network_sender = StateSyncSender::new(
            PeerManagerRequestSender::new(network_reqs_tx),
            ConnectionRequestSender::new(connection_reqs_tx),
        );
        let node_network_id = NodeNetworkId::new(NetworkId::Validator, 0);
        let network_senders = vec![(node_network_id, network_sender)]
            .into_iter()
            .collect::<HashMap<_, _>>();

        // Create channel senders and receivers
        let (_coordinator_sender, coordinator_receiver) = mpsc::unbounded();
        let (mempool_sender, _mempool_receiver) = mpsc::channel(1);

        // Return the new state sync coordinator
        StateSyncCoordinator::new(
            coordinator_receiver,
            mempool_sender,
            network_senders,
            RoleType::Validator,
            Waypoint::default(),
            StateSyncConfig::default(),
            UpstreamConfig::default(),
            executor_proxy,
            initial_state,
        )
        .unwrap()
    }
}

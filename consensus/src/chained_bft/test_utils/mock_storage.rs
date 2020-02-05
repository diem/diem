// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::persistent_liveness_storage::{
    LedgerRecoveryData, PersistentLivenessStorage, RecoveryData,
};

use anyhow::Result;
use consensus_types::{
    block::Block, common::Payload, quorum_cert::QuorumCert,
    timeout_certificate::TimeoutCertificate, vote::Vote,
};
use executor::ExecutedTrees;
use futures::executor::block_on;
use libra_crypto::HashValue;
use libra_types::crypto_proxies::ValidatorSet;
use libra_types::ledger_info::LedgerInfo;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub struct MockSharedStorage<T> {
    // Safety state
    pub block: Mutex<HashMap<HashValue, Block<T>>>,
    pub qc: Mutex<HashMap<HashValue, QuorumCert>>,
    pub last_vote: Mutex<Option<Vote>>,

    // Liveness state
    pub highest_timeout_certificate: Mutex<Option<TimeoutCertificate>>,
    pub validator_set: ValidatorSet,
}

/// A storage that simulates the operations in-memory, used in the tests that cares about storage
/// consistency.
pub struct MockStorage<T> {
    pub shared_storage: Arc<MockSharedStorage<T>>,
    storage_ledger: Mutex<LedgerInfo>,
}

impl<T: Payload> MockStorage<T> {
    pub fn new(shared_storage: Arc<MockSharedStorage<T>>) -> Self {
        MockStorage {
            shared_storage,
            storage_ledger: Mutex::new(LedgerInfo::genesis()),
        }
    }

    pub fn commit_to_storage(&self, ledger: LedgerInfo) {
        *self.storage_ledger.lock().unwrap() = ledger;

        if let Err(e) = self.verify_consistency() {
            panic!("invalid db after commit: {}", e);
        }
    }

    pub fn get_ledger_recovery_data(&self) -> LedgerRecoveryData<T> {
        LedgerRecoveryData::new(
            &self.storage_ledger.lock().unwrap(),
            self.shared_storage.validator_set.clone(),
        )
    }

    pub fn try_start(&self) -> Result<RecoveryData<T>> {
        let ledger_recovery_data = self.get_ledger_recovery_data();
        let mut blocks: Vec<_> = self
            .shared_storage
            .block
            .lock()
            .unwrap()
            .clone()
            .into_iter()
            .map(|(_, v)| v)
            .collect();
        let quorum_certs = self
            .shared_storage
            .qc
            .lock()
            .unwrap()
            .clone()
            .into_iter()
            .map(|(_, v)| v)
            .collect();
        blocks.sort_by_key(Block::round);
        RecoveryData::new(
            self.shared_storage.last_vote.lock().unwrap().clone(),
            ledger_recovery_data,
            blocks,
            quorum_certs,
            &self.storage_ledger.lock().unwrap(),
            ExecutedTrees::new_empty(),
            self.shared_storage
                .highest_timeout_certificate
                .lock()
                .unwrap()
                .clone(),
        )
    }

    pub fn verify_consistency(&self) -> Result<()> {
        self.try_start().map(|_| ())
    }

    pub fn start_for_testing(validator_set: ValidatorSet) -> (RecoveryData<T>, Arc<Self>) {
        let shared_storage = Arc::new(MockSharedStorage {
            block: Mutex::new(HashMap::new()),
            qc: Mutex::new(HashMap::new()),
            last_vote: Mutex::new(None),
            highest_timeout_certificate: Mutex::new(None),
            validator_set,
        });
        let storage = Arc::new(MockStorage::new(Arc::clone(&shared_storage)));

        (block_on(storage.start()), storage)
    }
}

// A impl that always start from genesis.
#[async_trait::async_trait]
impl<T: Payload> PersistentLivenessStorage<T> for MockStorage<T> {
    fn save_tree(&self, blocks: Vec<Block<T>>, quorum_certs: Vec<QuorumCert>) -> Result<()> {
        for block in blocks {
            self.shared_storage
                .block
                .lock()
                .unwrap()
                .insert(block.id(), block);
        }
        for qc in quorum_certs {
            self.shared_storage
                .qc
                .lock()
                .unwrap()
                .insert(qc.certified_block().id(), qc);
        }
        if let Err(e) = self.verify_consistency() {
            panic!("invalid db after save tree: {}", e);
        }
        Ok(())
    }

    fn prune_tree(&self, block_id: Vec<HashValue>) -> Result<()> {
        for id in block_id {
            self.shared_storage.block.lock().unwrap().remove(&id);
            self.shared_storage.qc.lock().unwrap().remove(&id);
        }
        if let Err(e) = self.verify_consistency() {
            panic!("invalid db after prune tree: {}", e);
        }
        Ok(())
    }

    fn save_state(&self, last_vote: &Vote) -> Result<()> {
        self.shared_storage
            .last_vote
            .lock()
            .unwrap()
            .replace(last_vote.clone());
        Ok(())
    }

    async fn recover_from_ledger(&self) -> LedgerRecoveryData<T> {
        self.get_ledger_recovery_data()
    }

    async fn start(&self) -> RecoveryData<T> {
        self.try_start().unwrap()
    }

    fn save_highest_timeout_cert(
        &self,
        highest_timeout_certificate: TimeoutCertificate,
    ) -> Result<()> {
        self.shared_storage
            .highest_timeout_certificate
            .lock()
            .unwrap()
            .replace(highest_timeout_certificate);
        Ok(())
    }
}

/// A storage that ignores any requests, used in the tests that don't care about the storage.
pub struct EmptyStorage;

impl EmptyStorage {
    pub fn start_for_testing<T: Payload>() -> (RecoveryData<T>, Arc<Self>) {
        let storage = Arc::new(EmptyStorage);
        (block_on(storage.start()), storage)
    }
}

#[async_trait::async_trait]
impl<T: Payload> PersistentLivenessStorage<T> for EmptyStorage {
    fn save_tree(&self, _: Vec<Block<T>>, _: Vec<QuorumCert>) -> Result<()> {
        Ok(())
    }

    fn prune_tree(&self, _: Vec<HashValue>) -> Result<()> {
        Ok(())
    }

    fn save_state(&self, _: &Vote) -> Result<()> {
        Ok(())
    }

    async fn recover_from_ledger(&self) -> LedgerRecoveryData<T> {
        LedgerRecoveryData::new(&LedgerInfo::genesis(), ValidatorSet::new(vec![]))
    }

    async fn start(&self) -> RecoveryData<T> {
        RecoveryData::new(
            None,
            self.recover_from_ledger().await,
            vec![],
            vec![],
            &LedgerInfo::genesis(),
            ExecutedTrees::new_empty(),
            None,
        )
        .unwrap()
    }
    fn save_highest_timeout_cert(&self, _: TimeoutCertificate) -> Result<()> {
        Ok(())
    }
}

// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::{
    epoch_manager::LivenessStorageData,
    persistent_liveness_storage::{
        LedgerRecoveryData, PersistentLivenessStorage, RecoveryData, RootMetadata,
    },
};
use anyhow::Result;
use consensus_types::{
    block::Block, common::Payload, quorum_cert::QuorumCert,
    timeout_certificate::TimeoutCertificate, vote::Vote,
};
use libra_crypto::HashValue;
use libra_types::{
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    on_chain_config::ValidatorSet,
    validator_change::ValidatorChangeProof,
};
use std::{
    collections::{BTreeMap, HashMap},
    marker::PhantomData,
    sync::{Arc, Mutex},
};
use storage_interface::DbReader;

pub struct MockSharedStorage<T> {
    // Safety state
    pub block: Mutex<HashMap<HashValue, Block<T>>>,
    pub qc: Mutex<HashMap<HashValue, QuorumCert>>,
    pub lis: Mutex<HashMap<u64, LedgerInfoWithSignatures>>,
    pub last_vote: Mutex<Option<Vote>>,

    // Liveness state
    pub highest_timeout_certificate: Mutex<Option<TimeoutCertificate>>,
    pub validator_set: ValidatorSet,
}

impl<T: Payload> MockSharedStorage<T> {
    pub fn new(validator_set: ValidatorSet) -> Self {
        MockSharedStorage {
            block: Mutex::new(HashMap::new()),
            qc: Mutex::new(HashMap::new()),
            lis: Mutex::new(HashMap::new()),
            last_vote: Mutex::new(None),
            highest_timeout_certificate: Mutex::new(None),
            validator_set,
        }
    }
}

/// A storage that simulates the operations in-memory, used in the tests that cares about storage
/// consistency.
pub struct MockStorage<T> {
    pub shared_storage: Arc<MockSharedStorage<T>>,
    storage_ledger: Mutex<LedgerInfo>,
}

impl<T: Payload> MockStorage<T> {
    pub fn new(shared_storage: Arc<MockSharedStorage<T>>) -> Self {
        let validator_set = Some(shared_storage.validator_set.clone());
        let li = LedgerInfo::mock_genesis(validator_set);
        let lis = LedgerInfoWithSignatures::new(li.clone(), BTreeMap::new());
        shared_storage
            .lis
            .lock()
            .unwrap()
            .insert(lis.ledger_info().version(), lis);
        MockStorage {
            shared_storage,
            storage_ledger: Mutex::new(li),
        }
    }

    pub fn new_with_ledger_info(
        shared_storage: Arc<MockSharedStorage<T>>,
        ledger_info: LedgerInfo,
    ) -> Self {
        let li = if ledger_info.next_validator_set().is_some() {
            ledger_info.clone()
        } else {
            let validator_set = Some(shared_storage.validator_set.clone());
            LedgerInfo::mock_genesis(validator_set)
        };
        let lis = LedgerInfoWithSignatures::new(li, BTreeMap::new());
        shared_storage
            .lis
            .lock()
            .unwrap()
            .insert(lis.ledger_info().version(), lis);
        MockStorage {
            shared_storage,
            storage_ledger: Mutex::new(ledger_info),
        }
    }

    pub fn get_ledger_info(&self) -> LedgerInfo {
        self.storage_ledger.lock().unwrap().clone()
    }

    pub fn commit_to_storage(&self, ledger: LedgerInfo) {
        *self.storage_ledger.lock().unwrap() = ledger;

        if let Err(e) = self.verify_consistency() {
            panic!("invalid db after commit: {}", e);
        }
    }

    pub fn get_validator_set(&self) -> &ValidatorSet {
        &self.shared_storage.validator_set
    }

    pub fn get_ledger_recovery_data(&self) -> LedgerRecoveryData {
        LedgerRecoveryData::new(self.storage_ledger.lock().unwrap().clone())
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
            RootMetadata::new_empty(),
            quorum_certs,
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
            lis: Mutex::new(HashMap::new()),
            last_vote: Mutex::new(None),
            highest_timeout_certificate: Mutex::new(None),
            validator_set,
        });
        let storage = Arc::new(MockStorage::new(Arc::clone(&shared_storage)));

        (
            storage
                .start()
                .expect_recovery_data("Mock storage should never fail constructing recovery data"),
            storage,
        )
    }
}

// A impl that always start from genesis.
impl<T: Payload> PersistentLivenessStorage<T> for MockStorage<T> {
    fn save_tree(&self, blocks: Vec<Block<T>>, quorum_certs: Vec<QuorumCert>) -> Result<()> {
        // When the shared storage is empty, we are expected to not able to construct an block tree
        // from it. During test we will intentionally clear shared_storage to simulate the situation
        // of restarting from an empty consensusDB
        let should_check_for_consistency = !(self.shared_storage.block.lock().unwrap().is_empty()
            && self.shared_storage.qc.lock().unwrap().is_empty());
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
        if should_check_for_consistency {
            if let Err(e) = self.verify_consistency() {
                panic!("invalid db after save tree: {}", e);
            }
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

    fn recover_from_ledger(&self) -> LedgerRecoveryData {
        self.get_ledger_recovery_data()
    }

    fn start(&self) -> LivenessStorageData<T> {
        match self.try_start() {
            Ok(recovery_data) => LivenessStorageData::RecoveryData(recovery_data),
            Err(_) => LivenessStorageData::LedgerRecoveryData(self.recover_from_ledger()),
        }
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

    fn retrieve_validator_change_proof(&self, version: u64) -> Result<ValidatorChangeProof> {
        let lis = self
            .shared_storage
            .lis
            .lock()
            .unwrap()
            .get(&version)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("LedgerInfo for version not found"))?;
        Ok(ValidatorChangeProof::new(vec![lis], false))
    }

    fn libra_db(&self) -> Arc<dyn DbReader> {
        unimplemented!()
    }
}

/// A storage that ignores any requests, used in the tests that don't care about the storage.
pub struct EmptyStorage<T> {
    _phantom: PhantomData<T>,
}

impl<T: Payload> EmptyStorage<T> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    pub fn start_for_testing() -> (RecoveryData<T>, Arc<Self>) {
        let storage = Arc::new(EmptyStorage::new());
        let recovery_data = storage
            .start()
            .expect_recovery_data("Empty storage should never fail constructing recovery data");
        (recovery_data, storage)
    }
}

impl<T: Payload> PersistentLivenessStorage<T> for EmptyStorage<T> {
    fn save_tree(&self, _: Vec<Block<T>>, _: Vec<QuorumCert>) -> Result<()> {
        Ok(())
    }

    fn prune_tree(&self, _: Vec<HashValue>) -> Result<()> {
        Ok(())
    }

    fn save_state(&self, _: &Vote) -> Result<()> {
        Ok(())
    }

    fn recover_from_ledger(&self) -> LedgerRecoveryData {
        LedgerRecoveryData::new(LedgerInfo::mock_genesis(None))
    }

    fn start(&self) -> LivenessStorageData<T> {
        match RecoveryData::new(
            None,
            self.recover_from_ledger(),
            vec![],
            RootMetadata::new_empty(),
            vec![],
            None,
        ) {
            Ok(recovery_data) => LivenessStorageData::RecoveryData(recovery_data),
            Err(e) => {
                eprintln!("{}", e);
                panic!("Construct recovery data during genesis should never fail");
            }
        }
    }

    fn save_highest_timeout_cert(&self, _: TimeoutCertificate) -> Result<()> {
        Ok(())
    }

    fn retrieve_validator_change_proof(&self, _version: u64) -> Result<ValidatorChangeProof> {
        unimplemented!()
    }

    fn libra_db(&self) -> Arc<dyn DbReader> {
        unimplemented!()
    }
}

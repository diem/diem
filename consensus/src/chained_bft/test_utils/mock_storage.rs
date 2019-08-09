// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::{
    common::Payload,
    consensus_types::{block::Block, quorum_cert::QuorumCert},
    liveness::pacemaker_timeout_manager::HighestTimeoutCertificates,
    persistent_storage::{PersistentLivenessStorage, PersistentStorage, RecoveryData},
    safety::safety_rules::ConsensusState,
};
use config::config::{NodeConfig, NodeConfigHelpers};
use crypto::HashValue;
use failure::Result;
use nextgen_crypto::ed25519::*;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub struct MockSharedStorage<T> {
    // Safety state
    pub block: Mutex<HashMap<HashValue, Block<T, Ed25519Signature>>>,
    pub qc: Mutex<HashMap<HashValue, QuorumCert<Ed25519Signature>>>,
    pub state: Mutex<ConsensusState>,

    // Liveness state
    pub highest_timeout_certificates: Mutex<HighestTimeoutCertificates<Ed25519Signature>>,
}

/// A storage that simulates the operations in-memory, used in the tests that cares about storage
/// consistency.
pub struct MockStorage<T> {
    pub shared_storage: Arc<MockSharedStorage<T>>,
}

impl<T: Payload> MockStorage<T> {
    pub fn new(shared_storage: Arc<MockSharedStorage<T>>) -> Self {
        MockStorage { shared_storage }
    }

    pub fn get_recovery_data(&self) -> Result<RecoveryData<T, Ed25519Signature>> {
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
        // There is no root_from_storage in MockStorage(unit tests), hence we use the consensus
        // root value;
        blocks.sort_by_key(Block::round);
        let root_from_storage = blocks[0].id();
        RecoveryData::new(
            self.shared_storage.state.lock().unwrap().clone(),
            blocks,
            quorum_certs,
            root_from_storage,
            self.shared_storage
                .highest_timeout_certificates
                .lock()
                .unwrap()
                .clone(),
        )
    }

    pub fn verify_consistency(&self) -> Result<()> {
        self.get_recovery_data().map(|_| ())
    }

    pub fn start_for_testing() -> (Arc<Self>, RecoveryData<T, Ed25519Signature>) {
        Self::start(&NodeConfigHelpers::get_single_node_test_config(false))
    }
}

impl<T: Payload> PersistentLivenessStorage<Ed25519Signature> for MockStorage<T> {
    fn save_highest_timeout_cert(
        &self,
        highest_timeout_certificates: HighestTimeoutCertificates<Ed25519Signature>,
    ) -> Result<()> {
        *self
            .shared_storage
            .highest_timeout_certificates
            .lock()
            .unwrap() = highest_timeout_certificates;
        Ok(())
    }
}

// A impl that always start from genesis.
impl<T: Payload> PersistentStorage<T, Ed25519Signature> for MockStorage<T> {
    fn persistent_liveness_storage(&self) -> Box<dyn PersistentLivenessStorage<Ed25519Signature>> {
        Box::new(MockStorage {
            shared_storage: Arc::clone(&self.shared_storage),
        })
    }

    fn save_tree(
        &self,
        blocks: Vec<Block<T, Ed25519Signature>>,
        quorum_certs: Vec<QuorumCert<Ed25519Signature>>,
    ) -> Result<()> {
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
                .insert(qc.certified_block_id(), qc);
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

    fn save_consensus_state(&self, state: ConsensusState) -> Result<()> {
        *self.shared_storage.state.lock().unwrap() = state;
        Ok(())
    }

    fn start(_config: &NodeConfig) -> (Arc<Self>, RecoveryData<T, Ed25519Signature>) {
        let shared_storage = Arc::new(MockSharedStorage {
            block: Mutex::new(HashMap::new()),
            qc: Mutex::new(HashMap::new()),
            state: Mutex::new(ConsensusState::default()),
            highest_timeout_certificates: Mutex::new(HighestTimeoutCertificates::new(None, None)),
        });
        let storage = MockStorage {
            shared_storage: Arc::clone(&shared_storage),
        };

        // The current assumption is that the genesis block version is 0.
        storage
            .save_tree(
                vec![Block::make_genesis_block()],
                vec![QuorumCert::certificate_for_genesis()],
            )
            .unwrap();
        (
            Arc::new(Self::new(shared_storage)),
            storage.get_recovery_data().unwrap(),
        )
    }
}

/// A storage that ignores any requests, used in the tests that don't care about the storage.
pub struct EmptyStorage;

impl EmptyStorage {
    pub fn start_for_testing<T: Payload>() -> (Arc<Self>, RecoveryData<T, Ed25519Signature>) {
        Self::start(&NodeConfigHelpers::get_single_node_test_config(false))
    }
}

impl PersistentLivenessStorage<Ed25519Signature> for EmptyStorage {
    fn save_highest_timeout_cert(
        &self,
        _: HighestTimeoutCertificates<Ed25519Signature>,
    ) -> Result<()> {
        Ok(())
    }
}

impl<T: Payload> PersistentStorage<T, Ed25519Signature> for EmptyStorage {
    fn persistent_liveness_storage(&self) -> Box<dyn PersistentLivenessStorage<Ed25519Signature>> {
        Box::new(EmptyStorage)
    }

    fn save_tree(
        &self,
        _: Vec<Block<T, Ed25519Signature>>,
        _: Vec<QuorumCert<Ed25519Signature>>,
    ) -> Result<()> {
        Ok(())
    }

    fn prune_tree(&self, _: Vec<HashValue>) -> Result<()> {
        Ok(())
    }

    fn save_consensus_state(&self, _: ConsensusState) -> Result<()> {
        Ok(())
    }

    fn start(_: &NodeConfig) -> (Arc<Self>, RecoveryData<T, Ed25519Signature>) {
        let genesis = Block::make_genesis_block();
        let genesis_qc = QuorumCert::certificate_for_genesis();
        let htc = HighestTimeoutCertificates::new(None, None);
        (
            Arc::new(EmptyStorage),
            RecoveryData::new(
                ConsensusState::default(),
                vec![genesis],
                vec![genesis_qc],
                HashValue::random(),
                htc,
            )
            .unwrap(),
        )
    }
}

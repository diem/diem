// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::{consensusdb::ConsensusDB, epoch_manager::LivenessStorageData};
use anyhow::{format_err, Context, Result};
use consensus_types::{
    block::Block, common::Payload, quorum_cert::QuorumCert,
    timeout_certificate::TimeoutCertificate, vote::Vote,
};
use debug_interface::prelude::*;
use executor_types::ExecutedTrees;
use libra_config::config::NodeConfig;
use libra_crypto::HashValue;
use libra_logger::prelude::*;
use libra_types::{
    block_info::Round, epoch_info::EpochInfo, ledger_info::LedgerInfo, transaction::Version,
    validator_info::ValidatorInfo, validator_set::ValidatorSet,
    validator_verifier::ValidatorVerifier,
};
use libradb::LibraDBTrait;
use std::{cmp::max, collections::HashSet, sync::Arc};

/// PersistentLivenessStorage is essential for maintaining liveness when a node crashes.  Specifically,
/// upon a restart, a correct node will recover.  Even if all nodes crash, liveness is
/// guaranteed.
/// Blocks persisted are proposed but not yet committed.  The committed state is persisted
/// via StateComputer.
pub trait PersistentLivenessStorage<T>: Send + Sync {
    /// Persist the blocks and quorum certs into storage atomically.
    fn save_tree(&self, blocks: Vec<Block<T>>, quorum_certs: Vec<QuorumCert>) -> Result<()>;

    /// Delete the corresponding blocks and quorum certs atomically.
    fn prune_tree(&self, block_ids: Vec<HashValue>) -> Result<()>;

    /// Persist consensus' state
    fn save_state(&self, vote: &Vote) -> Result<()>;

    /// Construct data that can be recovered from ledger
    fn recover_from_ledger(&self) -> LedgerRecoveryData;

    /// Construct necessary data to start consensus.
    fn start(&self) -> LivenessStorageData<T>;

    /// Persist the highest timeout certificate for improved liveness - proof for other replicas
    /// to jump to this round
    fn save_highest_timeout_cert(&self, highest_timeout_cert: TimeoutCertificate) -> Result<()>;
}

#[derive(Clone)]
pub struct RootInfo<T>(pub Block<T>, pub QuorumCert, pub QuorumCert);

/// LedgerRecoveryData is a subset of RecoveryData that we can get solely from ledger info.
#[derive(Clone)]
pub struct LedgerRecoveryData {
    epoch: u64,
    validators: Arc<ValidatorVerifier>,
    validator_keys: ValidatorSet,
    storage_ledger: LedgerInfo,
}

impl LedgerRecoveryData {
    pub fn new(storage_ledger: LedgerInfo, validator_keys: ValidatorSet) -> Self {
        let epoch = if storage_ledger.next_validator_set().is_some() {
            storage_ledger.epoch() + 1
        } else {
            storage_ledger.epoch()
        };
        LedgerRecoveryData {
            epoch,
            validators: Arc::new((&validator_keys).into()),
            validator_keys,
            storage_ledger,
        }
    }

    pub fn epoch_info(&self) -> EpochInfo {
        EpochInfo {
            epoch: self.epoch,
            verifier: Arc::clone(&self.validators),
        }
    }

    pub fn commit_round(&self) -> Round {
        self.storage_ledger.round()
    }

    pub fn validator_keys(&self) -> Vec<ValidatorInfo> {
        self.validator_keys.clone().to_vec()
    }

    /// Finds the root (last committed block) and returns the root block, the QC to the root block
    /// and the ledger info for the root block, return an error if it can not be found.
    ///
    /// We guarantee that the block corresponding to the storage's latest ledger info always exists.
    fn find_root<T: Payload>(
        &self,
        blocks: &mut Vec<Block<T>>,
        quorum_certs: &mut Vec<QuorumCert>,
    ) -> Result<RootInfo<T>> {
        info!(
            "The last committed block id as recorded in storage: {}",
            self.storage_ledger
        );

        // We start from the block that storage's latest ledger info, if storage has end-epoch
        // LedgerInfo, we generate the virtual genesis block
        let root_id = if self.storage_ledger.next_validator_set().is_some() {
            let genesis = Block::make_genesis_block_from_ledger_info(&self.storage_ledger);
            let genesis_qc = QuorumCert::certificate_for_genesis_from_ledger_info(
                &self.storage_ledger,
                genesis.id(),
            );
            let genesis_id = genesis.id();
            blocks.push(genesis);
            quorum_certs.push(genesis_qc);
            genesis_id
        } else {
            self.storage_ledger.consensus_block_id()
        };

        // sort by (epoch, round) to guarantee the topological order of parent <- child
        blocks.sort_by_key(|b| (b.epoch(), b.round()));

        let root_idx = blocks
            .iter()
            .position(|block| block.id() == root_id)
            .ok_or_else(|| format_err!("unable to find root: {}", root_id))?;
        let root_block = blocks.remove(root_idx);
        let root_quorum_cert = quorum_certs
            .iter()
            .find(|qc| qc.certified_block().id() == root_block.id())
            .ok_or_else(|| format_err!("No QC found for root: {}", root_id))?
            .clone();
        let root_ledger_info = quorum_certs
            .iter()
            .find(|qc| qc.commit_info().id() == root_block.id())
            .ok_or_else(|| format_err!("No LI found for root: {}", root_id))?
            .clone();

        info!("Consensus root block is {}", root_block);

        Ok(RootInfo(root_block, root_quorum_cert, root_ledger_info))
    }
}

pub struct RootMetadata {
    pub accu_hash: HashValue,
    pub frozen_root_hashes: Vec<HashValue>,
    pub num_leaves: Version,
}

impl RootMetadata {
    pub fn new(num_leaves: u64, accu_hash: HashValue, frozen_root_hashes: Vec<HashValue>) -> Self {
        Self {
            num_leaves,
            accu_hash,
            frozen_root_hashes,
        }
    }

    pub fn version(&self) -> Version {
        max(self.num_leaves, 1) - 1
    }

    #[cfg(any(test, feature = "fuzzing"))]
    pub fn new_empty() -> Self {
        Self::new(0, *libra_crypto::hash::ACCUMULATOR_PLACEHOLDER_HASH, vec![])
    }
}

/// The recovery data constructed from raw consensusdb data, it'll find the root value and
/// blocks that need cleanup or return error if the input data is inconsistent.
pub struct RecoveryData<T> {
    ledger_recovery_data: LedgerRecoveryData,
    // The last vote message sent by this validator.
    last_vote: Option<Vote>,
    root: RootInfo<T>,
    root_metadata: RootMetadata,
    // 1. the blocks guarantee the topological ordering - parent <- child.
    // 2. all blocks are children of the root.
    blocks: Vec<Block<T>>,
    quorum_certs: Vec<QuorumCert>,
    blocks_to_prune: Option<Vec<HashValue>>,

    // Liveness data
    highest_timeout_certificate: Option<TimeoutCertificate>,
}

impl<T: Payload> RecoveryData<T> {
    pub fn new(
        last_vote: Option<Vote>,
        ledger_recovery_data: LedgerRecoveryData,
        mut blocks: Vec<Block<T>>,
        root_metadata: RootMetadata,
        mut quorum_certs: Vec<QuorumCert>,
        highest_timeout_certificate: Option<TimeoutCertificate>,
    ) -> Result<Self> {
        let root = ledger_recovery_data
            .find_root(&mut blocks, &mut quorum_certs)
            .with_context(|| {
                // for better readability
                quorum_certs.sort_by_key(|qc| qc.certified_block().round());
                format!(
                    "\nRoot id: {}\nBlocks in db: {}\nQuorum Certs in db: {}\n",
                    ledger_recovery_data.storage_ledger.consensus_block_id(),
                    blocks
                        .iter()
                        .map(|b| format!("\n\t{}", b))
                        .collect::<Vec<String>>()
                        .concat(),
                    quorum_certs
                        .iter()
                        .map(|qc| format!("\n\t{}", qc))
                        .collect::<Vec<String>>()
                        .concat(),
                )
            })?;

        let blocks_to_prune = Some(Self::find_blocks_to_prune(
            root.0.id(),
            &mut blocks,
            &mut quorum_certs,
        ));
        let epoch = root.0.epoch();
        Ok(RecoveryData {
            ledger_recovery_data,
            last_vote: match last_vote {
                Some(v) if v.epoch() == epoch => Some(v),
                _ => None,
            },
            root,
            root_metadata,
            blocks,
            quorum_certs,
            blocks_to_prune,
            highest_timeout_certificate: match highest_timeout_certificate {
                Some(tc) if tc.epoch() == epoch => Some(tc),
                _ => None,
            },
        })
    }

    pub fn epoch_info(&self) -> EpochInfo {
        self.ledger_recovery_data.epoch_info()
    }

    pub fn root_block(&self) -> &Block<T> {
        &self.root.0
    }

    pub fn last_vote(&self) -> Option<Vote> {
        self.last_vote.clone()
    }

    pub fn take(self) -> (RootInfo<T>, RootMetadata, Vec<Block<T>>, Vec<QuorumCert>) {
        (
            self.root,
            self.root_metadata,
            self.blocks,
            self.quorum_certs,
        )
    }

    pub fn take_blocks_to_prune(&mut self) -> Vec<HashValue> {
        self.blocks_to_prune
            .take()
            .expect("blocks_to_prune already taken")
    }

    pub fn highest_timeout_certificate(&self) -> Option<TimeoutCertificate> {
        self.highest_timeout_certificate.clone()
    }

    pub fn validator_keys(&self) -> Vec<ValidatorInfo> {
        self.ledger_recovery_data.validator_keys()
    }

    fn find_blocks_to_prune(
        root_id: HashValue,
        blocks: &mut Vec<Block<T>>,
        quorum_certs: &mut Vec<QuorumCert>,
    ) -> Vec<HashValue> {
        // prune all the blocks that don't have root as ancestor
        let mut tree = HashSet::new();
        let mut to_remove = vec![];
        tree.insert(root_id);
        // assume blocks are sorted by round already
        blocks.retain(|block| {
            if tree.contains(&block.parent_id()) {
                tree.insert(block.id());
                true
            } else {
                to_remove.push(block.id());
                false
            }
        });
        quorum_certs.retain(|qc| tree.contains(&qc.certified_block().id()));
        to_remove
    }
}

/// The proxy we use to persist data in libra db storage service via grpc.
pub struct StorageWriteProxy {
    db: Arc<ConsensusDB>,
    libra_db: Arc<dyn LibraDBTrait>,
}

impl StorageWriteProxy {
    pub fn new(config: &NodeConfig, libra_db: Arc<dyn LibraDBTrait>) -> Self {
        let db = Arc::new(ConsensusDB::new(config.storage.dir()));
        StorageWriteProxy { db, libra_db }
    }
}

impl<T: Payload> PersistentLivenessStorage<T> for StorageWriteProxy {
    fn save_tree(&self, blocks: Vec<Block<T>>, quorum_certs: Vec<QuorumCert>) -> Result<()> {
        let mut trace_batch = vec![];
        for block in blocks.iter() {
            trace_code_block!("consensusdb::save_tree", {"block", block.id()}, trace_batch);
        }
        self.db
            .save_blocks_and_quorum_certificates(blocks, quorum_certs)
    }

    fn prune_tree(&self, block_ids: Vec<HashValue>) -> Result<()> {
        if !block_ids.is_empty() {
            // quorum certs that certified the block_ids will get removed
            self.db
                .delete_blocks_and_quorum_certificates::<T>(block_ids)?;
        }
        Ok(())
    }

    fn save_state(&self, vote: &Vote) -> Result<()> {
        self.db.save_state(lcs::to_bytes(vote)?)
    }

    fn recover_from_ledger(&self) -> LedgerRecoveryData {
        let startup_info = self
            .libra_db
            .get_startup_info()
            .expect("unable to read ledger info from storage")
            .expect("startup info is None");

        LedgerRecoveryData::new(
            startup_info.latest_ledger_info.ledger_info().clone(),
            startup_info.get_validator_set().clone(),
        )
    }

    fn start(&self) -> LivenessStorageData<T> {
        info!("Start consensus recovery.");
        let raw_data = self
            .db
            .get_data()
            .expect("unable to recover consensus data");

        let last_vote = raw_data.0.map(|vote_data| {
            lcs::from_bytes(&vote_data[..]).expect("unable to deserialize last vote msg")
        });

        let highest_timeout_certificate = raw_data.1.map(|ts| {
            lcs::from_bytes(&ts[..]).expect("unable to deserialize highest timeout certificate")
        });
        let blocks = raw_data.2;
        let quorum_certs: Vec<_> = raw_data.3;
        let blocks_repr: Vec<String> = blocks.iter().map(|b| format!("\n\t{}", b)).collect();
        info!(
            "The following blocks were restored from ConsensusDB : {}",
            blocks_repr.concat()
        );
        let qc_repr: Vec<String> = quorum_certs
            .iter()
            .map(|qc| format!("\n\t{}", qc))
            .collect();
        info!(
            "The following quorum certs were restored from ConsensusDB: {}",
            qc_repr.concat()
        );

        // find the block corresponding to storage latest ledger info
        let startup_info = self
            .libra_db
            .get_startup_info()
            .expect("unable to read ledger info from storage")
            .expect("startup info is None");
        let validator_set = startup_info.get_validator_set().clone();
        let ledger_recovery_data = LedgerRecoveryData::new(
            startup_info.latest_ledger_info.ledger_info().clone(),
            validator_set,
        );
        let frozen_root_hashes = startup_info
            .committed_tree_state
            .ledger_frozen_subtree_hashes
            .clone();
        let root_executed_trees = ExecutedTrees::from(startup_info.committed_tree_state);
        match RecoveryData::new(
            last_vote,
            ledger_recovery_data.clone(),
            blocks,
            RootMetadata::new(
                root_executed_trees.txn_accumulator().num_leaves(),
                root_executed_trees.state_id(),
                frozen_root_hashes,
            ),
            quorum_certs,
            highest_timeout_certificate,
        ) {
            Ok(mut initial_data) => {
                (self as &dyn PersistentLivenessStorage<T>)
                    .prune_tree(initial_data.take_blocks_to_prune())
                    .expect("unable to prune dangling blocks during restart");
                if initial_data.last_vote.is_none() {
                    self.db
                        .delete_last_vote_msg()
                        .expect("unable to cleanup last vote");
                }
                if initial_data.highest_timeout_certificate.is_none() {
                    self.db
                        .delete_highest_timeout_certificate()
                        .expect("unable to cleanup highest timeout cert");
                }
                info!(
                    "Starting up the consensus state machine with recovery data - [last_vote {}], [highest timeout certificate: {}]",
                    initial_data.last_vote.as_ref().map_or("None".to_string(), |v| v.to_string()),
                    initial_data.highest_timeout_certificate.as_ref().map_or("None".to_string(), |v| v.to_string()),
                );

                LivenessStorageData::RecoveryData(initial_data)
            }
            Err(e) => {
                error!("Failed to construct recovery data: {}", e);
                LivenessStorageData::LedgerRecoveryData(ledger_recovery_data)
            }
        }
    }

    fn save_highest_timeout_cert(&self, highest_timeout_cert: TimeoutCertificate) -> Result<()> {
        self.db
            .save_highest_timeout_certificate(lcs::to_bytes(&highest_timeout_cert)?)
    }
}

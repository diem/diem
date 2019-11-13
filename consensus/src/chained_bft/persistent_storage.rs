// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::consensusdb::ConsensusDB, consensus_provider::create_storage_read_client,
};
use consensus_types::{
    block::Block, common::Payload, quorum_cert::QuorumCert,
    timeout_certificate::TimeoutCertificate, vote::Vote,
};
use failure::{Result, ResultExt};
use libra_config::config::NodeConfig;
use libra_crypto::HashValue;
use libra_logger::prelude::*;
use libra_types::ledger_info::LedgerInfo;
use rmp_serde::{from_slice, to_vec_named};
use std::{collections::HashSet, sync::Arc};

/// Persistent storage for liveness data
pub trait PersistentLivenessStorage: Send + Sync {
    /// Persist the highest timeout certificate for improved liveness - proof for other replicas
    /// to jump to this round
    fn save_highest_timeout_cert(&self, highest_timeout_cert: TimeoutCertificate) -> Result<()>;
}

/// Persistent storage is essential for maintaining safety when a node crashes.  Specifically,
/// upon a restart, a correct node will not equivocate.  Even if all nodes crash, safety is
/// guaranteed.  This trait also also supports liveness aspects (i.e. highest timeout certificate)
/// and supports clean up (i.e. tree pruning).
/// Blocks persisted are proposed but not yet committed.  The committed state is persisted
/// via StateComputer.
pub trait PersistentStorage<T>: PersistentLivenessStorage + Send + Sync {
    /// Get an Arc to an instance of PersistentLivenessStorage
    /// (workaround for trait downcasting
    fn persistent_liveness_storage(&self) -> Box<dyn PersistentLivenessStorage>;

    /// Persist the blocks and quorum certs into storage atomically.
    fn save_tree(&self, blocks: Vec<Block<T>>, quorum_certs: Vec<QuorumCert>) -> Result<()>;

    /// Delete the corresponding blocks and quorum certs atomically.
    fn prune_tree(&self, block_ids: Vec<HashValue>) -> Result<()>;

    /// Persist consensus' state
    fn save_state(&self, vote: &Vote) -> Result<()>;

    /// When the node restart, construct the instance and returned the data read from db.
    /// This could guarantee we only read once during start, and we would panic if the
    /// read fails.
    /// It makes sense to be synchronous since we can't do anything else until this finishes.
    fn start(config: &NodeConfig) -> (Arc<Self>, RecoveryData<T>)
    where
        Self: Sized;
}

/// The recovery data constructed from raw consensusdb data, it'll find the root value and
/// blocks that need cleanup or return error if the input data is inconsistent.
#[derive(Debug)]
pub struct RecoveryData<T> {
    epoch: u64,
    // The last vote message sent by this validator.
    last_vote: Option<Vote>,
    root: (Block<T>, QuorumCert, QuorumCert),
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
        mut blocks: Vec<Block<T>>,
        mut quorum_certs: Vec<QuorumCert>,
        storage_ledger: &LedgerInfo,
        highest_timeout_certificate: Option<TimeoutCertificate>,
    ) -> Result<Self> {
        let root =
            Self::find_root(&mut blocks, &mut quorum_certs, storage_ledger).with_context(|e| {
                // for better readability
                quorum_certs.sort_by_key(|qc| qc.certified_block().round());
                format!(
                    "Blocks in db: {}\nQuorum Certs in db: {}\nerror: {}",
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
                    e,
                )
            })?;

        let blocks_to_prune = Some(Self::find_blocks_to_prune(
            root.0.id(),
            &mut blocks,
            &mut quorum_certs,
        ));
        Ok(RecoveryData {
            epoch: root.0.epoch(),
            last_vote,
            root,
            blocks,
            quorum_certs,
            blocks_to_prune,
            highest_timeout_certificate,
        })
    }

    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    pub fn root_block(&self) -> &Block<T> {
        &self.root.0
    }

    pub fn last_vote(&self) -> Option<Vote> {
        self.last_vote.clone()
    }

    pub fn take(
        self,
    ) -> (
        (Block<T>, QuorumCert, QuorumCert),
        Vec<Block<T>>,
        Vec<QuorumCert>,
    ) {
        (self.root, self.blocks, self.quorum_certs)
    }

    pub fn take_blocks_to_prune(&mut self) -> Vec<HashValue> {
        self.blocks_to_prune
            .take()
            .expect("blocks_to_prune already taken")
    }

    pub fn highest_timeout_certificate(&self) -> Option<TimeoutCertificate> {
        self.highest_timeout_certificate.clone()
    }

    /// Finds the root (last committed block) and returns the root block, the QC to the root block
    /// and the ledger info for the root block, return an error if it can not be found.
    ///
    /// We guarantee that the block corresponding to the storage's latest ledger info always exists.
    /// In the case of an epoch boundary ledger info(i.e. it has validator set), we generate the virtual genesis block.
    fn find_root(
        blocks: &mut Vec<Block<T>>,
        quorum_certs: &mut Vec<QuorumCert>,
        storage_ledger: &LedgerInfo,
    ) -> Result<(Block<T>, QuorumCert, QuorumCert)> {
        let root_from_storage = storage_ledger.consensus_block_id();
        info!(
            "The last committed block id as recorded in storage: {}",
            root_from_storage
        );

        // We start from the block that storage's latest ledger info, if storage has end-epoch
        // LedgerInfo, we generate the virtual genesis block
        let root_id = if storage_ledger.next_validator_set().is_some() {
            let genesis = Block::make_genesis_block_from_ledger_info(storage_ledger);
            let genesis_qc =
                QuorumCert::certificate_for_genesis_from_ledger_info(storage_ledger, genesis.id());
            let genesis_id = genesis.id();
            blocks.push(genesis);
            quorum_certs.push(genesis_qc);
            genesis_id
        } else {
            storage_ledger.consensus_block_id()
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

        Ok((root_block, root_quorum_cert, root_ledger_info))
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
}

impl StorageWriteProxy {
    pub fn new(db: Arc<ConsensusDB>) -> Self {
        StorageWriteProxy { db }
    }
}

impl PersistentLivenessStorage for StorageWriteProxy {
    fn save_highest_timeout_cert(&self, highest_timeout_cert: TimeoutCertificate) -> Result<()> {
        self.db
            .save_highest_timeout_certificate(to_vec_named(&highest_timeout_cert)?)
    }
}

impl<T: Payload> PersistentStorage<T> for StorageWriteProxy {
    fn persistent_liveness_storage(&self) -> Box<dyn PersistentLivenessStorage> {
        Box::new(StorageWriteProxy::new(Arc::clone(&self.db)))
    }

    fn save_tree(&self, blocks: Vec<Block<T>>, quorum_certs: Vec<QuorumCert>) -> Result<()> {
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
        self.db.save_state(to_vec_named(vote)?)
    }

    fn start(config: &NodeConfig) -> (Arc<Self>, RecoveryData<T>) {
        info!("Start consensus recovery.");
        let read_client = create_storage_read_client(config);
        let db = Arc::new(ConsensusDB::new(config.get_storage_dir()));
        let proxy = Arc::new(Self::new(Arc::clone(&db)));
        let initial_data = db.get_data().expect("unable to recover consensus data");

        let last_vote = initial_data.0.map(|vote_data| {
            from_slice(&vote_data[..]).expect("unable to deserialize last vote msg")
        });
        let last_vote_repr = match &last_vote {
            Some(vote) => format!("{}", vote),
            None => "None".to_string(),
        };
        debug!("Recovered last vote msg: {}", last_vote_repr);

        let highest_timeout_certificate = initial_data.1.map(|ts| {
            from_slice(&ts[..]).expect("unable to deserialize highest timeout certificate")
        });
        let blocks = initial_data.2;
        let quorum_certs: Vec<_> = initial_data.3;
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
        let (_, ledger_info, _, _) = read_client
            .update_to_latest_ledger(0, vec![])
            .expect("unable to read ledger info from storage");
        let mut initial_data = RecoveryData::new(
            last_vote,
            blocks,
            quorum_certs,
            ledger_info.ledger_info(),
            highest_timeout_certificate,
        )
        .unwrap_or_else(|e| panic!("Can not construct recovery data due to {}", e));

        <dyn PersistentStorage<T>>::prune_tree(proxy.as_ref(), initial_data.take_blocks_to_prune())
            .expect("unable to prune dangling blocks during restart");

        (proxy, initial_data)
    }
}

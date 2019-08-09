// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        common::Payload,
        consensus_types::{block::Block, quorum_cert::QuorumCert},
        consensusdb::ConsensusDB,
        liveness::pacemaker_timeout_manager::HighestTimeoutCertificates,
        safety::safety_rules::ConsensusState,
    },
    consensus_provider::create_storage_read_client,
};
use config::config::NodeConfig;
use crypto::HashValue;
use failure::Result;
use logger::prelude::*;
use nextgen_crypto::*;
use rmp_serde::{from_slice, to_vec_named};
use serde::{de::Deserialize, ser::Serialize};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

/// Persistent storage for liveness data
pub trait PersistentLivenessStorage<Sig>: Send + Sync {
    /// Persist the highest timeout certificate for improved liveness - proof for other replicas
    /// to jump to this round
    fn save_highest_timeout_cert(
        &self,
        highest_timeout_certs: HighestTimeoutCertificates<Sig>,
    ) -> Result<()>;
}

/// Persistent storage is essential for maintaining safety when a node crashes.  Specifically,
/// upon a restart, a correct node will not equivocate.  Even if all nodes crash, safety is
/// guaranteed.  This trait also also supports liveness aspects (i.e. highest timeout certificate)
/// and supports clean up (i.e. tree pruning).
/// Blocks persisted are proposed but not yet committed.  The committed state is persisted
/// via StateComputer.
pub trait PersistentStorage<T, Sig: Signature>:
    PersistentLivenessStorage<Sig> + Send + Sync
where
    Sig::SigningKeyMaterial: Genesis,
{
    /// Get an Arc to an instance of PersistentLivenessStorage
    /// (workaround for trait downcasting
    fn persistent_liveness_storage(&self) -> Box<dyn PersistentLivenessStorage<Sig>>;

    /// Persist the blocks and quorum certs into storage atomically.
    fn save_tree(
        &self,
        blocks: Vec<Block<T, Sig>>,
        quorum_certs: Vec<QuorumCert<Sig>>,
    ) -> Result<()>;

    /// Delete the corresponding blocks and quorum certs atomically.
    fn prune_tree(&self, block_ids: Vec<HashValue>) -> Result<()>;

    /// Persist the consensus state.
    fn save_consensus_state(&self, state: ConsensusState) -> Result<()>;

    /// When the node restart, construct the instance and returned the data read from db.
    /// This could guarantee we only read once during start, and we would panic if the
    /// read fails.
    /// It makes sense to be synchronous since we can't do anything else until this finishes.
    fn start(config: &NodeConfig) -> (Arc<Self>, RecoveryData<T, Sig>)
    where
        Self: Sized;
}

/// The recovery data constructed from raw consensusdb data, it'll find the root value and
/// blocks that need cleanup or return error if the input data is inconsistent.
#[derive(Debug)]
pub struct RecoveryData<T, Sig> {
    // Safety data
    state: ConsensusState,
    root: (Block<T, Sig>, QuorumCert<Sig>, QuorumCert<Sig>),
    // 1. the blocks guarantee the topological ordering - parent <- child.
    // 2. all blocks are children of the root.
    blocks: Vec<Block<T, Sig>>,
    quorum_certs: Vec<QuorumCert<Sig>>,
    blocks_to_prune: Option<Vec<HashValue>>,

    // Liveness data
    highest_timeout_certificates: HighestTimeoutCertificates<Sig>,

    // whether root is consistent with StateComputer, if not we need to do the state sync before
    // starting
    need_sync: bool,
}

impl<T, Sig> RecoveryData<T, Sig>
where
    T: Payload,
    Sig: Signature,
    Sig::SigningKeyMaterial: Genesis,
{
    pub fn new(
        state: ConsensusState,
        mut blocks: Vec<Block<T, Sig>>,
        mut quorum_certs: Vec<QuorumCert<Sig>>,
        root_from_storage: HashValue,
        highest_timeout_certificates: HighestTimeoutCertificates<Sig>,
    ) -> Result<Self> {
        let root =
            Self::find_root(&mut blocks, &mut quorum_certs, root_from_storage).map_err(|e| {
                format_err!(
                    "Blocks in db: {}\nQuorum Certs in db: {}, error: {}",
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
        // if the root is different than the LI(S).block, we need to sync before start
        let need_sync = root_from_storage != root.0.id();
        Ok(RecoveryData {
            state,
            root,
            blocks,
            quorum_certs,
            blocks_to_prune,
            highest_timeout_certificates,
            need_sync,
        })
    }

    pub fn state(&self) -> ConsensusState {
        self.state.clone()
    }

    pub fn take(
        self,
    ) -> (
        (Block<T, Sig>, QuorumCert<Sig>, QuorumCert<Sig>),
        Vec<Block<T, Sig>>,
        Vec<QuorumCert<Sig>>,
    ) {
        (self.root, self.blocks, self.quorum_certs)
    }

    pub fn take_blocks_to_prune(&mut self) -> Vec<HashValue> {
        self.blocks_to_prune
            .take()
            .expect("blocks_to_prune already taken")
    }

    pub fn highest_timeout_certificates(&self) -> &HighestTimeoutCertificates<Sig> {
        &self.highest_timeout_certificates
    }

    pub fn root_ledger_info(&self) -> QuorumCert<Sig> {
        self.root.2.clone()
    }

    pub fn need_sync(&self) -> bool {
        self.need_sync
    }

    /// Finds the root (last committed block) and returns the root block, the QC to the root block
    /// and the ledger info for the root block, return an error if it can not be found.
    ///
    /// LI(S) is the highest known ledger info determined by storage.
    /// LI(C) is determined by ConsensusDB: it's the highest block id that is certified as committed
    /// by one of the QC's ledger infos.
    ///
    /// We guarantee a few invariants:
    /// 1. LI(C) must exist in blocks
    /// 2. LI(S).block.round <= LI(C).block.round
    ///
    /// We use the following condition to decide the root:
    /// 1. LI(S) exist && LI(S) is ancestor of LI(C) according to blocks, root = LI(S)
    /// 2. else root = LI(C)
    ///
    /// In a typical case, the QC certifying a commit of a block is persisted to ConsensusDB before
    /// this block is committed to the storage. Hence, ConsensusDB contains the
    /// block corresponding to LI(S) id, which is going to become the root.
    /// An additional complication is added in this code in order to tolerate a potential failure
    /// during state synchronization. In this case LI(S) might not be found in the blocks of
    /// ConsensusDB: we're going to start with LI(C) and invoke state synchronizer in order to
    /// resume the synchronization.
    fn find_root(
        blocks: &mut Vec<Block<T, Sig>>,
        quorum_certs: &mut Vec<QuorumCert<Sig>>,
        root_from_storage: HashValue,
    ) -> Result<(Block<T, Sig>, QuorumCert<Sig>, QuorumCert<Sig>)> {
        // sort by round to guarantee the topological order of parent <- child
        blocks.sort_by_key(Block::round);
        let root_from_consensus = {
            let id_to_round: HashMap<_, _> = blocks
                .iter()
                .map(|block| (block.id(), block.round()))
                .collect();
            let mut round_and_id = None;
            for qc in quorum_certs.iter() {
                if let Some(committed_block_id) = qc.committed_block_id() {
                    if let Some(round) = id_to_round.get(&committed_block_id) {
                        match round_and_id {
                            Some((r, _)) if r > round => (),
                            _ => round_and_id = Some((round, committed_block_id)),
                        }
                    }
                }
            }
            match round_and_id {
                Some((_, id)) => id,
                None => return Err(format_err!("No LI found in quorum certs.")),
            }
        };
        let root_id = {
            let mut tree = HashSet::new();
            tree.insert(root_from_storage);
            blocks.iter().for_each(|block| {
                if tree.contains(&block.parent_id()) {
                    tree.insert(block.id());
                }
            });
            if !tree.contains(&root_from_consensus) {
                root_from_consensus
            } else {
                root_from_storage
            }
        };

        let root_idx = blocks
            .iter()
            .position(|block| block.id() == root_id)
            .ok_or_else(|| format_err!("unable to find root: {}", root_id))?;
        let root_block = blocks.remove(root_idx);
        let root_quorum_cert = quorum_certs
            .iter()
            .find(|qc| qc.certified_block_id() == root_block.id())
            .ok_or_else(|| format_err!("No QC found for root: {}", root_id))?
            .clone();
        let root_ledger_info = quorum_certs
            .iter()
            .find(|qc| qc.committed_block_id() == Some(root_block.id()))
            .ok_or_else(|| format_err!("No LI found for root: {}", root_id))?
            .clone();
        Ok((root_block, root_quorum_cert, root_ledger_info))
    }

    fn find_blocks_to_prune(
        root_id: HashValue,
        blocks: &mut Vec<Block<T, Sig>>,
        quorum_certs: &mut Vec<QuorumCert<Sig>>,
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
        quorum_certs.retain(|qc| tree.contains(&qc.certified_block_id()));
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

impl<Sig: Signature + Serialize> PersistentLivenessStorage<Sig> for StorageWriteProxy {
    fn save_highest_timeout_cert(
        &self,
        highest_timeout_certs: HighestTimeoutCertificates<Sig>,
    ) -> Result<()> {
        self.db
            .save_highest_timeout_certificates::<Sig>(to_vec_named(&highest_timeout_certs)?)
    }
}

impl<T, Sig> PersistentStorage<T, Sig> for StorageWriteProxy
where
    T: Payload,
    Sig: Signature + Serialize + for<'de> Deserialize<'de>,
    Sig::SigningKeyMaterial: Genesis,
{
    fn persistent_liveness_storage(&self) -> Box<dyn PersistentLivenessStorage<Sig>> {
        Box::new(StorageWriteProxy::new(Arc::clone(&self.db)))
    }

    fn save_tree(
        &self,
        blocks: Vec<Block<T, Sig>>,
        quorum_certs: Vec<QuorumCert<Sig>>,
    ) -> Result<()> {
        self.db
            .save_blocks_and_quorum_certificates(blocks, quorum_certs)
    }

    fn prune_tree(&self, block_ids: Vec<HashValue>) -> Result<()> {
        if !block_ids.is_empty() {
            // quorum certs that certified the block_ids will get removed
            self.db
                .delete_blocks_and_quorum_certificates::<T, Sig>(block_ids)?;
        }
        Ok(())
    }

    fn save_consensus_state(&self, state: ConsensusState) -> Result<()> {
        self.db.save_state(to_vec_named(&state)?)
    }

    fn start(config: &NodeConfig) -> (Arc<Self>, RecoveryData<T, Sig>) {
        info!("Start consensus recovery.");
        let read_client = create_storage_read_client(config);
        let db = Arc::new(ConsensusDB::new(config.storage.dir.clone()));
        let proxy = Arc::new(Self::new(Arc::clone(&db)));
        let initial_data = db.get_data().expect("unable to recover consensus data");
        let consensus_state = initial_data.0.map_or_else(ConsensusState::default, |s| {
            from_slice(&s[..]).expect("unable to deserialize consensus state")
        });
        debug!("Recovered consensus state: {}", consensus_state);
        let highest_timeout_certificates = initial_data
            .1
            .map_or_else(HighestTimeoutCertificates::default, |s| {
                from_slice(&s[..]).expect("unable to deserialize highest timeout certificates")
            });
        let mut blocks = initial_data.2;
        let mut quorum_certs: Vec<_> = initial_data.3;
        // bootstrap the empty store with genesis block and qc.
        if blocks.is_empty() && quorum_certs.is_empty() {
            blocks.push(Block::make_genesis_block());
            quorum_certs.push(QuorumCert::certificate_for_genesis());
            proxy
                .save_tree(vec![blocks[0].clone()], vec![quorum_certs[0].clone()])
                .expect("unable to bootstrap the storage with genesis block");
        }
        let blocks_repr: Vec<String> = blocks.iter().map(|b| format!("\n\t{}", b)).collect();
        debug!(
            "The following blocks were restored from ConsensusDB : {}",
            blocks_repr.concat()
        );
        let qc_repr: Vec<String> = quorum_certs
            .iter()
            .map(|qc| format!("\n\t{}", qc))
            .collect();
        debug!(
            "The following blocks were restored from ConsensusDB: {}",
            qc_repr.concat()
        );

        // find the block corresponding to storage latest ledger info
        let (_, ledger_info, _) = read_client
            .update_to_latest_ledger(0, vec![])
            .expect("unable to read ledger info from storage");
        let root_from_storage = ledger_info.ledger_info().consensus_block_id();
        debug!(
            "The last committed block id as recorded in storage: {}",
            root_from_storage
        );

        let mut initial_data = RecoveryData::new(
            consensus_state,
            blocks,
            quorum_certs,
            root_from_storage,
            highest_timeout_certificates,
        )
        .unwrap_or_else(|e| panic!("Can not construct recovery data due to {}", e));

        <dyn PersistentStorage<T, Sig>>::prune_tree(
            proxy.as_ref(),
            initial_data.take_blocks_to_prune(),
        )
        .expect("unable to prune dangling blocks during restart");

        debug!("Consensus root to start with: {}", initial_data.root.0);

        if initial_data.need_sync {
            info!("Consensus recovery done but additional state synchronization is required.");
        } else {
            info!("Consensus recovery completed.")
        }
        (proxy, initial_data)
    }
}

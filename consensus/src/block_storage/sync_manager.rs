// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    block_storage::{BlockReader, BlockStore},
    logging::{LogEvent, LogSchema},
    network::NetworkSender,
    network_interface::ConsensusMsg,
    persistent_liveness_storage::{PersistentLivenessStorage, RecoveryData},
    state_replication::StateComputer,
};
use anyhow::{bail, format_err};

use consensus_types::{
    block::Block,
    block_retrieval::{BlockRetrievalRequest, BlockRetrievalStatus, MAX_BLOCKS_PER_REQUEST},
    common::Author,
    quorum_cert::QuorumCert,
    sync_info::SyncInfo,
};
use diem_crypto::HashValue;
use diem_logger::prelude::*;
use diem_types::{
    account_address::AccountAddress, epoch_change::EpochChangeProof,
    ledger_info::LedgerInfoWithSignatures,
};

use mirai_annotations::checked_precondition;
use rand::{prelude::*, Rng};
use std::{clone::Clone, cmp::min, sync::Arc, time::Duration};

#[derive(Debug, PartialEq, Eq)]
/// Whether we need to do block retrieval if we want to insert a Quorum Cert.
pub enum NeedFetchResult {
    QCRoundBeforeRoot,
    QCAlreadyExist,
    QCBlockExist,
    NeedFetch,
}

impl BlockStore {
    /// Check if we're far away from this ledger info and need to sync.
    /// Returns false if we have this block in the tree or the root's round is higher than the
    /// block.
    pub fn need_sync_for_quorum_cert(&self, qc: &QuorumCert) -> bool {
        // This precondition ensures that the check in the following lines
        // does not result in an addition overflow.
        checked_precondition!(self.ordered_root().round() < std::u64::MAX - 1);

        // If we have the block locally, we're not far from this QC thus don't need to sync.
        // In case root().round() is greater than that the committed
        // block carried by LI is older than my current commit.
        !(self.block_exists(qc.commit_info().id())
            || self.ordered_root().round() >= qc.commit_info().round())
    }

    /// Checks if quorum certificate can be inserted in block store without RPC
    /// Returns the enum to indicate the detailed status.
    pub fn need_fetch_for_quorum_cert(&self, qc: &QuorumCert) -> NeedFetchResult {
        if qc.certified_block().round() < self.ordered_root().round() {
            return NeedFetchResult::QCRoundBeforeRoot;
        }
        if self
            .get_quorum_cert_for_block(qc.certified_block().id())
            .is_some()
        {
            return NeedFetchResult::QCAlreadyExist;
        }
        if self.block_exists(qc.certified_block().id()) {
            return NeedFetchResult::QCBlockExist;
        }
        NeedFetchResult::NeedFetch
    }

    /// Fetches dependencies for given sync_info.quorum_cert
    /// If gap is large, performs state sync using sync_to_highest_ordered_cert
    /// Inserts sync_info.quorum_cert into block store as the last step
    pub async fn add_certs(
        &self,
        sync_info: &SyncInfo,
        mut retriever: BlockRetriever,
    ) -> anyhow::Result<()> {
        self.sync_to_highest_ordered_cert(
            sync_info.highest_ordered_cert().clone(),
            sync_info.highest_ledger_info().clone(),
            &mut retriever,
        )
        .await?;

        self.insert_quorum_cert(sync_info.highest_ordered_cert(), &mut retriever)
            .await?;

        self.insert_quorum_cert(sync_info.highest_quorum_cert(), &mut retriever)
            .await?;

        if let Some(tc) = sync_info.highest_timeout_certificate() {
            self.insert_timeout_certificate(Arc::new(tc.clone()))?;
        }
        Ok(())
    }

    pub async fn insert_quorum_cert(
        &self,
        qc: &QuorumCert,
        retriever: &mut BlockRetriever,
    ) -> anyhow::Result<()> {
        match self.need_fetch_for_quorum_cert(qc) {
            NeedFetchResult::NeedFetch => self.fetch_quorum_cert(qc.clone(), retriever).await?,
            NeedFetchResult::QCBlockExist => self.insert_single_quorum_cert(qc.clone())?,
            _ => (),
        }
        if self.ordered_root().round() < qc.commit_info().round() {
            let finality_proof = qc.ledger_info();
            self.commit(finality_proof.clone()).await?;
            if qc.ends_epoch() {
                retriever
                    .network
                    .broadcast(ConsensusMsg::EpochChangeProof(Box::new(
                        EpochChangeProof::new(
                            vec![finality_proof.clone()],
                            /* more = */ false,
                        ),
                    )))
                    .await;
            }
        }
        Ok(())
    }

    /// Insert the quorum certificate separately from the block, used to split the processing of
    /// updating the consensus state(with qc) and deciding whether to vote(with block)
    /// The missing ancestors are going to be retrieved from the given peer. If a given peer
    /// fails to provide the missing ancestors, the qc is not going to be added.
    async fn fetch_quorum_cert(
        &self,
        qc: QuorumCert,
        retriever: &mut BlockRetriever,
    ) -> anyhow::Result<()> {
        let mut pending = vec![];
        let mut retrieve_qc = qc.clone();
        loop {
            if self.block_exists(retrieve_qc.certified_block().id()) {
                break;
            }
            let mut blocks = retriever.retrieve_block_for_qc(&retrieve_qc, 1).await?;
            // retrieve_block_for_qc guarantees that blocks has exactly 1 element
            let block = blocks.remove(0);
            retrieve_qc = block.quorum_cert().clone();
            pending.push(block);
        }
        // insert the qc <- block pair
        while let Some(block) = pending.pop() {
            let block_qc = block.quorum_cert().clone();
            self.insert_single_quorum_cert(block_qc)?;
            self.execute_and_insert_block(block)?;
        }
        self.insert_single_quorum_cert(qc)
    }

    /// Check the highest commit cert sent by peer to see if we're behind and start a fast
    /// forward sync if the committed block doesn't exist in our tree.
    /// It works as follows:
    /// 1. request the committed 3-chain from the peer, if C2 is the highest_commit_cert
    /// we request for B0 <- C0 <- B1 <- C1 <- B2 (<- C2)
    /// 2. We persist the 3-chain to storage before start sync to ensure we could restart if we
    /// crash in the middle of the sync.
    /// 3. We prune the old tree and replace with a new tree built with the 3-chain.
    async fn sync_to_highest_ordered_cert(
        &self,
        highest_ordered_cert: QuorumCert,
        highest_ledger_info: LedgerInfoWithSignatures,
        retriever: &mut BlockRetriever,
    ) -> anyhow::Result<()> {
        if !self.need_sync_for_quorum_cert(&highest_ordered_cert) {
            return Ok(());
        }
        let (root, root_metadata, blocks, quorum_certs) = Self::fast_forward_sync(
            &highest_ordered_cert,
            highest_ledger_info,
            retriever,
            self.storage.clone(),
            self.state_computer.clone(),
        )
        .await?
        .take();
        debug!(
            LogSchema::new(LogEvent::CommitViaSync).round(self.ordered_root().round()),
            committed_round = root.0.round(),
            block_id = root.0.id(),
        );
        self.rebuild(root, root_metadata, blocks, quorum_certs)
            .await;

        if highest_ordered_cert.ends_epoch() {
            retriever
                .network
                .notify_epoch_change(EpochChangeProof::new(
                    vec![highest_ordered_cert.ledger_info().clone()],
                    /* more = */ false,
                ))
                .await;
        }
        Ok(())
    }
    #[allow(clippy::needless_borrow)]
    pub async fn fast_forward_sync<'a>(
        highest_ordered_cert: &'a QuorumCert,
        highest_ledger_info: LedgerInfoWithSignatures,
        retriever: &'a mut BlockRetriever,
        storage: Arc<dyn PersistentLivenessStorage>,
        state_computer: Arc<dyn StateComputer>,
    ) -> anyhow::Result<RecoveryData> {
        debug!(
            LogSchema::new(LogEvent::StateSync).remote_peer(retriever.preferred_peer),
            "Start state sync with peer to block: {}",
            highest_ordered_cert.commit_info(),
        );

        // we fetch the blocks from
        let num_blocks = highest_ordered_cert.certified_block().round()
            - highest_ledger_info.ledger_info().round()
            + 1;

        let blocks = retriever
            .retrieve_block_for_qc(highest_ordered_cert, num_blocks)
            .await?;

        assert_eq!(
            blocks.first().expect("should have at least 3-chain").id(),
            highest_ordered_cert.certified_block().id(),
        );

        assert_eq!(
            blocks.last().expect("should have at least 3-chain").id(),
            highest_ledger_info.commit_info().id(),
        );

        // although unlikely, we might wrap num_blocks around on a 32-bit machine
        assert!(num_blocks < std::usize::MAX as u64);
        let mut quorum_certs = vec![highest_ordered_cert.clone()];
        quorum_certs.extend(
            blocks
                .iter()
                .take(num_blocks as usize - 1)
                .map(|block| block.quorum_cert().clone()),
        );
        for (i, block) in blocks.iter().enumerate() {
            assert_eq!(block.id(), quorum_certs[i].certified_block().id());
        }

        // If a node restarts in the middle of state synchronization, it is going to try to catch up
        // to the stored quorum certs as the new root.
        storage.save_tree(blocks.clone(), quorum_certs.clone())?;
        state_computer.sync_to(highest_ledger_info).await?;

        // we do not need to update block_tree.highest_commit_decision_ledger_info here
        // because the block_tree is going to rebuild itself.

        let recovery_data = storage
            .start()
            .expect_recovery_data("Failed to construct recovery data after fast forward sync");

        Ok(recovery_data)
    }
}

/// BlockRetriever is used internally to retrieve blocks
pub struct BlockRetriever {
    network: NetworkSender,
    preferred_peer: Author,
}

impl BlockRetriever {
    pub fn new(network: NetworkSender, preferred_peer: Author) -> Self {
        Self {
            network,
            preferred_peer,
        }
    }

    /// Retrieve n blocks for given block_id from peers
    ///
    /// Returns Result with Vec that has a guaranteed size of num_blocks
    /// This guarantee is based on BlockRetrievalResponse::verify that ensures that number of
    /// blocks in response is equal to number of blocks requested.  This method will
    /// continue until the quorum certificate members all fail to return the missing chain.
    ///
    /// The first attempt of block retrieval will always be sent to preferred_peer to allow the
    /// leader to drive quorum certificate creation The other peers from the quorum certificate
    /// will be randomly tried next.  If all members of the quorum certificate are exhausted, an
    /// error is returned
    async fn retrieve_block_for_id(
        &mut self,
        block_id: HashValue,
        peers: &mut Vec<&AccountAddress>,
        num_blocks: u64,
    ) -> anyhow::Result<Vec<Block>> {
        let mut attempt = 0_u32;
        let mut progress = 0;
        let mut last_block_id = block_id;
        let mut result_blocks: Vec<Block> = vec![];
        let mut retrieve_batch_size = MAX_BLOCKS_PER_REQUEST;
        if peers.is_empty() {
            bail!(
                "Failed to fetch block {} in {} attempts: no more peers available",
                block_id,
                attempt
            );
        }
        let mut peer = self.pick_peer(attempt, peers);
        while progress < num_blocks {
            // in case this is the last retrieval
            retrieve_batch_size = min(retrieve_batch_size, num_blocks - progress);

            attempt += 1;

            debug!(
                LogSchema::new(LogEvent::RetrieveBlock).remote_peer(peer),
                block_id = block_id,
                "Fetching {} blocks, attempt {}",
                retrieve_batch_size,
                attempt
            );
            let response = self
                .network
                .request_block(
                    BlockRetrievalRequest::new(last_block_id, retrieve_batch_size),
                    peer,
                    retrieval_timeout(attempt),
                )
                .await;
            match response.and_then(|result| {
                if result.status() == BlockRetrievalStatus::Succeeded {
                    Ok(result.blocks().clone())
                } else {
                    Err(format_err!("{:?}", result.status()))
                }
            }) {
                Ok(batch) => {
                    // extend the result blocks
                    progress += batch.len() as u64;
                    last_block_id = batch.last().unwrap().parent_id();
                    result_blocks.extend(batch);
                }
                Err(e) => {
                    warn!(
                        remote_peer = peer,
                        block_id = block_id,
                        error = ?e, "Failed to fetch block, trying another peer",
                    );
                    // select next peer to try
                    if peers.is_empty() {
                        bail!(
                            "Failed to fetch block {} in {} attempts: no more peers available",
                            block_id,
                            attempt
                        );
                    }
                    peer = self.pick_peer(attempt, peers);
                }
            }
        }
        assert_eq!(result_blocks.len() as u64, num_blocks);
        Ok(result_blocks)
    }

    /// Retrieve chain of n blocks for given QC
    async fn retrieve_block_for_qc<'a>(
        &'a mut self,
        qc: &'a QuorumCert,
        num_blocks: u64,
    ) -> anyhow::Result<Vec<Block>> {
        let mut peers = qc
            .ledger_info()
            .signatures()
            .keys()
            .collect::<Vec<&AccountAddress>>();
        self.retrieve_block_for_id(qc.certified_block().id(), &mut peers, num_blocks)
            .await
    }

    fn pick_peer(&self, attempt: u32, peers: &mut Vec<&AccountAddress>) -> AccountAddress {
        assert!(!peers.is_empty(), "pick_peer on empty peer list");

        if attempt == 0 {
            // remove preferred_peer if its in list of peers
            // (strictly speaking it is not required to be there)
            for i in 0..peers.len() {
                if *peers[i] == self.preferred_peer {
                    peers.remove(i);
                    break;
                }
            }
            return self.preferred_peer;
        }

        let peer_idx = thread_rng().gen_range(0..peers.len());
        *peers.remove(peer_idx)
    }
}

// Max timeout is 16s=RETRIEVAL_INITIAL_TIMEOUT*(2^RETRIEVAL_MAX_EXP)
const RETRIEVAL_INITIAL_TIMEOUT: Duration = Duration::from_millis(200);
const RETRIEVAL_MAX_EXP: u32 = 4;

/// Returns exponentially increasing timeout with
/// limit of RETRIEVAL_INITIAL_TIMEOUT*(2^RETRIEVAL_MAX_EXP)
#[allow(clippy::trivially_copy_pass_by_ref)]
fn retrieval_timeout(attempt: u32) -> Duration {
    assert!(attempt > 0, "retrieval_timeout attempt can't be 0");
    let exp = RETRIEVAL_MAX_EXP.min(attempt - 1); // [0..RETRIEVAL_MAX_EXP]
    RETRIEVAL_INITIAL_TIMEOUT * 2_u32.pow(exp)
}

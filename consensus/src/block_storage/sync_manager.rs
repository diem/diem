// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    block_storage::{BlockReader, BlockStore},
    network::NetworkSender,
    network_interface::ConsensusMsg,
    persistent_liveness_storage::{PersistentLivenessStorage, RecoveryData},
    state_replication::StateComputer,
};
use anyhow::{bail, format_err};
use consensus_types::{
    block::Block,
    block_retrieval::{BlockRetrievalRequest, BlockRetrievalStatus},
    common::Author,
    quorum_cert::QuorumCert,
    sync_info::SyncInfo,
};
use libra_logger::prelude::*;
use libra_types::{account_address::AccountAddress, epoch_change::EpochChangeProof};
use mirai_annotations::checked_precondition;
use rand::{prelude::*, Rng};
use std::{clone::Clone, sync::Arc, time::Duration};
use termion::color::*;

#[derive(Debug, PartialEq)]
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
        checked_precondition!(self.root().round() < std::u64::MAX - 1);

        // If we have the block locally, we're not far from this QC thus don't need to sync.
        // In case root().round() is greater than that the committed
        // block carried by LI is older than my current commit.
        !(self.block_exists(qc.commit_info().id())
            || self.root().round() >= qc.commit_info().round())
    }

    /// Checks if quorum certificate can be inserted in block store without RPC
    /// Returns the enum to indicate the detailed status.
    pub fn need_fetch_for_quorum_cert(&self, qc: &QuorumCert) -> NeedFetchResult {
        if qc.certified_block().round() < self.root().round() {
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
    /// If gap is large, performs state sync using process_highest_commit_cert
    /// Inserts sync_info.quorum_cert into block store as the last step
    pub async fn add_certs(
        &self,
        sync_info: &SyncInfo,
        mut retriever: BlockRetriever,
    ) -> anyhow::Result<()> {
        self.sync_to_highest_commit_cert(sync_info.highest_commit_cert().clone(), &mut retriever)
            .await?;

        self.insert_quorum_cert(sync_info.highest_commit_cert(), &mut retriever)
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
        match self.need_fetch_for_quorum_cert(&qc) {
            NeedFetchResult::NeedFetch => self.fetch_quorum_cert(qc.clone(), retriever).await?,
            NeedFetchResult::QCBlockExist => self.insert_single_quorum_cert(qc.clone())?,
            _ => (),
        }
        if self.root().round() < qc.commit_info().round() {
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
    async fn sync_to_highest_commit_cert(
        &self,
        highest_commit_cert: QuorumCert,
        retriever: &mut BlockRetriever,
    ) -> anyhow::Result<()> {
        if !self.need_sync_for_quorum_cert(&highest_commit_cert) {
            return Ok(());
        }
        let (root, root_metadata, blocks, quorum_certs) = Self::fast_forward_sync(
            &highest_commit_cert,
            retriever,
            self.storage.clone(),
            self.state_computer.clone(),
        )
        .await?
        .take();
        debug!("{}Sync to{} {}", Fg(Blue), Fg(Reset), root.0);
        self.rebuild(root, root_metadata, blocks, quorum_certs)
            .await;

        if highest_commit_cert.ends_epoch() {
            retriever
                .network
                .notify_epoch_change(EpochChangeProof::new(
                    vec![highest_commit_cert.ledger_info().clone()],
                    /* more = */ false,
                ))
                .await;
        }
        Ok(())
    }

    pub async fn fast_forward_sync<'a>(
        highest_commit_cert: &'a QuorumCert,
        retriever: &'a mut BlockRetriever,
        storage: Arc<dyn PersistentLivenessStorage>,
        state_computer: Arc<dyn StateComputer>,
    ) -> anyhow::Result<RecoveryData> {
        debug!(
            "Start state sync with peer: {}, to block: {}",
            retriever.preferred_peer.short_str(),
            highest_commit_cert.commit_info(),
        );

        let blocks = retriever
            .retrieve_block_for_qc(&highest_commit_cert, 3)
            .await?;
        assert_eq!(
            blocks.last().expect("should have 3-chain").id(),
            highest_commit_cert.commit_info().id(),
        );
        let mut quorum_certs = vec![];
        quorum_certs.push(highest_commit_cert.clone());
        quorum_certs.extend(
            blocks
                .iter()
                .take(2)
                .map(|block| block.quorum_cert().clone()),
        );
        for (i, block) in blocks.iter().enumerate() {
            assert_eq!(block.id(), quorum_certs[i].certified_block().id());
        }

        // If a node restarts in the middle of state synchronization, it is going to try to catch up
        // to the stored quorum certs as the new root.
        storage.save_tree(blocks.clone(), quorum_certs.clone())?;
        state_computer
            .sync_to(highest_commit_cert.ledger_info().clone())
            .await?;
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
    /// Retrieve chain of n blocks for given QC
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
    async fn retrieve_block_for_qc<'a>(
        &'a mut self,
        qc: &'a QuorumCert,
        num_blocks: u64,
    ) -> anyhow::Result<Vec<Block>> {
        let block_id = qc.certified_block().id();
        let mut peers: Vec<&AccountAddress> = qc.ledger_info().signatures().keys().collect();
        let mut attempt = 0_u32;
        loop {
            if peers.is_empty() {
                bail!(
                    "Failed to fetch block {} in {} attempts: no more peers available",
                    block_id,
                    attempt
                );
            }
            let peer = self.pick_peer(attempt, &mut peers);
            attempt += 1;

            debug!(
                "Fetching {} from {}, attempt {}",
                block_id,
                peer.short_str(),
                attempt
            );
            let response = self
                .network
                .request_block(
                    BlockRetrievalRequest::new(block_id, num_blocks),
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
                result @ Ok(_) => return result,
                Err(e) => warn!(
                    "Failed to fetch block {} from {}: {:?}, trying another peer",
                    block_id,
                    peer.short_str(),
                    e,
                ),
            }
        }
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

        let peer_idx = thread_rng().gen_range(0, peers.len());
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

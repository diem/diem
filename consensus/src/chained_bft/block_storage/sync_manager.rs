// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        block_storage::{BlockReader, BlockStore},
        network::NetworkSender,
    },
    counters,
};
use consensus_types::block_retrieval::{
    BlockRetrievalMode, BlockRetrievalRequest, BlockRetrievalStatus,
};
use consensus_types::common::Round;
use consensus_types::{
    block::Block,
    common::{Author, Payload},
    quorum_cert::QuorumCert,
    sync_info::SyncInfo,
};
use failure;
use libra_crypto::HashValue;
use libra_logger::prelude::*;
use libra_types::account_address::AccountAddress;
use rand::{prelude::*, Rng};
use std::{
    clone::Clone,
    time::{Duration, Instant},
};
use termion::color::*;

#[derive(Debug, PartialEq)]
/// Whether we need to do block retrieval if we want to insert a Quorum Cert.
pub enum NeedFetchResult {
    QCRoundBeforeRoot,
    QCAlreadyExist,
    QCBlockExist,
    NeedFetch,
}

impl<T: Payload> BlockStore<T> {
    /// Check whether the local tree is falling behind (and might need state synchronization):
    /// 1) the root round is lower than the given round.
    /// 2) the committed block is not present in the block store.
    pub fn is_tree_outdated(&self, committed_block_id: HashValue, committed_round: Round) -> bool {
        self.root().round() < committed_round && !self.block_exists(committed_block_id)
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
    /// If gap is large, performs state sync using process_highest_ledger_info
    /// Inserts sync_info.quorum_cert into block store as the last step
    pub async fn sync_to(
        &self,
        sync_info: &SyncInfo,
        mut retriever: BlockRetriever,
    ) -> failure::Result<()> {
        self.process_highest_ledger_info_deprecated(
            sync_info.highest_ledger_info().clone(),
            &mut retriever,
        )
        .await?;

        match self.need_fetch_for_quorum_cert(sync_info.highest_quorum_cert()) {
            NeedFetchResult::NeedFetch => {
                self.fetch_quorum_cert(sync_info.highest_quorum_cert().clone(), retriever)
                    .await?
            }
            NeedFetchResult::QCBlockExist => {
                self.insert_single_quorum_cert(sync_info.highest_quorum_cert().clone())?
            }
            _ => (),
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
        mut retriever: BlockRetriever,
    ) -> failure::Result<()> {
        let mut pending = vec![];
        let mut retrieve_qc = qc.clone();
        loop {
            if self.block_exists(retrieve_qc.certified_block().id()) {
                break;
            }
            let mut blocks = retriever
                .retrieve_blocks(
                    BlockRetrievalRequest::new(
                        retrieve_qc.certified_block().id(),
                        BlockRetrievalMode::Ancestors(1),
                    ),
                    retrieve_qc
                        .ledger_info()
                        .signatures()
                        .keys()
                        .copied()
                        .collect(),
                )
                .await?;
            // retrieve_block_for_qc guarantees that blocks has exactly 1 element
            let block = blocks.remove(0);
            retrieve_qc = block.quorum_cert().clone();
            pending.push(block);
        }
        // insert the qc <- block pair
        while let Some(block) = pending.pop() {
            let block_qc = block.quorum_cert().clone();
            self.insert_single_quorum_cert(block_qc)?;
            self.execute_and_insert_block(block).await?;
        }
        self.insert_single_quorum_cert(qc)
    }

    /// Check the highest ledger info sent by peer to see if we're behind and start a fast
    /// forward sync if the committed block doesn't exist in our tree.
    /// It works as follows:
    /// 1. request the committed 3-chain from the peer, if C2 is the highest_ledger_info
    /// we request for B0 <- C0 <- B1 <- C1 <- B2 (<- C2)
    /// 2. We persist the 3-chain to storage before start sync to ensure we could restart if we
    /// crash in the middle of the sync.
    /// 3. We prune the old tree and replace with a new tree built with the 3-chain.
    async fn process_highest_ledger_info_deprecated(
        &self,
        highest_ledger_info: QuorumCert,
        retriever: &mut BlockRetriever,
    ) -> failure::Result<()> {
        let committed_block_id = highest_ledger_info
            .committed_block_id()
            .ok_or_else(|| format_err!("highest ledger info has no committed block"))?;
        // Unfortunately we're leaking some information about the commit rule here:
        // qc.parent_block() must be a child of the commit due to the three chain commit rule.
        // Hence, commit_round is qc.parent_block() - 1.
        let commit_round = if highest_ledger_info.parent_block().round() > 0 {
            highest_ledger_info.parent_block().round() - 1
        } else {
            0
        };
        if !self.is_tree_outdated(committed_block_id, commit_round) {
            return Ok(());
        }
        debug!(
            "Start state sync with peer: {}, to block: {}, round: {} from {}",
            retriever.preferred_peer.short_str(),
            committed_block_id,
            highest_ledger_info.certified_block().round() - 2,
            self.root()
        );

        let mut blocks = retriever
            .retrieve_blocks(
                BlockRetrievalRequest::new(
                    highest_ledger_info.certified_block().id(),
                    BlockRetrievalMode::Ancestors(3),
                ),
                highest_ledger_info
                    .ledger_info()
                    .signatures()
                    .keys()
                    .copied()
                    .collect(),
            )
            .await?;
        assert_eq!(
            blocks.last().expect("should have 3-chain").id(),
            committed_block_id
        );
        let mut quorum_certs = vec![];
        quorum_certs.push(highest_ledger_info.clone());
        quorum_certs.push(blocks[0].quorum_cert().clone());
        quorum_certs.push(blocks[1].quorum_cert().clone());
        // If a node restarts in the middle of state synchronization, it is going to try to catch up
        // to the stored quorum certs as the new root.
        self.storage
            .save_tree(blocks.clone(), quorum_certs.clone())?;
        let pre_sync_instance = Instant::now();
        self.state_computer
            .sync_to_or_bail_deprecated(highest_ledger_info.ledger_info().clone());
        counters::STATE_SYNC_DURATION_S.observe_duration(pre_sync_instance.elapsed());
        let root = (
            blocks.pop().expect("should have 3-chain"),
            quorum_certs.last().expect("should have 3-chain").clone(),
            highest_ledger_info.clone(),
        );
        debug!("{}Sync to{} {}", Fg(Blue), Fg(Reset), root.0);
        // ensure it's [b1, b2]
        blocks.reverse();
        self.rebuild(root, blocks, quorum_certs).await;
        Ok(())
    }

    /// Check the highest ledger info sent by peer to see if we're behind and start
    /// state synchronization if the committed block doesn't exist in the tree (the whole tree is
    /// outdated).
    async fn process_highest_ledger_info(
        &self,
        highest_ledger_info: QuorumCert,
        retriever: &mut BlockRetriever,
    ) -> failure::Result<()> {
        let committed_block_id = highest_ledger_info
            .committed_block_id()
            .ok_or_else(|| format_err!("highest ledger info has no committed block"))?;
        // Unfortunately we're leaking some information about the commit rule here:
        // qc.parent_block() must be a child of the commit.
        // Hence, commit_round is qc.parent_block() - 1.
        let commit_round = if highest_ledger_info.parent_block().round() > 0 {
            highest_ledger_info.parent_block().round() - 1
        } else {
            0
        };
        if !self.is_tree_outdated(committed_block_id, commit_round) {
            return Ok(());
        }
        // TODO: store the pending LedgerInfo for the duration of the state synchronization
        let mut attempt = 0_u32;
        loop {
            debug!(
                "State sync attempt = {} to round {} from {}",
                attempt,
                commit_round,
                self.root()
            );
            match self
                .state_sync_attempt(highest_ledger_info.clone(), retriever)
                .await
            {
                Err(e) => {
                    error!(
                        "Failed state sync attempt {} to round {}: {}",
                        attempt, commit_round, e
                    );
                    attempt += 1;
                }
                Ok(_) => {
                    debug!("State sync success. New root: {}", self.root());
                    return Ok(());
                }
            }
        }
    }

    async fn state_sync_attempt(
        &self,
        highest_ledger_info: QuorumCert,
        retriever: &mut BlockRetriever,
    ) -> failure::Result<()> {
        // State computer synchronization must eventually succeed (an infinite retry):
        // it returns the LedgerInfo corresponding to the latest committed state in accumulator.
        let pre_sync_instance = Instant::now();
        let new_root_li = self
            .state_computer
            .state_sync_or_bail(highest_ledger_info.ledger_info().clone());
        counters::STATE_SYNC_DURATION_S.observe_duration(pre_sync_instance.elapsed());
        let new_root_id = new_root_li.ledger_info().consensus_block_id();
        // BlockRetrieval guarantees that upon success the returned chain certifies the commit of
        // the given root in one of the QuorumCerts carried by the blocks.
        // The root is the very last element in the chain.
        let mut new_root_chain = retriever
            .retrieve_blocks(
                BlockRetrievalRequest::new(new_root_id, BlockRetrievalMode::Descendants),
                highest_ledger_info
                    .ledger_info()
                    .signatures()
                    .keys()
                    .copied()
                    .collect(),
            )
            .await?;

        // An example chain looks like b3-->b2-->b1-->b0, where b0 is the new root.
        let new_root_block = new_root_chain
            .pop()
            .expect("Empty chain in a successful retrieval response");
        // QuorumCerts in descending order
        let quorum_certs = new_root_chain
            .iter()
            .map(|b| b.quorum_cert().clone())
            .collect::<Vec<QuorumCert>>();
        // Find the QC that carries a commit proof for the root.
        let new_root_li = new_root_chain
            .iter()
            .find(|b| b.quorum_cert().committed_block_id() == Some(new_root_id))
            .expect("No commit proof for root in a successful retrieval response")
            .quorum_cert()
            .clone();
        let root_info = (
            new_root_block,
            quorum_certs
                .last()
                .expect("No QC for root in a successful retrieval response")
                .clone(),
            new_root_li,
        );
        new_root_chain.reverse(); // Tree building process executes the blocks in ascending order.
        self.rebuild(root_info, new_root_chain, quorum_certs).await;
        Ok(())
    }
}

/// BlockRetriever is used internally to retrieve blocks
pub struct BlockRetriever {
    network: NetworkSender,
    deadline: Instant,
    preferred_peer: Author,
}

impl BlockRetriever {
    pub fn new(network: NetworkSender, deadline: Instant, preferred_peer: Author) -> Self {
        Self {
            network,
            deadline,
            preferred_peer,
        }
    }
    /// Retrieve chain of n blocks starting with the given block id and continuing to its
    /// ancestors (e.g., for num_blocks = 3 it'll be the block, its parent and grandparent).
    ///
    /// This method continues attempts to bring the requested chain either until the round deadline
    /// is reached or until all the members from the given list fail to return the requested chain.
    ///
    /// The first attempt of block retrieval will always be sent to preferred_peer to allow the
    /// leader to drive quorum certificate creation. The other peers from the given list will be
    /// randomly tried next.
    async fn retrieve_blocks<T>(
        &mut self,
        req: BlockRetrievalRequest,
        mut peers: Vec<AccountAddress>,
    ) -> failure::Result<Vec<Block<T>>>
    where
        T: Payload,
    {
        let mut attempt = 0_u32;
        loop {
            if peers.is_empty() {
                bail!(
                    "{} failed in {} attempts: no more peers available",
                    req.clone(),
                    attempt
                );
            }
            let peer = self.pick_peer(attempt, &mut peers);
            attempt += 1;

            let timeout = retrieval_timeout(&self.deadline, attempt);
            let timeout = timeout.ok_or_else(|| {
                format_err!("{} to {} failed, attempt {}: round deadline was reached, won't make more attempts", req, peer.short_str(), attempt)
            })?;
            debug!(
                "Sending {} to {}, attempt {}",
                req,
                peer.short_str(),
                attempt
            );
            let response = self.network.request_block(req.clone(), peer, timeout).await;
            let response = match response {
                Err(e) => {
                    warn!(
                        "{} to {} failed: {}, trying another peer",
                        req,
                        peer.short_str(),
                        e
                    );
                    continue;
                }
                Ok(response) => response,
            };
            if response.status() != BlockRetrievalStatus::Succeeded {
                warn!(
                    "{} to {} failed: {:?}, trying another peer",
                    req,
                    peer.short_str(),
                    response.status()
                );
                continue;
            }
            return Ok(response.blocks().clone());
        }
    }

    fn pick_peer(&self, attempt: u32, peers: &mut Vec<AccountAddress>) -> AccountAddress {
        assert!(!peers.is_empty(), "pick_peer on empty peer list");

        if attempt == 0 {
            // remove preferred_peer if its in list of peers
            // (strictly speaking it is not required to be there)
            for i in 0..peers.len() {
                if peers[i] == self.preferred_peer {
                    peers.remove(i);
                    break;
                }
            }
            return self.preferred_peer;
        }

        let peer_idx = thread_rng().gen_range(0, peers.len());
        peers.remove(peer_idx)
    }
}

// Max timeout is 16s=RETRIEVAL_INITIAL_TIMEOUT*(2^RETRIEVAL_MAX_EXP)
const RETRIEVAL_INITIAL_TIMEOUT: Duration = Duration::from_secs(1);
const RETRIEVAL_MAX_EXP: u32 = 4;

/// Returns exponentially increasing timeout with
/// limit of RETRIEVAL_INITIAL_TIMEOUT*(2^RETRIEVAL_MAX_EXP)
fn retrieval_timeout(deadline: &Instant, attempt: u32) -> Option<Duration> {
    assert!(attempt > 0, "retrieval_timeout attempt can't be 0");
    let exp = RETRIEVAL_MAX_EXP.min(attempt - 1); // [0..RETRIEVAL_MAX_EXP]
    let request_timeout = RETRIEVAL_INITIAL_TIMEOUT * 2_u32.pow(exp);
    let now = Instant::now();
    let deadline_timeout = if *deadline >= now {
        Some(deadline.duration_since(now))
    } else {
        None
    };
    deadline_timeout.map(|delay| request_timeout.min(delay))
}

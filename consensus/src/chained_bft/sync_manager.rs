// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        block_storage::{BlockReader, BlockStore, NeedFetchResult},
        common::{Author, Payload},
        consensus_types::{
            block::{Block, ExecutedBlock},
            quorum_cert::QuorumCert,
            sync_info::SyncInfo,
        },
        network::ConsensusNetworkImpl,
        persistent_storage::PersistentStorage,
    },
    counters,
    state_replication::StateComputer,
};
use failure;
use logger::prelude::*;
use network::proto::BlockRetrievalStatus;
use rand::{prelude::*, Rng};
use std::{
    clone::Clone,
    sync::Arc,
    time::{Duration, Instant},
};
use termion::color::*;
use types::account_address::AccountAddress;

/// SyncManager is responsible for fetching dependencies and 'catching up' for given qc/ledger info
pub struct SyncManager<T> {
    block_store: Arc<BlockStore<T>>,
    storage: Arc<dyn PersistentStorage<T>>,
    network: ConsensusNetworkImpl,
    state_computer: Arc<dyn StateComputer<Payload = T>>,
}

/// Keeps the necessary context for `SyncMgr` to bring the missing information.
pub struct SyncMgrContext {
    pub highest_ledger_info: QuorumCert,
    pub highest_quorum_cert: QuorumCert,
    /// Preferred peer: this is typically the peer that delivered the original QC and
    /// thus has higher chances to be able to return the information than the other
    /// peers that signed the QC.
    /// If no preferred peer provided, random peers from the given QC are going to be queried.
    pub preferred_peer: Author,
}

impl SyncMgrContext {
    pub fn new(sync_info: &SyncInfo, preferred_peer: Author) -> Self {
        Self {
            highest_ledger_info: sync_info.highest_ledger_info().clone(),
            highest_quorum_cert: sync_info.highest_quorum_cert().clone(),
            preferred_peer,
        }
    }
}

impl<T> SyncManager<T>
where
    T: Payload,
{
    pub fn new(
        block_store: Arc<BlockStore<T>>,
        storage: Arc<dyn PersistentStorage<T>>,
        network: ConsensusNetworkImpl,
        state_computer: Arc<dyn StateComputer<Payload = T>>,
    ) -> SyncManager<T> {
        // Our counters are initialized via lazy_static, so they're not going to appear in
        // Prometheus if some conditions never happen.  Invoking get() function enforces creation.
        counters::BLOCK_RETRIEVAL_COUNT.get();
        counters::STATE_SYNC_COUNT.get();
        SyncManager {
            block_store,
            storage,
            network,
            state_computer,
        }
    }

    /// Fetches dependencies for given sync_info.quorum_cert
    /// If gap is large, performs state sync using process_highest_ledger_info
    /// Inserts sync_info.quorum_cert into block store as the last step
    pub async fn sync_to(
        &mut self,
        deadline: Instant,
        sync_context: SyncMgrContext,
    ) -> failure::Result<()> {
        self.process_highest_ledger_info(
            sync_context.highest_ledger_info.clone(),
            sync_context.preferred_peer,
            deadline,
        )
        .await?;

        match self
            .block_store
            .need_fetch_for_quorum_cert(&sync_context.highest_quorum_cert)
        {
            NeedFetchResult::NeedFetch => {
                self.fetch_quorum_cert(
                    sync_context.highest_quorum_cert.clone(),
                    sync_context.preferred_peer,
                    deadline,
                )
                .await?
            }
            NeedFetchResult::QCBlockExist => self
                .block_store
                .insert_single_quorum_cert(sync_context.highest_quorum_cert)?,
            _ => (),
        }
        Ok(())
    }

    pub async fn execute_and_insert_block(
        &self,
        block: Block<T>,
    ) -> failure::Result<Arc<ExecutedBlock<T>>> {
        // execute_and_insert_block has shortcut to return block if it exists
        self.block_store.execute_and_insert_block(block).await
    }

    /// Insert the quorum certificate separately from the block, used to split the processing of
    /// updating the consensus state(with qc) and deciding whether to vote(with block)
    /// The missing ancestors are going to be retrieved from the given peer. If a given peer
    /// fails to provide the missing ancestors, the qc is not going to be added.
    pub async fn fetch_quorum_cert(
        &self,
        qc: QuorumCert,
        preferred_peer: Author,
        deadline: Instant,
    ) -> failure::Result<()> {
        let mut pending = vec![];
        let network = self.network.clone();
        let mut retriever = BlockRetriever {
            network,
            deadline,
            preferred_peer,
        };
        let mut retrieve_qc = qc.clone();
        loop {
            if self
                .block_store
                .block_exists(retrieve_qc.certified_block_id())
            {
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
            self.block_store.insert_single_quorum_cert(block_qc)?;
            self.block_store.execute_and_insert_block(block).await?;
        }
        self.block_store.insert_single_quorum_cert(qc)
    }

    /// Check the highest ledger info sent by peer to see if we're behind and start a fast
    /// forward sync if the committed block doesn't exist in our tree.
    /// It works as follows:
    /// 1. request the committed 3-chain from the peer, if C2 is the highest_ledger_info
    /// we request for B0 <- C0 <- B1 <- C1 <- B2 (<- C2)
    /// 2. We persist the 3-chain to storage before start sync to ensure we could restart if we
    /// crash in the middle of the sync.
    /// 3. We prune the old tree and replace with a new tree built with the 3-chain.
    async fn process_highest_ledger_info(
        &self,
        highest_ledger_info: QuorumCert,
        peer: Author,
        deadline: Instant,
    ) -> failure::Result<()> {
        let committed_block_id = highest_ledger_info
            .committed_block_id()
            .ok_or_else(|| format_err!("highest ledger info has no committed block"))?;
        if !self
            .block_store
            .need_sync_for_quorum_cert(committed_block_id, &highest_ledger_info)
        {
            return Ok(());
        }
        debug!(
            "Start state sync with peer: {}, to block: {}, round: {} from {}",
            peer.short_str(),
            committed_block_id,
            highest_ledger_info.certified_block_round() - 2,
            self.block_store.root()
        );
        let network = self.network.clone();
        let mut retriever = BlockRetriever {
            network,
            deadline,
            preferred_peer: peer,
        };
        let mut blocks = retriever
            .retrieve_block_for_qc(&highest_ledger_info, 3)
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
            .sync_to_or_bail(highest_ledger_info.clone());
        counters::STATE_SYNC_DURATION_S.observe_duration(pre_sync_instance.elapsed());
        let root = (
            blocks.pop().expect("should have 3-chain"),
            quorum_certs.last().expect("should have 3-chain").clone(),
            highest_ledger_info.clone(),
        );
        debug!("{}Sync to{} {}", Fg(Blue), Fg(Reset), root.0);
        // ensure it's [b1, b2]
        blocks.reverse();
        self.block_store.rebuild(root, blocks, quorum_certs).await;
        Ok(())
    }
}

/// BlockRetriever is used internally to retrieve blocks
struct BlockRetriever {
    network: ConsensusNetworkImpl,
    deadline: Instant,
    preferred_peer: Author,
}

impl BlockRetriever {
    /// Retrieve chain of n blocks for given QC
    ///
    /// Returns Result with Vec that has a guaranteed size of num_blocks
    /// This guarantee is based on BlockRetrievalResponse::verify that ensures that number of
    /// blocks in response is equal to number of blocks requested.  This method will
    /// continue until either the round deadline is reached or the quorum certificate members all
    /// fail to return the missing chain.
    ///
    /// The first attempt of block retrieval will always be sent to preferred_peer to allow the
    /// leader to drive quorum certificate creation The other peers from the quorum certificate
    /// will be randomly tried next.  If all members of the quorum certificate are exhausted, an
    /// error is returned
    pub async fn retrieve_block_for_qc<'a, T>(
        &'a mut self,
        qc: &'a QuorumCert,
        num_blocks: u64,
    ) -> failure::Result<Vec<Block<T>>>
    where
        T: Payload,
    {
        let block_id = qc.certified_block_id();
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

            let timeout = retrieval_timeout(&self.deadline, attempt);
            let timeout = if let Some(timeout) = timeout {
                timeout
            } else {
                bail!("Failed to fetch block {} from {}, attempt {}: round deadline was reached, won't make more attempts", block_id, peer, attempt);
            };
            debug!(
                "Fetching {} from {}, attempt {}",
                block_id,
                peer.short_str(),
                attempt
            );
            let response = self
                .network
                .request_block(block_id, num_blocks, peer, timeout)
                .await;
            let response = match response {
                Err(e) => {
                    warn!(
                        "Failed to fetch block {} from {}: {:?}, trying another peer",
                        block_id,
                        peer.short_str(),
                        e
                    );
                    continue;
                }
                Ok(response) => response,
            };
            if response.status != BlockRetrievalStatus::Succeeded {
                warn!(
                    "Failed to fetch block {} from {}: {:?}, trying another peer",
                    block_id,
                    peer.short_str(),
                    response.status
                );
                continue;
            }
            return Ok(response.blocks);
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

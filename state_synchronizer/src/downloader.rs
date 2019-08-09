// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{coordinator::CoordinatorMsg, counters::OP_COUNTERS, PeerId};
use failure::prelude::*;
use futures::{channel::mpsc, SinkExt, StreamExt};
use logger::prelude::*;
use network::{proto::RequestChunk, validator_network::ConsensusNetworkSender};
use rand::{thread_rng, Rng};
use std::time::Duration;
use types::{ledger_info::LedgerInfoWithSignatures, proto::transaction::TransactionListWithProof};

/// Used for communication between coordinator and downloader
/// and represents a single fetch request
#[derive(Clone)]
pub struct FetchChunkMsg {
    // target version that we want to fetch
    pub target: LedgerInfoWithSignatures,
    // version from which to start fetching (the offset version)
    pub start_version: u64,
}

/// Used to download chunks of transactions from peers
pub struct Downloader {
    receiver_from_coordinator: mpsc::Receiver<FetchChunkMsg>,
    sender_to_coordinator: mpsc::UnboundedSender<CoordinatorMsg>,
    network: ConsensusNetworkSender,
    batch_size: u64,
    retries: usize,
}

impl Downloader {
    pub fn new(
        receiver_from_coordinator: mpsc::Receiver<FetchChunkMsg>,
        sender_to_coordinator: mpsc::UnboundedSender<CoordinatorMsg>,
        network: ConsensusNetworkSender,
        batch_size: u64,
        retries: usize,
    ) -> Self {
        Self {
            receiver_from_coordinator,
            sender_to_coordinator,
            network,
            batch_size,
            retries,
        }
    }

    /// Starts chunk downloader that listens to FetchChunkMsgs
    pub async fn start(mut self) {
        while let Some(msg) = self.receiver_from_coordinator.next().await {
            for attempt in 0..self.retries {
                let peer_id = self.pick_peer_id(&msg);
                let download_result = self.download_chunk(peer_id, msg.clone()).await;
                if download_result.is_ok() || attempt == self.retries - 1 {
                    let send_result = self
                        .sender_to_coordinator
                        .send(CoordinatorMsg::Fetched(download_result, msg.target))
                        .await;
                    if send_result.is_err() {
                        log_collector_error!("[state synchronizer] failed to send chunk from downloader to coordinator");
                    }
                    break;
                }
            }
        }
    }

    /// Downloads a chunk from another validator or from a cloud provider.
    /// It then verifies that the data in the chunk is valid and returns the validated data.
    async fn download_chunk(
        &mut self,
        peer_id: PeerId,
        msg: FetchChunkMsg,
    ) -> Result<TransactionListWithProof> {
        // Construct the message and use rpc call via network stack
        let mut req = RequestChunk::new();
        req.set_start_version(msg.start_version);
        req.set_target_version(msg.target.ledger_info().version());
        req.set_batch_size(self.batch_size);
        // Longer-term, we will read from a cloud provider.  But for testnet, just read
        // from the node which is proposing this block
        let mut resp = self
            .network
            .request_chunk(peer_id, req, Duration::from_millis(1000))
            .await?;

        OP_COUNTERS.inc_by(
            "download",
            resp.get_txn_list_with_proof().get_transactions().len(),
        );
        Ok(resp.take_txn_list_with_proof())
    }

    fn pick_peer_id(&self, msg: &FetchChunkMsg) -> PeerId {
        let signatures = msg.target.signatures();
        let idx = thread_rng().gen_range(0, signatures.len());
        signatures
            .keys()
            .nth(idx)
            .cloned()
            .expect("[state synchronizer] failed to pick peer from ledger info")
    }
}

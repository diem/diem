// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    counters, metrics_safety_rules::MetricsSafetyRules, network::NetworkSender,
    network_interface::ConsensusMsg, state_replication::StateComputer,
};
use channel::Receiver;
use consensus_types::{
    common::Author,
    executed_block::ExecutedBlock,
    experimental::{commit_decision::CommitDecision, commit_vote::CommitVote},
};
use core::sync::atomic::Ordering;
use diem_crypto::ed25519::Ed25519Signature;
use diem_infallible::Mutex;
use diem_logger::prelude::*;
use diem_metrics::monitor;
use diem_types::{
    account_address::AccountAddress,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    validator_verifier::ValidatorVerifier,
};
use executor_types::Error as ExecutionError;
use futures::{select, StreamExt};
use safety_rules::TSafetyRules;
use std::{
    collections::BTreeMap,
    sync::{atomic::AtomicU64, Arc},
};

/*
 Commit phase takes in the executed blocks from the execution
 phase and commit them. Specifically, commit phase signs a commit
 vote message containing the execution result and broadcast it.
 Upon collecting a quorum of agreeing votes to a execution result,
 the commit phase commits the blocks as well as broadcasts a commit
 decision message together with the quorum of signatures. The commit
 decision message helps the slower nodes to quickly catch up without
 having to collect the signatures.
 */

#[derive(Clone)]
struct PendingBlocks {
    vecblocks: Vec<ExecutedBlock>,
}

impl PendingBlocks {
    pub fn new(vecblocks: Vec<ExecutedBlock>) -> Self {
        Self { vecblocks }
    }
}

#[derive(Debug)]
pub enum CommitPhaseMessage {
    CommitVote(Author, LedgerInfo, Ed25519Signature),
    CommitDecision(LedgerInfoWithSignatures),
}

pub struct CommitPhase {
    commit_channel_recv: Receiver<(Vec<ExecutedBlock>, LedgerInfoWithSignatures)>,
    execution_proxy: Arc<dyn StateComputer>,
    blocks: Option<(Vec<ExecutedBlock>, LedgerInfoWithSignatures)>,
    commit_msg_receiver: channel::Receiver<ConsensusMsg>,
    verifier: ValidatorVerifier,
    safety_rules: Arc<Mutex<MetricsSafetyRules>>,
    author: Author,
    back_pressure: Arc<AtomicU64>,
    network_sender: NetworkSender,
}

pub async fn commit(
    execution_proxy: &Arc<dyn StateComputer>,
    vecblock: &[ExecutedBlock],
    ledger_info: &LedgerInfoWithSignatures,
) -> Result<(), ExecutionError> {
    // debug!("New commit: {} {}", vecblock.len(), ledger_info);
    // have to maintain the order.
    execution_proxy
        .commit(
            &vecblock
                .iter()
                .map(|eb| Arc::new(eb.clone()))
                .collect::<Vec<Arc<ExecutedBlock>>>(),
            ledger_info.clone(),
        )
        .await
}

macro_rules! report_err {
    ($result:expr, $error_string:literal) => {
        if let Err(err) = $result {
            counters::ERROR_COUNT.inc();
            error!(error = err.to_string(), $error_string,)
        }
    };
}

impl CommitPhase {
    pub fn new(
        commit_channel_recv: Receiver<(Vec<ExecutedBlock>, LedgerInfoWithSignatures)>,
        execution_proxy: Arc<dyn StateComputer>,
        commit_msg_receiver: channel::Receiver<ConsensusMsg>,
        verifier: ValidatorVerifier,
        safety_rules: Arc<Mutex<MetricsSafetyRules>>,
        author: Author,
        back_pressure: Arc<AtomicU64>,
        network_sender: NetworkSender,
    ) -> Self {
        Self {
            commit_channel_recv,
            execution_proxy,
            blocks: None,
            commit_msg_receiver,
            verifier,
            safety_rules,
            author,
            back_pressure,
            network_sender,
        }
    }

    /// Notified when receiving a commit vote message
    pub async fn process_commit_vote(&mut self, commit_vote: &CommitVote) -> anyhow::Result<()> {
        // debug!("process_commit_vote {}", commit_vote);

        if let Some((vecblock, ledger_info)) = self.blocks.as_mut() {
            let li = commit_vote.ledger_info();

            if !(li.commit_info().clone() == vecblock.last().unwrap().block_info()) {
                return Ok(());  // ignore the message
            }

            commit_vote.verify(&self.verifier)?;

            ledger_info.add_signature(commit_vote.author(), commit_vote.signature().clone());
        }

        Ok(())
    }

    pub async fn process_commit_decision(
        &mut self,
        commit_decision: CommitDecision,
    ) -> anyhow::Result<()> {
        // debug!("process_commit_decision {}", commit_decision);

        if let Some((vecblock, leder_info)) = self.blocks.as_ref() {
            let li = commit_decision.ledger_info();

            if !(li.ledger_info().commit_info().clone() == leder_info.ledger_info().commit_info().clone()) {
                return Ok(());  // ignore the message
            }

            commit_decision.verify(&self.verifier)?;

            self.blocks = Some((vecblock.clone(), li.clone()));
        }

        Ok(())
    }

    pub async fn process_block(&mut self) -> anyhow::Result<()> {
        if let Some((vecblock, leder_info)) = self.blocks.as_ref() {
            if leder_info.verify_signatures(&self.verifier).is_ok() {
                commit(
                    &self.execution_proxy,
                    &vecblock,
                    leder_info,
                )
                    .await
                    .expect("Failed to commit the executed blocks.");

                let commit_round = vecblock.last().unwrap().block_info().round();
                self.back_pressure.store(commit_round, Ordering::SeqCst);

                // debug!("Commit End");

                let mut commit_sender = self.network_sender.clone();
                let ledger_info_clone = leder_info.clone();
                tokio::spawn(async move {
                    commit_sender
                        .broadcast(ConsensusMsg::CommitDecisionMsg(Box::new(
                            CommitDecision::new(ledger_info_clone),
                        )))
                        .await;
                });

                self.blocks = None; // prepare for the next batch of blocks
            }
        }

        Ok(())
    }

    pub async fn process_executed_blocks(
        &mut self,
        vecblock: Vec<ExecutedBlock>,
        ledger_info: LedgerInfoWithSignatures,
    ) -> anyhow::Result<()> {
        // debug!(
        //     "process_executed_blocks: vecblock {} ledger_info {}",
        //     vecblock.len(),
        //     ledger_info
        // );

            let new_ledger_info = LedgerInfo::new(
                vecblock.last().unwrap().block_info(),
                ledger_info.ledger_info().consensus_data_hash(),
            );
            // debug!("build new ledger info: {}", new_ledger_info);

            let signature = self
                .safety_rules
                .lock()
                .sign_commit_vote(ledger_info, new_ledger_info.clone())?;

            // debug!("signed");
            // if fails, it needs to resend, otherwise the liveness might compromise.

            let mut commit_sender = self.network_sender.clone();
            let author = self.author;
            let msg = ConsensusMsg::CommitVoteMsg(Box::new(CommitVote::new_with_signature(
                author,
                new_ledger_info.clone(),
                signature.clone(),
            )));

            let mut new_ledger_info_with_sig = LedgerInfoWithSignatures::new(
                new_ledger_info,
                BTreeMap::<AccountAddress, Ed25519Signature>::new(),
            );
            new_ledger_info_with_sig.add_signature(self.author, signature);

            self.blocks = Some((vecblock, new_ledger_info_with_sig));

            // debug!("sent commit vote");
            // note that this message will also reach the node itself

            tokio::spawn(async move {
                commit_sender.broadcast(msg).await;
            });

        Ok(())
    }

    pub async fn start(mut self) {
        loop {
            while let Some(_) = self.blocks.as_ref() {
                select! {
                    msg = self.commit_msg_receiver.select_next_some() => {
                        match msg {
                            ConsensusMsg::CommitVoteMsg(request) => {
                                monitor!(
                                    "process_commit_vote",
                                    report_err!(self.process_commit_vote(&*request).await, "Error in processing commit vote.")
                                );
                            }
                            ConsensusMsg::CommitDecisionMsg(request) => {
                                monitor!(
                                    "process_commit_decision",
                                    report_err!(self.process_commit_decision(*request).await, "Error in processing commit decision.")
                                );
                            }
                            _ => {}
                        };
                    }
                    // TODO: add a timer
                    complete => break,
                }
                report_err!(self.process_block().await, "Error in checking whether self.block is ready to commit.");
            }
            if let Some((vecblocks, ledger_info)) = self.commit_channel_recv.next().await {
                report_err!(self.process_executed_blocks(vecblocks, ledger_info).await, "Error in processing received blocks");
            } else { break; }
        }
    }
}

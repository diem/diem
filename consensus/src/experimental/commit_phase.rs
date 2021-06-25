// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use channel::Receiver;
use diem_infallible::Mutex;
use diem_types::ledger_info::LedgerInfoWithSignatures;
use consensus_types::executed_block::ExecutedBlock;
use futures::{select, StreamExt, SinkExt};
use std::collections::{HashMap, BTreeMap, VecDeque};
use diem_crypto::HashValue;
use crate::state_replication::StateComputer;
use consensus_types::experimental::commit_proposal::CommitProposal;
use consensus_types::experimental::commit_decision::CommitDecision;
use diem_types::account_address::AccountAddress;
use diem_crypto::ed25519::Ed25519Signature;
use crate::state_computer::ExecutionProxy;
use std::sync::Arc;
use crate::round_manager::RoundManager;
use diem_types::validator_verifier::{ValidatorVerifier, VerifyError as ValidatorVerifyError};
use crate::metrics_safety_rules::MetricsSafetyRules;
use consensus_types::common::Author;
use safety_rules::TSafetyRules;
use diem_logger::prelude::*;
use crate::network_interface::ConsensusMsg;
use diem_metrics::{monitor, IntGauge};
use std::collections::hash_map::Entry;
use tokio::runtime::{self, Runtime};
use tokio::sync::Mutex as AsyncMutex;

#[derive(Clone)]
struct CacheItem {
    vecblocks: Vec<ExecutedBlock>,
    ledger_info_sig: LedgerInfoWithSignatures,
}

impl CacheItem {
    pub fn new(
        vecblocks: Vec<ExecutedBlock>,
        ledger_info_sig: LedgerInfoWithSignatures,
    ) -> Self {
        Self {
            vecblocks,
            ledger_info_sig,
        }
    }
}

#[derive(Debug)]
pub enum CommitPhaseChannelMsgWrapper {
    CommitProposal(LedgerInfoWithSignatures),
    CommitDecision(LedgerInfoWithSignatures),
}

pub struct CommitPhase {
    commit_channel_recv: Receiver<(Vec<ExecutedBlock>, LedgerInfoWithSignatures)>,
    execution_proxy: Arc<dyn StateComputer>,
    local_cache: AsyncMutex<HashMap<HashValue, CacheItem>>,
    local_signed_cache: AsyncMutex<HashMap<HashValue, LedgerInfoWithSignatures>>,
    local_queue: AsyncMutex<VecDeque<CacheItem>>,
    commit_msg_sender: channel::Sender<CommitPhaseChannelMsgWrapper>,
    commit_msg_receiver: channel::Receiver<ConsensusMsg>,
    verifier: ValidatorVerifier,
    safety_rules: Arc<Mutex<MetricsSafetyRules>>,
    author: Author,
}

impl CommitPhase {
    pub fn new(
        commit_channel_recv: Receiver<(Vec<ExecutedBlock>, LedgerInfoWithSignatures)>,
        execution_proxy: Arc<dyn StateComputer>,
        commit_msg_sender: channel::Sender<CommitPhaseChannelMsgWrapper>,
        commit_msg_receiver: channel::Receiver<ConsensusMsg>,
        verifier: ValidatorVerifier,
        safety_rules: Arc<Mutex<MetricsSafetyRules>>,
        author: Author,
    ) -> Self {
        Self {
            commit_channel_recv,
            execution_proxy,
            local_cache: AsyncMutex::new(HashMap::<HashValue, CacheItem>::new()),
            local_signed_cache: AsyncMutex::new(HashMap::<HashValue, LedgerInfoWithSignatures>::new()),
            local_queue: AsyncMutex::new(VecDeque::<CacheItem>::new()),
            commit_msg_sender,
            commit_msg_receiver,
            verifier,
            safety_rules,
            author,
        }
    }

    /// Notified when receiving a commit proposal message
    pub async fn process_commit_proposal(&mut self, commit_proposal: &CommitProposal) {
        // verify the signature
        let li = commit_proposal.ledger_info();
        match self.verifier
            .verify(
                commit_proposal.author(),
                li,
                commit_proposal.signature()
            ) {
            Ok(_) => {
                // TODO: should we change chash to li.commit_info.executed_state_id?
                let chash = li.consensus_data_hash();
                let mut locked_local_signed_cache = self.local_signed_cache.lock().await; //.unwrap();
                match locked_local_signed_cache.get(&chash) {
                    Some(_) => {
                        // ignore the proposal as we have already done this.
                    }
                    None => {
                        let mut locked_local_cache = self.local_cache.lock().await; //.unwrap();
                        match locked_local_cache.entry(chash) {
                            Entry::Occupied(mut cio) => {
                                let (verification, li_copy) = {
                                    let ci = cio.get_mut();
                                    ci.ledger_info_sig.add_signature(commit_proposal.author(), commit_proposal.signature().clone());
                                    (self.verify_signature_tree(&ci.ledger_info_sig), ci.ledger_info_sig.clone())
                                };
                                match verification {
                                    Ok(_) => {
                                        cio.remove_entry();
                                        locked_local_signed_cache.insert(chash, li_copy);
                                    }
                                    Err(_) => {
                                        // TODO: panic?
                                    }
                                }
                            }
                            Entry::Vacant(_) => {
                                let mut ci = CacheItem::new(
                                    Vec::<ExecutedBlock>::new(),
                                    LedgerInfoWithSignatures::new(
                                        li.clone(),
                                        BTreeMap::<AccountAddress, Ed25519Signature>::new()
                                    )
                                );
                                let signature =
                                    {
                                        self.safety_rules.lock()
                                        .sign_commit_proposal(ci.ledger_info_sig.clone(), &self.verifier)
                                        .unwrap()
                                    };
                                ci.ledger_info_sig.add_signature(self.author, signature);
                                ci.ledger_info_sig.add_signature(commit_proposal.author(), commit_proposal.signature().clone());
                                if self.verify_signature_tree(&ci.ledger_info_sig).is_ok() {
                                    locked_local_signed_cache.insert(chash, ci.ledger_info_sig.clone());
                                } else {
                                    locked_local_cache.insert(chash, ci.clone());
                                }
                            }
                        }
                    }
                }
            }
            _ => {
                // ignore the proposal
                // Do we need to panic about this?
            }
        }
    }

    pub fn verify_signature_tree(&self, ledger_info: &LedgerInfoWithSignatures) -> Result<(), ValidatorVerifyError> {
        self.verifier
            .verify_aggregated_struct_signature(
                ledger_info.ledger_info(),
                ledger_info.signatures()
            )
    }

    pub async fn process_commit_decision(&self, commit_decision: CommitDecision) {
        let li = commit_decision.ledger_info();
        let chash = li.ledger_info().consensus_data_hash();
        let locked_local_signed_cache = &mut *self.local_signed_cache.lock().await;
        if locked_local_signed_cache.contains_key(&chash) {
            return // ignore the message
        }
        match self.verify_signature_tree(commit_decision.ledger_info()) {
            Ok(_) => {
                locked_local_signed_cache.insert(chash, commit_decision.ledger_info().clone());
            }
            _ => {
                // ignore the decision
                // Panic?
            }
        };
    }

    pub fn commit(&self, vecblock: Vec<ExecutedBlock>, ledger_info: LedgerInfoWithSignatures){
        // have to maintain the order.
        self.execution_proxy.commit(
            &vecblock.into_iter()
                .map(|eb|Arc::new(eb))
                .collect::<Vec<Arc<ExecutedBlock>>>(),
            ledger_info,
        );
    }


    pub async fn start(mut self){
        loop {
            select! {
                (vecblock, ledger_info) = self.commit_channel_recv.select_next_some() => {
                    let chash = ledger_info.ledger_info().consensus_data_hash();
                    let mut locked_signed_cache = self.local_signed_cache.lock().await;
                    let runtime = runtime::Handle::current();
                    match locked_signed_cache.entry(chash) {
                        Entry::Occupied(mut lio) => {
                            self.commit(vecblock, ledger_info); // commit right away
                            self.commit_msg_sender.send(
                                    CommitPhaseChannelMsgWrapper::CommitDecision(lio.get_mut().clone())
                                ).await.map_err(|e|{
                                //TODO: error handling
                            });
                            lio.remove_entry();
                        },
                        Entry::Vacant(_) => {
                            // sign the ledgerinfo and broadcast the signature
                            self.commit_msg_sender.send(
                                    CommitPhaseChannelMsgWrapper::CommitProposal(ledger_info.clone())
                                ).await.map_err(|e| {
                                // TODO: error handling
                            });

                            // check local cache
                            let mut locked_local_queue = self.local_queue.lock().await;
                            locked_local_queue.push_back(CacheItem::new(
                                vecblock,
                                ledger_info,
                            ));
                            loop {
                                let front = locked_local_queue.front().unwrap();
                                let front_chash = front.ledger_info_sig.ledger_info().consensus_data_hash();
                                match locked_signed_cache.entry(front_chash) {
                                    Entry::Occupied(mut lio) => {
                                        // cancel out an item from signed_cache and an item from local_queue
                                        self.commit(front.vecblocks.clone(), front.ledger_info_sig.clone());
                                        self.commit_msg_sender
                                            .send(
                                                CommitPhaseChannelMsgWrapper::CommitDecision(lio.get_mut().clone())
                                            ).await.map_err(|e| {
                                                // TODO: error handling
                                        });
                                        lio.remove_entry();
                                        locked_local_queue.pop_front();
                                    }
                                    Entry::Vacant(_) => {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                msg = self.commit_msg_receiver.select_next_some() => {
                    match msg {
                        ConsensusMsg::CommitProposalMsg(request) => {
                            monitor!(
                                "process_commit_proposal",
                                self.process_commit_proposal(&*request).await
                            );
                        }
                        ConsensusMsg::CommitDecisionMsg(request) => {
                            monitor!(
                                "process_commit_decision",
                                self.process_commit_decision(*request).await
                            );
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

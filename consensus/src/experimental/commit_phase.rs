// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use channel::Receiver;
use diem_infallible::Mutex;
use diem_logger::prelude::*;
use diem_metrics::monitor;
use diem_types::ledger_info::{LedgerInfoWithSignatures, LedgerInfo};
use execution_correctness::ExecutionCorrectness;
use consensus_types::executed_block::ExecutedBlock;
use state_sync::client::StateSyncClient;
use futures::{StreamExt, SinkExt};
use std::collections::{HashMap, HashSet, BTreeMap, VecDeque};
use diem_crypto::{HashValue, Signature};
use consensus_types::experimental::commit_proposal::CommitProposal;
use consensus_types::common::Author;
use diem_types::validator_signer::ValidatorSigner;
use crate::network_interface::{ConsensusNetworkSender, ConsensusMsg};
use crate::network::NetworkSender;
use consensus_types::experimental::commit_decision::CommitDecision;
use diem_types::account_address::AccountAddress;
use diem_crypto::ed25519::Ed25519Signature;
use crate::state_computer::ExecutionProxy;
use std::sync::Arc;
use crate::round_manager::RoundManager;
use safety_rules::TSafetyRules;

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

pub enum CommitPhaseChannelMsgWrapper {
    CommitProposal(LedgerInfoWithSignatures),
    CommitDecision(LedgerInfoWithSignatures),
}

pub struct CommitPhase {
    commit_channel_recv: Receiver<(Vec<ExecutedBlock>, LedgerInfoWithSignatures)>,
    execution_proxy: Arc<ExecutionProxy>,
    local_cache: Mutex<HashMap<HashValue, CacheItem>>,
    local_signed_cache: Mutex<HashMap<HashValue, LedgerInfoWithSignatures>>,
    local_queue: Mutex<VecDeque<CacheItem>>,
    commit_msg_sender: channel::Sender<CommitPhaseChannelMsgWrapper>,
}

impl CommitPhase {
    pub fn new(
        commit_channel_recv: Receiver<(Vec<ExecutedBlock>, LedgerInfoWithSignatures)>,
        execution_proxy: Arc<ExecutionProxy>,
        commit_msg_sender: channel::Sender<CommitPhaseChannelMsgWrapper>,
    ) -> Self {
        Self {
            commit_channel_recv,
            execution_proxy,
            local_cache: Mutex::new(HashMap::<HashValue, CacheItem>::new()),
            local_signed_cache: Mutex::new(HashMap::<HashValue, LedgerInfoWithSignatures>::new()),
            local_queue: Muex::new(VecDeque::<CacheItem>::new()),
            commit_msg_sender,
        }
    }

    /// Notified when receiving a commit proposal message
    pub async fn process_commit_proposal(self, commit_proposal: CommitProposal, round_manager_handle: &mut RoundManager) {
        // verify the signature
        let li = commit_proposal.ledger_info();
        match round_manager_handle
            .verify_signature(
                commit_proposal.author(),
                li,
                commit_proposal.signature()
            ) {
            Ok(_) => {
                // TODO: should we change chash to li.commit_info.executed_state_id?
                let chash = li.consensus_data_hash();
                let locked_local_signed_cache = self.local_signed_cache.lock().unwrap();
                match locked_local_signed_cache.get(&chash) {
                    Some(_) => {
                        // ignore the proposal as we have already done this.
                    }
                    None => {
                        let locked_local_cache = self.local_cache.lock().unwrap();
                        match locked_local_cache.get(&chash) {
                            Some(&ci) => {
                                ci.ledger_info_sig.insert(commit_proposal.author(), commit_proposal.signature());
                                if round_manager_handle.verify_signature_tree(ci.ledger_info_sig()) {
                                    locked_local_cache.remove(chash);
                                    locked_local_signed_cache.insert(chash);
                                }
                            }
                            None => {
                                let mut ci = CacheItem::new(
                                    Vec::<ExecutedBlock>::new(),
                                    LedgerInfoWithSignatures::new(
                                        li.clone(),
                                        BTreeMap::<AccountAddress, Ed25519Signature>::new()
                                    )
                                );
                                let signature = round_manager_handle
                                    .sign_commit_proposal(ci.ledger_info_sig);
                                ci.ledger_info_sig.insert(round_manager_handle.author(), signature);
                                ci.ledger_info_sig.insert(commit_proposal.author(), commit_proposal.signature());
                                if round_manager_handle.verify_signature_tree(ci.ledger_info_sig()) {
                                    locked_local_signed_cache.insert(chash, ci.ledger_info_sig());
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
                /// Do we need to panic about this?
            }
        }
    }

    pub async fn process_commit_decision(self, commit_decision: CommitDecision, round_manager_handle: &mut RoundManager) {
        let li = commit_decision.ledger_info();
        let chash = li.consensus_data_hash();
        let locked_local_signed_cache = self.local_signed_cache.lock().unwrap();
        if locked_local_signed_cache.contains_key(&chash) {
            return // ignore the message
        }
        match round_manager_handle.verify_signature_tree(commit_decision.ledger_info()) {
            Ok(_) => {
                locked_local_signed_cache.insert(chash, commit_decision.ledger_info())
            }
            _ => {
                // ignore the decision
                /// Panic?
            }
        }
    }

    pub fn commit(mut self, vecblock: Vec<ExecutedBlock>, ledger_info: LedgerInfoWithSignatures){
        // have to maintain the order.
        self.execution_proxy.unwrap().commit(
            vecblock.into_iter().map(|eb|Arc::new(eb)).collect(),
            ledger_info,
        );
    }


    pub async fn start(mut self){
        while let Some((vecblock, ledger_info)) = self.commit_channel_recv.next().await {
            let chash = ledger_info.ledger_info().consensus_data_hash();
            let mut locked_signed_cache = self.local_signed_cache.lock();
            match locked_signed_cache.get(&chash) {
                Some(&li) => {
                    self.commit(vecblock, ledger_info); // commit right away
                    self.commit_msg_sender.send(
                        CommitPhaseChannelMsgWrapper::CommitDecision(li)
                    );
                    locked_signed_cache.remove(&chash);
                },
                None => {
                    // sign the ledgerinfo and broadcast the signature
                    self.commit_msg_sender.send(
                        CommitPhaseChannelMsgWrapper::CommitProposal(ledger_info.clone())
                    );

                    // check local cache
                    let mut locked_local_queue = self.local_queue.lock();
                    locked_local_queue.push_back(CacheItem(
                        vecblock,
                        ledger_info,
                    ));
                    loop {
                        let &front = locked_local_queue.front().unwrap();
                        let front_chash = front.ledger_info_sig.ledger_info().consensus_data_hash();
                        match locked_signed_cache.get(&front_chash) {
                            Some(&li) => {
                                // cancel out an item from signed_cache and an item from local_queue
                                self.commit(front.vecblocks, front.ledger_info_sig);
                                self.commit_msg_sender.send(
                                    CommitPhaseChannelMsgWrapper::CommitDecision(li)
                                );
                                locked_signed_cache.remove(&cache);
                                locked_local_queue.pop_front();
                            }
                            None => {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}

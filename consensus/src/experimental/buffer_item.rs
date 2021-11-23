// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

use anyhow::anyhow;
use itertools::zip_eq;

use consensus_types::{
    common::Author, executed_block::ExecutedBlock, experimental::commit_vote::CommitVote,
};
use diem_crypto::ed25519::Ed25519Signature;
use diem_logger::prelude::*;
use diem_types::{
    account_address::AccountAddress,
    block_info::BlockInfo,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    validator_verifier::ValidatorVerifier,
};

use crate::{experimental::hashable::Hashable, state_replication::StateComputerCommitCallBackType};
use diem_crypto::HashValue;

fn generate_commit_proof(
    commit_info: &BlockInfo,
    ordered_proof: &LedgerInfoWithSignatures,
) -> LedgerInfo {
    LedgerInfo::new(
        commit_info.clone(),
        ordered_proof.ledger_info().consensus_data_hash(),
    )
}

fn aggregate_ledger_info(
    commit_ledger_info: &LedgerInfo,
    unverified_signatures: BTreeMap<AccountAddress, Ed25519Signature>,
    validator: &ValidatorVerifier,
) -> LedgerInfoWithSignatures {
    let valid_sigs = unverified_signatures
        .into_iter()
        .filter(|(author, sig)| validator.verify(*author, commit_ledger_info, sig).is_ok())
        .collect();

    LedgerInfoWithSignatures::new(commit_ledger_info.clone(), valid_sigs)
}

// we differentiate buffer items at different stages
// for better code readability
pub struct OrderedItem {
    pub unverified_signatures: BTreeMap<AccountAddress, Ed25519Signature>,
    pub callback: StateComputerCommitCallBackType,
    pub ordered_blocks: Vec<ExecutedBlock>,
    pub ordered_proof: LedgerInfoWithSignatures,
}

pub struct ExecutedItem {
    pub executed_blocks: Vec<ExecutedBlock>,
    pub commit_proof: LedgerInfoWithSignatures,
    pub callback: StateComputerCommitCallBackType,
    pub commit_info: BlockInfo,
    pub ordered_proof: LedgerInfoWithSignatures,
}

pub struct SignedItem {
    pub executed_blocks: Vec<ExecutedBlock>,
    pub commit_proof: LedgerInfoWithSignatures,
    pub callback: StateComputerCommitCallBackType,
    pub commit_vote: CommitVote,
}

pub struct AggregatedItem {
    pub executed_blocks: Vec<ExecutedBlock>,
    pub commit_proof: LedgerInfoWithSignatures,
    pub callback: StateComputerCommitCallBackType,
}

pub enum BufferItem {
    Ordered(Box<OrderedItem>),
    Executed(Box<ExecutedItem>),
    Signed(Box<SignedItem>),
    Aggregated(Box<AggregatedItem>),
}

impl Hashable for BufferItem {
    fn hash(&self) -> HashValue {
        self.block_id()
    }
}

impl BufferItem {
    pub fn new_ordered(
        ordered_blocks: Vec<ExecutedBlock>,
        ordered_proof: LedgerInfoWithSignatures,
        callback: StateComputerCommitCallBackType,
    ) -> Self {
        Self::Ordered(Box::new(OrderedItem {
            unverified_signatures: BTreeMap::new(),
            callback,
            ordered_blocks,
            ordered_proof,
        }))
    }

    // pipeline functions
    pub fn advance_to_executed_or_aggregated(
        self,
        executed_blocks: Vec<ExecutedBlock>,
        validator: &ValidatorVerifier,
    ) -> Self {
        match self {
            Self::Ordered(ordered_item) => {
                let OrderedItem {
                    ordered_blocks,
                    unverified_signatures,
                    callback,
                    ordered_proof,
                } = *ordered_item;
                for (b1, b2) in zip_eq(ordered_blocks.iter(), executed_blocks.iter()) {
                    assert_eq!(b1.id(), b2.id());
                }
                let mut commit_info = executed_blocks.last().unwrap().block_info();
                // Since proposal_generator is not aware of reconfiguration any more, the suffix blocks
                // will not have the same timestamp as the reconfig block which violates the invariant
                // that block.timestamp == state.timestamp because no txn is executed in suffix blocks.
                // We change the timestamp field of the block info to maintain the invariant.
                // If the executed blocks are b1 <- b2 <- r <- b4 <- b5 with timestamp t1..t5
                // we replace t5 with t3 (from reconfiguration block) since that's the last timestamp
                // being updated on-chain.
                let reconfig_ts = executed_blocks
                    .iter()
                    .find(|b| b.block_info().has_reconfiguration())
                    .map(|b| b.timestamp_usecs())
                    .filter(|ts| *ts != commit_info.timestamp_usecs());
                if let Some(ts) = reconfig_ts {
                    assert!(executed_blocks.last().unwrap().is_reconfiguration_suffix());
                    debug!(
                        "Reconfig happens, change timestamp of {} to {}",
                        commit_info, ts
                    );
                    commit_info.change_timestamp(ts);
                }
                let commit_proof = aggregate_ledger_info(
                    &generate_commit_proof(&commit_info, &ordered_proof),
                    unverified_signatures,
                    validator,
                );
                if commit_proof.check_voting_power(validator).is_ok() {
                    debug!(
                        "{} advance to aggregated from ordered",
                        commit_proof.commit_info()
                    );
                    Self::Aggregated(Box::new(AggregatedItem {
                        executed_blocks,
                        commit_proof,
                        callback,
                    }))
                } else {
                    debug!(
                        "{} advance to executed from ordered",
                        commit_proof.commit_info()
                    );
                    Self::Executed(Box::new(ExecutedItem {
                        executed_blocks,
                        commit_proof,
                        callback,
                        commit_info,
                        ordered_proof,
                    }))
                }
            }
            _ => {
                panic!("Only ordered blocks can advance to executed blocks.")
            }
        }
    }

    pub fn advance_to_signed(self, author: Author, signature: Ed25519Signature) -> Self {
        match self {
            Self::Executed(executed_item) => {
                let ExecutedItem {
                    executed_blocks,
                    callback,
                    commit_proof,
                    ..
                } = *executed_item;

                // we don't add the signature here, it'll be added when receiving the commit vote from self
                let commit_vote = CommitVote::new_with_signature(
                    author,
                    commit_proof.ledger_info().clone(),
                    signature,
                );
                debug!("{} advance to signed", commit_proof.commit_info());

                Self::Signed(Box::new(SignedItem {
                    executed_blocks,
                    callback,
                    commit_proof,
                    commit_vote,
                }))
            }
            _ => {
                panic!("Only executed buffer items can advance to signed blocks.")
            }
        }
    }

    /// this function assumes block id matches and the validity of ledger_info and that it has the voting power
    /// it returns an updated item
    pub fn try_advance_to_aggregated_with_ledger_info(
        self,
        commit_proof: LedgerInfoWithSignatures,
    ) -> Self {
        match self {
            Self::Signed(signed_item) => {
                let SignedItem {
                    executed_blocks,
                    callback,
                    commit_proof: local_commit_proof,
                    ..
                } = *signed_item;
                assert_eq!(local_commit_proof.commit_info(), commit_proof.commit_info(),);
                debug!(
                    "{} advance to aggregated with commit decision",
                    commit_proof.commit_info()
                );
                Self::Aggregated(Box::new(AggregatedItem {
                    executed_blocks,
                    callback,
                    commit_proof,
                }))
            }
            Self::Executed(executed_item) => {
                let ExecutedItem {
                    executed_blocks,
                    callback,
                    commit_info,
                    ..
                } = *executed_item;
                assert_eq!(commit_info, *commit_proof.commit_info());
                debug!(
                    "{} advance to aggregated with commit decision",
                    commit_proof.commit_info()
                );
                Self::Aggregated(Box::new(AggregatedItem {
                    executed_blocks,
                    callback,
                    commit_proof,
                }))
            }
            Self::Ordered(ordered_item) => {
                let ordered = *ordered_item;
                assert!(ordered
                    .ordered_proof
                    .commit_info()
                    .match_ordered_only(commit_proof.commit_info()));
                // can't aggregate it without execution, only store the signatures
                debug!(
                    "{} received commit decision in ordered stage",
                    commit_proof.commit_info()
                );
                Self::Ordered(Box::new(OrderedItem {
                    unverified_signatures: commit_proof.signatures().clone(),
                    ..ordered
                }))
            }
            Self::Aggregated(_) => {
                unreachable!("Found aggregated buffer item but any aggregated buffer item should get dequeued right away.");
            }
        }
    }

    pub fn try_advance_to_aggregated(self, validator: &ValidatorVerifier) -> Self {
        match self {
            Self::Signed(signed_item) => {
                if signed_item
                    .commit_proof
                    .check_voting_power(validator)
                    .is_ok()
                {
                    Self::Aggregated(Box::new(AggregatedItem {
                        executed_blocks: signed_item.executed_blocks,
                        commit_proof: signed_item.commit_proof,
                        callback: signed_item.callback,
                    }))
                } else {
                    Self::Signed(signed_item)
                }
            }
            Self::Executed(executed_item) => {
                if executed_item
                    .commit_proof
                    .check_voting_power(validator)
                    .is_ok()
                {
                    Self::Aggregated(Box::new(AggregatedItem {
                        executed_blocks: executed_item.executed_blocks,
                        commit_proof: executed_item.commit_proof,
                        callback: executed_item.callback,
                    }))
                } else {
                    Self::Executed(executed_item)
                }
            }
            _ => self,
        }
    }

    // generic functions
    pub fn get_blocks(&self) -> &Vec<ExecutedBlock> {
        match self {
            Self::Ordered(ordered) => &ordered.ordered_blocks,
            Self::Executed(executed) => &executed.executed_blocks,
            Self::Signed(signed) => &signed.executed_blocks,
            Self::Aggregated(aggregated) => &aggregated.executed_blocks,
        }
    }

    pub fn block_id(&self) -> HashValue {
        self.get_blocks().last().unwrap().id()
    }

    pub fn add_signature_if_matched(&mut self, vote: CommitVote) -> anyhow::Result<()> {
        let target_commit_info = vote.commit_info();
        let author = vote.author();
        let signature = vote.signature().clone();
        match self {
            Self::Ordered(ordered) => {
                if ordered
                    .ordered_proof
                    .commit_info()
                    .match_ordered_only(target_commit_info)
                {
                    // we optimistically assume the vote will be valid in the future.
                    // when advancing to executed item, we will check if the sigs are valid.
                    // each author at most stores a single sig for each item,
                    // so an adversary will not be able to flood our memory.
                    ordered.unverified_signatures.insert(author, signature);
                    return Ok(());
                }
            }
            Self::Executed(executed) => {
                if executed.commit_info == *target_commit_info {
                    executed.commit_proof.add_signature(author, signature);
                    return Ok(());
                }
            }
            Self::Signed(signed) => {
                if signed.commit_proof.commit_info() == target_commit_info {
                    signed.commit_proof.add_signature(author, signature);
                    return Ok(());
                }
            }
            Self::Aggregated(aggregated) => {
                // we do not need to do anything for aggregated
                // but return true is helpful to stop the outer loop early
                if aggregated.commit_proof.commit_info() == target_commit_info {
                    return Ok(());
                }
            }
        }
        Err(anyhow!("Inconsistent commit info."))
    }

    pub fn is_ordered(&self) -> bool {
        matches!(self, Self::Ordered(_))
    }

    pub fn is_executed(&self) -> bool {
        matches!(self, Self::Executed(_))
    }

    pub fn is_signed(&self) -> bool {
        matches!(self, Self::Signed(_))
    }

    pub fn is_aggregated(&self) -> bool {
        matches!(self, Self::Aggregated(_))
    }

    pub fn unwrap_signed_ref(&self) -> &SignedItem {
        match self {
            BufferItem::Signed(item) => item.as_ref(),
            _ => panic!("Not signed item"),
        }
    }

    pub fn unwrap_executed_ref(&self) -> &ExecutedItem {
        match self {
            BufferItem::Executed(item) => item.as_ref(),
            _ => panic!("Not executed item"),
        }
    }

    pub fn unwrap_aggregated(self) -> AggregatedItem {
        match self {
            BufferItem::Aggregated(item) => *item,
            _ => panic!("Not aggregated item"),
        }
    }
}

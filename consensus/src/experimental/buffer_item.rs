// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::state_replication::StateComputerCommitCallBackType;
use consensus_types::{common::Author, executed_block::ExecutedBlock};
use diem_crypto::ed25519::Ed25519Signature;
use diem_types::{
    account_address::AccountAddress,
    block_info::BlockInfo,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    validator_verifier::ValidatorVerifier,
};
use itertools::zip;
use std::collections::BTreeMap;

// we differentiate buffer items at different stages
// for better code readability
pub struct OrderedBufferItem {
    pub pending_votes: BTreeMap<AccountAddress, Ed25519Signature>,
    pub callback: StateComputerCommitCallBackType,
    pub ordered_blocks: Vec<ExecutedBlock>,
    pub ordered_proof: LedgerInfoWithSignatures,
}

pub struct ExecutedBufferItem {
    pub executed_blocks: Vec<ExecutedBlock>,
    pub pending_votes: BTreeMap<AccountAddress, Ed25519Signature>,
    pub callback: StateComputerCommitCallBackType,
    pub commit_info: BlockInfo,
    pub ordered_proof: LedgerInfoWithSignatures,
}

pub struct SignedBufferItem {
    pub executed_blocks: Vec<ExecutedBlock>,
    pub commit_proof: LedgerInfoWithSignatures,
    pub callback: StateComputerCommitCallBackType,
}

pub struct AggregatedBufferItem {
    pub executed_blocks: Vec<ExecutedBlock>,
    pub aggregated_proof: LedgerInfoWithSignatures,
    pub callback: StateComputerCommitCallBackType,
}

pub enum BufferItem {
    Ordered(Box<OrderedBufferItem>),
    Executed(Box<ExecutedBufferItem>),
    Signed(Box<SignedBufferItem>),
    Aggregated(Box<AggregatedBufferItem>),
}

impl BufferItem {
    pub fn new_ordered(
        ordered_blocks: Vec<ExecutedBlock>,
        ordered_proof: LedgerInfoWithSignatures,
        callback: StateComputerCommitCallBackType,
    ) -> Self {
        Self::Ordered(Box::new(OrderedBufferItem {
            pending_votes: BTreeMap::<AccountAddress, Ed25519Signature>::new(),
            callback,
            ordered_blocks,
            ordered_proof,
        }))
    }

    // pipeline functions
    pub fn advance_to_executed(self, executed_blocks: Vec<ExecutedBlock>) -> Self {
        match self {
            Self::Ordered(ordered_item_box) => {
                let ordered_item = *ordered_item_box;
                assert_eq!(ordered_item.ordered_blocks.len(), executed_blocks.len());
                for (b1, b2) in zip(ordered_item.ordered_blocks.iter(), executed_blocks.iter()) {
                    assert_eq!(b1.id(), b2.id());
                }
                let commit_info = executed_blocks.last().unwrap().block_info();
                Self::Executed(Box::new(ExecutedBufferItem {
                    executed_blocks,
                    pending_votes: ordered_item.pending_votes,
                    callback: ordered_item.callback,
                    commit_info,
                    ordered_proof: ordered_item.ordered_proof,
                }))
            }
            _ => {
                panic!("Only ordered blocks can advance to executed blocks.")
            }
        }
    }

    pub fn advance_to_signed(
        self,
        author: Author,
        signature: Ed25519Signature,
        verifier: &ValidatorVerifier,
    ) -> Self {
        match self {
            Self::Executed(executed_item_box) => {
                let executed_item = *executed_item_box;
                let mut valid_sigs = BTreeMap::<AccountAddress, Ed25519Signature>::new();
                valid_sigs.insert(author, signature);

                let commit_ledger_info = LedgerInfo::new(
                    executed_item.commit_info,
                    executed_item
                        .ordered_proof
                        .ledger_info()
                        .consensus_data_hash(),
                );

                for (author, sig) in executed_item.pending_votes.iter() {
                    if verifier.verify(*author, &commit_ledger_info, sig).is_ok() {
                        valid_sigs.insert(*author, sig.clone());
                    }
                }

                let commit_ledger_info_with_sigs =
                    LedgerInfoWithSignatures::new(commit_ledger_info, valid_sigs);

                Self::Signed(Box::new(SignedBufferItem {
                    executed_blocks: executed_item.executed_blocks,
                    callback: executed_item.callback,
                    commit_proof: commit_ledger_info_with_sigs,
                }))
            }
            _ => {
                panic!("Only executed buffer items can advance to signed blocks.")
            }
        }
    }

    pub fn try_advance_to_aggregated(self, validator: &ValidatorVerifier) -> Self {
        match self {
            Self::Signed(signed_item_box) => {
                let signed_item = *signed_item_box;
                if signed_item
                    .commit_proof
                    .check_voting_power(validator)
                    .is_ok()
                {
                    Self::Aggregated(Box::new(AggregatedBufferItem {
                        executed_blocks: signed_item.executed_blocks,
                        aggregated_proof: signed_item.commit_proof,
                        callback: signed_item.callback,
                    }))
                } else {
                    Self::Signed(Box::new(signed_item))
                }
            }
            _ => {
                panic!("Only signed buffer items can advance to aggregated blocks.")
            }
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

    pub fn get_commit_info(&self) -> &BlockInfo {
        match self {
            Self::Ordered(_) => {
                panic!("Ordered buffer item does not contain commit info");
            }
            Self::Executed(executed) => &executed.commit_info,
            Self::Signed(signed) => signed.commit_proof.ledger_info().commit_info(),
            Self::Aggregated(aggregated) => aggregated.aggregated_proof.ledger_info().commit_info(),
        }
    }
}

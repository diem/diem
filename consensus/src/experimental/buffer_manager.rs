// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    experimental::{
        execution_phase::{ExecutionRequest, ExecutionResponse},
        linkedlist::{Link, List},
        persisting_phase::{PersistingRequest, PersistingResponse},
        signing_phase::{SigningRequest, SigningResponse},
    },
    network::NetworkSender,
    round_manager::VerifiedEvent,
    state_replication::StateComputerCommitCallBackType,
};
use consensus_types::{common::Author, executed_block::ExecutedBlock};
use diem_crypto::ed25519::Ed25519Signature;
use diem_types::{
    account_address::AccountAddress, ledger_info::LedgerInfoWithSignatures,
    validator_verifier::ValidatorVerifier,
};
use futures::channel::{
    mpsc::{UnboundedReceiver, UnboundedSender},
    oneshot,
};
use std::{collections::BTreeMap, sync::Arc};

pub type ResetAck = ();
pub fn reset_ack_new() -> ResetAck {}

pub struct ResetRequest {
    tx: oneshot::Sender<ResetAck>,
    reconfig: bool,
}

pub struct OrderedBlocks {
    pub blocks: Vec<ExecutedBlock>,
    pub finality_proof: LedgerInfoWithSignatures,
    pub callback: StateComputerCommitCallBackType,
}

/*
 *                       LedgerInfo Buffer
 *
 *                      ┌────────────┐                    ┌────────────┐
 *                      │ LedgerInfo │                    │ LedgerInfo │
 *                  ────► BufferItem ├────────────────────► BufferItem ├───►
 *                      └─────┬──────┘                    └─────┬──────┘
 *                            │ Link                            │ Link
 *                            │                                 │
 *     ┌────────────┐   ┌─────▼──────┐   ┌────────────┐   ┌─────▼──────┐
 *     │ BufferItem │   │ BufferItem │   │ BufferItem │   │ BufferItem │
 * ────►            ├───►            ├───►            ├───►            ├───►
 *     │ Block      │   │ LedgerInfo │   │ Block      │   │ LedgerInfo │
 *     └────────────┘   └────────────┘   └────────────┘   └────────────┘
 *
 *      Buffer
 */

pub struct FinalityProofItem(
    LedgerInfoWithSignatures,
    BTreeMap<AccountAddress, Ed25519Signature>,
    StateComputerCommitCallBackType,
);

pub enum BufferItem {
    Block(Arc<ExecutedBlock>), // TODO: remove Arc
    // the second item is to store a signature received from a commit vote
    // before the prefix of blocks have been executed
    FinalityProof(Box<FinalityProofItem>), // use box to avoid large size difference in enum
}

pub struct LedgerInfoBufferItem {
    // signatures are collected in ledger_info
    pub commit_ledger_info: LedgerInfoWithSignatures, // duplicating for efficiency
    // jump back to BufferItem cursor
    pub link: Link<BufferItem>,
}

pub type BufferItemRootType = Link<BufferItem>;
pub type LedgerInfoRootType = Link<LedgerInfoBufferItem>;
pub type Sender<T> = UnboundedSender<T>;
pub type Receiver<T> = UnboundedReceiver<T>;

/// StateManager handles the states of ordered blocks and
/// interacts with the execution phase, the signing phase, and
/// the persisting phase.
pub struct StateManager {
    author: Author,

    buffer: List<BufferItem>,
    li_buffer: List<LedgerInfoBufferItem>,
    // the second item is for updating aggregation_root and jumping between LI's easily
    execution_root: BufferItemRootType,
    execution_phase_tx: Sender<ExecutionRequest>,
    execution_phase_rx: Receiver<ExecutionResponse>,

    signing_root: LedgerInfoRootType,
    signing_phase_tx: Sender<SigningRequest>,
    signing_phase_rx: Receiver<SigningResponse>,

    aggregation_root: BufferItemRootType,
    commit_msg_tx: NetworkSender,
    commit_msg_rx: channel::diem_channel::Receiver<AccountAddress, VerifiedEvent>,

    persisting_phase_tx: Sender<PersistingRequest>,
    persisting_phase_rx: Receiver<PersistingResponse>,

    block_rx: UnboundedReceiver<OrderedBlocks>,
    reset_rx: UnboundedReceiver<ResetRequest>,
    end_epoch: bool,

    verifier: ValidatorVerifier,
}

impl StateManager {
    pub fn new(
        author: Author,
        execution_phase_tx: Sender<ExecutionRequest>,
        execution_phase_rx: Receiver<ExecutionResponse>,
        signing_phase_tx: Sender<SigningRequest>,
        signing_phase_rx: Receiver<SigningResponse>,
        commit_msg_tx: NetworkSender,
        commit_msg_rx: channel::diem_channel::Receiver<AccountAddress, VerifiedEvent>,
        persisting_phase_tx: Sender<PersistingRequest>,
        persisting_phase_rx: Receiver<PersistingResponse>,
        block_rx: UnboundedReceiver<OrderedBlocks>,
        reset_rx: UnboundedReceiver<ResetRequest>,
        verifier: ValidatorVerifier,
    ) -> Self {
        let buffer = List::<BufferItem>::new();
        let li_buffer = List::<LedgerInfoBufferItem>::new();

        // point the roots to the head
        let execution_root = buffer.head.as_ref().cloned();
        let signing_root = li_buffer.head.as_ref().cloned();
        let aggregation_root = buffer.head.as_ref().cloned();

        Self {
            author,

            buffer,
            li_buffer,

            execution_root,
            execution_phase_tx,
            execution_phase_rx,

            signing_root,
            signing_phase_tx,
            signing_phase_rx,

            aggregation_root,
            commit_msg_tx,
            commit_msg_rx,

            persisting_phase_tx,
            persisting_phase_rx,

            block_rx,
            reset_rx,
            end_epoch: false,

            verifier,
        }
    }

    async fn start(self) {

        // loop receving new blocks or reset
        // while !self.end_epoch {

        // select from all rx channels,
        // if new from block_rx, push to buffer
        // if new from reset_rx, make a mark that stops all the following ops
        // if new from execution_phase_rx,
        //   if execution failure, send all the blocks to execution_phase.
        //   Otherwise,
        //     update execution_root and send the blocks from execution_root to end to execution_phase
        // if new from signing_phase_rx,
        //   update sigining_root and send the blocks from signing_root to execution_root to signing_phase
        // if new from commit_msg_rx,
        //   collect sig and update the sigs
        //   if aggregated,
        //     update aggregation_root
        // if new from persisting_phase_rx,
        //   pop blocks from buffer, and continue to post-committing ops
        //   send the blocks from aggregation_root to the end to persisting_phase

        // if not reset, retry sending the commit_vote msg via commit_msg_tx
        // }
    }
}

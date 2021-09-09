// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    experimental::{
        buffer_item::BufferItem,
        execution_phase::{ExecutionRequest, ExecutionResponse},
        linkedlist::{get_elem, get_next, link_eq, set_elem, take_elem, Link, List},
        persisting_phase::{PersistingRequest, PersistingResponse},
        signing_phase::{SigningRequest, SigningResponse},
    },
    network::NetworkSender,
    round_manager::VerifiedEvent,
    state_replication::StateComputerCommitCallBackType,
};
use consensus_types::{common::Author, executed_block::ExecutedBlock};
use diem_types::{
    account_address::AccountAddress,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    validator_verifier::ValidatorVerifier,
};
use futures::{
    channel::{
        mpsc::{UnboundedReceiver, UnboundedSender},
        oneshot,
    },
    SinkExt,
};

pub type SyncAck = ();
pub fn sync_ack_new() -> SyncAck {}

pub struct SyncRequest {
    tx: oneshot::Sender<SyncAck>,
    ledger_info: LedgerInfo,
    reconfig: bool,
}

pub struct OrderedBlocks {
    pub ordered_blocks: Vec<ExecutedBlock>,
    pub ordered_proof: LedgerInfoWithSignatures,
    pub callback: StateComputerCommitCallBackType,
}

pub type BufferItemRootType = Link<BufferItem>;
pub type Sender<T> = UnboundedSender<T>;
pub type Receiver<T> = UnboundedReceiver<T>;

/// StateManager handles the states of ordered blocks and
/// interacts with the execution phase, the signing phase, and
/// the persisting phase.
pub struct StateManager {
    author: Author,

    buffer: List<BufferItem>,

    execution_root: BufferItemRootType,
    execution_phase_tx: Sender<ExecutionRequest>,
    execution_phase_rx: Receiver<ExecutionResponse>,

    signing_root: BufferItemRootType,
    signing_phase_tx: Sender<SigningRequest>,
    signing_phase_rx: Receiver<SigningResponse>,

    aggregation_root: BufferItemRootType,
    commit_msg_tx: NetworkSender,
    commit_msg_rx: channel::diem_channel::Receiver<AccountAddress, VerifiedEvent>,

    persisting_phase_tx: Sender<PersistingRequest>,
    persisting_phase_rx: Receiver<PersistingResponse>,

    block_rx: UnboundedReceiver<OrderedBlocks>,
    sync_rx: UnboundedReceiver<SyncRequest>,
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
        sync_rx: UnboundedReceiver<SyncRequest>,
        verifier: ValidatorVerifier,
    ) -> Self {
        let buffer = List::<BufferItem>::new();

        // point the roots to the head
        let execution_root = buffer.head.as_ref().cloned();
        let signing_root = buffer.head.as_ref().cloned();
        let aggregation_root = buffer.head.as_ref().cloned();

        Self {
            author,

            buffer,

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
            sync_rx,
            end_epoch: false,

            verifier,
        }
    }

    async fn process_ordered_blocks(
        &mut self,
        ordered_blocks: OrderedBlocks,
    ) -> anyhow::Result<()> {
        let OrderedBlocks {
            ordered_blocks,
            ordered_proof,
            callback,
        } = ordered_blocks;

        // push blocks to buffer
        self.buffer.push_back(BufferItem::new_ordered(
            ordered_blocks.clone(),
            ordered_proof,
            callback,
        ));

        // send blocks to execution phase
        self.execution_phase_tx
            .send(ExecutionRequest { ordered_blocks })
            .await?;
        Ok(())
    }

    async fn process_sync_req(&mut self, sync_event: SyncRequest) -> anyhow::Result<()> {
        let SyncRequest {
            tx,
            ledger_info,
            reconfig,
        } = sync_event;

        if reconfig {
            // buffer manager will stop
            self.end_epoch = true;
        } else {
            // clear the buffer until (including) the ledger_info
            while let Some(buffer_item) = self.buffer.pop_front() {
                if buffer_item
                    .get_commit_info()
                    .match_ordered_only(ledger_info.commit_info())
                {
                    break;
                }
            }

            // reset roots
            self.execution_root = self.buffer.head.as_ref().cloned();
            self.signing_root = self.buffer.head.as_ref().cloned();
            self.aggregation_root = self.buffer.head.as_ref().cloned();
        }

        // ack reset
        tx.send(sync_ack_new()).unwrap();
        Ok(())
    }

    /// this function updates the buffer according to the response from the execution phase
    /// it also initiates a request to the signing phase.
    async fn process_successful_execution_response(
        &mut self,
        executed_blocks: Vec<ExecutedBlock>,
    ) -> anyhow::Result<()> {
        if self.execution_root.is_none() {
            // right after a sync
            return Ok(());
        }

        let current_cursor = get_next(&self.execution_root);

        if current_cursor.is_some() {
            // update buffer
            let buffer_item = take_elem(&current_cursor);
            if let BufferItem::Ordered(ordered_box) = &buffer_item {
                if ordered_box.ordered_blocks.first().unwrap().id()
                    != executed_blocks.first().unwrap().id()
                {
                    // an sync req happened before the response
                    // we do nothing except putting the item back
                    // the process_execution_resp function will retry the next ordered batch
                    set_elem(&current_cursor, buffer_item);
                    return Ok(());
                }

                // push to the signing phase
                let commit_ledger_info = LedgerInfo::new(
                    executed_blocks.last().unwrap().block_info(),
                    ordered_box
                        .ordered_proof
                        .ledger_info()
                        .consensus_data_hash(),
                );

                self.signing_phase_tx
                    .send(SigningRequest {
                        ordered_ledger_info: ordered_box.ordered_proof.clone(),
                        commit_ledger_info,
                    })
                    .await?;

                set_elem(
                    &current_cursor,
                    buffer_item.advance_to_executed(executed_blocks),
                );
                self.execution_root = current_cursor;
            } else {
                // even if there is a sync happened before the response
                // the buffer item right after execution root should be an ordered buffer item
                panic!("Inconsistent buffer item state");
            }
        }
        Ok(())
    }

    /// this function handles the execution response and updates the buffer
    /// if the execution fails: it re-collects a larger batch and retries.
    async fn process_execution_resp(
        &mut self,
        execution_resp: ExecutionResponse,
    ) -> anyhow::Result<()> {
        // we do not use callback from the execution phase to fetch the retry blocks
        // because we want the buffer accessed by a single thread

        let ExecutionResponse { inner } = execution_resp;

        if let Ok(executed_blocks) = inner {
            let res = self
                .process_successful_execution_response(executed_blocks)
                .await;
            // try the next item (even if sending to signing phase failed)
            let cursor = get_next(&self.execution_root);
            let buffer_item = get_elem(&cursor);
            self.execution_phase_tx
                .send(ExecutionRequest {
                    ordered_blocks: buffer_item.get_blocks().clone(),
                })
                .await?;
            res
        } else {
            // it might be possible that the buffer is already reset
            // in which case we are retrying an irrelevant large batch
            // this is ok as blocks can be executed more than once
            let mut cursor = self.buffer.head.clone();
            let mut ordered_blocks: Vec<ExecutedBlock> = vec![];
            while cursor.is_some() {
                ordered_blocks.extend(get_elem(&cursor).get_blocks().clone());
                if link_eq(&cursor, &self.execution_root) {
                    // there must be a successor since the last execution failed
                    cursor = get_next(&cursor);
                    ordered_blocks.extend(get_elem(&cursor).get_blocks().clone());
                    // retry execution with the larger batch
                    // send blocks to execution phase
                    self.execution_phase_tx
                        .send(ExecutionRequest { ordered_blocks })
                        .await?;
                    break;
                }
                cursor = get_next(&cursor);
            }
            // the only case that cursor did not meet execution root is when the buffer is empty
            // in which case we do nothing
            Ok(())
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

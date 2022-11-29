// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    experimental::tests::test_utils::prepare_executed_blocks_with_ordered_ledger_info,
    round_manager::UnverifiedEvent,
    test_utils::{consensus_runtime, timed_block_on, RandomComputeResultStateComputer},
};
use futures::{SinkExt, StreamExt};
use network::protocols::network::Event;

use crate::{
    block_storage::BlockReader,
    experimental::{
        execution_phase::{ExecutionChannelType, ExecutionPhase},
        ordering_state_computer::OrderingStateComputer,
    },
};
use std::sync::Arc;

use crate::network_interface::ConsensusMsg;

use consensus_types::block::block_test_utils::certificate_for_genesis;

use crate::{
    experimental::{
        commit_phase::CommitChannelType, execution_phase::ResetAck,
        tests::test_utils::prepare_commit_phase_with_block_store_state_computer,
    },
    state_replication::empty_state_computer_call_back,
    test_utils::{EmptyStateComputer, TreeInserter},
};
use consensus_types::executed_block::ExecutedBlock;
use executor_types::StateComputeResult;
use futures::channel::oneshot;

#[test]
fn decoupled_execution_integration() {
    let channel_size = 30;
    let mut runtime = consensus_runtime();

    let (execution_phase_tx, execution_phase_rx) =
        channel::new_test::<ExecutionChannelType>(channel_size);

    let (execution_phase_reset_tx, execution_phase_reset_rx) =
        channel::new_test::<oneshot::Sender<ResetAck>>(1);

    let state_computer = Arc::new(OrderingStateComputer::new(
        execution_phase_tx,
        Arc::new(EmptyStateComputer {}), // we will not call sync_to in this test
        execution_phase_reset_tx,
    ));

    // now we need to replace the state computer instance (previously the one directly outputs to commit_result_rx)
    // to the outside one that connects with the execution phase.
    let (
        mut commit_tx,
        mut msg_tx,
        commit_phase_reset_tx,
        mut commit_result_rx,
        mut self_loop_rx,
        _safety_rules_container,
        signers,
        _state_computer,
        validator,
        commit_phase,
        block_store_handle,
    ) = prepare_commit_phase_with_block_store_state_computer(&runtime, state_computer, 1);

    let mut inserter = TreeInserter::new_with_store(signers[0].clone(), block_store_handle.clone());

    let genesis = block_store_handle.ordered_root();
    let genesis_block_id = genesis.id();
    let genesis_block = block_store_handle
        .get_block(genesis_block_id)
        .expect("genesis block must exist");

    // genesis --> a1 --> a2 --> a3 --> a4
    let a1 = inserter.insert_block_with_qc(certificate_for_genesis(), &genesis_block, 1);
    let a2 = inserter.insert_block(&a1, 2, None);
    let a3 = inserter.insert_block(&a2, 3, Some(genesis.block_info()));
    let a4 = inserter.insert_block(&a3, 4, Some(a3.block_info()));

    let ledger_info_with_sigs = a4.quorum_cert().ledger_info().clone();

    let random_state_computer = RandomComputeResultStateComputer::new();
    let random_execute_result_root_hash = random_state_computer.get_root_hash();

    let execution_phase = ExecutionPhase::new(
        execution_phase_rx,
        Arc::new(random_state_computer),
        commit_tx.clone(),
        execution_phase_reset_rx,
        commit_phase_reset_tx,
    );

    runtime.spawn(execution_phase.start());

    runtime.spawn(commit_phase.start());

    timed_block_on(&mut runtime, async move {
        // commit the block
        block_store_handle.commit(ledger_info_with_sigs).await.ok();

        // the pruning should be delayed
        assert!(block_store_handle.block_exists(a1.block().id()));

        match self_loop_rx.next().await {
            Some(Event::Message(_, msg)) => {
                let event: UnverifiedEvent = msg.into();
                // verify the message and send the message into self loop
                msg_tx.send(event.verify(&validator).unwrap()).await.ok();
            }
            _ => {
                panic!("We are expecting a commit vote message.");
            }
        };

        // it commits the block
        if let Some(ExecutionChannelType(executed_blocks, finality_proof, callback)) =
            commit_result_rx.next().await
        {
            assert_eq!(executed_blocks.len(), 3); // a1 a2 a3
            assert_eq!(
                finality_proof
                    .ledger_info()
                    .commit_info()
                    .executed_state_id(),
                random_execute_result_root_hash
            );
            callback(
                &executed_blocks
                    .into_iter()
                    .map(|b| Arc::new(ExecutedBlock::new(b, StateComputeResult::new_dummy())))
                    .collect::<Vec<Arc<ExecutedBlock>>>(),
                finality_proof,
            ); // call the callback
        } else {
            panic!("Expecting a commited block")
        }

        // and it sends a commit decision
        assert!(matches!(
            self_loop_rx.next().await,
            Some(Event::Message(_, ConsensusMsg::CommitDecisionMsg(_))),
        ));

        // fill in two dummy items to commit_tx to make sure commit_phase::check_commit has finished
        let (blocks_1, li_1) = prepare_executed_blocks_with_ordered_ledger_info(&signers[0]);
        let (blocks_2, li_2) = prepare_executed_blocks_with_ordered_ledger_info(&signers[0]);
        commit_tx
            .send(CommitChannelType(
                blocks_1,
                li_1,
                empty_state_computer_call_back(),
            ))
            .await
            .ok();
        commit_tx
            .send(CommitChannelType(
                blocks_2,
                li_2,
                empty_state_computer_call_back(),
            ))
            .await
            .ok();

        // ..also the block is gone
        assert!(!block_store_handle.block_exists(genesis_block_id));
        assert!(!block_store_handle.block_exists(a1.block().id()));
        assert!(!block_store_handle.block_exists(a2.block().id()));
        // ..until a3
        assert!(block_store_handle.block_exists(a3.block().id()));
    });
}

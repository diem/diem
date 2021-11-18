// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::{
    binary_views::FunctionView,
    control_flow_graph::{BlockId, ControlFlowGraph},
    file_format::{Bytecode, CodeOffset},
};
use std::collections::BTreeMap;

/// Trait for finite-height abstract domains. Infinite height domains would require a more complex
/// trait with widening and a partial order.
pub(crate) trait AbstractDomain: Clone + Sized {
    fn join(&mut self, other: &Self) -> JoinResult;
}

#[derive(Debug)]
pub(crate) enum JoinResult {
    Changed,
    Unchanged,
}

#[derive(Clone)]
pub(crate) enum BlockPostcondition<AnalysisError> {
    /// Block not yet analyzed
    Unprocessed,
    /// Analyzing block was successful
    /// TODO might carry post state at some point
    Success,
    /// Analyzing block resulted in an error
    Error(AnalysisError),
}

#[allow(dead_code)]
#[derive(Clone)]
pub(crate) struct BlockInvariant<State, AnalysisError> {
    /// Precondition of the block
    pub(crate) pre: State,
    /// Postcondition of the block
    pub(crate) post: BlockPostcondition<AnalysisError>,
}

/// A map from block id's to the pre/post of each block after a fixed point is reached.
#[allow(dead_code)]
pub(crate) type InvariantMap<State, AnalysisError> =
    BTreeMap<BlockId, BlockInvariant<State, AnalysisError>>;

/// Take a pre-state + instruction and mutate it to produce a post-state
/// Auxiliary data can be stored in self.
pub(crate) trait TransferFunctions {
    type State: AbstractDomain;
    type AnalysisError: Clone;

    /// Execute local@instr found at index local@index in the current basic block from pre-state
    /// local@pre.
    /// Should return an AnalysisError if executing the instruction is unsuccessful, and () if
    /// the effects of successfully executing local@instr have been reflected by mutatating
    /// local@pre.
    /// Auxilary data from the analysis that is not part of the abstract state can be collected by
    /// mutating local@self.
    /// The last instruction index in the current block is local@last_index. Knowing this
    /// information allows clients to detect the end of a basic block and special-case appropriately
    /// (e.g., normalizing the abstract state before a join).
    fn execute(
        &mut self,
        pre: &mut Self::State,
        instr: &Bytecode,
        index: CodeOffset,
        last_index: CodeOffset,
    ) -> Result<(), Self::AnalysisError>;
}

pub(crate) trait AbstractInterpreter: TransferFunctions {
    /// Analyze procedure local@function_view starting from pre-state local@initial_state.
    fn analyze_function(
        &mut self,
        initial_state: Self::State,
        function_view: &FunctionView,
    ) -> InvariantMap<Self::State, Self::AnalysisError> {
        let mut inv_map: InvariantMap<Self::State, Self::AnalysisError> = InvariantMap::new();
        let entry_block_id = function_view.cfg().entry_block_id();
        let mut next_block = Some(entry_block_id);
        inv_map.insert(
            entry_block_id,
            BlockInvariant {
                pre: initial_state,
                post: BlockPostcondition::Unprocessed,
            },
        );

        macro_rules! set_next_block {
            ($block_id:ident) => {
                let loop_last_continue_blocks = function_view.cfg().loop_last_continue_blocks();
                // if the current block has the last continue of a loop,
                // check that any of its associated starts need to be reprocessed
                match loop_last_continue_blocks.get(&$block_id) {
                    Some(loop_id)
                        if matches!(&inv_map[loop_id].post, BlockPostcondition::Unprocessed) =>
                    {
                        next_block = Some(*loop_id);
                    }
                    _ => {
                        next_block = function_view.cfg().next_block($block_id);
                    }
                }
            };
        }

        while let Some(block_id) = next_block {
            let block_invariant = match inv_map.get_mut(&block_id) {
                Some(invariant) => invariant,
                None => {
                    // This can only happen when all predecessors have errors,
                    // so skip the block and move on to the next one
                    set_next_block!(block_id);
                    continue;
                }
            };

            let pre_state = &block_invariant.pre;
            let post_state = match self.execute_block(block_id, pre_state, function_view) {
                Err(e) => {
                    block_invariant.post = BlockPostcondition::Error(e);
                    set_next_block!(block_id);
                    continue;
                }
                Ok(s) => {
                    block_invariant.post = BlockPostcondition::Success;
                    s
                }
            };

            // propagate postcondition of this block to successor blocks
            for next_block_id in function_view.cfg().successors(block_id) {
                match inv_map.get_mut(next_block_id) {
                    Some(next_block_invariant) => {
                        let join_result = {
                            let old_pre = &mut next_block_invariant.pre;
                            old_pre.join(&post_state)
                        };
                        match join_result {
                            JoinResult::Unchanged => {
                                // Pre is the same after join. Reanalyzing this block would produce
                                // the same post
                            }
                            JoinResult::Changed => {
                                // Pre has changed, the post condition is now unknown for the block
                                next_block_invariant.post = BlockPostcondition::Unprocessed
                            }
                        }
                    }
                    None => {
                        // Haven't visited the next block yet. Use the post of the current block as
                        // its pre
                        inv_map.insert(
                            *next_block_id,
                            BlockInvariant {
                                pre: post_state.clone(),
                                post: BlockPostcondition::Success,
                            },
                        );
                    }
                }
            }
            set_next_block!(block_id);
        }
        inv_map
    }

    fn execute_block(
        &mut self,
        block_id: BlockId,
        pre_state: &Self::State,
        function_view: &FunctionView,
    ) -> Result<Self::State, Self::AnalysisError> {
        let mut state_acc = pre_state.clone();
        let block_end = function_view.cfg().block_end(block_id);
        for offset in function_view.cfg().instr_indexes(block_id) {
            let instr = &function_view.code().code[offset as usize];
            self.execute(&mut state_acc, instr, offset, block_end)?
        }
        Ok(state_acc)
    }
}

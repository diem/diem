// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::control_flow_graph::{BlockId, ControlFlowGraph};
use std::collections::HashMap;
use vm::{
    file_format::{Bytecode, CompiledModule},
    views::FunctionDefinitionView,
};

/// Trait for finite-height abstract domains. Infinite height domains would require a more complex
/// trait with widening and a partial order.
pub trait AbstractDomain: Clone + Sized {
    fn join(&mut self, other: &Self) -> JoinResult;
}

#[derive(Debug)]
pub enum JoinResult {
    Unchanged,
    Changed,
    Error,
}

#[derive(Clone)]
pub enum BlockPrecondition<State> {
    State(State),
    /// joining postconditions of previous blocks ended in failure
    JoinFailure,
}

#[derive(Clone)]
pub enum BlockPostcondition<AnalysisError> {
    /// Analyzing block was successful
    Success,
    /// Analyzing block ended in an error
    Error(AnalysisError),
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct BlockInvariant<State, AnalysisError> {
    /// Precondition of the block
    pub pre: BlockPrecondition<State>,
    /// Postcondition of the block---just success/error for now
    pub post: BlockPostcondition<AnalysisError>,
}

/// A map from block id's to the pre/post of each block after a fixed point is reached.
#[allow(dead_code)]
pub type InvariantMap<State, AnalysisError> =
    HashMap<BlockId, BlockInvariant<State, AnalysisError>>;

/// Take a pre-state + instruction and mutate it to produce a post-state
/// Auxiliary data can be stored in self.
pub trait TransferFunctions {
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
        index: usize,
        last_index: usize,
    ) -> Result<(), Self::AnalysisError>;
}

pub trait AbstractInterpreter: TransferFunctions {
    /// Analyze procedure local@function_view starting from pre-state local@initial_state.
    fn analyze_function(
        &mut self,
        initial_state: Self::State,
        function_view: &FunctionDefinitionView<CompiledModule>,
        cfg: &dyn ControlFlowGraph,
    ) -> InvariantMap<Self::State, Self::AnalysisError> {
        let mut inv_map: InvariantMap<Self::State, Self::AnalysisError> = InvariantMap::new();
        let entry_block_id = cfg.entry_block_id();
        let mut work_list = vec![entry_block_id];
        inv_map.insert(
            entry_block_id,
            BlockInvariant {
                pre: BlockPrecondition::State(initial_state),
                post: BlockPostcondition::Success,
            },
        );

        while let Some(block_id) = work_list.pop() {
            let block_invariant = match inv_map.get_mut(&block_id) {
                Some(invariant) => invariant,
                None => unreachable!("Missing invariant for block {}", block_id),
            };

            let mut state = match &block_invariant.pre {
                BlockPrecondition::State(s) => s.clone(),
                BlockPrecondition::JoinFailure =>
                // Can't analyze the block from a failing precondition
                {
                    continue
                }
            };
            match self.execute_block(block_id, &mut state, &function_view, cfg) {
                Err(errors) => {
                    block_invariant.post = BlockPostcondition::Error(errors);
                    continue;
                }
                Ok(()) => {
                    block_invariant.post = BlockPostcondition::Success;
                }
            };
            // propagate postcondition of this block to successor blocks
            for next_block_id in cfg.successors(block_id) {
                match inv_map.get_mut(next_block_id) {
                    Some(next_block_invariant) => {
                        let join_result = match &mut next_block_invariant.pre {
                            BlockPrecondition::State(old_pre) => old_pre.join(&state),
                            BlockPrecondition::JoinFailure => JoinResult::Error,
                        };
                        match join_result {
                            JoinResult::Unchanged => {
                                // Pre is the same after join. Reanalyzing this block would produce
                                // the same post. Don't schedule it.
                                continue;
                            }
                            JoinResult::Changed => {
                                // The pre changed. Schedule the next block.
                                work_list.push(*next_block_id);
                            }
                            JoinResult::Error => {
                                // This join produced an error. Don't schedule the block.
                                next_block_invariant.pre = BlockPrecondition::JoinFailure;
                                continue;
                            }
                        }
                    }
                    None => {
                        // Haven't visited the next block yet. Use the post of the current block as
                        // its pre and schedule it.
                        inv_map.insert(
                            *next_block_id,
                            BlockInvariant {
                                pre: BlockPrecondition::State(state.clone()),
                                post: BlockPostcondition::Success,
                            },
                        );
                        work_list.push(*next_block_id);
                    }
                }
            }
        }

        inv_map
    }

    fn execute_block(
        &mut self,
        block_id: BlockId,
        state: &mut Self::State,
        function_view: &FunctionDefinitionView<CompiledModule>,
        cfg: &dyn ControlFlowGraph,
    ) -> Result<(), Self::AnalysisError> {
        let block_end = cfg.block_end(block_id);
        for offset in cfg.instr_indexes(block_id) {
            let instr = &function_view.code().code[offset as usize];
            self.execute(state, instr, offset as usize, block_end as usize)?
        }

        Ok(())
    }
}

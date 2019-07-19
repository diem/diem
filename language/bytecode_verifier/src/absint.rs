use crate::control_flow_graph::{BlockId, ControlFlowGraph, VMControlFlowGraph};
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
pub enum BlockPostcondition {
    /// Analyzing block was successful
    Success,
    /// Analyzing block ended in an error
    Error,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct BlockInvariant<State> {
    /// Precondition of the block
    pre: State,
    /// Postcondition of the block---just success/error for now
    post: BlockPostcondition,
}

/// A map from block id's to the pre/post of each block after a fixed point is reached.
#[allow(dead_code)]
pub type InvariantMap<State> = HashMap<BlockId, BlockInvariant<State>>;

/// Take a pre-state + instruction and mutate it to produce a post-state
/// Auxiliary data can be stored in self.
pub trait TransferFunctions {
    type State: AbstractDomain;
    type AnalysisError;

    /// Execute local@instr found at index local@index in the current basic block from pre-state
    /// local@pre.
    /// Should return an AnalysisError if executing the instruction is unsuccessful, and () if
    /// the effects of successfully executing local@instr have been reflected by mutatating
    /// local@pre.
    /// Auxiliary data from the analysis that is not part of the abstract state can be collected by
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
        cfg: &VMControlFlowGraph,
    ) -> InvariantMap<Self::State> {
        let mut inv_map: InvariantMap<Self::State> = InvariantMap::new();
        let entry_block_id = 0; // 0 is always the entry block
                                // seed worklist/precondition map with initial block/state
        let mut work_list = vec![entry_block_id];
        inv_map.insert(
            entry_block_id,
            BlockInvariant {
                pre: initial_state,
                post: BlockPostcondition::Success,
            },
        );

        while let Some(block_id) = work_list.pop() {
            let mut block_invariant = match inv_map.get_mut(&block_id) {
                Some(BlockInvariant {
                    post: BlockPostcondition::Error,
                    ..
                }) =>
                // Analyzing this block previously resulted in an error. Avoid double-reporting.
                {
                    continue
                }
                Some(invariant) => invariant.clone(),
                None => unreachable!("Missing invariant for block {}", block_id),
            };

            let mut state = block_invariant.pre.clone();
            let block_ends_in_error = self
                .execute_block(block_id, &mut state, &function_view, &cfg)
                .is_err();
            if block_ends_in_error {
                block_invariant.post = BlockPostcondition::Error;
                continue;
            } else {
                block_invariant.post = BlockPostcondition::Success;
            };

            let block = cfg
                .block_of_id(block_id)
                .expect("block_id is not the start offset of a block");
            // propagate postcondition of this block to successor blocks
            for next_block_id in &block.successors {
                match inv_map.get_mut(next_block_id) {
                    Some(next_block_invariant) => {
                        let join_result = next_block_invariant.pre.join(&state);
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
                                next_block_invariant.post = BlockPostcondition::Error;
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
                                pre: state.clone(),
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
        cfg: &VMControlFlowGraph,
    ) -> Result<(), Self::AnalysisError> {
        let block = cfg
            .block_of_id(block_id)
            .expect("block_id is not the start offset of a block");

        for offset in block.entry..=block.exit {
            let instr = &function_view.code().code[offset as usize];
            self.execute(state, instr, offset as usize, block.exit as usize)?
        }

        Ok(())
    }
}

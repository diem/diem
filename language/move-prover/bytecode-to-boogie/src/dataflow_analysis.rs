// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Adapted from AbstractInterpreter for Bytecode, this module defines the data-flow analysis
//! framework for stackless bytecode.

use bytecode_verifier::absint::{AbstractDomain, JoinResult};
use bytecode_verifier::control_flow_graph::{BlockId, ControlFlowGraph};
use std::collections::HashMap;

pub struct BlockState<State: Clone> {
    pub pre: State,
    pub post: State,
}

pub type StateMap<State> = HashMap<BlockId, BlockState<State>>;

/// Take a pre-state + instruction and mutate it to produce a post-state。
pub trait TransferFunctions {
    type State: AbstractDomain + Clone;
    type InstrType;

    /// Execute instr found at index in the current basic block from pre-state
    fn execute(&mut self, pre: &Self::State, instr: &Self::InstrType) -> Self::State;
}

pub trait DataflowAnalysis: TransferFunctions {
    fn analyze_function(
        &mut self,
        initial_state: Self::State,
        instrs: &[Self::InstrType],
        cfg: &dyn ControlFlowGraph,
    ) -> StateMap<Self::State> {
        let mut state_map: StateMap<Self::State> = StateMap::new();
        let entry_block_id = cfg.entry_block_id();
        let mut work_list = vec![entry_block_id];
        state_map.insert(
            entry_block_id,
            BlockState {
                pre: initial_state.clone(),
                post: initial_state.clone(),
            },
        );

        while let Some(block_id) = work_list.pop() {
            let block_res = state_map
                .get_mut(&block_id)
                .unwrap_or_else(|| panic!("Missing result for block {}", block_id));

            let pre_state = block_res.pre.clone();
            let post_state = self.execute_block(block_id, &pre_state, &instrs, cfg);

            // propagate postcondition of this block to successor blocks
            for next_block_id in cfg.successors(block_id) {
                match state_map.get_mut(next_block_id) {
                    Some(next_block_res) => {
                        let join_result = next_block_res.pre.join(&post_state);

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
                            _ => unreachable!(), // There shouldn't be any error at this point
                        }
                    }
                    None => {
                        // Haven't visited the next block yet. Use the post of the current block as
                        // its pre and schedule it.
                        state_map.insert(
                            *next_block_id,
                            BlockState {
                                pre: post_state.clone(),
                                post: initial_state.clone(),
                            },
                        );
                        work_list.push(*next_block_id);
                    }
                }
            }

            state_map.insert(
                block_id,
                BlockState {
                    pre: pre_state,
                    post: post_state,
                },
            );
        }

        state_map
    }

    fn execute_block(
        &mut self,
        block_id: BlockId,
        pre_state: &Self::State,
        instrs: &[Self::InstrType],
        cfg: &dyn ControlFlowGraph,
    ) -> Self::State {
        let mut state = pre_state.clone();
        for offset in cfg.instr_indexes(block_id) {
            let instr = &instrs[offset as usize];
            state = self.execute(&state, instr);
        }
        state
    }
}

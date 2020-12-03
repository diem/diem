// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Adapted from AbstractInterpreter for Bytecode, this module defines the data-flow analysis
//! framework for stackless bytecode.

use crate::{
    stackless_bytecode::Bytecode,
    stackless_control_flow_graph::{BlockId, StacklessControlFlowGraph},
};
use std::collections::{BTreeMap, VecDeque};
use vm::file_format::CodeOffset;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinResult {
    Unchanged,
    Changed,
}

impl JoinResult {
    pub fn join(self, other: JoinResult) -> JoinResult {
        use JoinResult::*;
        match (self, other) {
            (Unchanged, Unchanged) => Unchanged,
            _ => Changed,
        }
    }
}

pub trait AbstractDomain: Clone + Sized {
    fn join(&mut self, other: &Self) -> JoinResult;
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BlockState<State: Clone> {
    pub pre: State,
    pub post: State,
}

pub type StateMap<State> = BTreeMap<BlockId, BlockState<State>>;

/// Take a pre-state + instruction and mutate it to produce a post-state。
pub trait TransferFunctions {
    type State: AbstractDomain + Clone;
    const BACKWARD: bool;

    fn execute_block(
        &self,
        block_id: BlockId,
        mut state: Self::State,
        instrs: &[Bytecode],
        cfg: &StacklessControlFlowGraph,
    ) -> Self::State {
        if Self::BACKWARD {
            for offset in cfg.instr_indexes(block_id).rev() {
                let instr = &instrs[offset as usize];
                self.execute(&mut state, instr, offset);
            }
        } else {
            for offset in cfg.instr_indexes(block_id) {
                let instr = &instrs[offset as usize];
                self.execute(&mut state, instr, offset);
            }
        }
        state
    }

    fn execute(&self, state: &mut Self::State, instr: &Bytecode, offset: CodeOffset);
}

pub trait DataflowAnalysis: TransferFunctions {
    fn analyze_function(
        &mut self,
        initial_state: Self::State,
        instrs: &[Bytecode],
        cfg: &StacklessControlFlowGraph,
    ) -> StateMap<Self::State> {
        let mut state_map: StateMap<Self::State> = StateMap::new();
        let mut work_list = VecDeque::new();
        for entry_block_id in cfg.entry_blocks() {
            work_list.push_back(entry_block_id);
            state_map.insert(
                entry_block_id,
                BlockState {
                    pre: initial_state.clone(),
                    post: initial_state.clone(),
                },
            );
        }
        while let Some(block_id) = work_list.pop_front() {
            let pre = state_map.remove(&block_id).expect("basic block").pre;
            let post = self.execute_block(block_id, pre.clone(), &instrs, cfg);

            // propagate postcondition of this block to successor blocks
            for next_block_id in cfg.successors(block_id) {
                match state_map.get_mut(next_block_id) {
                    Some(next_block_res) => {
                        let join_result = next_block_res.pre.join(&post);

                        match join_result {
                            JoinResult::Unchanged => {
                                // Pre is the same after join. Reanalyzing this block would produce
                                // the same post. Don't schedule it.
                                continue;
                            }
                            JoinResult::Changed => {
                                // The pre changed. Schedule the next block.
                                work_list.push_back(*next_block_id);
                            }
                        }
                    }
                    None => {
                        // Haven't visited the next block yet. Use the post of the current block as
                        // its pre and schedule it.
                        state_map.insert(
                            *next_block_id,
                            BlockState {
                                pre: post.clone(),
                                post: initial_state.clone(),
                            },
                        );
                        work_list.push_back(*next_block_id);
                    }
                }
            }
            state_map.insert(block_id, BlockState { pre, post });
        }

        state_map
    }

    /// Takes the StateMap resulting from `analyze_function` and converts it into a map
    /// from each code offset into a derived state `A`. This re-executes the analysis for
    /// each instruction within a basic block to reconstruct the intermediate results
    /// from block begin to block end. The function `f` gets passed the before/after state
    /// of the instruction at a code offset. Returns a map from code offset to `A`.
    fn state_per_instruction<A, F>(
        &self,
        state_map: StateMap<Self::State>,
        instrs: &[Bytecode],
        cfg: &StacklessControlFlowGraph,
        mut f: F,
    ) -> BTreeMap<CodeOffset, A>
    where
        F: FnMut(&Self::State, &Self::State) -> A,
    {
        let mut result = BTreeMap::new();
        for (block_id, block_state) in state_map {
            let mut state = block_state.pre;
            if Self::BACKWARD {
                for offset in cfg.instr_indexes(block_id).rev() {
                    let after = state.clone();
                    self.execute(&mut state, &instrs[offset as usize], offset);
                    result.insert(offset, f(&state, &after));
                }
            } else {
                for offset in cfg.instr_indexes(block_id) {
                    let before = state.clone();
                    self.execute(&mut state, &instrs[offset as usize], offset);
                    result.insert(offset, f(&before, &state));
                }
            }
        }
        result
    }
}

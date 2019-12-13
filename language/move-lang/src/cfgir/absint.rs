// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::{ast::*, cfg::BlockCFG};
use crate::errors::*;
use std::collections::HashMap;

/// Trait for finite-height abstract domains. Infinite height domains would require a more complex
/// trait with widening and a partial order.
pub trait AbstractDomain: Clone + Sized {
    fn join(&mut self, other: &Self) -> JoinResult;
}

#[derive(Debug)]
pub enum JoinResult {
    Unchanged,
    Changed,
}

#[derive(Clone)]
pub enum BlockPostcondition {
    /// Unprocessed block
    Unprocessed,
    /// Analyzing block was successful
    Success,
    /// Analyzing block ended in an error
    Error(Errors),
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct BlockInvariant<State> {
    /// Precondition of the block
    pre: State,
    /// Postcondition of the block---just success/error for now
    post: BlockPostcondition,
}

impl<State> BlockInvariant<State> {
    #[allow(dead_code)]
    pub fn pre(&self) -> &State {
        &self.pre
    }

    #[allow(dead_code)]
    pub fn post(&self) -> &BlockPostcondition {
        &self.post
    }
}

/// A map from block id's to the pre/post of each block after a fixed point is reached.
#[allow(dead_code)]
pub type InvariantMap<State> = HashMap<Label, BlockInvariant<State>>;

fn collect_errors<State>(map: InvariantMap<State>) -> Errors {
    let mut errors = Errors::new();
    for (_, BlockInvariant { post, .. }) in map {
        if let BlockPostcondition::Error(mut es) = post {
            errors.append(&mut es)
        }
    }
    errors
}

/// Take a pre-state + instruction and mutate it to produce a post-state
/// Auxiliary data can be stored in self.
pub trait TransferFunctions {
    type State: AbstractDomain;

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
    fn execute(&mut self, pre: &mut Self::State, command: &Command) -> Errors;
}

pub trait AbstractInterpreter: TransferFunctions {
    /// Analyze procedure local@function_view starting from pre-state local@initial_state.
    fn analyze_function(&mut self, cfg: &BlockCFG, initial_state: Self::State) -> Errors {
        let mut inv_map: InvariantMap<Self::State> = InvariantMap::new();
        let start = cfg.start_block();
        let mut work_list = vec![start];
        inv_map.insert(
            start.clone(),
            BlockInvariant {
                pre: initial_state,
                post: BlockPostcondition::Unprocessed,
            },
        );

        while let Some(block_lbl) = work_list.pop() {
            let block_invariant = match inv_map.get_mut(&block_lbl) {
                // Analyzing this block previously resulted in an error. Avoid double-reporting.
                Some(BlockInvariant {
                    post: BlockPostcondition::Error(_),
                    ..
                }) => continue,
                Some(invariant) => invariant,
                None => unreachable!("Missing invariant for block"),
            };

            let mut state = block_invariant.pre.clone();
            match self.execute_block(cfg, &mut state, block_lbl) {
                Err(e) => {
                    block_invariant.post = BlockPostcondition::Error(e);
                    continue;
                }
                Ok(()) => {
                    block_invariant.post = BlockPostcondition::Success;
                }
            }

            // propagate postcondition of this block to successor blocks
            for next_lbl in cfg.successors(&block_lbl) {
                match inv_map.get_mut(next_lbl) {
                    Some(next_block_invariant) => {
                        match next_block_invariant.pre.join(&state) {
                            JoinResult::Unchanged => {
                                // Pre is the same after join. Reanalyzing this block would produce
                                // the same post. Don't schedule it.
                                continue;
                            }
                            JoinResult::Changed => {
                                // The pre changed. Schedule the next block.
                                work_list.push(next_lbl.clone());
                            }
                        }
                    }
                    None => {
                        // Haven't visited the next block yet. Use the post of the current block as
                        // its pre and schedule it.
                        inv_map.insert(
                            next_lbl.clone(),
                            BlockInvariant {
                                pre: state.clone(),
                                post: BlockPostcondition::Unprocessed,
                            },
                        );
                        work_list.push(next_lbl.clone());
                    }
                }
            }
        }

        collect_errors(inv_map)
    }

    fn execute_block(
        &mut self,
        cfg: &BlockCFG,
        state: &mut Self::State,
        block_lbl: Label,
    ) -> Result<(), Errors> {
        let mut errors = vec![];
        for cmd in cfg.block(&block_lbl) {
            errors.append(&mut self.execute(state, cmd));
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

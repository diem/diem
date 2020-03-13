// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::cfg::CFG;
use crate::{errors::*, hlir::ast::*};
use std::collections::BTreeMap;

/// Trait for finite-height abstract domains. Infinite height domains would require a more complex
/// trait with widening and a partial order.
pub trait AbstractDomain: Clone + Sized {
    fn join(&mut self, other: &Self) -> JoinResult;
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum JoinResult {
    Unchanged,
    Changed,
}

#[derive(Clone)]
enum BlockPostcondition {
    /// Unprocessed block
    Unprocessed,
    /// Analyzing block was successful
    Success,
    /// Analyzing block ended in an error
    Error(Errors),
}

#[derive(Clone)]
struct BlockInvariant<State> {
    /// Precondition of the block
    pre: State,
    /// Postcondition of the block---just success/error for now
    post: BlockPostcondition,
}

/// A map from block id's to the pre/post of each block after a fixed point is reached.
type InvariantMap<State> = BTreeMap<Label, BlockInvariant<State>>;

fn collect_states_and_errors<State>(map: InvariantMap<State>) -> (BTreeMap<Label, State>, Errors) {
    let mut errors = Errors::new();
    let final_states = map
        .into_iter()
        .map(|(lbl, BlockInvariant { pre, post })| {
            if let BlockPostcondition::Error(mut es) = post {
                errors.append(&mut es)
            }
            (lbl, pre)
        })
        .collect();
    (final_states, errors)
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
    fn execute(
        &mut self,
        pre: &mut Self::State,
        lbl: Label,
        idx: usize,
        command: &Command,
    ) -> Errors;
}

pub trait AbstractInterpreter: TransferFunctions {
    /// Analyze procedure local@function_view starting from pre-state local@initial_state.
    fn analyze_function(
        &mut self,
        cfg: &dyn CFG,
        initial_state: Self::State,
    ) -> (BTreeMap<Label, Self::State>, Errors) {
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
                Some(invariant) => invariant,
                None => unreachable!("Missing invariant for block"),
            };

            let (post_state, errors) = self.execute_block(cfg, &block_invariant.pre, block_lbl);
            block_invariant.post = if errors.is_empty() {
                BlockPostcondition::Success
            } else {
                BlockPostcondition::Error(errors)
            };
            // propagate postcondition of this block to successor blocks
            for next_lbl in cfg.successors(block_lbl) {
                match inv_map.get_mut(next_lbl) {
                    Some(next_block_invariant) => {
                        match next_block_invariant.pre.join(&post_state) {
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
                                pre: post_state.clone(),
                                post: BlockPostcondition::Unprocessed,
                            },
                        );
                        work_list.push(next_lbl.clone());
                    }
                }
            }
        }

        collect_states_and_errors(inv_map)
    }

    fn execute_block(
        &mut self,
        cfg: &dyn CFG,
        pre_state: &Self::State,
        block_lbl: Label,
    ) -> (Self::State, Errors) {
        let mut state = pre_state.clone();
        let mut errors = vec![];
        for (idx, cmd) in cfg.commands(block_lbl) {
            errors.append(&mut self.execute(&mut state, block_lbl, idx, cmd));
        }
        (state, errors)
    }
}

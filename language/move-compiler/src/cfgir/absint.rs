// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::cfg::CFG;
use crate::{diagnostics::Diagnostics, hlir::ast::*};
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
    Error(Diagnostics),
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

fn collect_states_and_diagnostics<State>(
    map: InvariantMap<State>,
) -> (BTreeMap<Label, State>, Diagnostics) {
    let mut diags = Diagnostics::new();
    let final_states = map
        .into_iter()
        .map(|(lbl, BlockInvariant { pre, post })| {
            if let BlockPostcondition::Error(ds) = post {
                diags.extend(ds)
            }
            (lbl, pre)
        })
        .collect();
    (final_states, diags)
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
    ) -> Diagnostics;
}

pub trait AbstractInterpreter: TransferFunctions {
    /// Analyze procedure local@function_view starting from pre-state local@initial_state.
    fn analyze_function(
        &mut self,
        cfg: &dyn CFG,
        initial_state: Self::State,
    ) -> (BTreeMap<Label, Self::State>, Diagnostics) {
        let mut inv_map: InvariantMap<Self::State> = InvariantMap::new();
        let start = cfg.start_block();
        let mut next_block = Some(start);

        while let Some(block_label) = next_block {
            let block_invariant = inv_map
                .entry(block_label)
                .or_insert_with(|| BlockInvariant {
                    pre: initial_state.clone(),
                    post: BlockPostcondition::Unprocessed,
                });

            let (post_state, errors) = self.execute_block(cfg, &block_invariant.pre, block_label);
            block_invariant.post = if errors.is_empty() {
                BlockPostcondition::Success
            } else {
                BlockPostcondition::Error(errors)
            };

            // propagate postcondition of this block to successor blocks
            for next_block_id in cfg.successors(block_label) {
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

            // if the current block has the last continue of a loop,
            // check that any of its associated starts need to be reprocessed
            match cfg.last_continue_to_loop(block_label) {
                Some(loop_head)
                    if matches!(&inv_map[&loop_head].post, BlockPostcondition::Unprocessed) =>
                {
                    next_block = Some(loop_head);
                }
                _ => {
                    next_block = cfg.next_block(block_label);
                }
            }
        }
        collect_states_and_diagnostics(inv_map)
    }

    fn execute_block(
        &mut self,
        cfg: &dyn CFG,
        pre_state: &Self::State,
        block_lbl: Label,
    ) -> (Self::State, Diagnostics) {
        let mut state = pre_state.clone();
        let mut diags = Diagnostics::new();
        for (idx, cmd) in cfg.commands(block_lbl) {
            diags.extend(self.execute(&mut state, block_lbl, idx, cmd));
        }
        (state, diags)
    }
}

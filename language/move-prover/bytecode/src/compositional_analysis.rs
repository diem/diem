// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    dataflow_analysis::{AbstractDomain, DataflowAnalysis},
    function_target::FunctionTargetData,
    function_target_pipeline::FunctionTargetsHolder,
    stackless_control_flow_graph::StacklessControlFlowGraph,
};
use move_model::model::{FunId, FunctionEnv, GlobalEnv, QualifiedId};

/// Provides access to procedure summaries that have already been computed
pub struct SummaryCache<'a> {
    targets: &'a FunctionTargetsHolder,
    global_env: &'a GlobalEnv,
}

impl<'a> SummaryCache<'a> {
    pub fn new(targets: &'a FunctionTargetsHolder, global_env: &'a GlobalEnv) -> Self {
        Self {
            targets,
            global_env,
        }
    }

    /// Return a summary for `fun_id`. Returns None if `fun_id` is a native function
    pub fn get<Summary: 'static>(&self, fun_id: QualifiedId<FunId>) -> Option<&Summary> {
        self.targets
            .get_target_data(&fun_id)
            .map(|fun_data| {
                if self.global_env.get_function(fun_id).is_native() {
                    None
                } else {
                    Some(
                        fun_data.annotations.get::<Summary>().expect(
                            "Failed to resolve summary; recursion or scheduler bug suspected",
                        ),
                    )
                }
            })
            .flatten()
    }

    pub fn global_env(&self) -> &GlobalEnv {
        &self.global_env
    }
}

/// Trait that lifts an intraprocedural analysis into a bottom-up, compositional interprocedural
/// analysis. The derived `summarize` function computes a postcondition for the procedure under
/// analysis, then stores it in the summary cache for other procedures to use.
// TODO: allow client analyses to customize this by providing a function that converts a
// postcondition into a summary
pub trait CompositionalAnalysis: DataflowAnalysis
where
    Self::State: 'static,
{
    fn summarize(
        &self,
        func_env: &FunctionEnv<'_>,
        initial_state: Self::State,
        mut data: FunctionTargetData,
    ) -> FunctionTargetData {
        if !func_env.is_native() {
            let cfg = if Self::BACKWARD {
                unimplemented!("backward compositional analysis")
            } else {
                StacklessControlFlowGraph::new_forward(&data.code)
            };
            let instrs = &data.code;
            let state_map = self.analyze_function(initial_state, instrs, &cfg);
            let exit_states: Vec<Self::State> = cfg
                .exit_blocks(instrs)
                .iter()
                .map(|block_id| state_map[block_id].post.clone())
                .collect();
            // Join exit states to create summary
            let mut acc = exit_states[0].clone();
            for exit_state in exit_states.iter().skip(1) {
                acc.join(&exit_state);
            }
            data.annotations.set(acc)
        } else {
            // TODO: not clear that this is desired, but some clients rely on
            // every function having a summary, even natives
            data.annotations.set(initial_state)
        }
        data
    }
}

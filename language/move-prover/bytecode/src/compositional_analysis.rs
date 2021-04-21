// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    dataflow_analysis::{AbstractDomain, DataflowAnalysis},
    function_target::FunctionTarget,
    function_target_pipeline::{FunctionTargetsHolder, FunctionVariant},
    stackless_control_flow_graph::StacklessControlFlowGraph,
};
use move_model::model::{FunId, GlobalEnv, QualifiedId};

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

    /// Return a summary for a variant of `fun_id`. Returns None if `fun_id` is a native function
    pub fn get<Summary: 'static>(
        &self,
        fun_id: QualifiedId<FunId>,
        variant: &FunctionVariant,
    ) -> Option<&Summary> {
        let fun_env = self.global_env.get_function(fun_id);
        self.targets
            .get_data(&fun_id, variant)
            .map(|fun_data| {
                if fun_env.is_native_or_intrinsic() {
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
/// analysis. Here, the type `Summary` represents a transformation of the final data flow analysis
/// state.
pub trait CompositionalAnalysis<Summary: AbstractDomain + 'static>: DataflowAnalysis
where
    Self::State: AbstractDomain + 'static,
{
    /// Specifies mapping from elements of dataflow analysis domain to elements of `Domain`.
    fn to_summary(&self, state: Self::State, fun_target: &FunctionTarget) -> Summary;

    /// Computes a postcondition for the function `data` and then maps the postcondition to an
    /// element of abstract domain `Domain` by applying `to_summary` function. The result is stored
    /// in the summary cache of `data`.
    fn summarize(&self, fun_target: &FunctionTarget<'_>, initial_state: Self::State) -> Summary {
        if !fun_target.func_env.is_native() {
            let instrs = &fun_target.data.code;
            let cfg = if Self::BACKWARD {
                unimplemented!("backward compositional analysis")
            } else {
                StacklessControlFlowGraph::new_forward(instrs)
            };
            let state_map = self.analyze_function(initial_state.clone(), instrs, &cfg);
            if let Some(exit_state) = state_map.get(&cfg.exit_block()) {
                self.to_summary(exit_state.post.clone(), fun_target)
            } else {
                self.to_summary(initial_state, fun_target)
            }
        } else {
            // TODO: not clear that this is desired, but some clients rely on
            // every function having a summary, even natives
            self.to_summary(initial_state, fun_target)
        }
    }
}

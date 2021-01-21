// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    dataflow_analysis::{AbstractDomain, DataflowAnalysis},
    function_target::FunctionData,
    function_target_pipeline::{FunctionTargetsHolder, FunctionVariant},
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
        // TODO(wrwg): this currently only works for FunctionVariant::Baseline. If needed
        //   we may need to extend this. However, we expect to not perform analysis
        //   once function variants are introduced beyond baseline, as this happens
        //   at the end of transformation pipeline, as part of instrumentation with
        //   the specification.
        let fun_env = self.global_env.get_function(fun_id);
        assert_eq!(
            self.targets.get_target_variants(&fun_env),
            vec![FunctionVariant::Baseline]
        );
        self.targets
            .get_target_data(&fun_id, FunctionVariant::Baseline)
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
/// analysis.
pub trait CompositionalAnalysis<Domain: AbstractDomain + 'static>: DataflowAnalysis
where
    Self::State: AbstractDomain + 'static,
{
    /// Specifies mapping from elements of dataflow analysis domain to elements of `Domain`.
    fn to_summary(&self, state: Self::State) -> Domain;

    /// Computes a postcondition for the procedure `data` and then maps the postcondition to an
    /// element of abstract domain `Domain` by applying `to_summary` function. The result is stored
    /// in the summary cache of `data`.
    fn summarize(
        &self,
        func_env: &FunctionEnv<'_>,
        initial_state: Self::State,
        mut data: FunctionData,
    ) -> FunctionData {
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
            data.annotations.set(self.to_summary(acc))
        } else {
            // TODO: not clear that this is desired, but some clients rely on
            // every function having a summary, even natives
            data.annotations.set(self.to_summary(initial_state))
        }
        data
    }
}

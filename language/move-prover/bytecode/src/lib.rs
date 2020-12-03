// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::function_target_pipeline::FunctionTargetsHolder;
use spec_lang::env::GlobalEnv;

pub mod annotations;
pub mod borrow_analysis;
pub mod clean_and_optimize;
pub mod dataflow_analysis;
pub mod eliminate_imm_refs;
pub mod eliminate_mut_refs;
pub mod function_target;
pub mod function_target_pipeline;
pub mod graph;
pub mod livevar_analysis;
pub mod memory_instrumentation;
pub mod reaching_def_analysis;
pub mod stackless_bytecode;
pub mod stackless_bytecode_generator;
pub mod stackless_control_flow_graph;
pub mod test_instrumenter;
pub mod usage_analysis;

#[cfg(test)]
pub mod unit_tests;

/// Print function targets for testing and debugging.
pub fn print_targets_for_test(
    env: &GlobalEnv,
    header: &str,
    targets: &FunctionTargetsHolder,
) -> String {
    let mut text = String::new();
    text.push_str(&format!("============ {} ================\n", header));
    for module_env in env.get_modules() {
        for func_env in module_env.get_functions() {
            let target = targets.get_target(&func_env);
            target.register_annotation_formatters_for_test();
            text += &format!("\n{}\n", target);
        }
    }
    text
}

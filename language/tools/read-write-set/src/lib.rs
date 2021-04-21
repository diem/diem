// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_binary_format::file_format::CompiledModule;
use move_core_types::{identifier::Identifier, language_storage::ModuleId};
use move_model::model::{FunctionEnv, GlobalEnv};
use prover_bytecode::{
    function_target_pipeline::{FunctionTargetPipeline, FunctionTargetsHolder, FunctionVariant},
    read_write_set_analysis::{ReadWriteSetProcessor, ReadWriteSetState},
};

pub struct ReadWriteSet {
    targets: FunctionTargetsHolder,
    env: GlobalEnv,
}

/// Infer read/write set results for `modules`.
/// The `modules` list must be topologically sorted by the dependency relation
/// (i.e., a child node in the dependency graph should appear earlier in the
/// vector than its parents), and all dependencies of each module must be
/// included.
pub fn analyze<'a>(modules: impl IntoIterator<Item = &'a CompiledModule>) -> Result<ReadWriteSet> {
    let env = move_model::run_bytecode_model_builder(modules)?;
    let mut pipeline = FunctionTargetPipeline::default();
    pipeline.add_processor(ReadWriteSetProcessor::new());
    let mut targets = FunctionTargetsHolder::default();
    for module_env in env.get_modules() {
        for func_env in module_env.get_functions() {
            targets.add_target(&func_env)
        }
    }
    pipeline.run(&env, &mut targets);

    Ok(ReadWriteSet { targets, env })
}

impl ReadWriteSet {
    /// Return the read/write set for `module`::`fun`.
    /// Returns `None` if the read/write set does not exist.
    pub fn get(&self, module: &ModuleId, fun: &Identifier) -> Option<&ReadWriteSetState> {
        self.get_function_env(module, fun)
            .map(|fenv| {
                self.targets
                    .get_data(&fenv.get_qualified_id(), &FunctionVariant::Baseline)
                    .map(|data| data.annotations.get::<ReadWriteSetState>())
                    .flatten()
            })
            .flatten()
    }

    /// Returns the FunctionEnv for `module`::`fun`
    /// Returns `None` if this function does not exist
    pub fn get_function_env(&self, module: &ModuleId, fun: &Identifier) -> Option<FunctionEnv> {
        self.env
            .find_function_by_language_storage_id_name(module, fun)
    }
}

// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    function_target::{FunctionTarget, FunctionTargetData},
    stackless_bytecode_generator::StacklessBytecodeGenerator,
};
use itertools::Itertools;
use spec_lang::env::{FunId, FunctionEnv, GlobalEnv, ModuleId};
use std::collections::BTreeMap;

/// A globally unique key to identify a function.
type FunKey = (ModuleId, FunId);

/// A data structure which holds data for multiple function targets, and allows to
/// manipulate them as part of a transformation pipeline.
#[derive(Debug, Default)]
pub struct FunctionTargetsHolder {
    targets: BTreeMap<FunKey, FunctionTargetData>,
}

/// A trait for processing a function target. Takes as parameter a target holder which can be
/// mutated, the env of the function being processed, and the target data. During the time
/// the processor is called, the target data is removed from the holder, and added back once
/// transformation has finished. This allows the processor to take ownership
/// on the target data. Notice that you can use `FunctionTarget{func_env, &data}` to temporarily
/// construct a function target for access to the underlying data.
pub trait FunctionTargetProcessor {
    fn process(
        &self,
        targets: &mut FunctionTargetsHolder,
        func_env: &FunctionEnv<'_>,
        data: FunctionTargetData,
    ) -> FunctionTargetData;
}

/// A processing pipeline for function targets.
#[derive(Default)]
pub struct FunctionTargetPipeline {
    processors: Vec<Box<dyn FunctionTargetProcessor>>,
}

impl FunctionTargetsHolder {
    /// Adds a new function target. The target will be initialized from the Move byte code.
    pub fn add_target(&mut self, func_env: &FunctionEnv<'_>) {
        let generator = StacklessBytecodeGenerator::new(func_env);
        let data = generator.generate_function();
        self.targets
            .insert((func_env.module_env.get_id(), func_env.get_id()), data);
    }

    /// Gets a function target for read-only consumption.
    pub fn get_target<'env>(&'env self, func_env: &'env FunctionEnv<'env>) -> FunctionTarget<'env> {
        let data = self
            .targets
            .get(&(func_env.module_env.get_id(), func_env.get_id()))
            .expect("function target exists");
        FunctionTarget::new(func_env, data)
    }

    /// Processes the function target data for given function.
    fn process(&mut self, func_env: &FunctionEnv<'_>, processor: &dyn FunctionTargetProcessor) {
        let key = (func_env.module_env.get_id(), func_env.get_id());
        let data = self.targets.remove(&key).expect("function target exists");
        let processed_data = processor.process(self, func_env, data);
        self.targets.insert(key, processed_data);
    }
}

impl FunctionTargetPipeline {
    /// Adds a processor to this pipeline. Processor will be called in the order they have been
    /// added.
    pub fn add_processor(&mut self, processor: Box<dyn FunctionTargetProcessor>) {
        self.processors.push(processor)
    }

    /// Runs the pipeline on all functions in the targets holder. This happens in breadth-first
    /// fashion, i.e. a processor can expect that processors preceding it in the pipeline have been
    /// executed for all functions before it is called.
    pub fn run(&self, env: &GlobalEnv, targets: &mut FunctionTargetsHolder) {
        let keys = targets.targets.keys().cloned().collect_vec();
        for processor in &self.processors {
            for (mid, fid) in &keys {
                let func_env = env.get_module(*mid).into_function(*fid);
                targets.process(&func_env, processor.as_ref());
            }
        }
    }
}

// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    function_target::{FunctionTarget, FunctionTargetData},
    print_targets_for_test,
    stackless_bytecode_generator::StacklessBytecodeGenerator,
};
use log::debug;
use spec_lang::env::{FunId, FunctionEnv, GlobalEnv, QualifiedId};
use std::{collections::BTreeMap, fs};

/// A data structure which holds data for multiple function targets, and allows to
/// manipulate them as part of a transformation pipeline.
#[derive(Debug, Default)]
pub struct FunctionTargetsHolder {
    targets: BTreeMap<QualifiedId<FunId>, FunctionTargetData>,
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

    /// Returns a name for this processor. This should be suitable as a file suffix.
    fn name(&self) -> String;
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
        self.targets.insert(func_env.get_qualified_id(), data);
    }

    /// Gets a function target for read-only consumption.
    pub fn get_target<'env>(&'env self, func_env: &'env FunctionEnv<'env>) -> FunctionTarget<'env> {
        let data = self
            .targets
            .get(&func_env.get_qualified_id())
            .expect("function target exists");
        FunctionTarget::new(func_env, data)
    }

    /// Processes the function target data for given function.
    fn process(&mut self, func_env: &FunctionEnv<'_>, processor: &dyn FunctionTargetProcessor) {
        let key = func_env.get_qualified_id();
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

    /// Runs the pipeline on all functions in the targets holder. Functions are analyzed in
    /// topological order, so a caller can guarantee that its callees have already been analyzed
    /// in programs without recursion or mutual recursion. Processors are run on an individual
    /// function in breadth-first fashion; i.e. a processor can expect that processors preceding it
    /// in the pipeline have been executed for all functions before it is called.
    pub fn run(
        &self,
        env: &GlobalEnv,
        targets: &mut FunctionTargetsHolder,
        dump_to_file: Option<String>,
    ) {
        let mut worklist = vec![];
        for (key, data) in &targets.targets {
            worklist.push((*key, data.get_callees()))
        }
        let mut to_remove = vec![];
        let mut topological_order = vec![];
        // analyze bottom-up from the leaves of the call graph
        loop {
            let last = worklist.last();
            if last.is_none() {
                break;
            }
            // front of the worklist has a nonempty list of callees to analyze. walk through the
            // worklist and remove the analyzed callees from `to_remove` from each entry in the
            // worklist.
            if !last.unwrap().1.is_empty() {
                for (_f, f_callees) in &mut worklist {
                    for f in &to_remove {
                        f_callees.retain(|e| e != f);
                    }
                }
                // Put functions with 0 calls first in line, at the end of the vector
                worklist
                    .sort_by(|(_, callees1), (_, callees2)| callees2.len().cmp(&callees1.len()));
            }
            let (call_id, callees) = worklist.pop().unwrap();
            // At this point, one of two things is true:
            // 1. callees is empty (common case)
            // 2. callees is nonempty and mid is part of a recursive or mutually recursive
            //    intra-module call cycle (possible in theory, but doesn't happen in the current
            //    implementation of the Diem framework).
            to_remove.push(call_id);
            let func_env = env.get_function(call_id);
            if !callees.is_empty() {
                // The right long-term thing to do here is to allow analysis in case (2) and ask the
                // analysis processors to deal gracefully with the absence of summaries. But for
                // now, we intentionally fail because recursion is not expected in Diem Framework
                // code
                unimplemented!("Recursion or mutual recursion detected in {:?}. Make sure that all analyses in  self.processors are prepared to handle recursion", func_env.get_identifier());
            }
            topological_order.push(func_env);
        }

        // Now that we determined the topological order, run each processor on it, breadth-first.
        let dump_to_file = |step_count: usize, name: &str, targets: &FunctionTargetsHolder| {
            // Dump result to file if requested
            if let Some(base_name) = &dump_to_file {
                let dump =
                    print_targets_for_test(env, &format!("after processor `{}`", name), targets);
                let file_name = format!("{}_{}_{}.bytecode", base_name, step_count, name);
                debug!("dumping bytecode to `{}`", file_name);
                fs::write(&file_name, &dump).expect("dumping bytecode");
            }
        };
        dump_to_file(0, "stackless", targets);
        for (step_count, processor) in self.processors.iter().enumerate() {
            for func_env in &topological_order {
                targets.process(func_env, processor.as_ref());
            }
            dump_to_file(step_count + 1, &processor.name(), targets);
        }
    }
}

// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    function_target::{FunctionData, FunctionTarget},
    print_targets_for_test,
    stackless_bytecode_generator::StacklessBytecodeGenerator,
};
use itertools::Itertools;
use log::debug;
use move_model::model::{FunId, FunctionEnv, GlobalEnv, QualifiedId};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Formatter,
    fs,
};

/// A data structure which holds data for multiple function targets, and allows to
/// manipulate them as part of a transformation pipeline.
#[derive(Debug, Default)]
pub struct FunctionTargetsHolder {
    targets: BTreeMap<QualifiedId<FunId>, BTreeMap<FunctionVariant, FunctionData>>,
}

/// Describes a function target variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FunctionVariant {
    /// The baseline variant which was created from the original Move bytecode and is then
    /// subject of multiple transformations.
    Baseline,
    /// The variant which is instrumented for verification. Only functions which are target
    /// of verification have one of those.
    Verification,
}

impl std::fmt::Display for FunctionVariant {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use FunctionVariant::*;
        match self {
            Baseline => write!(f, "baseline"),
            Verification => write!(f, "verification"),
        }
    }
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
        data: FunctionData,
    ) -> FunctionData;

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
    pub fn add_target(&mut self, func_env: &FunctionEnv<'_>, for_v2: bool) {
        let generator = StacklessBytecodeGenerator::new(func_env, for_v2);
        let data = generator.generate_function();
        self.targets
            .entry(func_env.get_qualified_id())
            .or_default()
            .insert(FunctionVariant::Baseline, data);
    }

    /// Gets a function target for read-only consumption, for the given variant.
    pub fn get_target<'env>(
        &'env self,
        func_env: &'env FunctionEnv<'env>,
        variant: FunctionVariant,
    ) -> FunctionTarget<'env> {
        let data = self
            .get_target_data(&func_env.get_qualified_id(), variant)
            .expect("function target exists");
        FunctionTarget::new(func_env, &data)
    }

    /// Gets the function target from the variant which owns the annotations.
    /// TODO(refactoring): the need for this function should be removed once refactoring
    ///    finishes and old boilerplate can be removed.
    pub fn get_annotated_target<'env>(
        &'env self,
        func_env: &'env FunctionEnv<'env>,
    ) -> FunctionTarget<'env> {
        if self
            .get_target_variants(func_env)
            .contains(&FunctionVariant::Verification)
        {
            self.get_target(func_env, FunctionVariant::Verification)
        } else {
            self.get_target(func_env, FunctionVariant::Baseline)
        }
    }

    /// Gets all available variants for function.
    pub fn get_target_variants(&self, func_env: &FunctionEnv<'_>) -> Vec<FunctionVariant> {
        self.targets
            .get(&func_env.get_qualified_id())
            .expect("function targets exist")
            .keys()
            .cloned()
            .collect_vec()
    }

    /// Gets targets for all available variants.
    pub fn get_targets<'env>(
        &'env self,
        func_env: &'env FunctionEnv<'env>,
    ) -> Vec<(FunctionVariant, FunctionTarget<'env>)> {
        self.targets
            .get(&func_env.get_qualified_id())
            .expect("function targets exist")
            .iter()
            .map(|(v, d)| (*v, FunctionTarget::new(func_env, d)))
            .collect_vec()
    }

    /// Gets function data for a variant.
    pub fn get_target_data(
        &self,
        id: &QualifiedId<FunId>,
        variant: FunctionVariant,
    ) -> Option<&FunctionData> {
        self.targets.get(id).and_then(|vs| vs.get(&variant))
    }

    /// Removes function data for a variant.
    pub fn remove_target_data(
        &mut self,
        id: &QualifiedId<FunId>,
        variant: FunctionVariant,
    ) -> FunctionData {
        self.targets
            .get_mut(id)
            .expect("function target exists")
            .remove(&variant)
            .expect("variant exists")
    }

    /// Sets function data for a function's variant.
    pub fn insert_target_data(
        &mut self,
        id: &QualifiedId<FunId>,
        variant: FunctionVariant,
        data: FunctionData,
    ) {
        self.targets.entry(*id).or_default().insert(variant, data);
    }

    /// Processes the function target data for given function.
    fn process(&mut self, func_env: &FunctionEnv<'_>, processor: &dyn FunctionTargetProcessor) {
        let id = func_env.get_qualified_id();
        for variant in self.get_target_variants(func_env) {
            // Remove data so we can own it.
            let data = self.remove_target_data(&id, variant);
            let processed_data = processor.process(self, func_env, data);
            // Put back processed data.
            self.insert_target_data(&id, variant, processed_data);
        }
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
        for (key, variants) in &targets.targets {
            // Use the union of the callees of all variants
            let mut callees = BTreeSet::new();
            for data in variants.values() {
                callees.extend(data.get_callees());
            }
            worklist.push((*key, callees.into_iter().collect_vec()));
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
                unimplemented!(
                    "Recursion or mutual recursion detected in {:?}. \
                     Make sure that all analyses in  self.processors are prepared to handle recursion",
                    func_env.get_identifier()
                );
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

// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    compositional_analysis::{CompositionalAnalysis, SummaryCache},
    dataflow_analysis::{DataflowAnalysis, TransferFunctions},
    dataflow_domains::{AbstractDomain, JoinResult, SetDomain},
    function_target::{FunctionData, FunctionTarget},
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder, FunctionVariant},
    stackless_bytecode::{BorrowNode, Bytecode, Operation, PropKind},
    verification_analysis,
};

use move_binary_format::file_format::CodeOffset;
use move_model::{
    model::{FunctionEnv, GlobalEnv, QualifiedId, QualifiedInstId, StructId},
    ty::Type,
};

use itertools::Itertools;
use move_model::ast::{ConditionKind, Spec};
use paste::paste;
use std::{collections::BTreeSet, fmt, fmt::Formatter};

pub fn get_memory_usage<'env>(target: &FunctionTarget<'env>) -> &'env UsageState {
    target
        .get_annotations()
        .get::<UsageState>()
        .expect("Invariant violation: target not analyzed")
}

/// A summary of the memory accessed / modified per function, both directly and transitively.
#[derive(Default, Clone)]
pub struct MemoryUsage {
    // The memory directly used in the function.
    pub direct: SetDomain<QualifiedInstId<StructId>>,
    // The memory transitively used in either the function itself or at least one of its callees.
    pub transitive: SetDomain<QualifiedInstId<StructId>>,
    // The union of the above sets
    pub all: SetDomain<QualifiedInstId<StructId>>,
}

#[derive(Default, Clone)]
pub struct UsageState {
    /// The memory accessed by this function. This is the union of the three individual fields
    /// below.
    pub accessed: MemoryUsage,
    /// The memory modified by this function.
    pub modified: MemoryUsage,
    /// The memory mentioned by the assume expressions in this function.
    pub assumed: MemoryUsage,
    /// The memory mentioned by the assert expressions in this function.
    pub asserted: MemoryUsage,
}

impl MemoryUsage {
    //
    // setters that insert element(s) to related sets
    //

    fn add_direct(&mut self, mem: QualifiedInstId<StructId>) {
        self.direct.insert(mem.clone());
        self.all.insert(mem);
    }

    fn add_transitive(&mut self, mem: QualifiedInstId<StructId>) {
        self.transitive.insert(mem.clone());
        self.all.insert(mem);
    }

    //
    // accessors that further instantiate the memories
    //

    pub fn get_direct_inst(&self, inst: &[Type]) -> BTreeSet<QualifiedInstId<StructId>> {
        self.direct
            .iter()
            .map(|mem| mem.instantiate_ref(inst))
            .collect()
    }

    pub fn get_transitive_inst(&self, inst: &[Type]) -> BTreeSet<QualifiedInstId<StructId>> {
        self.transitive
            .iter()
            .map(|mem| mem.instantiate_ref(inst))
            .collect()
    }

    pub fn get_all_inst(&self, inst: &[Type]) -> BTreeSet<QualifiedInstId<StructId>> {
        self.all
            .iter()
            .map(|mem| mem.instantiate_ref(inst))
            .collect()
    }

    //
    // accessors that uninstantiate the memories
    //

    pub fn get_direct_uninst(&self) -> BTreeSet<QualifiedId<StructId>> {
        self.direct
            .iter()
            .map(|mem| mem.module_id.qualified(mem.id))
            .collect()
    }

    pub fn get_transitive_uninst(&self) -> BTreeSet<QualifiedId<StructId>> {
        self.transitive
            .iter()
            .map(|mem| mem.module_id.qualified(mem.id))
            .collect()
    }

    pub fn get_all_uninst(&self) -> BTreeSet<QualifiedId<StructId>> {
        self.all
            .iter()
            .map(|mem| mem.module_id.qualified(mem.id))
            .collect()
    }
}

impl AbstractDomain for MemoryUsage {
    fn join(&mut self, other: &Self) -> JoinResult {
        match (
            self.direct.join(&other.direct),
            self.transitive.join(&other.transitive),
            self.all.join(&other.all),
        ) {
            (JoinResult::Unchanged, JoinResult::Unchanged, JoinResult::Unchanged) => {
                JoinResult::Unchanged
            }
            _ => JoinResult::Changed,
        }
    }
}

macro_rules! generate_inserter {
    ($field: ident, $method: ident) => {
        paste! {
            #[allow(dead_code)]
            fn [<$method _ $field>](&mut self, mem: QualifiedInstId<StructId>) {
                self.$field.$method(mem.clone());
                self.accessed.$method(mem);
            }

            #[allow(dead_code)]
            fn [<$method _ $field _iter>](
                &mut self,
                mems: impl Iterator<Item = QualifiedInstId<StructId>>
            ) {
                for mem in mems {
                    self.[<$method _ $field>](mem);
                }
            }
        }
    };
}

/// Generated functions
impl UsageState {
    generate_inserter!(accessed, add_direct);
    generate_inserter!(accessed, add_transitive);

    generate_inserter!(modified, add_direct);
    generate_inserter!(modified, add_transitive);

    generate_inserter!(assumed, add_direct);
    generate_inserter!(assumed, add_transitive);

    generate_inserter!(asserted, add_direct);
    generate_inserter!(asserted, add_transitive);
}

/// Helpers for the abstract interpretation process
impl UsageState {
    fn subsume_callee_as_direct(&mut self, callee: &Self, inst: &[Type]) {
        self.add_direct_accessed_iter(callee.accessed.get_all_inst(inst).into_iter());
        self.add_direct_modified_iter(callee.modified.get_all_inst(inst).into_iter());
        self.add_direct_assumed_iter(callee.assumed.get_all_inst(inst).into_iter());
        self.add_direct_asserted_iter(callee.asserted.get_all_inst(inst).into_iter());
    }

    fn subsume_callee_as_transitive(&mut self, callee: &Self, inst: &[Type]) {
        self.add_transitive_accessed_iter(callee.accessed.get_all_inst(inst).into_iter());
        self.add_transitive_modified_iter(callee.modified.get_all_inst(inst).into_iter());
        self.add_transitive_assumed_iter(callee.assumed.get_all_inst(inst).into_iter());
        self.add_transitive_asserted_iter(callee.asserted.get_all_inst(inst).into_iter());
    }
}

impl AbstractDomain for UsageState {
    fn join(&mut self, other: &Self) -> JoinResult {
        match (
            self.accessed.join(&other.accessed),
            self.modified.join(&other.modified),
            self.assumed.join(&other.assumed),
            self.asserted.join(&other.asserted),
        ) {
            (
                JoinResult::Unchanged,
                JoinResult::Unchanged,
                JoinResult::Unchanged,
                JoinResult::Unchanged,
            ) => JoinResult::Unchanged,
            _ => JoinResult::Changed,
        }
    }
}

struct MemoryUsageAnalysis<'a> {
    cache: SummaryCache<'a>,
}

impl<'a> DataflowAnalysis for MemoryUsageAnalysis<'a> {}

impl<'a> CompositionalAnalysis<UsageState> for MemoryUsageAnalysis<'a> {
    fn to_summary(&self, state: UsageState, _fun_target: &FunctionTarget) -> UsageState {
        state
    }
}

impl<'a> TransferFunctions for MemoryUsageAnalysis<'a> {
    type State = UsageState;
    const BACKWARD: bool = false;

    fn execute(&self, state: &mut Self::State, code: &Bytecode, _offset: CodeOffset) {
        use Bytecode::*;
        use Operation::*;
        use PropKind::*;

        match code {
            // memory accesses in operations
            Call(_, _, oper, _, _) => match oper {
                Function(mid, fid, inst)
                | OpaqueCallBegin(mid, fid, inst)
                | OpaqueCallEnd(mid, fid, inst) => {
                    let callee_id = mid.qualified(*fid);
                    if let Some(summary) = self
                        .cache
                        .get::<UsageState>(callee_id, &FunctionVariant::Baseline)
                    {
                        let callee_env = self.cache.global_env().get_function(callee_id);
                        if verification_analysis::is_invariant_checking_delegated(&callee_env) {
                            // NOTE: memories touched by callees, including opaque callees, are
                            // considered as directly touched by the caller. We need this info in
                            // global invariant instrumentation to calculate
                            // 1) the correct set of invariants applicable (assumed/asserted) at the
                            //    caller function, and
                            // 2) the complete set of instantiations for each of the applicable
                            //    invariant.
                            //
                            // An alternative is to keep these memory as transitive for now,
                            // duplicate this information in invariant analysis and mark these
                            // memory as direct in the duplicated copy. Given currently the global
                            // invariant instrumentation pipeline is the only client of the
                            // usage analysis pass, we encode the special treatment up here.
                            state.subsume_callee_as_direct(summary, inst);
                        } else {
                            state.subsume_callee_as_transitive(summary, inst);
                        }
                    }
                }
                MoveTo(mid, sid, inst)
                | MoveFrom(mid, sid, inst)
                | BorrowGlobal(mid, sid, inst) => {
                    let mem = mid.qualified_inst(*sid, inst.to_owned());
                    state.add_direct_modified(mem);
                }
                WriteBack(BorrowNode::GlobalRoot(mem), _) => {
                    state.add_direct_modified(mem.clone());
                }
                Exists(mid, sid, inst) | GetGlobal(mid, sid, inst) => {
                    let mem = mid.qualified_inst(*sid, inst.to_owned());
                    state.add_direct_accessed(mem);
                }
                _ => {}
            },
            // memory accesses in expressions
            Prop(_, kind, exp) => match kind {
                Assume => state.add_direct_assumed_iter(
                    exp.used_memory(self.cache.global_env())
                        .into_iter()
                        .map(|(usage, _)| usage),
                ),
                Assert => state.add_direct_asserted_iter(
                    exp.used_memory(self.cache.global_env())
                        .into_iter()
                        .map(|(usage, _)| usage),
                ),
                Modifies => {
                    unreachable!("`modifies` expressions are not expected in the function body")
                }
            },
            _ => {}
        }
    }
}

impl<'a> MemoryUsageAnalysis<'a> {
    /// Compute usage information for the given spec. This spec is injected in later
    /// phases into the code, but we need to account for it's memory usage already here
    /// as spec injection itself depends on this information.
    fn compute_spec_usage(&self, spec: &Spec, state: &mut UsageState) {
        use ConditionKind::*;
        for cond in &spec.conditions {
            let mut used_memory = cond.exp.used_memory(self.cache.global_env());
            for exp in &cond.additional_exps {
                used_memory.extend(exp.used_memory(self.cache.global_env()));
            }
            match &cond.kind {
                Ensures | AbortsIf | Emits => {
                    state.add_direct_asserted_iter(used_memory.into_iter().map(|(usage, _)| usage));
                }
                _ => {
                    state.add_direct_assumed_iter(used_memory.into_iter().map(|(usage, _)| usage));
                }
            }
            if matches!(cond.kind, Update) {
                // Add target of spec update to modified memory
                if let Some((mem, _, _)) =
                    cond.additional_exps[0].extract_ghost_mem_access(self.cache.global_env())
                {
                    state.add_direct_modified(mem);
                }
            }
        }
    }
}

pub struct UsageProcessor();

impl UsageProcessor {
    pub fn new() -> Box<Self> {
        Box::new(UsageProcessor())
    }
}

impl FunctionTargetProcessor for UsageProcessor {
    fn process(
        &self,
        targets: &mut FunctionTargetsHolder,
        func_env: &FunctionEnv<'_>,
        mut data: FunctionData,
    ) -> FunctionData {
        let func_target = FunctionTarget::new(func_env, &data);
        let cache = SummaryCache::new(targets, func_env.module_env.env);
        let analysis = MemoryUsageAnalysis { cache };
        let mut summary = analysis.summarize(&func_target, UsageState::default());
        analysis.compute_spec_usage(func_env.get_spec(), &mut summary);
        data.annotations.set(summary);
        data
    }

    fn name(&self) -> String {
        "usage_analysis".to_string()
    }

    fn dump_result(
        &self,
        f: &mut Formatter<'_>,
        env: &GlobalEnv,
        targets: &FunctionTargetsHolder,
    ) -> fmt::Result {
        writeln!(f, "\n\n********* Result of usage analysis *********\n\n")?;
        for module in env.get_modules() {
            if !module.is_target() {
                continue;
            }
            for fun in module.get_functions() {
                for (_, ref target) in targets.get_targets(&fun) {
                    let usage = get_memory_usage(target);
                    writeln!(
                        f,
                        "function {} [{}] {{",
                        target.func_env.get_full_name_str(),
                        target.data.variant
                    )?;

                    let mut print_usage = |set: &MemoryUsage, name: &str| -> fmt::Result {
                        writeln!(
                            f,
                            "  {} = {{{}}}",
                            name,
                            set.all
                                .iter()
                                .map(|qid| env.display(qid).to_string())
                                .join(", ")
                        )?;
                        writeln!(
                            f,
                            "  directly {} = {{{}}}",
                            name,
                            set.direct
                                .iter()
                                .map(|qid| env.display(qid).to_string())
                                .join(", ")
                        )
                    };

                    print_usage(&usage.accessed, "accessed")?;
                    print_usage(&usage.modified, "modified")?;
                    print_usage(&usage.assumed, "assumed")?;
                    print_usage(&usage.asserted, "asserted")?;

                    writeln!(f, "}}")?;
                }
            }
        }
        writeln!(f)?;
        Ok(())
    }
}

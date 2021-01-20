// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Reaching definition analysis with subsequent copy propagation.
//
// This analysis and transformation only propagates definitions, leaving dead assignments
// in the code. The subsequent livevar_analysis takes care of removing those.

use crate::{
    dataflow_analysis::{AbstractDomain, DataflowAnalysis, JoinResult, TransferFunctions},
    function_target::{FunctionData, FunctionTarget},
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder},
    stackless_bytecode::{BorrowNode, Bytecode, Operation, TempIndex},
    stackless_control_flow_graph::StacklessControlFlowGraph,
};
use itertools::Itertools;
use move_model::model::FunctionEnv;
use std::collections::{BTreeMap, BTreeSet};
use vm::file_format::CodeOffset;

/// The reaching definitions we are capturing. Currently we only capture
/// aliases (assignment).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Def {
    Alias(TempIndex),
}

/// The annotation for reaching definitions. For each code position, we have a map of local
/// indices to the set of definitions reaching the code position.
#[derive(Default)]
pub struct ReachingDefAnnotation(BTreeMap<CodeOffset, BTreeMap<TempIndex, BTreeSet<Def>>>);

pub struct ReachingDefProcessor {
    /// If true, user locals will not be renamed during copy propagation.
    preserve_user_locals: bool,
}

type DefMap = BTreeMap<TempIndex, BTreeSet<Def>>;

impl ReachingDefProcessor {
    pub fn new() -> Box<Self> {
        Box::new(ReachingDefProcessor {
            preserve_user_locals: true,
        })
    }

    pub fn new_no_preserve_user_locals() -> Box<Self> {
        Box::new(ReachingDefProcessor {
            preserve_user_locals: false,
        })
    }

    /// Returns Some(temp, def) if temp has a unique reaching definition and None otherwise.
    fn get_unique_def(temp: TempIndex, defs: &BTreeSet<Def>) -> Option<(TempIndex, TempIndex)> {
        if defs.len() != 1 {
            return None;
        }
        let Def::Alias(def) = defs.iter().next().unwrap();
        Some((temp, *def))
    }

    /// Gets the propagated local resolving aliases using the reaching definitions.
    fn get_propagated_local(temp: TempIndex, reaching_defs: &DefMap) -> TempIndex {
        // For being robust, we protect this function against cycles in alias definitions. If
        // a cycle is detected, alias resolution stops.
        fn get(
            temp: TempIndex,
            reaching_defs: &DefMap,
            visited: &mut BTreeSet<TempIndex>,
        ) -> TempIndex {
            if let Some(defs) = reaching_defs.get(&temp) {
                if let Some((_, def_temp)) = ReachingDefProcessor::get_unique_def(temp, defs) {
                    if visited.insert(def_temp) {
                        return get(def_temp, reaching_defs, visited);
                    }
                }
            }
            temp
        }
        let mut visited = BTreeSet::new();
        get(temp, reaching_defs, &mut visited)
    }

    /// Perform copy propagation based on reaching definitions analysis results.
    pub fn copy_propagation(
        target: &FunctionTarget<'_>,
        code: Vec<Bytecode>,
        defs: &ReachingDefAnnotation,
    ) -> Vec<Bytecode> {
        let mut res = vec![];
        for (pc, bytecode) in code.into_iter().enumerate() {
            let no_defs = BTreeMap::new();
            let reaching_defs = defs.0.get(&(pc as CodeOffset)).unwrap_or(&no_defs);
            let mut propagate = |local| Self::get_propagated_local(local, reaching_defs);
            res.push(bytecode.remap_src_vars(target, &mut propagate));
        }
        res
    }

    /// Compute the set of locals which are borrowed from. We can't alias such locals
    /// to other locals because of reference semantics.
    fn borrowed_locals(&self, code: &[Bytecode]) -> BTreeSet<TempIndex> {
        use Bytecode::*;
        code.iter()
            .filter_map(|bc| {
                if let Call(_, _, Operation::BorrowLoc, srcs) = bc {
                    Some(srcs[0])
                } else {
                    None
                }
            })
            .collect()
    }

    /// Determines whether code is suitable for copy propagation. Currently we cannot
    /// do this for code with embedded spec blocks, because those refer to locals
    /// which might be substituted via copy propagation.
    /// TODO(wrwg): verify that spec blocks are the actual cause, it could be also a bug elsewhere.
    ///     Currently functional/verify_vector fails without this and it uses spec blocks all
    ///     over the place.
    fn suitable_for_copy_propagation(&self, code: &[Bytecode]) -> bool {
        !code.iter().any(|bc| matches!(bc, Bytecode::SpecBlock(..)))
    }
}

impl FunctionTargetProcessor for ReachingDefProcessor {
    fn process(
        &self,
        _targets: &mut FunctionTargetsHolder,
        func_env: &FunctionEnv<'_>,
        mut data: FunctionData,
    ) -> FunctionData {
        if func_env.is_native() || !self.suitable_for_copy_propagation(&data.code) {
            // Nothing to do
            data
        } else {
            let cfg = StacklessControlFlowGraph::new_forward(&data.code);
            let analyzer = ReachingDefAnalysis {
                target: FunctionTarget::new(func_env, &data),
                preserve_user_locals: self.preserve_user_locals,
                borrowed_locals: self.borrowed_locals(&data.code),
            };
            let block_state_map = analyzer.analyze_function(
                ReachingDefState {
                    map: BTreeMap::new(),
                },
                &data.code,
                &cfg,
            );
            let defs =
                analyzer.state_per_instruction(block_state_map, &data.code, &cfg, |before, _| {
                    before.map.clone()
                });

            // Run copy propagation transformation.
            let annotations = ReachingDefAnnotation(defs);
            let code = std::mem::take(&mut data.code);
            let target = FunctionTarget::new(func_env, &data);
            let new_code = Self::copy_propagation(&target, code, &annotations);
            data.code = new_code;

            // Currently we do not need reaching defs after this phase. If so in the future, we
            // need to uncomment this statement.
            //data.annotations.set(annotations);
            data
        }
    }

    fn name(&self) -> String {
        "reaching_def_analysis".to_string()
    }
}

struct ReachingDefAnalysis<'a> {
    target: FunctionTarget<'a>,
    preserve_user_locals: bool,
    borrowed_locals: BTreeSet<TempIndex>,
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd)]
struct ReachingDefState {
    map: BTreeMap<TempIndex, BTreeSet<Def>>,
}

impl<'a> ReachingDefAnalysis<'a> {}

impl<'a> TransferFunctions for ReachingDefAnalysis<'a> {
    type State = ReachingDefState;
    const BACKWARD: bool = false;

    fn execute(&self, state: &mut ReachingDefState, instr: &Bytecode, _offset: CodeOffset) {
        use BorrowNode::*;
        use Bytecode::*;
        use Operation::*;
        match instr {
            Assign(_, dest, src, _) => {
                state.kill(*dest);
                // On `self.preserve_user_locals`, only define aliases for temporaries.
                // Also don't alias proxied parameters. The later is currently needed because the
                // Boogie backend does not allow to write to such values, which happens via
                // WriteBack instructions.
                // TODO(refactoring): this can be removed once the old boogie backend is retired,
                //   as in the new world, we emit `trace_local` instructions before this phase,
                //   and this way remember user local names.
                if !self.borrowed_locals.contains(dest)
                    && (!self.preserve_user_locals
                        || self.target.is_temporary(*dest)
                            && self.target.get_proxy_index(*src).is_none())
                {
                    state.def_alias(*dest, *src);
                }
            }
            Load(_, dest, ..) => {
                state.kill(*dest);
            }
            Call(_, dests, oper, ..) => {
                if let WriteBack(LocalRoot(dest)) = oper {
                    state.kill(*dest);
                }
                for dest in dests {
                    state.kill(*dest);
                }
            }
            OnAbort(_, _, code_dest) => state.kill(*code_dest),
            _ => {}
        }
    }
}

impl<'a> DataflowAnalysis for ReachingDefAnalysis<'a> {}

impl AbstractDomain for ReachingDefState {
    fn join(&mut self, other: &Self) -> JoinResult {
        let mut result = JoinResult::Unchanged;
        for idx in self.map.keys().cloned().collect_vec() {
            if let Some(other_defs) = other.map.get(&idx) {
                // Union of definitions
                let defs = self.map.get_mut(&idx).unwrap();
                for d in other_defs {
                    if defs.insert(d.clone()) {
                        result = JoinResult::Changed;
                    }
                }
            } else {
                // Kill this definition as it is not contained in both incoming states.
                self.map.remove(&idx);
                result = JoinResult::Changed;
            }
        }
        result
    }
}

impl ReachingDefState {
    fn def_alias(&mut self, dest: TempIndex, src: TempIndex) {
        let set = self.map.entry(dest).or_insert_with(BTreeSet::new);
        // Kill previous definitions.
        set.clear();
        set.insert(Def::Alias(src));
    }

    fn kill(&mut self, dest: TempIndex) {
        self.map.remove(&dest);
    }
}

// =================================================================================================
// Formatting

/// Format a reaching definition annotation.
pub fn format_reaching_def_annotation(
    target: &FunctionTarget<'_>,
    code_offset: CodeOffset,
) -> Option<String> {
    if let Some(ReachingDefAnnotation(map)) =
        target.get_annotations().get::<ReachingDefAnnotation>()
    {
        if let Some(map_at) = map.get(&code_offset) {
            let mut res = map_at
                .iter()
                .map(|(idx, defs)| {
                    let name = target.get_local_name(*idx);
                    format!(
                        "{} -> {{{}}}",
                        name.display(target.symbol_pool()),
                        defs.iter()
                            .map(|def| {
                                match def {
                                    Def::Alias(a) => format!(
                                        "{}",
                                        target.get_local_name(*a).display(target.symbol_pool())
                                    ),
                                }
                            })
                            .join(", ")
                    )
                })
                .join(", ");
            res.insert_str(0, "reach: ");
            return Some(res);
        }
    }
    None
}

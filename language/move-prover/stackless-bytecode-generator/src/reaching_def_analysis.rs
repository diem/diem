// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    dataflow_analysis::{AbstractDomain, DataflowAnalysis, JoinResult, TransferFunctions},
    function_target::{FunctionTarget, FunctionTargetData},
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder},
    stackless_bytecode::{Bytecode, Constant, TempIndex},
    stackless_control_flow_graph::{BlockId, StacklessControlFlowGraph},
};
use itertools::Itertools;
use spec_lang::env::FunctionEnv;
use std::collections::{BTreeMap, BTreeSet};
use vm::file_format::CodeOffset;

/// The definitions we are capturing. Currently we only capture constants
/// and aliases (assignment).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Def {
    Const(Constant),
    Alias(TempIndex),
}

/// The annotation for reaching definitions. For each code position, we have a map of local
/// indices to the set of definitions reaching the code position.
#[derive(Default)]
pub struct ReachingDefAnnotation(BTreeMap<CodeOffset, BTreeMap<TempIndex, BTreeSet<Def>>>);

pub struct ReachingDefProcessor();

type DefMap = BTreeMap<TempIndex, BTreeSet<Def>>;
impl ReachingDefProcessor {
    pub fn new() -> Box<Self> {
        Box::new(ReachingDefProcessor())
    }

    /// Union all the reaching definitions of a temp.
    fn union_defs(one: &mut DefMap, other: &DefMap) {
        for (temp, defs) in other {
            one.entry(*temp)
                .and_modify(|e| {
                    e.extend(defs.clone());
                })
                .or_insert_with(|| defs.clone());
        }
    }

    /// Returns Some(temp, def) if temp has a unique reaching definition and None otherwise.
    fn get_unique_def(temp: TempIndex, defs: BTreeSet<Def>) -> Option<(TempIndex, TempIndex)> {
        if defs.len() != 1 {
            return None;
        }
        if let Def::Alias(def) = defs.iter().next().unwrap() {
            return Some((temp, *def));
        }
        None
    }

    fn defs_reaching_exits(
        exit_offsets: Vec<CodeOffset>,
        annotations: &BTreeMap<CodeOffset, DefMap>,
        func_target: FunctionTarget,
    ) -> BTreeMap<TempIndex, TempIndex> {
        let mut res = BTreeMap::new();
        let user_local_count = func_target.get_user_local_count();
        for offset in exit_offsets {
            if let Some(defs) = annotations.get(&offset) {
                Self::union_defs(&mut res, &defs);
            }
        }
        res.into_iter()
            .filter_map(|(temp, defs)| Self::get_unique_def(temp, defs))
            .filter(|(t, _)| {
                *t >= user_local_count && !func_target.get_local_type(*t).is_reference()
            }) // keeping only newly added temps
            .collect()
    }

    fn get_src_local(idx: TempIndex, reaching_defs: &BTreeMap<TempIndex, TempIndex>) -> TempIndex {
        let mut res = idx;
        while reaching_defs.contains_key(&res) {
            res = *reaching_defs.get(&res).unwrap();
            if res == idx {
                break;
            }
        }
        res
    }

    /// Eliminate assignments of temps if they reach the return of a function
    /// and replace temps accordingly. For example, if assignment 'a=b' reaches
    /// the end of the function, then it will be eliminated and all appearances
    /// of a will be replaced by b.
    pub fn transform_code(
        code: &[Bytecode],
        defs: &ReachingDefAnnotation,
        func_target: FunctionTarget,
    ) -> Vec<Bytecode> {
        use Bytecode::*;
        let exit_offsets = Bytecode::get_exits(code);
        let user_local_count = func_target.get_user_local_count();
        let reaching_defs = Self::defs_reaching_exits(exit_offsets, &defs.0, func_target);
        let mut res = vec![];
        let transform_local = |local| Self::get_src_local(local, &reaching_defs);
        for bytecode in code {
            match bytecode {
                Assign(attr, dest, src, a_kind) => {
                    if *dest < user_local_count
                        || !reaching_defs.contains_key(dest)
                        || reaching_defs.get(dest).unwrap() != src
                    {
                        res.push(Assign(*attr, *dest, transform_local(*src), *a_kind));
                    }
                }
                Call(attr, dests, op, srcs) => {
                    let transformed_dests = dests.iter().map(|d| transform_local(*d)).collect();
                    let transformed_srcs = srcs.iter().map(|s| transform_local(*s)).collect();
                    res.push(Call(*attr, transformed_dests, op.clone(), transformed_srcs));
                }
                Ret(attr, rets) => {
                    let transformed_rets = rets.iter().map(|r| transform_local(*r)).collect();
                    res.push(Ret(*attr, transformed_rets));
                }

                Branch(attr, if_label, else_label, cond) => {
                    res.push(Branch(
                        *attr,
                        *if_label,
                        *else_label,
                        transform_local(*cond),
                    ));
                }

                Abort(attr, cond) => {
                    res.push(Abort(*attr, transform_local(*cond)));
                }
                _ => {
                    res.push(bytecode.clone());
                }
            }
        }
        res
    }
}

impl FunctionTargetProcessor for ReachingDefProcessor {
    fn process(
        &self,
        _targets: &mut FunctionTargetsHolder,
        func_env: &FunctionEnv<'_>,
        mut data: FunctionTargetData,
    ) -> FunctionTargetData {
        let defs = if func_env.is_native() {
            BTreeMap::new()
        } else {
            let cfg = StacklessControlFlowGraph::new_forward(&data.code);
            let mut analyzer = ReachingDefAnalysis {
                _target: FunctionTarget::new(func_env, &data),
            };
            let mut block_state_map = analyzer.analyze_function(
                ReachingDefState {
                    map: BTreeMap::new(),
                },
                &data.code,
                &cfg,
            );
            // Convert the block state map into a map for each code offset. We achieve this by
            // replaying execution of the instructions of each individual block, based on the pre
            // state. Because the analyzes converged to a fixpoint, the result is sound.
            let mut res = BTreeMap::new();
            for block_id in cfg.blocks() {
                let mut state = block_state_map.remove(&block_id).expect("basic block").pre;
                for code_offset in cfg.instr_indexes(block_id) {
                    res.insert(code_offset, state.map.clone());
                    state = analyzer.execute(state, &data.code[code_offset as usize], code_offset);
                }
            }
            res
        };
        let annotations = ReachingDefAnnotation(defs);
        let func_target = FunctionTarget::new(func_env, &data);
        let transformed_code = Self::transform_code(&data.code, &annotations, func_target);
        data.code = transformed_code;
        data.annotations.set(annotations);
        data
    }
}

struct ReachingDefAnalysis<'a> {
    _target: FunctionTarget<'a>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct ReachingDefState {
    map: BTreeMap<TempIndex, BTreeSet<Def>>,
}

impl<'a> ReachingDefAnalysis<'a> {
    fn execute(
        &mut self,
        pre: ReachingDefState,
        instr: &Bytecode,
        _idx: CodeOffset,
    ) -> ReachingDefState {
        use Bytecode::*;
        let mut post = pre;
        match instr {
            Assign(_, dst, src, _) => {
                post.def_alias(*dst, *src);
            }
            Load(_, dst, cons) => {
                post.def_const(*dst, cons.clone());
            }
            Call(_, dsts, ..) => {
                for dst in dsts {
                    post.kill(*dst);
                }
            }
            _ => {}
        }
        post
    }
}

impl<'a> TransferFunctions for ReachingDefAnalysis<'a> {
    type State = ReachingDefState;
    type AnalysisError = ();

    fn execute_block(
        &mut self,
        block_id: BlockId,
        pre_state: Self::State,
        instrs: &[Bytecode],
        cfg: &StacklessControlFlowGraph,
    ) -> Result<Self::State, Self::AnalysisError> {
        let mut state = pre_state;
        for offset in cfg.instr_indexes(block_id) {
            let instr = &instrs[offset as usize];
            state = self.execute(state, instr, offset);
        }
        Ok(state)
    }
}

impl<'a> DataflowAnalysis for ReachingDefAnalysis<'a> {}

impl AbstractDomain for ReachingDefState {
    fn join(&mut self, other: &Self) -> JoinResult {
        let mut result = JoinResult::Unchanged;
        for (idx, other_defs) in &other.map {
            if !self.map.contains_key(idx) {
                self.map.insert(*idx, other_defs.clone());
                result = JoinResult::Changed;
            } else {
                let defs = self.map.get_mut(idx).unwrap();
                for d in other_defs {
                    if defs.insert(d.clone()) {
                        result = JoinResult::Changed;
                    }
                }
            }
        }
        result
    }
}

impl ReachingDefState {
    fn def_alias(&mut self, dest: TempIndex, src: TempIndex) {
        let set = self.map.entry(dest).or_insert_with(BTreeSet::new);
        set.clear();
        set.insert(Def::Alias(src));
    }

    fn def_const(&mut self, dest: TempIndex, cons: Constant) {
        let set = self.map.entry(dest).or_insert_with(BTreeSet::new);
        set.clear();
        set.insert(Def::Const(cons));
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
                                    Def::Const(c) => format!("{}", c),
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

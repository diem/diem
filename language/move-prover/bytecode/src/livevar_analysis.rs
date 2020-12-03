// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Live variable analysis with subsequent dead assignment elimination and
// computation of new Destroy instructions.

use crate::{
    dataflow_analysis::{AbstractDomain, DataflowAnalysis, JoinResult, TransferFunctions},
    function_target::{FunctionTarget, FunctionTargetData},
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder},
    stackless_bytecode::{AttrId, Bytecode, Label, Operation, TempIndex},
    stackless_control_flow_graph::StacklessControlFlowGraph,
};
use itertools::Itertools;
use spec_lang::{env::FunctionEnv, ty::Type};
use std::collections::{BTreeMap, BTreeSet};
use vm::file_format::CodeOffset;

/// The annotation for live variable analysis. For each code position, we have a set of local
/// variable indices that are live just before the code offset, i.e. these variables are used
/// before being overwritten.
#[derive(Debug, Default)]
pub struct LiveVarInfoAtCodeOffset {
    pub before: BTreeSet<TempIndex>,
    pub after: BTreeSet<TempIndex>,
}

#[derive(Default)]
pub struct LiveVarAnnotation(BTreeMap<CodeOffset, LiveVarInfoAtCodeOffset>);

impl LiveVarAnnotation {
    pub fn get_live_var_info_at(
        &self,
        code_offset: CodeOffset,
    ) -> Option<&LiveVarInfoAtCodeOffset> {
        self.0.get(&code_offset)
    }
}

pub struct LiveVarAnalysisProcessor {}

impl LiveVarAnalysisProcessor {
    pub fn new() -> Box<Self> {
        Box::new(LiveVarAnalysisProcessor {})
    }
}

impl FunctionTargetProcessor for LiveVarAnalysisProcessor {
    fn process(
        &self,
        _targets: &mut FunctionTargetsHolder,
        func_env: &FunctionEnv<'_>,
        mut data: FunctionTargetData,
    ) -> FunctionTargetData {
        let offset_to_live_refs = if func_env.is_native() {
            // Native functions have no byte code.
            LiveVarAnnotation(BTreeMap::new())
        } else {
            let code = std::mem::take(&mut data.code);
            let func_target = FunctionTarget::new(func_env, &data);

            // Call 1st time
            let (code, _) = Self::analyze_and_transform(&func_target, code);

            // Eliminate unused locals after dead code elimination.
            let (code, local_types, remap) = Self::eliminate_unused_vars(&func_target, code);
            data.rename_vars(&|idx| {
                if let Some(new_idx) = remap.get(&idx) {
                    *new_idx
                } else {
                    idx
                }
            });
            data.local_types = local_types;
            data.code = code;
            let func_target = FunctionTarget::new(func_env, &data);

            // Call analysis 2nd time on transformed code.
            let annotations = Self::analyze(&func_target, &data.code);
            LiveVarAnnotation(annotations)
        };
        // Annotate function target with computed life variable data.
        data.annotations
            .set::<LiveVarAnnotation>(offset_to_live_refs);
        data
    }

    fn name(&self) -> String {
        "livevar_analysis".to_string()
    }
}

impl LiveVarAnalysisProcessor {
    fn analyze_and_transform(
        func_target: &FunctionTarget,
        code: Vec<Bytecode>,
    ) -> (Vec<Bytecode>, BTreeMap<CodeOffset, LiveVarInfoAtCodeOffset>) {
        let annotations = Self::analyze(func_target, &code);
        let mut analyzer = LiveVarAnalysis::new(&func_target);
        let new_bytecode = analyzer.transform_code(&annotations, code);
        (new_bytecode, annotations)
    }

    fn analyze(
        func_target: &FunctionTarget,
        code: &[Bytecode],
    ) -> BTreeMap<CodeOffset, LiveVarInfoAtCodeOffset> {
        let cfg = StacklessControlFlowGraph::new_backward(&code);
        let mut analyzer = LiveVarAnalysis::new(&func_target);
        let state_map = analyzer.analyze_function(
            LiveVarState {
                livevars: BTreeSet::new(),
            },
            &code,
            &cfg,
        );
        analyzer.state_per_instruction(state_map, code, &cfg, |before, after| {
            LiveVarInfoAtCodeOffset {
                before: before.livevars.clone(),
                after: after.livevars.clone(),
            }
        })
    }

    fn eliminate_unused_vars(
        func_target: &FunctionTarget,
        code: Vec<Bytecode>,
    ) -> (Vec<Bytecode>, Vec<Type>, BTreeMap<TempIndex, TempIndex>) {
        if code.iter().any(|c| matches!(c, Bytecode::SpecBlock(..))) {
            // TODO(wrwg): SpecBlock currently does not work with variable renaming.
            return (code, func_target.data.local_types.clone(), BTreeMap::new());
        }
        let mut new_code = vec![];
        let mut new_vars = vec![];
        let mut remap = BTreeMap::new();
        // Do not change user declared vars, so populate remap info with them first.
        for local in 0..func_target.get_user_local_count() {
            let ty = func_target.get_local_type(local);
            new_vars.push(ty.clone());
            remap.insert(local, local);
        }
        let mut transform_local = |local: TempIndex| {
            if let Some(new_idx) = remap.get(&local) {
                *new_idx
            } else {
                let new_idx = new_vars.len();
                let ty = func_target.get_local_type(local);
                new_vars.push(ty.clone());
                remap.insert(local, new_idx);
                new_idx
            }
        };
        for bytecode in code {
            new_code.push(bytecode.remap_vars(&mut transform_local));
        }
        (new_code, new_vars, remap)
    }
}

struct LiveVarAnalysis<'a> {
    func_target: &'a FunctionTarget<'a>,
    next_label_id: usize,
    next_attr_id: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct LiveVarState {
    livevars: BTreeSet<TempIndex>,
}

impl LiveVarState {
    fn remove(&mut self, vars: &[TempIndex]) -> bool {
        let mut removed = false;
        for v in vars {
            if self.livevars.remove(v) {
                removed = true;
            }
        }
        removed
    }

    fn insert(&mut self, vars: Vec<TempIndex>) {
        for v in vars {
            self.livevars.insert(v);
        }
    }
}

impl<'a> LiveVarAnalysis<'a> {
    fn new(func_target: &'a FunctionTarget) -> Self {
        Self {
            func_target,
            next_label_id: 0,
            next_attr_id: 0,
        }
    }

    fn transform_code(
        &mut self,
        annotations: &BTreeMap<CodeOffset, LiveVarInfoAtCodeOffset>,
        mut code: Vec<Bytecode>,
    ) -> Vec<Bytecode> {
        let label_to_code_offset = Bytecode::label_offsets(&code);
        let mut transformed_code = vec![];
        let mut new_bytecodes = vec![];
        let mut skip_next = false;
        self.next_label_id = code.len();
        self.next_attr_id = code.len();
        for code_offset in 0..code.len() {
            if skip_next {
                skip_next = false;
                continue;
            }
            let bytecode = std::mem::replace(&mut code[code_offset], Bytecode::Nop(AttrId::new(0)));
            let annotation_at = &annotations[&(code_offset as CodeOffset)];
            match bytecode {
                Bytecode::Branch(attr_id, then_label, else_label, src) => {
                    let (then_label, mut bytecodes) = self.create_block_to_destroy_refs(
                        then_label,
                        self.lost_refs_along_edge(
                            annotations,
                            code_offset as CodeOffset,
                            label_to_code_offset[&then_label],
                        ),
                    );
                    new_bytecodes.append(&mut bytecodes);
                    let (else_label, mut bytecodes) = self.create_block_to_destroy_refs(
                        else_label,
                        self.lost_refs_along_edge(
                            annotations,
                            code_offset as CodeOffset,
                            label_to_code_offset[&else_label],
                        ),
                    );
                    new_bytecodes.append(&mut bytecodes);
                    transformed_code.push(Bytecode::Branch(attr_id, then_label, else_label, src));
                }
                Bytecode::Assign(_, dest, _, _) if !annotation_at.after.contains(&dest) => {
                    // Drop this assign as it is not used.
                }
                Bytecode::Call(attr_id, dests, oper, srcs)
                    if code_offset + 1 < code.len() && dests.len() == 1 =>
                {
                    // Catch the common case where we have:
                    //
                    //   $t := call(...)
                    //   x := $t
                    //   <$t is dead>
                    //
                    // This is an artifact from transformation from stack to stackless bytecode.
                    // Copy propagation cannot catch this case because it does not have the
                    // livevar information about $t.
                    let next_code_offset = code_offset + 1;
                    if let Bytecode::Assign(_, dest, src, _) = &code[next_code_offset] {
                        let annotation_at = &annotations[&(next_code_offset as CodeOffset)];
                        if src == &dests[0] && !annotation_at.after.contains(src) {
                            transformed_code.push(Bytecode::Call(attr_id, vec![*dest], oper, srcs));
                            skip_next = true;
                        } else {
                            transformed_code.push(Bytecode::Call(attr_id, dests, oper, srcs));
                        }
                    } else {
                        transformed_code.push(Bytecode::Call(attr_id, dests, oper, srcs));
                    }
                }
                _ => {
                    transformed_code.push(bytecode);
                }
            }
        }
        transformed_code.append(&mut new_bytecodes);
        transformed_code
    }

    fn new_label(&mut self) -> Label {
        let label = Label::new(self.next_label_id);
        self.next_label_id += 1;
        label
    }

    fn new_attr_id(&mut self) -> AttrId {
        let attr_id = AttrId::new(self.next_attr_id);
        self.next_attr_id += 1;
        attr_id
    }

    fn create_block_to_destroy_refs(
        &mut self,
        jump_label: Label,
        refs: Vec<TempIndex>,
    ) -> (Label, Vec<Bytecode>) {
        let mut start_label = jump_label;
        let mut new_bytecodes = vec![];
        if !refs.is_empty() {
            start_label = self.new_label();
            new_bytecodes.push(Bytecode::Label(self.new_attr_id(), start_label));
            for idx in refs {
                new_bytecodes.push(Bytecode::Call(
                    self.new_attr_id(),
                    vec![],
                    Operation::Destroy,
                    vec![idx],
                ));
            }
            new_bytecodes.push(Bytecode::Jump(self.new_attr_id(), jump_label));
        }
        (start_label, new_bytecodes)
    }

    fn lost_refs_along_edge(
        &self,
        annotations: &BTreeMap<CodeOffset, LiveVarInfoAtCodeOffset>,
        src_code_offset: CodeOffset,
        dest_code_offset: CodeOffset,
    ) -> Vec<TempIndex> {
        annotations[&src_code_offset]
            .after
            .iter()
            .filter(|x| {
                self.func_target.get_local_type(**x).is_reference()
                    && !annotations[&dest_code_offset].before.contains(x)
            })
            .copied()
            .collect()
    }
}

impl<'a> TransferFunctions for LiveVarAnalysis<'a> {
    type State = LiveVarState;
    const BACKWARD: bool = true;

    fn execute(&self, state: &mut LiveVarState, instr: &Bytecode, _idx: CodeOffset) {
        use Bytecode::*;
        match instr {
            Assign(_, dst, src, _) => {
                if state.remove(&[*dst]) {
                    state.insert(vec![*src]);
                }
            }
            Load(_, dst, _) => {
                state.remove(&[*dst]);
            }
            Call(_, dsts, oper, srcs) => {
                state.remove(dsts);
                state.insert(srcs.clone());
                if let Operation::Splice(map) = oper {
                    state.insert(map.values().cloned().collect());
                }
            }
            Ret(_, srcs) => {
                state.insert(srcs.clone());
            }
            Abort(_, src) | Branch(_, _, _, src) => {
                state.insert(vec![*src]);
            }
            _ => {}
        }
    }
}

impl<'a> DataflowAnalysis for LiveVarAnalysis<'a> {}

impl AbstractDomain for LiveVarState {
    fn join(&mut self, other: &Self) -> JoinResult {
        let mut result = JoinResult::Unchanged;
        for livevar in &other.livevars {
            if !self.livevars.contains(livevar) {
                self.livevars.insert(*livevar);
                result = JoinResult::Changed;
            }
        }
        result
    }
}

// =================================================================================================
// Formatting

/// Format a live variable annotation.
pub fn format_livevar_annotation(
    target: &FunctionTarget<'_>,
    code_offset: CodeOffset,
) -> Option<String> {
    if let Some(LiveVarAnnotation(map)) = target.get_annotations().get::<LiveVarAnnotation>() {
        if let Some(map_at) = map.get(&code_offset) {
            let mut res = map_at
                .before
                .iter()
                .map(|idx| {
                    let name = target.get_local_name(*idx);
                    format!("{}", name.display(target.symbol_pool()),)
                })
                .join(", ");
            res.insert_str(0, "live vars: ");
            return Some(res);
        }
    }
    None
}

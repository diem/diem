// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    dataflow_analysis::{
        AbstractDomain, DataflowAnalysis, JoinResult, StateMap, TransferFunctions,
    },
    function_target::{FunctionTarget, FunctionTargetData},
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder},
    stackless_bytecode::{AttrId, Bytecode, Label, Operation, TempIndex},
    stackless_control_flow_graph::{BlockId, StacklessControlFlowGraph},
};
use itertools::Itertools;
use spec_lang::env::FunctionEnv;
use std::collections::{BTreeMap, BTreeSet};
use vm::file_format::CodeOffset;

/// The annotation for live variable analysis. For each code position, we have a set of local
/// variable indices that are live just before the code offset, i.e. these variables are used
/// before being overwritten.

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

pub struct LiveVarAnalysisProcessor();

impl LiveVarAnalysisProcessor {
    pub fn new() -> Box<Self> {
        Box::new(LiveVarAnalysisProcessor())
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
            let (code, _) = Self::analyze_and_transform(&func_target, code);
            let (code, annotations) = Self::analyze_and_transform(&func_target, code);
            data.code = code;
            LiveVarAnnotation(annotations)
        };
        // Annotate function target with computed life variable data.
        data.annotations
            .set::<LiveVarAnnotation>(offset_to_live_refs);
        data
    }
}

impl LiveVarAnalysisProcessor {
    fn analyze_and_transform(
        func_target: &FunctionTarget,
        code: Vec<Bytecode>,
    ) -> (Vec<Bytecode>, BTreeMap<CodeOffset, LiveVarInfoAtCodeOffset>) {
        let cfg = StacklessControlFlowGraph::new_backward(&code);
        let mut analyzer = LiveVarAnalysis::new(&func_target);
        let state_map = analyzer.analyze_function(
            LiveVarState {
                livevars: BTreeSet::new(),
            },
            &code,
            &cfg,
        );
        let mut annotations = analyzer.post_process(&cfg, &code, state_map);
        let new_bytecode = analyzer.possibly_transform_code(&mut annotations, code);
        (new_bytecode, annotations)
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

    fn possibly_transform_code(
        &mut self,
        annotations: &mut BTreeMap<CodeOffset, LiveVarInfoAtCodeOffset>,
        code: Vec<Bytecode>,
    ) -> Vec<Bytecode> {
        let label_to_code_offset: BTreeMap<Label, usize> = code
            .iter()
            .enumerate()
            .map(|(code_offset, bytecode)| {
                use Bytecode::*;
                match bytecode {
                    Label(_, label) => Some((*label, code_offset)),
                    _ => None,
                }
            })
            .flatten()
            .collect();
        let mut transformed_code = vec![];
        let mut new_bytecodes = vec![];
        self.next_label_id = code.len();
        self.next_attr_id = code.len();
        for (code_offset, bytecode) in code.into_iter().enumerate() {
            match bytecode {
                Bytecode::Branch(attr_id, then_label, else_label, src) => {
                    let (then_label, mut bytecodes) = self.create_block_to_destroy_refs(
                        then_label,
                        self.lost_refs_along_edge(
                            annotations,
                            code_offset as CodeOffset,
                            label_to_code_offset[&then_label] as CodeOffset,
                        ),
                    );
                    new_bytecodes.append(&mut bytecodes);
                    let (else_label, mut bytecodes) = self.create_block_to_destroy_refs(
                        else_label,
                        self.lost_refs_along_edge(
                            annotations,
                            code_offset as CodeOffset,
                            label_to_code_offset[&else_label] as CodeOffset,
                        ),
                    );
                    new_bytecodes.append(&mut bytecodes);
                    transformed_code.push(Bytecode::Branch(attr_id, then_label, else_label, src));
                }
                Bytecode::Assign(attr_id, dest, _, _) => {
                    let annotation_at = &annotations[&(code_offset as CodeOffset)];
                    if annotation_at.after.contains(&dest) {
                        transformed_code.push(bytecode);
                    } else {
                        transformed_code.push(Bytecode::Nop(attr_id));
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

    fn post_process(
        &mut self,
        cfg: &StacklessControlFlowGraph,
        instrs: &[Bytecode],
        state_map: StateMap<LiveVarState, ()>,
    ) -> BTreeMap<CodeOffset, LiveVarInfoAtCodeOffset> {
        let mut result = BTreeMap::new();
        for (block_id, block_state) in state_map {
            let mut state = block_state.pre;
            for offset in cfg.instr_indexes(block_id).rev() {
                let instr = &instrs[offset as usize];
                let after = state.livevars.clone();
                state = self.execute(state, instr, offset);
                let before = state.livevars.clone();
                result.insert(offset, LiveVarInfoAtCodeOffset { before, after });
            }
        }
        result
    }

    fn execute(&mut self, pre: LiveVarState, instr: &Bytecode, _idx: CodeOffset) -> LiveVarState {
        use Bytecode::*;
        let mut post = pre;
        match instr {
            Assign(_, dst, src, _) => {
                if post.remove(&[*dst]) {
                    post.insert(vec![*src]);
                }
            }
            Load(_, dst, _) => {
                post.remove(&[*dst]);
            }
            Call(_, dsts, _, srcs) => {
                post.remove(dsts);
                post.insert(srcs.clone());
            }
            Ret(_, srcs) => {
                post.insert(srcs.clone());
            }
            Abort(_, src) | Branch(_, _, _, src) => {
                post.insert(vec![*src]);
            }
            _ => {}
        }
        post
    }
}

impl<'a> TransferFunctions for LiveVarAnalysis<'a> {
    type State = LiveVarState;
    type AnalysisError = ();

    fn execute_block(
        &mut self,
        block_id: BlockId,
        pre_state: Self::State,
        instrs: &[Bytecode],
        cfg: &StacklessControlFlowGraph,
    ) -> Result<Self::State, Self::AnalysisError> {
        let mut state = pre_state;
        for offset in cfg.instr_indexes(block_id).rev() {
            let instr = &instrs[offset as usize];
            state = self.execute(state, instr, offset);
        }
        Ok(state)
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

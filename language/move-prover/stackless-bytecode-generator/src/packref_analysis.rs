// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    borrow_analysis::BorrowAnnotation,
    dataflow_analysis::{
        AbstractDomain, DataflowAnalysis, JoinResult, StateMap, TransferFunctions,
    },
    function_target::{FunctionTarget, FunctionTargetData},
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder},
    stackless_bytecode::{
        AssignKind, AttrId,
        Bytecode::{self, *},
        Operation, TempIndex,
    },
    stackless_control_flow_graph::{BlockId, StacklessControlFlowGraph},
};
use itertools::Itertools;
use spec_lang::env::FunctionEnv;
use std::collections::{BTreeMap, BTreeSet};
use vm::file_format::CodeOffset;

/// Copy analysis
#[derive(Debug, Clone, PartialEq)]
struct CopyState {
    copies: BTreeSet<TempIndex>,
}

impl CopyState {}

struct CopyAnalysis<'a> {
    func_target: &'a FunctionTarget<'a>,
    borrow_annotation: &'a BorrowAnnotation,
}

impl<'a> CopyAnalysis<'a> {
    fn new(func_target: &'a FunctionTarget<'a>, borrow_annotation: &'a BorrowAnnotation) -> Self {
        Self {
            func_target,
            borrow_annotation,
        }
    }

    fn analyze(
        &mut self,
        copies: BTreeSet<TempIndex>,
        instrs: &[Bytecode],
    ) -> BTreeMap<CodeOffset, CopyState> {
        let cfg = StacklessControlFlowGraph::new_forward(instrs);
        let initial_state = CopyState { copies };
        let state_map = self.analyze_function(initial_state, instrs, &cfg);
        self.post_process(&cfg, instrs, state_map)
    }

    fn post_process(
        &mut self,
        cfg: &StacklessControlFlowGraph,
        instrs: &[Bytecode],
        state_map: StateMap<CopyState, ()>,
    ) -> BTreeMap<CodeOffset, CopyState> {
        let mut result = BTreeMap::new();
        for (block_id, block_state) in state_map {
            let mut state = block_state.pre;
            for offset in cfg.instr_indexes(block_id) {
                let instr = &instrs[offset as usize];
                result.insert(offset, state.clone());
                state = self.execute(state, instr, offset).unwrap();
            }
        }
        result
    }

    fn execute(
        &mut self,
        pre: CopyState,
        instr: &Bytecode,
        code_offset: CodeOffset,
    ) -> Result<CopyState, ()> {
        use Bytecode::*;
        let mut post = pre;
        match instr {
            Assign(_, dst, _, AssignKind::Copy) => {
                if self.func_target.get_local_type(*dst).is_reference() {
                    post.copies.insert(*dst);
                }
            }
            Assign(_, dst, src, AssignKind::Move) | Assign(_, dst, src, AssignKind::Store) => {
                if post.copies.remove(src) {
                    post.copies.insert(*dst);
                }
            }
            _ => {}
        }
        let after_refs = &self
            .borrow_annotation
            .get_borrow_info_at(code_offset)
            .unwrap()
            .after
            .all_refs();
        post.copies = post.copies.intersection(after_refs).cloned().collect();
        Ok(post)
    }
}

impl<'a> TransferFunctions for CopyAnalysis<'a> {
    type State = CopyState;
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
            state = self.execute(state, instr, offset)?;
        }
        Ok(state)
    }
}

impl<'a> DataflowAnalysis for CopyAnalysis<'a> {}

impl AbstractDomain for CopyState {
    fn join(&mut self, other: &Self) -> JoinResult {
        if self.copies == other.copies {
            JoinResult::Unchanged
        } else {
            JoinResult::Error
        }
    }
}
/// Copy Analysis ends

pub struct PackrefAnalysisProcessor {}

impl PackrefAnalysisProcessor {
    pub fn new() -> Box<Self> {
        Box::new(PackrefAnalysisProcessor {})
    }
}

impl FunctionTargetProcessor for PackrefAnalysisProcessor {
    fn process(
        &self,
        _targets: &mut FunctionTargetsHolder,
        func_env: &FunctionEnv<'_>,
        mut data: FunctionTargetData,
    ) -> FunctionTargetData {
        if func_env.is_native() {
            return data;
        }
        let func_target = FunctionTarget::new(func_env, &data);
        let borrow_annotation = func_target
            .get_annotations()
            .get::<BorrowAnnotation>()
            .expect("borrow annotation");
        let copy_annotation = CopyAnalysis::new(&func_target, borrow_annotation).analyze(
            if func_target.call_ends_lifetime() {
                BTreeSet::new()
            } else {
                (0..func_target.get_parameter_count())
                    .filter(|idx| func_target.get_local_type(*idx).is_reference())
                    .collect()
            },
            &data.code,
        );
        let mut pack_analysis = PackrefAnalysis::new(
            &func_target,
            borrow_annotation,
            &copy_annotation,
            &data.code,
        );
        let mut new_code = BTreeMap::new();
        for (code_offset, bytecode) in data.code.iter().enumerate() {
            new_code.insert(
                code_offset as CodeOffset,
                pack_analysis.compute_instrumentation(code_offset as CodeOffset, bytecode),
            );
        }
        data.annotations
            .set::<PackrefAnnotation>(PackrefAnnotation(new_code));
        data
    }
}

pub struct PackrefInstrumentation {
    pub before: Vec<Bytecode>,
    pub after: Vec<Bytecode>,
}

impl PackrefInstrumentation {
    fn is_empty(&self) -> bool {
        self.before.is_empty() && self.after.is_empty()
    }
}

pub struct PackrefAnnotation(BTreeMap<CodeOffset, PackrefInstrumentation>);

impl PackrefAnnotation {
    pub fn get_packref_instrumentation_at(
        &self,
        code_offset: CodeOffset,
    ) -> Option<&PackrefInstrumentation> {
        self.0.get(&code_offset)
    }
}

pub struct PackrefAnalysis<'a> {
    func_target: &'a FunctionTarget<'a>,
    borrow_annotation: &'a BorrowAnnotation,
    copy_annotation: &'a BTreeMap<CodeOffset, CopyState>,
    next_attr_id: usize,
}

impl<'a> PackrefAnalysis<'a> {
    fn new(
        func_target: &'a FunctionTarget<'a>,
        borrow_annotation: &'a BorrowAnnotation,
        copy_annotation: &'a BTreeMap<CodeOffset, CopyState>,
        code: &[Bytecode],
    ) -> Self {
        Self {
            func_target,
            borrow_annotation,
            copy_annotation,
            next_attr_id: code.len(),
        }
    }

    fn compute_instrumentation(
        &mut self,
        code_offset: CodeOffset,
        bytecode: &Bytecode,
    ) -> PackrefInstrumentation {
        let (before, mut after) = self.public_function_instrumentation(code_offset, bytecode);
        match bytecode {
            SpecBlock(..) | Assign(..) | Ret(..) | Load(..) | Branch(..) | Jump(..) | Label(..)
            | Abort(..) | Nop(..) => {}
            _ => {
                after.append(&mut self.ref_create_destroy_instrumentation(code_offset, bytecode));
            }
        };
        PackrefInstrumentation { before, after }
    }

    fn entry_exit_instrumentation(
        &mut self,
        code_offset: CodeOffset,
        at_entry: bool,
        bytecodes: &mut Vec<Bytecode>,
    ) {
        if self.func_target.call_ends_lifetime() {
            let borrow_annotation_at = self
                .borrow_annotation
                .get_borrow_info_at(code_offset)
                .unwrap();
            for idx in 0..self.func_target.get_parameter_count() {
                if borrow_annotation_at.before.live_refs.contains(&idx) {
                    bytecodes.push(if at_entry {
                        Bytecode::UnpackRef(self.new_attr_id(), idx)
                    } else {
                        Bytecode::PackRef(self.new_attr_id(), idx)
                    });
                }
            }
        }
    }

    fn public_function_instrumentation(
        &mut self,
        code_offset: CodeOffset,
        bytecode: &Bytecode,
    ) -> (Vec<Bytecode>, Vec<Bytecode>) {
        let mut before = vec![];
        let mut after = vec![];
        if code_offset == 0 {
            self.entry_exit_instrumentation(code_offset, true, &mut before);
        }
        match &bytecode {
            Call(_, dests, Operation::Function(mid, fid, _), srcs) => {
                let module_env = self.func_target.module_env();
                let call_ends_lifetime = dests
                    .iter()
                    .all(|idx| !self.func_target.get_local_type(*idx).is_reference())
                    && (module_env.get_id() != *mid || module_env.get_function(*fid).is_public());
                if call_ends_lifetime {
                    let pack_refs: Vec<&TempIndex> = srcs
                        .iter()
                        .filter(|idx| self.func_target.get_local_type(**idx).is_reference())
                        .collect();
                    before.append(
                        &mut pack_refs
                            .iter()
                            .map(|idx| Bytecode::PackRef(self.new_attr_id(), **idx))
                            .collect(),
                    );
                    after.append(
                        &mut pack_refs
                            .into_iter()
                            .map(|idx| Bytecode::UnpackRef(self.new_attr_id(), *idx))
                            .collect(),
                    );
                }
            }
            Ret(..) => {
                self.entry_exit_instrumentation(code_offset, false, &mut before);
            }
            _ => {}
        };
        (before, after)
    }

    fn new_attr_id(&mut self) -> AttrId {
        let attr_id = AttrId::new(self.next_attr_id);
        self.next_attr_id += 1;
        attr_id
    }

    fn ref_create_destroy_instrumentation(
        &mut self,
        code_offset: CodeOffset,
        bytecode: &Bytecode,
    ) -> Vec<Bytecode> {
        let borrow_annotation_at = self
            .borrow_annotation
            .get_borrow_info_at(code_offset)
            .unwrap();
        let copy_annotation_at = &self.copy_annotation[&code_offset];
        let mut instrumented_bytecodes = vec![];
        match bytecode {
            Call(_, dests, op, _) => {
                use Operation::*;
                match op {
                    BorrowLoc | BorrowField(..) | BorrowGlobal(..) => {
                        instrumented_bytecodes
                            .push(Bytecode::UnpackRef(self.new_attr_id(), dests[0]));
                    }
                    _ => {
                        let before_refs = borrow_annotation_at.before.all_refs();
                        let after_refs = borrow_annotation_at.after.all_refs();
                        for idx in before_refs.difference(&after_refs) {
                            if !copy_annotation_at.copies.contains(idx) {
                                instrumented_bytecodes
                                    .push(Bytecode::PackRef(self.new_attr_id(), *idx));
                            }
                        }
                    }
                }
            }
            _ => unreachable!(),
        }
        instrumented_bytecodes
    }
}

// =================================================================================================
// Formatting

/// Format a packref annotation.
pub fn format_packref_annotation(
    func_target: &FunctionTarget<'_>,
    code_offset: CodeOffset,
) -> Option<String> {
    if let Some(PackrefAnnotation(map)) = func_target.get_annotations().get::<PackrefAnnotation>() {
        if let Some(map_at) = map.get(&code_offset) {
            if !map_at.is_empty() {
                let before_str = format!(
                    "before: {}",
                    map_at
                        .before
                        .iter()
                        .map(|bytecode| bytecode.display(func_target))
                        .join(", ")
                );
                let after_str = format!(
                    "after: {}",
                    map_at
                        .after
                        .iter()
                        .map(|bytecode| bytecode.display(func_target))
                        .join(", ")
                );
                return Some(format!("{} {}", before_str, after_str));
            }
        }
    }
    None
}

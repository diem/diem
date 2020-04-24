// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    borrow_analysis::{BorrowAnnotation, BorrowInfo},
    function_target::{FunctionTarget, FunctionTargetData},
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder},
    stackless_bytecode::{
        AssignKind, AttrId, BorrowNode,
        Bytecode::{self, *},
        Operation, TempIndex,
    },
};
use itertools::Itertools;
use spec_lang::env::FunctionEnv;
use std::collections::{BTreeMap, BTreeSet};
use vm::file_format::CodeOffset;

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
        let func_target = FunctionTarget::new(func_env, &data);
        let mut pack_analysis = PackrefAnalysis::new(&func_target, &data.code);
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
    next_attr_id: usize,
}

impl<'a> PackrefAnalysis<'a> {
    fn new(func_target: &'a FunctionTarget<'a>, code: &[Bytecode]) -> Self {
        let borrow_annotation = func_target
            .get_annotations()
            .get::<BorrowAnnotation>()
            .expect("borrow annotation");

        Self {
            func_target,
            borrow_annotation,
            next_attr_id: code.len(),
        }
    }

    fn compute_instrumentation(
        &mut self,
        code_offset: CodeOffset,
        bytecode: &Bytecode,
    ) -> PackrefInstrumentation {
        let borrow_annotation_at = self
            .borrow_annotation
            .get_borrow_info_at(code_offset)
            .unwrap();
        let (before, mut after) = self.public_function_instrumentation(code_offset, &bytecode);
        match bytecode {
            SpecBlock(..)
            | Assign(_, _, _, AssignKind::Move)
            | Assign(_, _, _, AssignKind::Store)
            | Ret(..)
            | Load(..)
            | Branch(..)
            | Jump(..)
            | Label(..)
            | Abort(..)
            | Nop(..) => {}
            _ => {
                after.append(&mut self.ref_create_destroy_instrumentation(
                    bytecode,
                    &borrow_annotation_at.before,
                    &borrow_annotation_at.after,
                ));
            }
        };
        PackrefInstrumentation { before, after }
    }

    fn call_ends_lifetime(&self) -> bool {
        self.func_target.is_public()
            && self
                .func_target
                .get_return_types()
                .iter()
                .all(|ty| !ty.is_reference())
    }

    fn public_function_instrumentation(
        &mut self,
        code_offset: CodeOffset,
        bytecode: &Bytecode,
    ) -> (Vec<Bytecode>, Vec<Bytecode>) {
        let mut before = vec![];
        let mut after = vec![];
        if self.call_ends_lifetime() && code_offset == 0 {
            for idx in 0..self.func_target.get_parameter_count() {
                if self.func_target.get_local_type(idx).is_reference() {
                    before.push(Bytecode::UnpackRef(self.new_attr_id(), idx));
                }
            }
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
                if self.call_ends_lifetime() {
                    for idx in 0..self.func_target.get_parameter_count() {
                        if self.func_target.get_local_type(idx).is_reference() {
                            before.push(Bytecode::PackRef(self.new_attr_id(), idx));
                        }
                    }
                }
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
        bytecode: &Bytecode,
        before: &BorrowInfo,
        after: &BorrowInfo,
    ) -> Vec<Bytecode> {
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
                        let filter_fn = |node: &BorrowNode| {
                            if let BorrowNode::Reference(idx) = node {
                                Some(*idx)
                            } else {
                                None
                            }
                        };
                        let before_borrowed_by =
                            before.borrowed_by.keys().filter_map(filter_fn).collect();
                        let before_refs: BTreeSet<&TempIndex> =
                            before.live_refs.union(&before_borrowed_by).collect();
                        let after_borrowed_by =
                            after.borrowed_by.keys().filter_map(filter_fn).collect();
                        let after_refs: BTreeSet<&TempIndex> =
                            after.live_refs.union(&after_borrowed_by).collect();
                        for idx in before_refs.difference(&after_refs) {
                            instrumented_bytecodes
                                .push(Bytecode::PackRef(self.new_attr_id(), **idx));
                        }
                    }
                }
            }
            Assign(_, dest, _, AssignKind::Copy) => {
                if after
                    .borrowed_by
                    .contains_key(&BorrowNode::Reference(*dest))
                {
                    instrumented_bytecodes.push(Bytecode::UnpackRef(self.new_attr_id(), *dest));
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

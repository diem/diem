// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    borrow_analysis::{BorrowAnnotation, BorrowInfo},
    function_target::{FunctionTarget, FunctionTargetData},
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder},
    stackless_bytecode::{
        AttrId, BorrowNode,
        Bytecode::{self, *},
        TempIndex,
    },
};
use itertools::Itertools;
use spec_lang::env::FunctionEnv;
use std::collections::{BTreeMap, BTreeSet};
use vm::file_format::CodeOffset;

pub struct WritebackAnalysisProcessor {}

impl WritebackAnalysisProcessor {
    pub fn new() -> Box<Self> {
        Box::new(WritebackAnalysisProcessor {})
    }
}

impl FunctionTargetProcessor for WritebackAnalysisProcessor {
    fn process(
        &self,
        _targets: &mut FunctionTargetsHolder,
        func_env: &FunctionEnv<'_>,
        mut data: FunctionTargetData,
    ) -> FunctionTargetData {
        let func_target = FunctionTarget::new(func_env, &data);
        let mut writeback_analysis = WritebackAnalysis::new(&func_target, &data.code);
        let mut new_code = BTreeMap::new();
        for (code_offset, bytecode) in data.code.iter().enumerate() {
            new_code.insert(
                code_offset as CodeOffset,
                writeback_analysis.compute_instrumentation(code_offset as CodeOffset, bytecode),
            );
        }
        data.annotations
            .set::<WritebackAnnotation>(WritebackAnnotation(new_code));
        data
    }
}

// WriteBack instructions to be inserted right after a code offset
pub struct WritebackAnnotation(BTreeMap<CodeOffset, Vec<Bytecode>>);

impl WritebackAnnotation {
    pub fn get_writeback_instrs_at(&self, code_offset: CodeOffset) -> Option<&Vec<Bytecode>> {
        self.0.get(&code_offset)
    }
}

pub struct WritebackAnalysis<'a> {
    _func_target: &'a FunctionTarget<'a>,
    borrow_annotation: &'a BorrowAnnotation,
    next_attr_id: usize,
}

impl<'a> WritebackAnalysis<'a> {
    fn new(func_target: &'a FunctionTarget<'a>, code: &[Bytecode]) -> Self {
        let borrow_annotation = func_target
            .get_annotations()
            .get::<BorrowAnnotation>()
            .expect("borrow annotation");

        Self {
            _func_target: func_target,
            borrow_annotation,
            next_attr_id: code.len(),
        }
    }

    fn compute_instrumentation(
        &mut self,
        offset: CodeOffset,
        bytecode: &Bytecode,
    ) -> Vec<Bytecode> {
        let borrow_annotation_at = self.borrow_annotation.get_borrow_info_at(offset).unwrap();
        match bytecode {
            SpecBlock(..) | Assign(..) | Ret(..) | Load(..) | Branch(..) | Jump(..) | Label(..)
            | Abort(..) | Nop(..) => vec![],
            _ => {
                self.writeback_bytecodes(&borrow_annotation_at.before, &borrow_annotation_at.after)
            }
        }
    }

    fn visit(
        idx: TempIndex,
        borrows_from: &BTreeMap<BorrowNode, BTreeSet<BorrowNode>>,
        visited: &mut BTreeSet<TempIndex>,
        dfs_order: &mut Vec<TempIndex>,
    ) {
        visited.insert(idx);
        let node = BorrowNode::Reference(idx);
        if borrows_from.contains_key(&node) {
            for n in &borrows_from[&node] {
                if let BorrowNode::Reference(next_idx) = n {
                    if !visited.contains(next_idx) {
                        Self::visit(*next_idx, borrows_from, visited, dfs_order);
                    }
                }
            }
        }
        dfs_order.push(idx);
    }

    fn new_attr_id(&mut self) -> AttrId {
        let attr_id = AttrId::new(self.next_attr_id);
        self.next_attr_id += 1;
        attr_id
    }

    fn writeback_bytecodes(&mut self, before: &BorrowInfo, after: &BorrowInfo) -> Vec<Bytecode> {
        let mut instrumented_bytecodes = vec![];
        // add writebacks (youngest first) for all references that are live before but not after
        let mut visited = BTreeSet::new();
        let mut dfs_order = vec![];
        for idx in before.live_refs.difference(&after.live_refs) {
            Self::visit(*idx, &before.borrows_from, &mut visited, &mut dfs_order);
        }
        for idx in dfs_order {
            if before.live_refs.contains(&idx) && !after.live_refs.contains(&idx) {
                let node = BorrowNode::Reference(idx);
                for n in &before.borrows_from[&node] {
                    instrumented_bytecodes.push(Bytecode::WriteBack(
                        self.new_attr_id(),
                        n.clone(),
                        idx,
                    ));
                }
            }
        }
        instrumented_bytecodes
    }
}

// =================================================================================================
// Formatting

/// Format a writeback annotation.
pub fn format_writeback_annotation(
    func_target: &FunctionTarget<'_>,
    code_offset: CodeOffset,
) -> Option<String> {
    if let Some(WritebackAnnotation(map)) =
        func_target.get_annotations().get::<WritebackAnnotation>()
    {
        if let Some(map_at) = map.get(&code_offset) {
            if !map_at.is_empty() {
                return Some(
                    map_at
                        .iter()
                        .map(|bytecode| bytecode.display(func_target))
                        .join(", "),
                );
            }
        }
    }
    None
}

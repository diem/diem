// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    function_data_builder::FunctionDataBuilder,
    function_target::{FunctionData, FunctionTarget},
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder},
    graph::Graph,
    stackless_bytecode::{BorrowNode, Bytecode, Label, Operation, PropKind},
    stackless_control_flow_graph::{BlockContent, BlockId, StacklessControlFlowGraph},
};
use move_model::{
    ast::{self, TempIndex},
    model::FunctionEnv,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Default, Clone)]
pub struct LoopAnnotation {
    pub loop_targets: BTreeMap<Label, BTreeSet<usize>>,
}

pub struct LoopAnalysisProcessor {}

impl LoopAnalysisProcessor {
    pub fn new() -> Box<Self> {
        Box::new(LoopAnalysisProcessor {})
    }
}

impl FunctionTargetProcessor for LoopAnalysisProcessor {
    fn process(
        &self,
        _targets: &mut FunctionTargetsHolder,
        func_env: &FunctionEnv<'_>,
        data: FunctionData,
    ) -> FunctionData {
        if func_env.is_native() {
            return data;
        }
        let loop_targets = Self::collect_loop_targets(func_env, &data);
        let mut data = Self::transform(func_env, data, &loop_targets);
        let loop_annotation = LoopAnnotation { loop_targets };
        data.annotations.set::<LoopAnnotation>(loop_annotation);
        data
    }

    fn name(&self) -> String {
        "loop_analysis".to_string()
    }
}

impl LoopAnalysisProcessor {
    fn transform(
        func_env: &FunctionEnv<'_>,
        mut data: FunctionData,
        loop_targets: &BTreeMap<Label, BTreeSet<usize>>,
    ) -> FunctionData {
        let code = std::mem::take(&mut data.code);
        let mut builder = FunctionDataBuilder::new(func_env, data);
        for bytecode in code {
            match bytecode {
                Bytecode::Label(attr_id, label) => {
                    builder.emit(bytecode);
                    builder.set_loc_from_attr(attr_id);
                    if loop_targets.contains_key(&label) {
                        let exp =
                            builder.mk_not(builder.mk_bool_call(ast::Operation::AbortFlag, vec![]));
                        builder.emit_with(|attr_id| Bytecode::Prop(attr_id, PropKind::Assume, exp));
                        for idx in &loop_targets[&label] {
                            let exp = builder.mk_bool_call(
                                ast::Operation::WellFormed,
                                vec![builder.mk_temporary(*idx)],
                            );
                            builder.emit_with(|attr_id| {
                                Bytecode::Prop(attr_id, PropKind::Assume, exp)
                            });
                        }
                    }
                }
                _ => {
                    builder.emit(bytecode);
                }
            }
        }
        builder.data
    }

    fn collect_loop_targets(
        func_env: &FunctionEnv<'_>,
        data: &FunctionData,
    ) -> BTreeMap<Label, BTreeSet<usize>> {
        // build for natural loops
        let func_target = FunctionTarget::new(func_env, data);
        let code = func_target.get_bytecode();
        let cfg = StacklessControlFlowGraph::new_forward(code);
        let entry = cfg.entry_block();
        let nodes = cfg.blocks();
        let edges: Vec<(BlockId, BlockId)> = nodes
            .iter()
            .map(|x| {
                cfg.successors(*x)
                    .iter()
                    .map(|y| (*x, *y))
                    .collect::<Vec<(BlockId, BlockId)>>()
            })
            .flatten()
            .collect();
        let graph = Graph::new(entry, nodes, edges);
        let natural_loops = graph.compute_reducible().expect(
            "A well-formed Move function is expected to have a reducible control-flow graph",
        );

        // collect labels where loop invariant instrumentations are expected to be placed at
        let loop_header_to_label: BTreeMap<BlockId, Label> = natural_loops
            .iter()
            .filter_map(|l| match cfg.content(l.loop_header) {
                BlockContent::Dummy => None,
                BlockContent::Basic { lower, upper: _ } => {
                    if let Bytecode::Label(_, label) = code[*lower as usize] {
                        Some((l.loop_header, label))
                    } else {
                        None
                    }
                }
            })
            .collect();
        if loop_header_to_label.len() != natural_loops.len() {
            panic!(
                "Natural loops in a well-formed Move function are expected to have unique headers \
                and each loop header is expected to start with a Label bytecode"
            );
        }

        // find loop targets per loop (also means, per label)
        let mut loop_targets = BTreeMap::new();
        for single_loop in natural_loops {
            let targets: BTreeSet<TempIndex> = single_loop
                .loop_body
                .into_iter()
                .map(|block_id| {
                    cfg.instr_indexes(block_id)
                        .expect("A loop body should never contain a dummy block")
                })
                .flatten()
                .map(|code_offset| Self::targets(&code[code_offset as usize]))
                .flatten()
                .collect();
            loop_targets.insert(loop_header_to_label[&single_loop.loop_header], targets);
        }
        loop_targets
    }

    fn targets(bytecode: &Bytecode) -> Vec<TempIndex> {
        use BorrowNode::*;
        match bytecode {
            Bytecode::Assign(_, dest, _, _) => vec![*dest],
            Bytecode::Load(_, dest, _) => vec![*dest],
            Bytecode::Call(_, _, Operation::WriteBack(LocalRoot(dest), ..), ..) => vec![*dest],
            Bytecode::Call(_, _, Operation::WriteBack(Reference(dest), ..), ..) => vec![*dest],
            Bytecode::Call(_, dests, ..) => dests.clone(),
            _ => vec![],
        }
    }
}

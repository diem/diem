// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    function_data_builder::FunctionDataBuilder,
    function_target::{FunctionData, FunctionTarget},
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder},
    graph::{Graph, Reducible},
    stackless_bytecode::{
        BorrowNode,
        Bytecode::{self, *},
        Label, Operation, PropKind,
    },
    stackless_control_flow_graph::{BlockContent, BlockId, StacklessControlFlowGraph},
};
use move_model::{ast, model::FunctionEnv};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Default)]
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
                Label(attr_id, label) => {
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
        let mut loop_targets = BTreeMap::new();
        if let Some(Reducible {
            loop_headers,
            natural_loops,
        }) = graph.compute_reducible()
        {
            let block_id_to_label: BTreeMap<BlockId, Label> = loop_headers
                .iter()
                .map(|x| match cfg.content(*x) {
                    BlockContent::Dummy => None,
                    BlockContent::Basic { lower, upper: _ } => {
                        if let Label(_, label) = code[*lower as usize] {
                            Some((*x, label))
                        } else {
                            None
                        }
                    }
                })
                .flatten()
                .collect();
            for (back_edge, natural_loop) in natural_loops {
                let loop_header_label = block_id_to_label[&back_edge.1];
                loop_targets
                    .entry(loop_header_label)
                    .or_insert_with(BTreeSet::new);
                let natural_loop_targets = natural_loop
                    .iter()
                    .map(|block_id| match cfg.is_dummmy(*block_id) {
                        true => BTreeSet::new(),
                        false => cfg
                            .instr_indexes(*block_id)
                            .unwrap()
                            .map(|x| Self::targets(&code[x as usize]))
                            .flatten()
                            .collect::<BTreeSet<usize>>(),
                    })
                    .flatten()
                    .collect::<BTreeSet<usize>>();
                for target in natural_loop_targets {
                    loop_targets.entry(loop_header_label).and_modify(|x| {
                        x.insert(target);
                    });
                }
            }
        }
        loop_targets
    }

    fn targets(bytecode: &Bytecode) -> Vec<usize> {
        use BorrowNode::*;
        match bytecode {
            Assign(_, dest, _, _) => vec![*dest],
            Load(_, dest, _) => vec![*dest],
            Call(_, _, Operation::WriteBack(LocalRoot(dest), ..), ..) => vec![*dest],
            Call(_, _, Operation::WriteBack(Reference(dest), ..), ..) => vec![*dest],
            Call(_, dests, ..) => dests.clone(),
            _ => vec![],
        }
    }
}

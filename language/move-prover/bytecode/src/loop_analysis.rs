// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    function_data_builder::FunctionDataBuilder,
    function_target::{FunctionData, FunctionTarget},
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder},
    graph::Graph,
    stackless_bytecode::{AttrId, BorrowNode, Bytecode, Label, Operation, PropKind},
    stackless_control_flow_graph::{BlockContent, BlockId, StacklessControlFlowGraph},
};
use move_model::{
    ast::{self, TempIndex},
    model::FunctionEnv,
};
use std::collections::{BTreeMap, BTreeSet};
use vm::file_format::CodeOffset;

#[derive(Debug, Default, Clone)]
pub struct LoopInfo {
    pub invariants: Vec<(AttrId, ast::Exp)>,
    pub targets: BTreeSet<TempIndex>,
    pub back_edge_location: CodeOffset,
}

#[derive(Debug, Default, Clone)]
pub struct LoopAnnotation {
    pub loops: BTreeMap<Label, LoopInfo>,
}

impl LoopAnnotation {
    fn back_edges_locations(&self) -> BTreeSet<CodeOffset> {
        self.loops.values().map(|l| l.back_edge_location).collect()
    }
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
        let loop_annotation = Self::build_loop_annotation(func_env, &data);
        Self::transform(func_env, data, &loop_annotation)
    }

    fn name(&self) -> String {
        "loop_analysis".to_string()
    }
}

impl LoopAnalysisProcessor {
    fn transform(
        func_env: &FunctionEnv<'_>,
        data: FunctionData,
        loop_annotation: &LoopAnnotation,
    ) -> FunctionData {
        let back_edge_locs = loop_annotation.back_edges_locations();
        let mut builder = FunctionDataBuilder::new(func_env, data);
        let mut goto_fixes = vec![];
        let code = std::mem::take(&mut builder.data.code);
        for (offset, bytecode) in code.into_iter().enumerate() {
            match bytecode {
                Bytecode::Label(attr_id, label) => {
                    builder.emit(bytecode);
                    builder.set_loc_from_attr(attr_id);
                    if let Some(loop_info) = loop_annotation.loops.get(&label) {
                        // assert loop invariants -> this is the base case
                        for (_, exp) in &loop_info.invariants {
                            builder.emit_with(|attr_id| {
                                Bytecode::Prop(attr_id, PropKind::Assert, exp.clone())
                            });
                        }

                        // havoc all loop targets
                        for idx in &loop_info.targets {
                            builder.emit_with(|attr_id| {
                                Bytecode::Call(attr_id, vec![], Operation::Havoc, vec![*idx], None)
                            });
                        }

                        // add additional assumptions
                        let exp =
                            builder.mk_not(builder.mk_bool_call(ast::Operation::AbortFlag, vec![]));
                        builder.emit_with(|attr_id| Bytecode::Prop(attr_id, PropKind::Assume, exp));
                        for idx in &loop_info.targets {
                            let exp = builder.mk_bool_call(
                                ast::Operation::WellFormed,
                                vec![builder.mk_temporary(*idx)],
                            );
                            builder.emit_with(|attr_id| {
                                Bytecode::Prop(attr_id, PropKind::Assume, exp)
                            });
                        }

                        // re-assume loop invariants
                        for (attr_id, exp) in &loop_info.invariants {
                            builder.emit(Bytecode::Prop(*attr_id, PropKind::Assume, exp.clone()));
                        }
                    }
                }
                Bytecode::Prop(_, PropKind::Invariant, _) => {
                    // do nothing, as the invariants should have been translated to asserts
                    //
                    // TODO (mengxu): we might want to print out warnings for invariants not
                    // specified at the loop header block?
                }
                _ => {
                    builder.emit(bytecode);
                }
            }
            // mark that the goto labels in this bytecode needs to be updated to a new label
            // representing the invariant-checking block for the loop.
            if back_edge_locs.contains(&(offset as CodeOffset)) {
                goto_fixes.push(builder.data.code.len() - 1);
            }
        }

        // create one invariant-checking block for each loop
        let invariant_checker_labels: BTreeMap<_, _> = loop_annotation
            .loops
            .keys()
            .map(|label| (*label, builder.new_label()))
            .collect();

        for (label, loop_info) in &loop_annotation.loops {
            let checker_label = invariant_checker_labels.get(label).unwrap();
            builder.set_next_debug_comment(format!(
                "Loop invariant checking block for the loop started with header: L{}",
                label.as_usize()
            ));
            builder.emit_with(|attr_id| Bytecode::Label(attr_id, *checker_label));
            builder.clear_next_debug_comment();

            // add instrumentations to assert loop invariants -> this is the induction case
            for (_, exp) in &loop_info.invariants {
                builder.emit_with(|attr_id| Bytecode::Prop(attr_id, PropKind::Assert, exp.clone()));
            }

            // stop the checking
            builder.emit_with(|attr_id| {
                Bytecode::Call(attr_id, vec![], Operation::Stop, vec![], None)
            });
        }

        // fix the goto statements in the loop latch blocks
        for code_offset in goto_fixes {
            let updated_goto = match &builder.data.code[code_offset] {
                Bytecode::Jump(attr_id, old_label) => {
                    Bytecode::Jump(*attr_id, *invariant_checker_labels.get(old_label).unwrap())
                }
                Bytecode::Branch(attr_id, if_label, else_label, idx) => {
                    let new_if_label = *invariant_checker_labels.get(if_label).unwrap_or(if_label);
                    let new_else_label = *invariant_checker_labels
                        .get(else_label)
                        .unwrap_or(else_label);
                    Bytecode::Branch(*attr_id, new_if_label, new_else_label, *idx)
                }
                _ => panic!("Expect a branch statement"),
            };
            builder.data.code[code_offset] = updated_goto;
        }

        builder.data
    }

    fn build_loop_annotation(func_env: &FunctionEnv<'_>, data: &FunctionData) -> LoopAnnotation {
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

        // find loop targets and invariants per loop (also means, per label)
        let mut loops = BTreeMap::new();
        for single_loop in &natural_loops {
            let loop_label = loop_header_to_label[&single_loop.loop_header];

            let targets = single_loop
                .loop_body
                .iter()
                .map(|block_id| {
                    cfg.instr_indexes(*block_id)
                        .expect("A loop body should never contain a dummy block")
                })
                .flatten()
                .map(|code_offset| Self::targets(&code[code_offset as usize]))
                .flatten()
                .collect();

            let invariants = cfg
                .instr_indexes(single_loop.loop_header)
                .unwrap()
                .filter_map(|code_offset| {
                    let instruction = &code[code_offset as usize];
                    match instruction {
                        Bytecode::Prop(attr_id, PropKind::Invariant, exp) => {
                            Some((*attr_id, exp.clone()))
                        }
                        _ => None,
                    }
                })
                .collect();

            let back_edge_location = match cfg.content(single_loop.loop_latch) {
                BlockContent::Dummy => panic!("A loop body should never contain a dummy block"),
                BlockContent::Basic { upper, .. } => *upper,
            };
            match &code[back_edge_location as usize] {
                Bytecode::Jump(_, goto_label) if *goto_label == loop_label => {}
                Bytecode::Branch(_, if_label, else_label, _)
                    if *if_label == loop_label || *else_label == loop_label => {}
                _ => panic!("The latch bytecode of a loop does not branch into the header"),
            }

            loops.insert(
                loop_label,
                LoopInfo {
                    invariants,
                    targets,
                    back_edge_location,
                },
            );
        }

        LoopAnnotation { loops }
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

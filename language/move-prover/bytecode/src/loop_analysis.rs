// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    function_data_builder::{FunctionDataBuilder, FunctionDataBuilderOptions},
    function_target::{FunctionData, FunctionTarget},
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder},
    graph::Graph,
    stackless_bytecode::{AttrId, BorrowNode, Bytecode, HavocKind, Label, Operation, PropKind},
    stackless_control_flow_graph::{BlockContent, BlockId, StacklessControlFlowGraph},
};
use move_model::{
    ast::{self, TempIndex},
    model::FunctionEnv,
};
use std::collections::{BTreeMap, BTreeSet};
use vm::file_format::CodeOffset;

const LOOP_INVARIANT_BASE_FAILED: &str = "base case of the loop invariant does not hold";
const LOOP_INVARIANT_INDUCTION_FAILED: &str = "induction case of the loop invariant does not hold";

/// A fat-loop captures the information of one or more natural loops that share the same loop
/// header. This shared header is called the header of the fat-loop.
///
/// Conceptually, every back edge defines a unique natural loop and different back edges may points
/// to the same loop header (e.g., when there are two "continue" statements in the loop body).
///
/// However, since these natural loops share the same loop header, they share the same loop
/// invariants too and the fat-loop targets (i.e., variables that may be changed in any sub-loop)
/// is the union of loop targets per each natural loop that share the header.
#[derive(Debug, Clone)]
pub struct FatLoop {
    pub invariants: BTreeMap<CodeOffset, (AttrId, ast::Exp)>,
    pub val_targets: BTreeSet<TempIndex>,
    pub mut_targets: BTreeMap<TempIndex, bool>,
    pub back_edges: BTreeSet<CodeOffset>,
}

#[derive(Debug, Clone)]
pub struct LoopAnnotation {
    pub fat_loops: BTreeMap<Label, FatLoop>,
}

impl LoopAnnotation {
    fn back_edges_locations(&self) -> BTreeSet<CodeOffset> {
        self.fat_loops
            .values()
            .map(|l| l.back_edges.iter())
            .flatten()
            .copied()
            .collect()
    }

    fn invariants_locations(&self) -> BTreeSet<CodeOffset> {
        self.fat_loops
            .values()
            .map(|l| l.invariants.keys())
            .flatten()
            .copied()
            .collect()
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
        let invariant_locs = loop_annotation.invariants_locations();
        let mut builder = FunctionDataBuilder::new_with_options(
            func_env,
            data,
            FunctionDataBuilderOptions {
                no_fallthrough_jump_removal: true,
            },
        );
        let mut goto_fixes = vec![];
        let code = std::mem::take(&mut builder.data.code);
        for (offset, bytecode) in code.into_iter().enumerate() {
            match bytecode {
                Bytecode::Label(attr_id, label) => {
                    builder.emit(bytecode);
                    builder.set_loc_from_attr(attr_id);
                    if let Some(loop_info) = loop_annotation.fat_loops.get(&label) {
                        // assert loop invariants -> this is the base case
                        for (attr_id, exp) in loop_info.invariants.values() {
                            builder.set_loc_and_vc_info(
                                builder.get_loc(*attr_id),
                                LOOP_INVARIANT_BASE_FAILED,
                            );
                            builder.emit_with(|attr_id| {
                                Bytecode::Prop(attr_id, PropKind::Assert, exp.clone())
                            });
                        }

                        // havoc all loop targets
                        for idx in &loop_info.val_targets {
                            builder.emit_with(|attr_id| {
                                Bytecode::Call(
                                    attr_id,
                                    vec![],
                                    Operation::Havoc(HavocKind::Value),
                                    vec![*idx],
                                    None,
                                )
                            });
                        }
                        for (idx, havoc_all) in &loop_info.mut_targets {
                            let havoc_kind = if *havoc_all {
                                HavocKind::MutationAll
                            } else {
                                HavocKind::MutationValue
                            };
                            builder.emit_with(|attr_id| {
                                Bytecode::Call(
                                    attr_id,
                                    vec![],
                                    Operation::Havoc(havoc_kind),
                                    vec![*idx],
                                    None,
                                )
                            });
                        }

                        // add an additional assumption that the loop did not abort
                        let exp =
                            builder.mk_not(builder.mk_bool_call(ast::Operation::AbortFlag, vec![]));
                        builder.emit_with(|attr_id| Bytecode::Prop(attr_id, PropKind::Assume, exp));

                        // re-assume loop invariants
                        for (attr_id, exp) in loop_info.invariants.values() {
                            builder.emit(Bytecode::Prop(*attr_id, PropKind::Assume, exp.clone()));
                        }
                    }
                }
                Bytecode::Prop(_, PropKind::Assert, _)
                    if invariant_locs.contains(&(offset as CodeOffset)) =>
                {
                    // skip it, as the invariant should have been added as an assert after the label
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

        // create one invariant-checking block for each fat loop
        let invariant_checker_labels: BTreeMap<_, _> = loop_annotation
            .fat_loops
            .keys()
            .map(|label| (*label, builder.new_label()))
            .collect();

        for (label, loop_info) in &loop_annotation.fat_loops {
            let checker_label = invariant_checker_labels.get(label).unwrap();
            builder.set_next_debug_comment(format!(
                "Loop invariant checking block for the loop started with header: L{}",
                label.as_usize()
            ));
            builder.emit_with(|attr_id| Bytecode::Label(attr_id, *checker_label));
            builder.clear_next_debug_comment();

            // add instrumentations to assert loop invariants -> this is the induction case
            for (attr_id, exp) in loop_info.invariants.values() {
                builder.set_loc_and_vc_info(
                    builder.get_loc(*attr_id),
                    LOOP_INVARIANT_INDUCTION_FAILED,
                );
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

        // collect shared headers from loops
        let mut fat_headers = BTreeMap::new();
        for single_loop in natural_loops {
            fat_headers
                .entry(single_loop.loop_header)
                .or_insert_with(Vec::new)
                .push(single_loop);
        }

        // build fat loops by label
        let mut fat_loops = BTreeMap::new();
        for (fat_root, sub_loops) in fat_headers {
            // get the label of the scc root
            let label = match cfg.content(fat_root) {
                BlockContent::Dummy => panic!("A loop header should never be a dummy block"),
                BlockContent::Basic { lower, upper: _ } => match code[*lower as usize] {
                    Bytecode::Label(_, label) => label,
                    _ => panic!("A loop header block is expected to start with a Label bytecode"),
                },
            };

            // collect invariants
            //
            // Loop invariants are defined as the longest sequence of consecutive 'assert'
            // statements in the loop header block, immediately after the Label statement.
            //
            // In other words, for the loop header block:
            // - the first statement must be a 'label',
            // - followed by N 'assert' statements, N >= 0
            // - and N + 1 must not be an 'assert' statement.
            let mut invariants = BTreeMap::new();
            for (index, code_offset) in cfg.instr_indexes(fat_root).unwrap().enumerate() {
                let bytecode = &code[code_offset as usize];
                if index == 0 {
                    assert!(matches!(bytecode, Bytecode::Label(_, _)));
                } else {
                    match bytecode {
                        Bytecode::Prop(attr_id, PropKind::Assert, exp) => {
                            invariants.insert(code_offset, (*attr_id, exp.clone()));
                        }
                        _ => break,
                    }
                }
            }

            // collect fat loop targets
            let mut val_targets = BTreeSet::new();
            let mut mut_targets = BTreeMap::new();
            let fat_loop_body: BTreeSet<_> = sub_loops
                .iter()
                .map(|l| l.loop_body.iter())
                .flatten()
                .copied()
                .collect();
            for block_id in fat_loop_body {
                for code_offset in cfg
                    .instr_indexes(block_id)
                    .expect("A loop body should never contain a dummy block")
                {
                    let (bc_val_targets, bc_mut_targets) =
                        Self::targets(&func_target, &code[code_offset as usize]);
                    val_targets.extend(bc_val_targets);
                    for (idx, is_full_havoc) in bc_mut_targets {
                        mut_targets
                            .entry(idx)
                            .and_modify(|v| {
                                *v = *v || is_full_havoc;
                            })
                            .or_insert(is_full_havoc);
                    }
                }
            }

            // collect back edge locations
            let back_edges = sub_loops
                .iter()
                .map(|l| {
                    let code_offset = match cfg.content(l.loop_latch) {
                        BlockContent::Dummy => {
                            panic!("A loop body should never contain a dummy block")
                        }
                        BlockContent::Basic { upper, .. } => *upper,
                    };
                    match &code[code_offset as usize] {
                        Bytecode::Jump(_, goto_label) if *goto_label == label => {}
                        Bytecode::Branch(_, if_label, else_label, _)
                            if *if_label == label || *else_label == label => {}
                        _ => panic!("The latch bytecode of a loop does not branch into the header"),
                    };
                    code_offset
                })
                .collect();

            // done with all information collection
            fat_loops.insert(
                label,
                FatLoop {
                    invariants,
                    val_targets,
                    mut_targets,
                    back_edges,
                },
            );
        }

        LoopAnnotation { fat_loops }
    }

    fn targets(
        func_target: &FunctionTarget<'_>,
        bytecode: &Bytecode,
    ) -> (Vec<TempIndex>, Vec<(TempIndex, bool)>) {
        use BorrowNode::*;
        match bytecode {
            Bytecode::Assign(_, dest, _, _) => {
                if func_target.get_local_type(*dest).is_mutable_reference() {
                    (vec![], vec![(*dest, true)])
                } else {
                    (vec![*dest], vec![])
                }
            }
            Bytecode::Load(_, dest, _) => (vec![*dest], vec![]),
            Bytecode::Call(_, _, Operation::WriteBack(LocalRoot(dest), ..), ..) => {
                (vec![*dest], vec![])
            }
            Bytecode::Call(_, _, Operation::WriteBack(Reference(dest), ..), ..) => {
                // write-backs only distorts the value
                (vec![], vec![(*dest, false)])
            }
            Bytecode::Call(_, _, Operation::WriteRef, srcs, ..) => {
                // write-ref only distorts the value
                (vec![], vec![(*srcs.get(0).unwrap(), false)])
            }
            Bytecode::Call(_, dests, ..) => {
                let mut val_targets = vec![];
                let mut mut_targets = vec![];
                for dest in dests {
                    if func_target.get_local_type(*dest).is_mutable_reference() {
                        mut_targets.push((*dest, true));
                    } else {
                        val_targets.push(*dest);
                    }
                }
                (val_targets, mut_targets)
            }
            _ => (vec![], vec![]),
        }
    }
}

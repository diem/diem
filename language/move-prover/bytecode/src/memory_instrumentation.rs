// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    borrow_analysis::{BorrowAnnotation, EdgeDomain},
    function_target::{FunctionData, FunctionTarget},
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder},
    stackless_bytecode::{
        AttrId, BorrowEdge, BorrowNode,
        Bytecode::{self, *},
        Operation,
    },
};
use move_binary_format::file_format::CodeOffset;
use move_model::{
    ast::{ConditionKind, TempIndex},
    model::{FunctionEnv, Loc, StructEnv},
    ty::Type,
};
use std::collections::BTreeMap;

pub struct MemoryInstrumentationProcessor {}

impl MemoryInstrumentationProcessor {
    pub fn new() -> Box<Self> {
        Box::new(MemoryInstrumentationProcessor {})
    }
}

impl FunctionTargetProcessor for MemoryInstrumentationProcessor {
    fn process(
        &self,
        _targets: &mut FunctionTargetsHolder,
        func_env: &FunctionEnv<'_>,
        mut data: FunctionData,
    ) -> FunctionData {
        if func_env.is_native() {
            return data;
        }
        let borrow_annotation = data
            .annotations
            .remove::<BorrowAnnotation>()
            .expect("borrow annotation");
        let next_attr_id = data.next_free_attr_index();
        let code = std::mem::take(&mut data.code);
        let func_target = FunctionTarget::new(func_env, &data);
        let mut instrumenter = Instrumenter::new(&func_target, &borrow_annotation, next_attr_id);
        let mut new_code = vec![];
        for (code_offset, bytecode) in code.into_iter().enumerate() {
            let (before, after) =
                instrumenter.compute_instrumentation(code_offset as CodeOffset, &bytecode);
            new_code.extend(before);
            new_code.push(bytecode);
            new_code.extend(after);
        }
        let new_locations = std::mem::take(&mut instrumenter.new_locations);
        data.code = new_code;
        data.locations.extend(new_locations.into_iter());
        data
    }

    fn name(&self) -> String {
        "memory_instr".to_string()
    }
}

struct Instrumenter<'a> {
    func_target: &'a FunctionTarget<'a>,
    borrow_annotation: &'a BorrowAnnotation,
    next_attr_id: usize,
    new_locations: BTreeMap<AttrId, Loc>,
}

impl<'a> Instrumenter<'a> {
    fn new(
        func_target: &'a FunctionTarget<'a>,
        borrow_annotation: &'a BorrowAnnotation,
        next_attr_id: usize,
    ) -> Self {
        Self {
            func_target,
            borrow_annotation,
            next_attr_id,
            new_locations: BTreeMap::new(),
        }
    }

    fn compute_instrumentation(
        &mut self,
        code_offset: CodeOffset,
        bytecode: &Bytecode,
    ) -> (Vec<Bytecode>, Vec<Bytecode>) {
        let (mut before, mut after) = self.public_function_instrumentation(code_offset, bytecode);
        let destroy_instr = self.ref_create_destroy_instrumentation(code_offset, bytecode);
        if matches!(
            bytecode,
            Bytecode::Ret(..) | Bytecode::Branch(..) | Bytecode::Jump(..) | Bytecode::Abort(..)
        ) {
            // Add this to before instrumentation.
            before.extend(destroy_instr);
        } else {
            after.extend(destroy_instr);
        }
        (before, after)
    }

    fn public_function_instrumentation(
        &mut self,
        _code_offset: CodeOffset,
        bytecode: &Bytecode,
    ) -> (Vec<Bytecode>, Vec<Bytecode>) {
        let mut before = vec![];
        let mut after = vec![];
        if let Call(attr_id, _, Operation::Function(mid, fid, _), srcs, _) = bytecode {
            let callee_env = self
                .func_target
                .module_env()
                .env
                .get_module(*mid)
                .into_function(*fid);
            // TODO: this will instrument not only `public` functions, but also `public(script)`
            // and `public(friend)` as well.
            if callee_env.is_exposed() {
                let pack_refs: Vec<&TempIndex> = srcs
                    .iter()
                    .filter(|idx| self.is_pack_ref_ty(self.func_target.get_local_type(**idx)))
                    .collect();
                before.append(
                    &mut pack_refs
                        .iter()
                        .map(|idx| {
                            Bytecode::Call(
                                self.clone_attr(*attr_id),
                                vec![],
                                Operation::PackRef,
                                vec![**idx],
                                None,
                            )
                        })
                        .collect(),
                );
                after.append(
                    &mut pack_refs
                        .into_iter()
                        .map(|idx| {
                            Bytecode::Call(
                                self.clone_attr(*attr_id),
                                vec![],
                                Operation::UnpackRef,
                                vec![*idx],
                                None,
                            )
                        })
                        .collect(),
                );
            }
        }
        (before, after)
    }

    /// Determines whether the type needs a pack ref.
    fn is_pack_ref_ty(&self, ty: &Type) -> bool {
        let env = self.func_target.global_env();
        if let Some((struct_env, _)) = ty.skip_reference().get_struct(env) {
            self.is_pack_ref_struct(&struct_env)
        } else {
            // TODO: vectors
            false
        }
    }

    /// Determines whether the struct needs a pack ref.
    fn is_pack_ref_struct(&self, struct_env: &StructEnv<'_>) -> bool {
        use ConditionKind::*;
        struct_env.get_spec().any(|c| matches!(c.kind, VarUpdate(..)|VarPack(..)|Invariant))
        // If any of the fields has it, it inherits to the struct.
        ||  struct_env
            .get_fields()
            .any(|fe| self.is_pack_ref_ty(&fe.get_type()))
    }

    fn new_attr_id(&mut self, loc: Loc) -> AttrId {
        let attr_id = AttrId::new(self.next_attr_id);
        self.next_attr_id += 1;
        self.new_locations.insert(attr_id, loc);
        attr_id
    }

    fn clone_attr(&mut self, id: AttrId) -> AttrId {
        let loc = self.func_target.get_bytecode_loc(id);
        self.new_attr_id(loc)
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
        let before = &borrow_annotation_at.before;
        let after = &borrow_annotation_at.after;

        let mut instrumented_bytecodes = vec![];

        // Generate UnpackRef from Borrow instructions.
        if let Call(attr_id, dests, op, _, _) = bytecode {
            use Operation::*;
            match op {
                BorrowLoc | BorrowField(..) | BorrowGlobal(..) => {
                    let ty = self.func_target.get_local_type(dests[0]);
                    let node = BorrowNode::Reference(dests[0]);
                    if self.is_pack_ref_ty(ty)
                        && after.is_in_use(&node)
                        && !after.is_unchecked(&node)
                    {
                        instrumented_bytecodes.push(Bytecode::Call(
                            self.clone_attr(*attr_id),
                            vec![],
                            if after.is_spliced(&node) {
                                Operation::UnpackRefDeep
                            } else {
                                Operation::UnpackRef
                            },
                            vec![dests[0]],
                            None,
                        ));
                    }
                }
                _ => {}
            }
        }

        // Generate PackRef for nodes which go out of scope, as well as generate WriteBack.
        let attr_id = bytecode.get_attr_id();
        for node in before.dying_nodes(after) {
            if let BorrowNode::Reference(idx) = &node {
                let ty = self.func_target.get_local_type(*idx);
                if self.is_pack_ref_ty(ty) {
                    // Generate a pack_ref for this reference, unless: (a) the node is marked
                    // as unchecked (b) the node is marked as having been moved to somewhere else.
                    if !before.is_unchecked(&node) && !before.is_moved(&node) {
                        instrumented_bytecodes.push(Bytecode::Call(
                            self.clone_attr(attr_id),
                            vec![],
                            if before.is_spliced(&node) {
                                // If this node has been spliced, we need to perform a deep pack.
                                // A spliced node is one which has a child at some unknown,
                                // dynamically defined path, derived by some function from the parent.
                                // The nodes on this path have not been packed yet, and we therefore
                                // need to do a deep pack.
                                Operation::PackRefDeep
                            } else {
                                Operation::PackRef
                            },
                            vec![*idx],
                            None,
                        ));
                    }
                }
                // Generate write_back for this reference.
                for (parent, edge_dom_ele) in before.get_incoming(&node) {
                    match edge_dom_ele {
                        EdgeDomain::Top => instrumented_bytecodes.push(Bytecode::Call(
                            self.clone_attr(attr_id),
                            vec![],
                            Operation::WriteBack(parent.clone(), BorrowEdge::Weak),
                            vec![*idx],
                            None,
                        )),
                        EdgeDomain::EdgeSet(edges) => {
                            for edge in edges {
                                instrumented_bytecodes.push(Bytecode::Call(
                                    self.clone_attr(attr_id),
                                    vec![],
                                    Operation::WriteBack(parent.clone(), BorrowEdge::Strong(edge)),
                                    vec![*idx],
                                    None,
                                ))
                            }
                        }
                    }
                }
            }
        }
        instrumented_bytecodes
    }
}

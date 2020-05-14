// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    borrow_analysis::BorrowAnnotation,
    function_target::{FunctionTarget, FunctionTargetData},
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder},
    packref_analysis::{PackrefAnnotation, PackrefInstrumentation},
    stackless_bytecode::{
        AssignKind, AttrId, BorrowNode,
        Bytecode::{self, *},
        Operation::*,
        TempIndex,
    },
    writeback_analysis::WritebackAnnotation,
};
use spec_lang::{env::FunctionEnv, ty::Type};
use std::collections::BTreeMap;
use vm::file_format::CodeOffset;

pub struct EliminateMutRefsProcessor {}

impl EliminateMutRefsProcessor {
    pub fn new() -> Box<Self> {
        Box::new(EliminateMutRefsProcessor {})
    }
}

impl FunctionTargetProcessor for EliminateMutRefsProcessor {
    fn process(
        &self,
        _targets: &mut FunctionTargetsHolder,
        func_env: &FunctionEnv<'_>,
        mut data: FunctionTargetData,
    ) -> FunctionTargetData {
        let local_types = &mut data.local_types;
        let return_types = &mut data.return_types;

        let mut param_proxy_map = BTreeMap::new();
        let mut ref_param_proxy_map = BTreeMap::new();
        for idx in 0..func_env.get_parameter_count() {
            param_proxy_map.insert(idx, local_types.len());
            local_types.push(EliminateMutRefs::transform_type(local_types[idx].clone()));
            if local_types[idx].is_reference() {
                let ty = local_types[idx].clone();
                ref_param_proxy_map.insert(idx, local_types.len());
                local_types.push(ty.clone());
                return_types.push(EliminateMutRefs::transform_type(ty));
            }
        }

        let mut max_ref_params_per_type = BTreeMap::new();
        for bytecode in &data.code {
            if let Call(_, _, Function(..), srcs) = bytecode {
                let mut ref_params_per_type = BTreeMap::new();
                for idx in srcs {
                    let ty = &local_types[*idx];
                    if ty.is_reference() {
                        ref_params_per_type
                            .entry(ty)
                            .and_modify(|x| *x += 1)
                            .or_insert_with(|| 1);
                    }
                }
                for (ty, ref_params) in ref_params_per_type {
                    max_ref_params_per_type
                        .entry(ty.clone())
                        .and_modify(|x| {
                            if *x < ref_params {
                                *x = ref_params;
                            }
                        })
                        .or_insert_with(|| ref_params);
                }
            }
        }
        let mut ref_param_inout_proxy_map = BTreeMap::new();
        for (ty, n) in max_ref_params_per_type {
            let curr_len = local_types.len();
            for _ in 0..n {
                local_types.push(EliminateMutRefs::transform_type(ty.clone()));
            }
            ref_param_inout_proxy_map.insert(ty, curr_len);
        }

        // move inputs into the temporaries
        let mut next_attr_id = data.code.len();
        let mut new_code = vec![];
        for idx in 0..func_env.get_parameter_count() {
            new_code.push(Bytecode::Assign(
                AttrId::new(next_attr_id),
                param_proxy_map[&idx],
                idx,
                AssignKind::Move,
            ));
            next_attr_id += 1;
        }

        // take address of inputs that are references
        for idx in 0..func_env.get_parameter_count() {
            let ty = &local_types[idx];
            if ty.is_reference() {
                new_code.push(Bytecode::Call(
                    AttrId::new(next_attr_id),
                    vec![ref_param_proxy_map[&idx]],
                    BorrowLoc,
                    vec![param_proxy_map[&idx]],
                ));
                next_attr_id += 1;
            }
        }

        // transform parameter types
        data.local_types = data
            .local_types
            .into_iter()
            .enumerate()
            .map(|(idx, ty)| {
                if idx < func_env.get_parameter_count() {
                    EliminateMutRefs::transform_type(ty)
                } else {
                    ty
                }
            })
            .collect();

        // transform original code
        data.annotations
            .remove::<BorrowAnnotation>()
            .expect("borrow annotation");
        let writeback_annotation = *data
            .annotations
            .remove::<WritebackAnnotation>()
            .expect("writeback annotation");
        let packref_annotation = *data
            .annotations
            .remove::<PackrefAnnotation>()
            .expect("packref annotation");
        let code = std::mem::take(&mut data.code);
        let func_target = FunctionTarget::new(func_env, &data);
        let mut elim_mut_refs = EliminateMutRefs::new(
            &func_target,
            param_proxy_map,
            ref_param_proxy_map,
            ref_param_inout_proxy_map,
            next_attr_id,
        );
        for (code_offset, bytecode) in code.into_iter().enumerate() {
            let PackrefInstrumentation {
                before: packref_instrs_before,
                after: packref_instrs_after,
            } = packref_annotation
                .get_packref_instrumentation_at(code_offset as CodeOffset)
                .unwrap();
            let writeback_instrs = writeback_annotation
                .get_writeback_instrs_at(code_offset as CodeOffset)
                .unwrap();
            new_code.append(&mut elim_mut_refs.transform_bytecodes(packref_instrs_before));
            new_code.append(&mut elim_mut_refs.transform_bytecode(bytecode));
            new_code.append(&mut elim_mut_refs.transform_bytecodes(writeback_instrs));
            new_code.append(&mut elim_mut_refs.transform_bytecodes(packref_instrs_after));
        }
        data.code = new_code;
        data
    }
}

pub struct EliminateMutRefs<'a> {
    func_target: &'a FunctionTarget<'a>,
    param_proxy_map: BTreeMap<TempIndex, TempIndex>,
    ref_param_proxy_map: BTreeMap<TempIndex, TempIndex>,
    ref_param_inout_proxy_map: BTreeMap<Type, TempIndex>,
    next_attr_id: usize,
}

impl<'a> EliminateMutRefs<'a> {
    fn new(
        func_target: &'a FunctionTarget,
        param_proxy_map: BTreeMap<TempIndex, TempIndex>,
        ref_param_proxy_map: BTreeMap<TempIndex, TempIndex>,
        ref_param_inout_proxy_map: BTreeMap<Type, TempIndex>,
        next_attr_id: usize,
    ) -> Self {
        Self {
            func_target,
            param_proxy_map,
            ref_param_proxy_map,
            ref_param_inout_proxy_map,
            next_attr_id,
        }
    }

    fn transform_type(ty: Type) -> Type {
        if let Type::Reference(_, y) = ty {
            *y
        } else {
            ty
        }
    }

    fn transform_index(&self, idx: TempIndex) -> TempIndex {
        if self.ref_param_proxy_map.contains_key(&idx) {
            self.ref_param_proxy_map[&idx]
        } else if self.param_proxy_map.contains_key(&idx) {
            self.param_proxy_map[&idx]
        } else {
            idx
        }
    }

    fn transform_index_for_local_root(&self, idx: TempIndex) -> TempIndex {
        if self.param_proxy_map.contains_key(&idx) {
            self.param_proxy_map[&idx]
        } else {
            idx
        }
    }

    fn transform_indices(&self, indices: Vec<TempIndex>) -> Vec<TempIndex> {
        indices
            .into_iter()
            .map(|idx| self.transform_index(idx))
            .collect()
    }

    fn transform_bytecode_indices(&self, bytecode: Bytecode) -> Bytecode {
        use BorrowNode::*;
        match bytecode {
            Assign(attr_id, dest, src, kind) => Assign(
                attr_id,
                self.transform_index(dest),
                self.transform_index(src),
                kind,
            ),
            Call(attr_id, dests, op, srcs) => Call(
                attr_id,
                self.transform_indices(dests),
                op,
                self.transform_indices(srcs),
            ),
            Ret(attr_id, srcs) => Ret(attr_id, self.transform_indices(srcs)),
            Load(attr_id, dest, c) => Load(attr_id, self.transform_index(dest), c),
            Branch(attr_id, then_label, else_label, src) => {
                Branch(attr_id, then_label, else_label, self.transform_index(src))
            }
            WriteBack(attr_id, GlobalRoot(struct_decl), src) => {
                WriteBack(attr_id, GlobalRoot(struct_decl), self.transform_index(src))
            }
            WriteBack(attr_id, LocalRoot(dest), src) => WriteBack(
                attr_id,
                LocalRoot(self.transform_index_for_local_root(dest)),
                self.transform_index(src),
            ),
            WriteBack(attr_id, Reference(dest), src) => WriteBack(
                attr_id,
                Reference(self.transform_index(dest)),
                self.transform_index(src),
            ),
            UnpackRef(attr_id, src) => UnpackRef(attr_id, self.transform_index(src)),
            PackRef(attr_id, src) => PackRef(attr_id, self.transform_index(src)),
            _ => bytecode,
        }
    }

    fn new_attr_id(&mut self) -> AttrId {
        let attr_id = AttrId::new(self.next_attr_id);
        self.next_attr_id += 1;
        attr_id
    }

    fn transform_bytecode(&mut self, bytecode: Bytecode) -> Vec<Bytecode> {
        let bytecode = self.transform_bytecode_indices(bytecode);
        match bytecode {
            Call(attr_id, mut dests, Function(mid, fid, type_actuals), mut srcs) => {
                let mut ref_param_count_per_type = BTreeMap::new();
                let old_srcs = std::mem::take(&mut srcs);
                let mut read_ref_bytecodes = vec![];
                let mut write_ref_bytecodes = vec![];
                let mut splice_map = BTreeMap::new();
                for (pos, idx) in old_srcs.into_iter().enumerate() {
                    let ty = self.func_target.get_local_type(idx);
                    if ty.is_reference() {
                        let ref_param_count =
                            ref_param_count_per_type.entry(ty).or_insert_with(|| 0);
                        let read_ref_dest_idx =
                            self.ref_param_inout_proxy_map[ty] + *ref_param_count;
                        srcs.push(read_ref_dest_idx);
                        dests.push(read_ref_dest_idx);
                        *ref_param_count += 1;
                        read_ref_bytecodes.push(Call(
                            self.new_attr_id(),
                            vec![read_ref_dest_idx],
                            ReadRef,
                            vec![idx],
                        ));
                        write_ref_bytecodes.push(Call(
                            self.new_attr_id(),
                            vec![],
                            WriteRef,
                            vec![idx, read_ref_dest_idx],
                        ));
                        splice_map.insert(pos, idx);
                    } else {
                        srcs.push(idx);
                    }
                }
                let mut splice_bytecodes = vec![];
                for idx in &dests {
                    let ty = self.func_target.get_local_type(*idx);
                    if ty.is_reference() {
                        splice_bytecodes.push(Splice(self.new_attr_id(), *idx, splice_map.clone()));
                    }
                }

                let mut return_bytecodes = vec![];
                return_bytecodes.append(&mut read_ref_bytecodes);
                return_bytecodes.push(Call(attr_id, dests, Function(mid, fid, type_actuals), srcs));
                return_bytecodes.append(&mut write_ref_bytecodes);
                return_bytecodes.append(&mut splice_bytecodes);
                return_bytecodes
            }
            Ret(attr_id, mut srcs) => {
                for idx in self.ref_param_proxy_map.keys() {
                    srcs.push(self.param_proxy_map[idx]);
                }
                vec![Ret(attr_id, srcs)]
            }
            _ => vec![bytecode],
        }
    }

    fn transform_bytecodes(&mut self, instrs: &[Bytecode]) -> Vec<Bytecode> {
        instrs
            .iter()
            .flat_map(|bytecode| self.transform_bytecode(bytecode.clone()))
            .collect()
    }
}

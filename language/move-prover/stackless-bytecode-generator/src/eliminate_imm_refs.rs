// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    function_target::{FunctionTarget, FunctionTargetData},
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder},
    stackless_bytecode::{
        AssignKind,
        Bytecode::{self, *},
        Operation::*,
    },
};
use spec_lang::{env::FunctionEnv, ty::Type};

pub struct EliminateImmRefsProcessor {}

impl EliminateImmRefsProcessor {
    pub fn new() -> Box<Self> {
        Box::new(EliminateImmRefsProcessor {})
    }
}

impl FunctionTargetProcessor for EliminateImmRefsProcessor {
    fn process(
        &self,
        _targets: &mut FunctionTargetsHolder,
        func_env: &FunctionEnv<'_>,
        mut data: FunctionTargetData,
    ) -> FunctionTargetData {
        let code = std::mem::replace(&mut data.code, vec![]);
        data.code = code
            .into_iter()
            .map(|bytecode| {
                EliminateImmRefs::new(&FunctionTarget::new(func_env, &data))
                    .transform_bytecode(bytecode)
            })
            .collect();
        let local_types = std::mem::replace(&mut data.local_types, vec![]);
        data.local_types = local_types
            .into_iter()
            .map(|ty| {
                EliminateImmRefs::new(&FunctionTarget::new(func_env, &data)).transform_type(ty)
            })
            .collect();
        let return_types = std::mem::replace(&mut data.return_types, vec![]);
        data.return_types = return_types
            .into_iter()
            .map(|ty| {
                EliminateImmRefs::new(&FunctionTarget::new(func_env, &data)).transform_type(ty)
            })
            .collect();
        data
    }
}

pub struct EliminateImmRefs<'a> {
    func_target: &'a FunctionTarget<'a>,
}

impl<'a> EliminateImmRefs<'a> {
    fn new(func_target: &'a FunctionTarget) -> Self {
        Self { func_target }
    }

    fn transform_type(&self, ty: Type) -> Type {
        if let Type::Reference(false, y) = ty {
            *y
        } else {
            ty
        }
    }

    fn transform_bytecode(&self, bytecode: Bytecode) -> Bytecode {
        match &bytecode {
            Call(attr_id, dests, op, srcs) => match op {
                ReadRef => {
                    let src = srcs[0];
                    let dest = dests[0];
                    if self
                        .func_target
                        .get_local_type(src)
                        .is_immutable_reference()
                    {
                        Assign(*attr_id, dest, src, AssignKind::Move)
                    } else {
                        bytecode
                    }
                }
                FreezeRef => Call(*attr_id, dests.to_vec(), ReadRef, srcs.to_vec()),
                BorrowLoc => {
                    let src = srcs[0];
                    let dest = dests[0];
                    if self
                        .func_target
                        .get_local_type(dest)
                        .is_immutable_reference()
                    {
                        Assign(*attr_id, dest, src, AssignKind::Copy)
                    } else {
                        bytecode
                    }
                }
                BorrowField(mid, sid, type_actuals, field_offset) => {
                    let dest = dests[0];
                    if self
                        .func_target
                        .get_local_type(dest)
                        .is_immutable_reference()
                    {
                        Call(
                            *attr_id,
                            dests.to_vec(),
                            GetField(*mid, *sid, type_actuals.to_vec(), *field_offset),
                            srcs.to_vec(),
                        )
                    } else {
                        bytecode
                    }
                }
                BorrowGlobal(mid, sid, type_actuals) => {
                    let dest = dests[0];
                    if self
                        .func_target
                        .get_local_type(dest)
                        .is_immutable_reference()
                    {
                        Call(
                            *attr_id,
                            dests.to_vec(),
                            GetGlobal(*mid, *sid, type_actuals.to_vec()),
                            srcs.to_vec(),
                        )
                    } else {
                        bytecode
                    }
                }
                _ => bytecode,
            },
            _ => bytecode,
        }
    }
}

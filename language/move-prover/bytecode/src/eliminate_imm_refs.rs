// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    function_data_builder::FunctionDataBuilder,
    function_target::FunctionData,
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder},
    stackless_bytecode::{AssignKind, Bytecode, Operation},
};
use move_model::{ast::TempIndex, model::FunctionEnv, ty::Type};

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
        data: FunctionData,
    ) -> FunctionData {
        let mut elim = EliminateImmRefs::new(FunctionDataBuilder::new(func_env, data));
        elim.run();
        elim.builder.data
    }

    fn name(&self) -> String {
        "eliminate_imm_refs".to_string()
    }
}

pub struct EliminateImmRefs<'a> {
    builder: FunctionDataBuilder<'a>,
}

impl<'a> EliminateImmRefs<'a> {
    fn new(builder: FunctionDataBuilder<'a>) -> Self {
        Self { builder }
    }

    fn run(&mut self) {
        for bc in std::mem::take(&mut self.builder.data.code) {
            self.transform_bytecode(bc);
        }
        self.builder.data.local_types = std::mem::take(&mut self.builder.data.local_types)
            .into_iter()
            .map(|ty| self.transform_type(ty))
            .collect();
        self.builder.data.return_types = std::mem::take(&mut self.builder.data.return_types)
            .into_iter()
            .map(|ty| self.transform_type(ty))
            .collect();
    }

    fn transform_type(&self, ty: Type) -> Type {
        if let Type::Reference(false, y) = ty {
            *y
        } else {
            ty
        }
    }

    fn is_imm_ref(&self, idx: TempIndex) -> bool {
        self.builder
            .get_target()
            .get_local_type(idx)
            .is_immutable_reference()
    }

    fn transform_bytecode(&mut self, bytecode: Bytecode) {
        use Bytecode::*;
        use Operation::*;
        match bytecode {
            Call(attr_id, dests, op, srcs, aa) => match op {
                ReadRef if self.is_imm_ref(srcs[0]) => {
                    self.builder
                        .emit(Assign(attr_id, dests[0], srcs[0], AssignKind::Move));
                }
                FreezeRef => self.builder.emit(Call(attr_id, dests, ReadRef, srcs, None)),
                BorrowLoc if self.is_imm_ref(dests[0]) => {
                    self.builder
                        .emit(Assign(attr_id, dests[0], srcs[0], AssignKind::Copy));
                }
                BorrowField(mid, sid, type_actuals, offset) if self.is_imm_ref(dests[0]) => {
                    self.builder.emit(Call(
                        attr_id,
                        dests,
                        GetField(mid, sid, type_actuals, offset),
                        srcs,
                        aa,
                    ));
                }
                BorrowGlobal(mid, sid, type_actuals) if self.is_imm_ref(dests[0]) => {
                    self.builder.emit(Call(
                        attr_id,
                        dests,
                        GetGlobal(mid, sid, type_actuals),
                        srcs,
                        aa,
                    ));
                }
                _ => self.builder.emit(Call(attr_id, dests, op, srcs, aa)),
            },
            _ => self.builder.emit(bytecode),
        };
    }
}

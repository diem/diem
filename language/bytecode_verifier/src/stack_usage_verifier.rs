// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements a checker for verifying that basic blocks in the bytecode instruction
//! sequence of a function use the evaluation stack in a balanced manner. Every basic block,
//! except those that end in Ret (return to caller) opcode, must leave the stack height the
//! same as at the beginning of the block. A basic block that ends in Ret opcode must increase
//! the stack height by the number of values returned by the function as indicated in its
//! signature. Additionally, the stack height must not dip below that at the beginning of the
//! block for any basic block.
use crate::{
    code_unit_verifier::VerificationPass,
    control_flow_graph::{BasicBlock, VMControlFlowGraph},
};
use vm::{
    access::{BaseAccess, ModuleAccess},
    errors::VMStaticViolation,
    file_format::{Bytecode, CompiledModule, FunctionDefinition},
    views::FunctionDefinitionView,
};

pub struct StackUsageVerifier<'a> {
    module: &'a CompiledModule,
    function_definition_view: FunctionDefinitionView<'a, CompiledModule>,
    cfg: &'a VMControlFlowGraph,
}

impl<'a> VerificationPass<'a> for StackUsageVerifier<'a> {
    fn new(
        module: &'a CompiledModule,
        function_definition: &'a FunctionDefinition,
        cfg: &'a VMControlFlowGraph,
    ) -> Self {
        let function_definition_view = FunctionDefinitionView::new(module, function_definition);
        Self {
            module,
            function_definition_view,
            cfg,
        }
    }

    fn verify(self) -> Vec<VMStaticViolation> {
        let mut errors = vec![];
        for (_, block) in self.cfg.blocks.iter() {
            errors.append(&mut self.verify_block(&block));
        }
        errors
    }
}

impl<'a> StackUsageVerifier<'a> {
    fn verify_block(&self, block: &BasicBlock) -> Vec<VMStaticViolation> {
        let code = &self.function_definition_view.code().code;
        let mut stack_size_increment = 0;
        for i in block.entry..=block.exit {
            stack_size_increment += self.instruction_effect(&code[i as usize]);
            if stack_size_increment < 0 {
                return vec![VMStaticViolation::NegativeStackSizeInsideBlock(
                    block.entry as usize,
                    i as usize,
                )];
            }
        }

        if stack_size_increment == 0 {
            vec![]
        } else {
            vec![VMStaticViolation::PositiveStackSizeAtBlockEnd(
                block.entry as usize,
            )]
        }
    }

    fn instruction_effect(&self, instruction: &Bytecode) -> i32 {
        match instruction {
            Bytecode::Pop | Bytecode::BrTrue(_) | Bytecode::BrFalse(_) | Bytecode::StLoc(_) => -1,

            Bytecode::Ret => {
                let return_count = self.function_definition_view.signature().return_count() as i32;
                -return_count
            }

            Bytecode::Branch(_) | Bytecode::BorrowField(_) => 0,

            Bytecode::LdConst(_)
            | Bytecode::LdAddr(_)
            | Bytecode::LdStr(_)
            | Bytecode::LdTrue
            | Bytecode::LdFalse
            | Bytecode::CopyLoc(_)
            | Bytecode::MoveLoc(_)
            | Bytecode::BorrowLoc(_) => 1,

            Bytecode::Call(idx) => {
                let function_handle = self.module.function_handle_at(*idx);
                let signature = self.module.function_signature_at(function_handle.signature);
                let arg_count = signature.arg_types.len() as i32;
                let return_count = signature.return_types.len() as i32;
                return_count - arg_count
            }

            Bytecode::Pack(idx) => {
                let struct_definition = self.module.struct_def_at(*idx);
                let num_fields = i32::from(struct_definition.field_count);
                1 - num_fields
            }

            Bytecode::Unpack(idx) => {
                let struct_definition = self.module.struct_def_at(*idx);
                let num_fields = i32::from(struct_definition.field_count);
                num_fields - 1
            }

            Bytecode::ReadRef => 0,

            Bytecode::WriteRef | Bytecode::Assert => -2,

            Bytecode::Add
            | Bytecode::Sub
            | Bytecode::Mul
            | Bytecode::Mod
            | Bytecode::Div
            | Bytecode::BitOr
            | Bytecode::BitAnd
            | Bytecode::Xor
            | Bytecode::Or
            | Bytecode::And
            | Bytecode::Eq
            | Bytecode::Neq
            | Bytecode::Lt
            | Bytecode::Gt
            | Bytecode::Le
            | Bytecode::Ge => -1,

            Bytecode::Not => 0,

            Bytecode::FreezeRef => 0,
            Bytecode::Exists(_) => 0,
            Bytecode::BorrowGlobal(_) => 0,
            Bytecode::ReleaseRef => -1,
            Bytecode::MoveFrom(_) => 0,
            Bytecode::MoveToSender(_) => -1,

            Bytecode::GetTxnGasUnitPrice
            | Bytecode::GetTxnMaxGasUnits
            | Bytecode::GetGasRemaining
            | Bytecode::GetTxnPublicKey
            | Bytecode::GetTxnSequenceNumber
            | Bytecode::GetTxnSenderAddress => 1,
            Bytecode::CreateAccount => -1,
            Bytecode::EmitEvent => -3,

            Bytecode::LdByteArray(_) => 1,
        }
    }
}

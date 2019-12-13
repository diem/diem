// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements a checker for verifying that basic blocks in the bytecode instruction
//! sequence of a function use the evaluation stack in a balanced manner. Every basic block,
//! except those that end in Ret (return to caller) opcode, must leave the stack height the
//! same as at the beginning of the block. A basic block that ends in Ret opcode must increase
//! the stack height by the number of values returned by the function as indicated in its
//! signature. Additionally, the stack height must not dip below that at the beginning of the
//! block for any basic block.
use crate::control_flow_graph::{BlockId, ControlFlowGraph, VMControlFlowGraph};
use libra_types::vm_error::{StatusCode, VMStatus};
use vm::{
    access::ModuleAccess,
    errors::err_at_offset,
    file_format::{Bytecode, CompiledModule, FunctionDefinition, StructFieldInformation},
    views::FunctionDefinitionView,
};

pub struct StackUsageVerifier<'a> {
    module: &'a CompiledModule,
    function_definition_view: FunctionDefinitionView<'a, CompiledModule>,
}

impl<'a> StackUsageVerifier<'a> {
    pub fn verify(
        module: &'a CompiledModule,
        function_definition: &'a FunctionDefinition,
        cfg: &'a VMControlFlowGraph,
    ) -> Vec<VMStatus> {
        let function_definition_view = FunctionDefinitionView::new(module, function_definition);
        let verifier = Self {
            module,
            function_definition_view,
        };

        let mut errors = vec![];
        for block_id in cfg.blocks() {
            errors.append(&mut verifier.verify_block(&block_id, cfg));
        }
        errors
    }

    fn verify_block(&self, block_id: &BlockId, cfg: &dyn ControlFlowGraph) -> Vec<VMStatus> {
        let code = &self.function_definition_view.code().code;
        let mut stack_size_increment = 0;
        let block_start = cfg.block_start(block_id);
        for i in block_start..=cfg.block_end(block_id) {
            let (num_pops, num_pushes) = self.instruction_effect(&code[i as usize]);
            // Check that the stack height is sufficient to accomodate the number
            // of pops this instruction does
            if stack_size_increment < num_pops {
                return vec![err_at_offset(
                    StatusCode::NEGATIVE_STACK_SIZE_WITHIN_BLOCK,
                    block_start as usize,
                )];
            }
            stack_size_increment -= num_pops;
            stack_size_increment += num_pushes;
        }

        if stack_size_increment == 0 {
            vec![]
        } else {
            vec![err_at_offset(
                StatusCode::POSITIVE_STACK_SIZE_AT_BLOCK_END,
                block_start as usize,
            )]
        }
    }

    /// The effect of an instruction is a tuple where the first element
    /// is the number of pops it does, and the second element is the number
    /// of pushes it does
    fn instruction_effect(&self, instruction: &Bytecode) -> (u32, u32) {
        match instruction {
            // Instructions that pop, but don't push
            Bytecode::Pop
            | Bytecode::BrTrue(_)
            | Bytecode::BrFalse(_)
            | Bytecode::Abort
            | Bytecode::MoveToSender(_, _)
            | Bytecode::StLoc(_) => (1, 0),

            // Instructions that push, but don't pop
            Bytecode::LdConst(_)
            | Bytecode::LdAddr(_)
            | Bytecode::LdStr(_)
            | Bytecode::LdTrue
            | Bytecode::LdFalse
            | Bytecode::LdByteArray(_)
            | Bytecode::CopyLoc(_)
            | Bytecode::MoveLoc(_)
            | Bytecode::MutBorrowLoc(_)
            | Bytecode::ImmBorrowLoc(_)
            | Bytecode::GetTxnGasUnitPrice
            | Bytecode::GetTxnMaxGasUnits
            | Bytecode::GetGasRemaining
            | Bytecode::GetTxnPublicKey
            | Bytecode::GetTxnSequenceNumber
            | Bytecode::GetTxnSenderAddress => (0, 1),

            // Instructions that pop and push once
            Bytecode::Not
            | Bytecode::FreezeRef
            | Bytecode::ReadRef
            | Bytecode::Exists(_, _)
            | Bytecode::MutBorrowGlobal(_, _)
            | Bytecode::ImmBorrowGlobal(_, _)
            | Bytecode::MutBorrowField(_)
            | Bytecode::ImmBorrowField(_)
            | Bytecode::MoveFrom(_, _) => (1, 1),

            // Binary operations (pop twice and push once)
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
            | Bytecode::Ge => (2, 1),

            // WriteRef pops twice but does not push
            Bytecode::WriteRef => (2, 0),

            // Branch neither pops nor pushes
            Bytecode::Branch(_) => (0, 0),

            // Return performs `return_count` pops
            Bytecode::Ret => {
                let return_count = self.function_definition_view.signature().return_count() as u32;
                (return_count, 0)
            }

            // Call performs `arg_count` pops and `return_count` pushes
            Bytecode::Call(idx, _) => {
                let function_handle = self.module.function_handle_at(*idx);
                let signature = self.module.function_signature_at(function_handle.signature);
                let arg_count = signature.arg_types.len() as u32;
                let return_count = signature.return_types.len() as u32;
                (arg_count, return_count)
            }

            // Pack performs `num_fields` pops and one push
            Bytecode::Pack(idx, _) => {
                let struct_definition = self.module.struct_def_at(*idx);
                let field_count = match &struct_definition.field_information {
                    // 'Native' here is an error that will be caught by the bytecode verifier later
                    StructFieldInformation::Native => 0,
                    StructFieldInformation::Declared { field_count, .. } => *field_count,
                };
                let num_fields = u32::from(field_count);
                (num_fields, 1)
            }

            // Unpack performs one pop and `num_fields` pushes
            Bytecode::Unpack(idx, _) => {
                let struct_definition = self.module.struct_def_at(*idx);
                let field_count = match &struct_definition.field_information {
                    // 'Native' here is an error that will be caught by the bytecode verifier later
                    StructFieldInformation::Native => 0,
                    StructFieldInformation::Declared { field_count, .. } => *field_count,
                };
                let num_fields = u32::from(field_count);
                (1, num_fields)
            }
        }
    }
}

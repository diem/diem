// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements a checker for verifying that basic blocks in the bytecode instruction
//! sequence of a function use the evaluation stack in a balanced manner. Every basic block,
//! except those that end in Ret (return to caller) opcode, must leave the stack height the
//! same as at the beginning of the block. A basic block that ends in Ret opcode must increase
//! the stack height by the number of values returned by the function as indicated in its
//! signature. Additionally, the stack height must not dip below that at the beginning of the
//! block for any basic block.
use crate::{
    binary_views::{BinaryIndexedView, FunctionView},
    control_flow_graph::{BlockId, ControlFlowGraph},
};
use diem_types::vm_status::StatusCode;
use vm::{
    errors::{PartialVMError, PartialVMResult},
    file_format::{Bytecode, CodeUnit, FunctionDefinitionIndex, Signature, StructFieldInformation},
};

pub(crate) struct StackUsageVerifier<'a> {
    resolver: &'a BinaryIndexedView<'a>,
    current_function: Option<FunctionDefinitionIndex>,
    code: &'a CodeUnit,
    return_: &'a Signature,
}

impl<'a> StackUsageVerifier<'a> {
    pub(crate) fn verify(
        resolver: &'a BinaryIndexedView<'a>,
        function_view: &'a FunctionView,
    ) -> PartialVMResult<()> {
        let verifier = Self {
            resolver,
            current_function: function_view.index(),
            code: function_view.code(),
            return_: function_view.return_(),
        };

        for block_id in function_view.cfg().blocks() {
            verifier.verify_block(block_id, function_view.cfg())?
        }
        Ok(())
    }

    fn verify_block(&self, block_id: BlockId, cfg: &dyn ControlFlowGraph) -> PartialVMResult<()> {
        let code = &self.code.code;
        let mut stack_size_increment = 0;
        let block_start = cfg.block_start(block_id);
        for i in block_start..=cfg.block_end(block_id) {
            let (num_pops, num_pushes) = self.instruction_effect(&code[i as usize])?;
            // Check that the stack height is sufficient to accomodate the number
            // of pops this instruction does
            if stack_size_increment < num_pops {
                return Err(
                    PartialVMError::new(StatusCode::NEGATIVE_STACK_SIZE_WITHIN_BLOCK)
                        .at_code_offset(self.current_function(), block_start),
                );
            }
            stack_size_increment -= num_pops;
            stack_size_increment += num_pushes;
        }

        if stack_size_increment == 0 {
            Ok(())
        } else {
            Err(
                PartialVMError::new(StatusCode::POSITIVE_STACK_SIZE_AT_BLOCK_END)
                    .at_code_offset(self.current_function(), block_start),
            )
        }
    }

    /// The effect of an instruction is a tuple where the first element
    /// is the number of pops it does, and the second element is the number
    /// of pushes it does
    fn instruction_effect(&self, instruction: &Bytecode) -> PartialVMResult<(u32, u32)> {
        Ok(match instruction {
            // Instructions that pop, but don't push
            Bytecode::Pop
            | Bytecode::BrTrue(_)
            | Bytecode::BrFalse(_)
            | Bytecode::Abort
            | Bytecode::StLoc(_) => (1, 0),

            // Instructions that push, but don't pop
            Bytecode::LdU8(_)
            | Bytecode::LdU64(_)
            | Bytecode::LdU128(_)
            | Bytecode::LdTrue
            | Bytecode::LdFalse
            | Bytecode::LdConst(_)
            | Bytecode::CopyLoc(_)
            | Bytecode::MoveLoc(_)
            | Bytecode::MutBorrowLoc(_)
            | Bytecode::ImmBorrowLoc(_) => (0, 1),

            // Instructions that pop and push once
            Bytecode::Not
            | Bytecode::FreezeRef
            | Bytecode::ReadRef
            | Bytecode::Exists(_)
            | Bytecode::ExistsGeneric(_)
            | Bytecode::MutBorrowGlobal(_)
            | Bytecode::MutBorrowGlobalGeneric(_)
            | Bytecode::ImmBorrowGlobal(_)
            | Bytecode::ImmBorrowGlobalGeneric(_)
            | Bytecode::MutBorrowField(_)
            | Bytecode::MutBorrowFieldGeneric(_)
            | Bytecode::ImmBorrowField(_)
            | Bytecode::ImmBorrowFieldGeneric(_)
            | Bytecode::MoveFrom(_)
            | Bytecode::MoveFromGeneric(_)
            | Bytecode::CastU8
            | Bytecode::CastU64
            | Bytecode::CastU128 => (1, 1),

            // Binary operations (pop twice and push once)
            Bytecode::Add
            | Bytecode::Sub
            | Bytecode::Mul
            | Bytecode::Mod
            | Bytecode::Div
            | Bytecode::BitOr
            | Bytecode::BitAnd
            | Bytecode::Xor
            | Bytecode::Shl
            | Bytecode::Shr
            | Bytecode::Or
            | Bytecode::And
            | Bytecode::Eq
            | Bytecode::Neq
            | Bytecode::Lt
            | Bytecode::Gt
            | Bytecode::Le
            | Bytecode::Ge => (2, 1),

            // MoveTo and WriteRef pop twice but do not push
            Bytecode::MoveTo(_) | Bytecode::MoveToGeneric(_) | Bytecode::WriteRef => (2, 0),

            // Branch and Nop neither pops nor pushes
            Bytecode::Branch(_) | Bytecode::Nop => (0, 0),

            // Return performs `return_count` pops
            Bytecode::Ret => {
                let return_count = self.return_.len();
                (return_count as u32, 0)
            }

            // Call performs `arg_count` pops and `return_count` pushes
            Bytecode::Call(idx) => {
                let function_handle = self.resolver.function_handle_at(*idx);
                let arg_count = self.resolver.signature_at(function_handle.parameters).len() as u32;
                let return_count = self.resolver.signature_at(function_handle.return_).len() as u32;
                (arg_count, return_count)
            }
            Bytecode::CallGeneric(idx) => {
                let func_inst = self.resolver.function_instantiation_at(*idx);
                let function_handle = self.resolver.function_handle_at(func_inst.handle);
                let arg_count = self.resolver.signature_at(function_handle.parameters).len() as u32;
                let return_count = self.resolver.signature_at(function_handle.return_).len() as u32;
                (arg_count, return_count)
            }

            // Pack performs `num_fields` pops and one push
            Bytecode::Pack(idx) => {
                let struct_definition = self.resolver.struct_def_at(*idx)?;
                let field_count = match &struct_definition.field_information {
                    // 'Native' here is an error that will be caught by the bytecode verifier later
                    StructFieldInformation::Native => 0,
                    StructFieldInformation::Declared(fields) => fields.len(),
                };
                (field_count as u32, 1)
            }
            Bytecode::PackGeneric(idx) => {
                let struct_inst = self.resolver.struct_instantiation_at(*idx)?;
                let struct_definition = self.resolver.struct_def_at(struct_inst.def)?;
                let field_count = match &struct_definition.field_information {
                    // 'Native' here is an error that will be caught by the bytecode verifier later
                    StructFieldInformation::Native => 0,
                    StructFieldInformation::Declared(fields) => fields.len(),
                };
                (field_count as u32, 1)
            }

            // Unpack performs one pop and `num_fields` pushes
            Bytecode::Unpack(idx) => {
                let struct_definition = self.resolver.struct_def_at(*idx)?;
                let field_count = match &struct_definition.field_information {
                    // 'Native' here is an error that will be caught by the bytecode verifier later
                    StructFieldInformation::Native => 0,
                    StructFieldInformation::Declared(fields) => fields.len(),
                };
                (1, field_count as u32)
            }
            Bytecode::UnpackGeneric(idx) => {
                let struct_inst = self.resolver.struct_instantiation_at(*idx)?;
                let struct_definition = self.resolver.struct_def_at(struct_inst.def)?;
                let field_count = match &struct_definition.field_information {
                    // 'Native' here is an error that will be caught by the bytecode verifier later
                    StructFieldInformation::Native => 0,
                    StructFieldInformation::Declared(fields) => fields.len(),
                };
                (1, field_count as u32)
            }
        })
    }

    fn current_function(&self) -> FunctionDefinitionIndex {
        self.current_function.unwrap_or(FunctionDefinitionIndex(0))
    }
}

// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements the checker for verifying correctness of function bodies.
//! The overall verification is split between stack_usage_verifier.rs and
//! abstract_interpreter.rs. CodeUnitVerifier simply orchestrates calls into these two files.
use crate::{
    acquires_list_verifier::AcquiresVerifier, control_flow_graph::VMControlFlowGraph,
    locals_safety, reference_safety, stack_usage_verifier::StackUsageVerifier, type_safety,
};
use libra_types::vm_error::{StatusCode, VMStatus};
use vm::{
    access::ModuleAccess,
    errors::{append_err_info, err_at_offset, VMResult},
    file_format::{CompiledModule, FunctionDefinition},
    IndexKind,
};

pub struct CodeUnitVerifier<'a> {
    module: &'a CompiledModule,
}

impl<'a> CodeUnitVerifier<'a> {
    pub fn verify(module: &'a CompiledModule) -> VMResult<()> {
        let verifier = Self { module };
        for (idx, function_definition) in verifier.module.function_defs().iter().enumerate() {
            verifier
                .verify_function(function_definition)
                .map_err(|err| append_err_info(err, IndexKind::FunctionDefinition, idx))?
        }
        Ok(())
    }

    fn verify_function(&self, function_definition: &FunctionDefinition) -> VMResult<()> {
        let code = match &function_definition.code {
            Some(code) => &code.code,
            None => return Ok(()),
        };

        // Check to make sure that the bytecode vector ends with a branching instruction.
        match code.last() {
            None => return Err(VMStatus::new(StatusCode::EMPTY_CODE_UNIT)),
            Some(bytecode) if !bytecode.is_unconditional_branch() => {
                return Err(err_at_offset(
                    StatusCode::INVALID_FALL_THROUGH,
                    code.len() - 1,
                ))
            }
            Some(_) => (),
        }
        self.verify_function_inner(function_definition, &VMControlFlowGraph::new(code))
    }

    fn verify_function_inner(
        &self,
        function_definition: &FunctionDefinition,
        cfg: &VMControlFlowGraph,
    ) -> VMResult<()> {
        StackUsageVerifier::verify(self.module, function_definition, cfg)?;
        AcquiresVerifier::verify(self.module, function_definition)?;
        type_safety::verify(self.module, function_definition, cfg)?;
        locals_safety::verify(self.module, function_definition, cfg)?;
        reference_safety::verify(self.module, function_definition, cfg)
    }
}

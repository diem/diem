// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements the checker for verifying correctness of function bodies.
//! The overall verification is split between stack_usage_verifier.rs and
//! abstract_interpreter.rs. CodeUnitVerifier simply orchestrates calls into these two files.
use crate::control_flow_graph::VMControlFlowGraph;
use libra_types::vm_error::{StatusCode, VMStatus};
use vm::{
    access::ModuleAccess,
    errors::append_err_info,
    file_format::{CompiledModule, FunctionDefinition},
    IndexKind,
};

use crate::{
    acquires_list_verifier::AcquiresVerifier, stack_usage_verifier::StackUsageVerifier,
    type_memory_safety::TypeAndMemorySafetyAnalysis,
};

pub struct CodeUnitVerifier<'a> {
    module: &'a CompiledModule,
}

impl<'a> CodeUnitVerifier<'a> {
    pub fn verify(module: &'a CompiledModule) -> Vec<VMStatus> {
        let verifier = Self { module };
        verifier
            .module
            .function_defs()
            .iter()
            .enumerate()
            .flat_map(move |(idx, function_definition)| {
                verifier
                    .verify_function(function_definition)
                    .into_iter()
                    .map(move |err| append_err_info(err, IndexKind::FunctionDefinition, idx))
            })
            .collect()
    }

    fn verify_function(&self, function_definition: &FunctionDefinition) -> Vec<VMStatus> {
        if function_definition.is_native() {
            return vec![];
        }

        let code = &function_definition.code.code;

        // Check to make sure that the bytecode vector ends with a branching instruction.
        if let Some(bytecode) = code.last() {
            if !bytecode.is_unconditional_branch() {
                return vec![VMStatus::new(StatusCode::INVALID_FALL_THROUGH)];
            }
        } else {
            return vec![VMStatus::new(StatusCode::INVALID_FALL_THROUGH)];
        }

        self.verify_function_inner(function_definition, &VMControlFlowGraph::new(code))
    }

    fn verify_function_inner(
        &self,
        function_definition: &FunctionDefinition,
        cfg: &VMControlFlowGraph,
    ) -> Vec<VMStatus> {
        let errors = StackUsageVerifier::verify(self.module, function_definition, cfg);
        if !errors.is_empty() {
            return errors;
        }
        let errors = AcquiresVerifier::verify(self.module, function_definition);
        if !errors.is_empty() {
            return errors;
        }
        TypeAndMemorySafetyAnalysis::verify(self.module, function_definition, cfg)
    }
}

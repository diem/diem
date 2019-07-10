// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements the checker for verifying correctness of function bodies.
//! The overall verification is split between stack_usage_verifier.rs and
//! abstract_interpreter.rs. CodeUnitVerifier simply orchestrates calls into these two files.
use crate::control_flow_graph::{ControlFlowGraph, VMControlFlowGraph};
use vm::{
    access::ModuleAccess,
    errors::{VMStaticViolation, VerificationError},
    file_format::{CompiledModule, FunctionDefinition},
    IndexKind,
};

use crate::{abstract_interpreter::AbstractInterpreter, stack_usage_verifier::StackUsageVerifier};

pub struct CodeUnitVerifier<'a> {
    module: &'a CompiledModule,
}

impl<'a> CodeUnitVerifier<'a> {
    pub fn verify(module: &'a CompiledModule) -> Vec<VerificationError> {
        let verifier = Self { module };
        verifier
            .module
            .function_defs()
            .iter()
            .enumerate()
            .map(move |(idx, function_definition)| {
                verifier
                    .verify_function(function_definition)
                    .into_iter()
                    .map(move |err| VerificationError {
                        kind: IndexKind::FunctionDefinition,
                        idx,
                        err,
                    })
            })
            .flatten()
            .collect()
    }

    fn verify_function(&self, function_definition: &FunctionDefinition) -> Vec<VMStaticViolation> {
        if function_definition.is_native() {
            return vec![];
        }
        let result: Result<VMControlFlowGraph, VMStaticViolation> =
            VMControlFlowGraph::new(&function_definition.code.code);
        match result {
            Ok(cfg) => self.verify_function_inner(function_definition, &cfg),
            Err(e) => vec![e],
        }
    }

    fn verify_function_inner(
        &self,
        function_definition: &FunctionDefinition,
        cfg: &VMControlFlowGraph,
    ) -> Vec<VMStaticViolation> {
        let errors = StackUsageVerifier::verify(self.module, function_definition, cfg);
        if !errors.is_empty() {
            return errors;
        }
        AbstractInterpreter::verify(self.module, function_definition, cfg)
    }
}

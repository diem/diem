// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    function_target::FunctionTargetData,
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder},
};
use spec_lang::{
    ast::{Condition, ConditionKind, Exp, Spec, Value},
    env::{ConditionInfo, FunctionEnv, VerificationScope, ALWAYS_ABORTS_TEST_PRAGMA},
    ty::BOOL_TYPE,
};

/// A function target processor which instruments specifications and code for the purpose
/// of specification testing.
pub struct TestInstrumenter {
    verification_scope: VerificationScope,
}

impl TestInstrumenter {
    /// Creates a new test instrumenter.
    pub fn new(verification_scope: VerificationScope) -> Box<Self> {
        Box::new(TestInstrumenter { verification_scope })
    }
}

impl FunctionTargetProcessor for TestInstrumenter {
    /// Implements the FunctionTargetProcessor trait.
    fn process(
        &self,
        _targets: &mut FunctionTargetsHolder,
        func_env: &FunctionEnv<'_>,
        mut data: FunctionTargetData,
    ) -> FunctionTargetData {
        if !func_env.should_verify(self.verification_scope) {
            // Do not instrument if function is not in verification scope.
            return data;
        }
        if func_env.is_pragma_true(ALWAYS_ABORTS_TEST_PRAGMA, || false) {
            self.instrument_always_aborts(func_env, &mut data);
        }
        data
    }
}

// ==============================================================================
// AbortsIf True Instrumentation

impl TestInstrumenter {
    /// Instrument the function to check whether it always aborts. This drops existing spec
    /// conditions and adds a single `aborts_if true` condition. This is marked as "negative",
    /// letting the boogie wrapper check whether it was not violated, this way identifying whether
    /// a function always aborts.
    fn instrument_always_aborts(&self, func_env: &FunctionEnv<'_>, data: &mut FunctionTargetData) {
        // Override specification with `aborts_if true`.
        let cond = self.make_condition(
            func_env,
            ConditionKind::AbortsIf,
            self.make_bool_exp(func_env, true),
        );
        let cond_loc = cond.loc.clone();
        let spec = Spec {
            conditions: vec![cond],
            properties: Default::default(),
            on_impl: Default::default(),
        };
        data.rewritten_spec = Some(spec);
        // Add ConditionInfo to global environment for backend error reporting.
        let info = ConditionInfo {
            message: "function always aborts".to_string(),
            message_if_requires: None,
            omit_trace: true,    // no error trace needed
            negative_cond: true, // this is a negative condition: we report above error if it passes
        };
        // Register the condition info for location we assigned to the condition. This way the
        // backend will find it.
        func_env.module_env.env.set_condition_info(cond_loc, info);
    }
}

// ==============================================================================
// Helpers

impl TestInstrumenter {
    /// Helper to create a boolean constant expression.
    fn make_bool_exp(&self, func_env: &FunctionEnv<'_>, val: bool) -> Exp {
        let node_id = func_env
            .module_env
            .new_node(func_env.get_loc(), BOOL_TYPE.clone());
        Exp::Value(node_id, Value::Bool(val))
    }

    /// Helper to create a specification condition.
    fn make_condition(
        &self,
        func_env: &FunctionEnv<'_>,
        kind: ConditionKind,
        exp: Exp,
    ) -> Condition {
        Condition {
            loc: func_env.get_loc(),
            kind,
            exp,
        }
    }
}

// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    function_target::FunctionTargetData,
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder},
    stackless_bytecode::{Bytecode, Operation, TEMP_DEFAULT_VALUE_INDEX},
};
use spec_lang::{
    ast::{
        Condition,
        ConditionKind,
        Exp,
        PropertyBag,
        Spec,
    },
    env::{
        Loc,
        ConditionInfo,
        ConditionTag,
        ModuleEnv,
        FunctionEnv,
        StructEnv,
        ModuleId,
        StructId,
        FieldEnv,
        VerificationScope,
        ALWAYS_ABORTS_TEST_PRAGMA,
        CONST_FIELD_TEST_PRAGMA,
        CONST_SC_ADDR,
        CONST_SUBEXP_TEST_PRAGMA,
        WRITEREF_TEST_PRAGMA,
    },
    ty::{
        BOOL_TYPE,
        ADDRESS_TYPE,
    },
};
use codespan::{Span, ByteOffset};

use num::BigUint;

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
        targets: &mut FunctionTargetsHolder,
        func_env: &FunctionEnv<'_>,
        mut data: FunctionTargetData,
    ) -> FunctionTargetData {
        if func_env.should_verify(self.verification_scope) {
            // Do not instrument if function is in verification scope.
            return data;
        }
        if func_env.is_pragma_true(ALWAYS_ABORTS_TEST_PRAGMA, || false) {
            self.instrument_always_aborts(func_env, &mut data);
        }
        if func_env.is_pragma_true(CONST_FIELD_TEST_PRAGMA, || false) {
            self.instrument_const_fields(func_env, &mut data, targets);
        }
        if func_env.is_pragma_true(CONST_SUBEXP_TEST_PRAGMA, || false) {
            let mut count = 0;
            self.instrument_const_precond_subexp(func_env, &mut data, &mut count);
            self.instrument_const_postcond_subexp(func_env, &mut data, &mut count);
        }
        if func_env.is_pragma_true(WRITEREF_TEST_PRAGMA, || false) {
            self.instrument_writeref(func_env, &mut data);
        }
        data
    }
}

// ==============================================================================
// Aborts If Test Instrumentation

impl TestInstrumenter {
    /// Instrument the function to check whether it always aborts. This drops existing spec
    /// conditions and adds a single `aborts_if true` condition. This is marked as "negative",
    /// letting the boogie wrapper check whether it was not violated, this way identifying whether
    /// a function always aborts.
    fn instrument_always_aborts(&self, func_env: &FunctionEnv<'_>, data: &mut FunctionTargetData) {
        let mut conds = vec![];

        // Keep the preconditions
        conds.append(&mut self.get_smoke_test_preconditions(func_env));

        // Override specification with `aborts_if true`.
        let cond = self.make_condition(
            func_env.get_loc(),
            ConditionKind::EnsuresSmokeTest,
            self.make_abort_flag_bool_exp(&func_env.module_env),
        );
        let cond_loc = cond.loc.clone();
        conds.push(cond);
        let spec = Spec::new(conds, Default::default(), Default::default());
        data.add_spec_check(spec, None);
        // Add ConditionInfo to global environment for backend error reporting.
        let info = ConditionInfo {
            message: "function always aborts".to_string(),
            omit_trace: true,    // no error trace needed
            negative_cond: true, // this is a negative condition: we report above error if it passes
        };
        // Register the condition info for location we assigned to the condition. This way the
        // backend will find it.
        func_env
            .module_env
            .env
            .set_condition_info(cond_loc, ConditionTag::NegativeTest, info);
    }
}

// ==============================================================================
// Constant global struct expression Instrumentation

impl TestInstrumenter {
    /// Instrument the function to check whether there are constant global expressions.
    fn instrument_const_fields(&self, func_env: &FunctionEnv<'_>, data: &mut FunctionTargetData, targets: &mut FunctionTargetsHolder) {
        let module_env = &func_env.module_env;
        let mut conds = vec![];

        // Keep the precondtions
        conds.append(&mut self.get_smoke_test_preconditions(func_env));

        let struct_envs = func_env
            .module_env
            .get_structs();
        for struct_env in struct_envs {
            // For every struct, add condition that struct fields don't change
            if struct_env.get_field_count() == 1
                || !self.can_mutate(data, &struct_env.get_id()) {
                // Ignore struct with no fields (except the dummy field) and functions
                // that don't potentially mutate the global resource
                continue;
            }

            // Check if the struct is moved to the top level of the global store
            // and if the current procedure modifies the struct
            if !self.is_top_level_struct(&targets, &data, &struct_env) ||
                !self.code_has_borrow_global(&data.code, &struct_env.module_env.get_id(), &struct_env.get_id()) {
                continue;
            }

            let const_addr = BigUint::from(func_env.get_num_pragma(CONST_SC_ADDR, || 0));
            // Add assumption that $sc_addr == const_addr
            // if the `const_sc_addr` pragma is declared
            let cond = self.make_condition(
                    struct_env.get_loc().clone(),
                    ConditionKind::RequiresSmokeTest,
                    self.sc_addr_is_const(module_env, const_addr)
                );
            conds.push(cond);

            // Add the unchanged field post conditions
            for field_env in struct_env.get_fields() {
                // Vector of conditions for the specific field
                let mut conds_for_field = conds.clone();

                // Create a condition and spec for the field
                // FIXME: Location is not exact. Simply added the offset to the beginning of the span
                // to create a dummy unqiue span.
                let field_span =
                    Span::new(struct_env.get_loc().span().start() + ByteOffset(field_env.get_offset() as i64), struct_env.get_loc().span().end());
                let field_loc =
                    Loc::new(struct_env.get_loc().file_id(), field_span);
                let field_unchanged_exp = self.field_unchanged_exp(func_env, &struct_env, &field_env);
                let cond = self.make_condition(
                        field_loc.clone(),
                        ConditionKind::EnsuresSmokeTest,
                        field_unchanged_exp,
                    );
                let info = ConditionInfo {
                    message: format!("{}.{} never changes value",
                        struct_env.get_name().display(struct_env.symbol_pool()),
                        field_env.get_name().display(struct_env.symbol_pool())),
                    omit_trace: true,
                    negative_cond: true,
                };
                func_env.module_env.env.set_condition_info(field_loc, ConditionTag::NegativeTest, info);
                conds_for_field.push(cond);

                // Add the specification check
                let spec = Spec::new(conds_for_field, Default::default(), Default::default());
                data.add_spec_check(spec, None);
            }
        }
    }

    /// Helper function to check if the current struct is moved to the top level of the global store
    fn is_top_level_struct(&self, targets: &FunctionTargetsHolder, data: &FunctionTargetData, struct_env: &StructEnv<'_>) -> bool {
        let func_envs = struct_env
            .module_env
            .get_functions();
        let module_id = struct_env.module_env.get_id();
        let struct_id = struct_env.get_id();
        // Check if the current function has a move_to<T>, where T is the struct described
        // by `struct_env`
        if self.code_has_move_to(&data.code, &module_id, &struct_id) {
            return true;
        }
        // Check if the other functions has a move_to<T>, where T is the struct described
        // by `struct_env`
        for func_env in func_envs {
            if !targets.has_target(&func_env) {
                continue;
            }
            if self.code_has_move_to(&targets.get_target(&func_env).data.code, &module_id, &struct_id) {
                return true;
            }
        }
        false
    }

    fn code_has_move_to(&self, code: &Vec<Bytecode>, module_id: &ModuleId, struct_id: &StructId) -> bool {
        for bytecode in code {
            if self.is_struct_move_to(&bytecode, module_id, struct_id) {
                return true;
            }
        }
        false
    }

    /// Helper function to check if the bytecode is a move_to with the given struct
    fn is_struct_move_to(&self, bytecode: &Bytecode, module_id: &ModuleId, struct_id: &StructId) -> bool {
        match bytecode {
            Bytecode::Call(_, _, Operation::MoveTo(module_id_, struct_id_, _), _) =>
                module_id == module_id_ && struct_id == struct_id_,
            _ => false,
        }
    }

    fn code_has_borrow_global(&self, code: &Vec<Bytecode>, module_id: &ModuleId, struct_id: &StructId) -> bool {
        for bytecode in code {
            if self.is_struct_borrow_global(&bytecode, module_id, struct_id) {
                return true;
            }
        }
        false
    }

    /// Helper function to check if the bytecode is a move_to with the given struct
    fn is_struct_borrow_global(&self, bytecode: &Bytecode, module_id: &ModuleId, struct_id: &StructId) -> bool {
        match bytecode {
            Bytecode::Call(_, _, Operation::BorrowGlobal(module_id_, struct_id_, _), _) =>
                module_id == module_id_ && struct_id == struct_id_,
            _ => false,
        }
    }

    /// Helper to create the expression `old(exists<T>($sc_addr)) ==> global<T>($sc_addr) == old(global<T>($sc_addr))`
    fn field_unchanged_exp(&self, func_env: &FunctionEnv<'_>, struct_env: &StructEnv<'_>, field_env: &FieldEnv<'_>) -> Exp {
        let module_env = &func_env.module_env;
        let sc_addr_var = Exp::make_localvar(module_env, "$sc_addr", ADDRESS_TYPE.clone());
        // Make the expression `global<T>($sc_addr) == old(global<T>($sc_addr))`
        let global_struct = Exp::make_call_global(module_env, struct_env.get_type(), sc_addr_var.clone());
        let global_field_value = Exp::make_call_select(module_env, struct_env.get_id(), field_env.get_id(), global_struct);
        let unchanged_field_exp = self.make_value_unchanged_exp(module_env, global_field_value);
        // Generate the expression `old(exists<T>($sc_addr))`
        let global_exists = Exp::make_call_exists(module_env, struct_env.get_type(), sc_addr_var.clone());
        let old_global_exists = Exp::make_call_old(module_env, global_exists);
        // Return the implication `old(exists<T>($sc_addr)) ==> global<T>($sc_addr) == old(global<T>($sc_addr))`
        Exp::make_call_implies(module_env, old_global_exists, unchanged_field_exp)
    }

    /// Helper to create the expression `exp == old(exp)`
    fn make_value_unchanged_exp(&self, module_env: &ModuleEnv<'_>, exp: Exp) -> Exp {
        // old(exp)
        let old_value = Exp::make_call_old(module_env, exp.clone());
        // exp == old(exp)
        Exp::make_call_eq(module_env, exp.clone(), old_value)
    }

    // Expression that says `$sc_addr` is equal to the given BigUInt constant
    fn sc_addr_is_const(&self, module_env: &ModuleEnv<'_>, const_addr: BigUint) -> Exp {
        let sc_addr_var = Exp::make_localvar(module_env, "$sc_addr", ADDRESS_TYPE.clone());
        let const_addr = Exp::make_value_address(module_env, const_addr);
        Exp::make_call_eq(module_env, sc_addr_var, const_addr)
    }
}

// ==============================================================================
// Constant expression checker

impl TestInstrumenter {
    /// Instrument the function to check for constant predicate subexpressions in
    /// the requires specifications
    fn instrument_const_precond_subexp(&self, func_env: &FunctionEnv<'_>, data: &mut FunctionTargetData, count: &mut i64) {
        let preconds = self.get_smoke_test_preconditions(func_env);
        let mut conds = vec![];
        for precond in preconds {
            self.create_const_subexp_condition(&ConditionKind::RequiresSmokeTestAssert, func_env, &precond.exp, &precond.loc, data, count, true, &conds, " in the precondition");
            conds.push(precond.clone());
        }
    }

    /// Instrument the function to check for constant predicate subexpressions in
    /// the ensures specifications
    fn instrument_const_postcond_subexp(&self, func_env: &FunctionEnv<'_>, data: &mut FunctionTargetData, count: &mut i64) {
        let preconds = self.get_smoke_test_preconditions(func_env);
        for cond in &func_env.get_spec().conditions {
            self.create_const_subexp_condition(&ConditionKind::EnsuresSmokeTest, func_env, &cond.exp, &cond.loc, data, count, true, &preconds, " in the postcondition");
        }
    }

    fn create_const_subexp_condition(
        &self,
        kind: &ConditionKind,
        func_env: &FunctionEnv<'_>,
        exp: &Exp,
        loc: &Loc,
        data: &mut FunctionTargetData,
        count: &mut i64,
        first_level: bool,
        preconds: &Vec<Condition>,
        msg_suffix: &str) {
        let module_env = &func_env.module_env;
        if func_env.module_env.get_node_type(exp.node_id()) == BOOL_TYPE && (!first_level || *kind == ConditionKind::RequiresSmokeTestAssert) {
            let exp_is_true = exp.clone();
            let exp_is_false = Exp::make_call_not(&module_env, exp.clone());
            let exps = vec![exp_is_true, exp_is_false];

            // Create a specification check for each boolean polarity true and false
            for exp in exps {
                let mut conds = preconds.clone();

                *count = *count + 1;
                let span = Span::new(loc.span().start() + ByteOffset(*count), loc.span().end());
                let loc = Loc::new(func_env.get_loc().file_id(), span);

                let cond = self.make_condition(
                    loc.clone(),
                    kind.clone(),
                    exp.clone(),
                );
                conds.push(cond);
                let info = ConditionInfo {
                    message: format!("subexpression in {} is always constant: {:?} {}",
                        func_env.get_name().display(func_env.symbol_pool()),
                        &exp,
                        msg_suffix),
                    omit_trace: true,
                    negative_cond: true,
                };

                func_env.module_env.env.set_condition_info(loc, ConditionTag::NegativeTest, info);
                data.add_spec_check(Spec::new(conds, Default::default(), Default::default()), None);
            }
        }
        use Exp::*;
        match exp {
            Call(_, _, exps) => {
                for exp in exps {
                    self.create_const_subexp_condition(kind, func_env, &exp, loc, data, count, false, preconds, msg_suffix);
                }
            }
            Block(_, _, exp) => {
                self.create_const_subexp_condition(kind, func_env, &exp, loc, data, count, false, preconds, msg_suffix);
            }
            IfElse(_, cond, then_, else_) => {
                self.create_const_subexp_condition(kind, func_env, &cond, loc, data, count, false, preconds, msg_suffix);
                self.create_const_subexp_condition(kind, func_env, &then_, loc, data, count, false, preconds, msg_suffix);
                self.create_const_subexp_condition(kind, func_env, &else_, loc, data, count, false, preconds, msg_suffix);
            }
            _ => ()
        }
    }
}

// ==============================================================================
// WriteRef mutation testing

impl TestInstrumenter {

    /// Injects a $DefaultValue in place of the source value for each WriteRef
    /// in the function code.
    fn instrument_writeref(&self, func_env: &FunctionEnv<'_>, data: &mut FunctionTargetData) {
        let module_env = &func_env.module_env;
        let preconds = self.get_smoke_test_preconditions(func_env);
        let postconds = self.get_smoke_test_postconditions(func_env);
        let mut count = 0;
        for (i, bytecode) in data.code.clone().iter().enumerate() {
            match bytecode {
                Bytecode::Call(id, dsts, Operation::WriteRef, srcs) => {
                    let mut conds = preconds.clone();

                    let mut rewritten_bytecode = data.code.clone();
                    let write_ref = &mut rewritten_bytecode[i];
                    let mutated_srcs = vec![srcs[0], TEMP_DEFAULT_VALUE_INDEX];
                    *write_ref = Bytecode::Call(id.clone(), dsts.clone(), Operation::WriteRef, mutated_srcs);

                    // Conjunction of all postconditions
                    let exp = postconds
                        .iter()
                        .fold(Exp::make_value_bool(&module_env, true), |acc, cond| {
                            Exp::make_call_and(&module_env, acc, cond.exp.clone())
                        });
                    let not_abort_flag = Exp::make_call_not(&module_env, self.make_abort_flag_bool_exp(&module_env));
                    let exp = Exp::make_call_implies(&module_env, not_abort_flag, exp);

                    count = count + 1;
                    let span = Span::new(func_env.get_loc().span().start() + ByteOffset(count), func_env.get_loc().span().end());
                    let loc = Loc::new(func_env.get_loc().file_id(), span);

                    let cond = self.make_condition(
                        loc.clone(),
                        ConditionKind::EnsuresSmokeTest,
                        exp);
                    conds.push(cond);

                    let info = ConditionInfo {
                        message: "missing writeref specification".to_string(),
                        omit_trace: true,
                        negative_cond: true,
                    };
                    func_env.module_env.env.set_condition_info(loc.clone(), ConditionTag::NegativeTest, info);
                    data.add_spec_check(Spec::new(conds, Default::default(), Default::default()), Some(rewritten_bytecode));
                },
                _ => (),
            }
        }
    }
}

// ==============================================================================
// Helpers

impl TestInstrumenter {
    /// Helper to return the preconditions of the `func_env` as a list of
    /// `RequiresSmokeTest` conditions.
    /// These conditions are assumed at the top level `_smoke_test_` function only.
    fn get_smoke_test_preconditions(&self, func_env: &FunctionEnv<'_>) -> Vec<Condition> {
        let mut conds = vec![];
        for cond in &func_env.get_spec().conditions {
            match cond.kind {
                ConditionKind::Requires | ConditionKind::RequiresModule => {
                    let st_requires = Condition {
                        loc: cond.loc.clone(),
                        kind: ConditionKind::RequiresSmokeTest,
                        properties: Default::default(),
                        exp: cond.exp.clone(),
                    };
                    conds.push(st_requires);
                }
                _ => ()
            }
        }
        conds
    }

    /// Helper to return the postconditions of the `func_env` as a list of
    /// `Ensures` conditions.
    /// These conditions are asserted at the top level `_smoke_test_` function.
    fn get_smoke_test_postconditions(&self, func_env: &FunctionEnv<'_>) -> Vec<Condition> {
        let mut conds = vec![];
        for cond in &func_env.get_spec().conditions {
            match cond.kind {
                ConditionKind::Ensures => {
                    let st_ensures = Condition {
                        loc: cond.loc.clone(),
                        kind: ConditionKind::Ensures,
                        properties: Default::default(),
                        exp: cond.exp.clone(),
                    };
                    conds.push(st_ensures);
                }
                _ => ()
            }
        }
        conds
    }

    /// Helper to create a specification condition.
    fn make_condition(
        &self,
        loc: Loc,
        kind: ConditionKind,
        exp: Exp,
    ) -> Condition {
        Condition {
            loc,
            kind,
            properties: PropertyBag::default(),
            exp,
        }
    }

    /// Helper to create the $abort_flag variable.
    fn make_abort_flag_bool_exp(&self, module_env: &ModuleEnv<'_>) -> Exp {
        Exp::make_localvar(module_env, "$Boolean($abort_flag)", BOOL_TYPE.clone())
    }

    /// Helper function to statically check if a struct is potentially modified
    fn can_mutate(&self, data: &FunctionTargetData, struct_id: &StructId) -> bool {
        data.code
            .iter()
            .find(|bc| match bc {
                Bytecode::Call(_, _, op, _) => {
                    use Operation::*;
                    match op {
                        Pack(_, struct_id_, _) => struct_id == struct_id_,
                        Unpack(_, struct_id_, _) => struct_id == struct_id_,
                        BorrowGlobal(_, struct_id_, _) => struct_id == struct_id_,
                        _ => false
                    }
                },
                _ => false
            })
            .is_some()
    }
}

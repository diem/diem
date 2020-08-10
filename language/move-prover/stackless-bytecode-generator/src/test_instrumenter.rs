// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    function_target::FunctionTargetData,
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder},
    stackless_bytecode::{TempIndex, AttrId, Bytecode, AssignKind, Operation, Label, BorrowNode},
};

use std::collections::BTreeMap;

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
        MUTATED_ARGS_TEST_PRAGMA,
    },
    symbol::Symbol,
    ty::{
        Type,
        BOOL_TYPE,
        ADDRESS_TYPE,
    },
};

use codespan::{Span, ByteOffset};

use num::BigUint;

/// Specification check with an id, the specification to check,
/// and the mutated bytecode to check.
#[derive(Debug)]
pub struct SpecCheck {
    /// Id of the current spec check
    pub id: usize,
    /// The specification to check
    pub spec: Spec,
    /// The rewritten code for specification checks.
    /// If this is none, then the original code from the function should be used.
    pub code: Option<Vec<Bytecode>>,
    /// Location annotation
    pub loc: Loc,
}

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
            self.instrument_const_precond_subexp(func_env, &mut data);
            self.instrument_const_postcond_subexp(func_env, &mut data);
        }
        if func_env.is_pragma_true(WRITEREF_TEST_PRAGMA, || false) {
            self.instrument_writeref(func_env, &mut data);
        }
        if func_env.is_pragma_true(MUTATED_ARGS_TEST_PRAGMA, || false) {
            self.instrument_mutable_args(func_env, &mut data);
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
            ConditionKind::Ensures,
            self.make_abort_flag_bool_exp(&func_env.module_env),
        );
        let cond_loc = cond.loc.clone();
        conds.push(cond);
        let spec = Spec::new(conds, Default::default(), Default::default());
        data.add_spec_check(spec, None, cond_loc.clone());

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

        // For every struct, add the condition that struct fields don't change
        let struct_envs = func_env
            .module_env
            .get_structs();
        for struct_env in struct_envs {
            if struct_env.get_field_count() == 1
                || !self.can_mutate(data, &struct_env.get_id()) {
                // Ignore struct with no fields (except the dummy field, hence == 1)
                // and functions that don't potentially mutate the global resource.
                continue;
            }

            // Check if the struct is moved to the top level of the global store
            // and if the current procedure modifies the struct.
            if !self.is_top_level_struct(&targets, &data, &struct_env) ||
                !self.code_has_borrow_global(&data.code, &struct_env.module_env.get_id(), &struct_env.get_id()) {
                continue;
            }

            let const_addr = BigUint::from(func_env.get_num_pragma(CONST_SC_ADDR, || 0));
            // Add assumption that $sc_addr == const_addr
            // if the `const_sc_addr` pragma is declared
            let cond = self.make_condition(
                    struct_env.get_loc().clone(),
                    ConditionKind::RequiresSpecCheck,
                    self.sc_addr_is_const(module_env, const_addr)
                );
            conds.push(cond);

            // Add the unchanged field post conditions.
            for field_env in struct_env.get_fields() {
                // Vector of conditions for the specific field.
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
                        ConditionKind::Ensures,
                        field_unchanged_exp,
                    );
                conds_for_field.push(cond);

                let info = ConditionInfo {
                    message: format!("{}.{} never changes value",
                        struct_env.get_name().display(struct_env.symbol_pool()),
                        field_env.get_name().display(struct_env.symbol_pool())),
                    omit_trace: true,
                    negative_cond: true,
                };
                func_env.module_env.env.set_condition_info(field_loc.clone(), ConditionTag::NegativeTest, info);


                // Add the specification check
                let spec = Spec::new(conds_for_field, Default::default(), Default::default());
                data.add_spec_check(spec, None, field_loc.clone());
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
    /// Instrument the function to check for constant
    /// predicate subexpressions in the requires specifications
    fn instrument_const_precond_subexp(&self, func_env: &FunctionEnv<'_>, data: &mut FunctionTargetData) {
        let preconds = self.get_smoke_test_preconditions(func_env);
        let mut conds = vec![];
        for precond in preconds {
            self.create_const_subexp_condition(&ConditionKind::RequiresSpecCheckAssert, func_env, &precond.exp, &precond.loc, data, true, &conds);
            conds.push(precond.clone());
        }
    }

    /// Instrument the function to check for constant
    /// predicate subexpressions in the ensures specifications
    fn instrument_const_postcond_subexp(&self, func_env: &FunctionEnv<'_>, data: &mut FunctionTargetData) {
        let preconds = self.get_smoke_test_preconditions(func_env);
        for cond in &func_env.get_spec().conditions {
            self.create_const_subexp_condition(&ConditionKind::Ensures, func_env, &cond.exp, &cond.loc, data, true, &preconds);
        }
    }

    /// Intruments the function with the specification check that finds
    /// constant predicate (only boolean valued) subexpressions.
    /// Creates a new specification check, one for each polarity.
    /// For example, in p ==> q, we add the checks p, !p, q, !q, and
    /// recursively add checks for any subexpressions of p and q.
    fn create_const_subexp_condition(
        &self,
        kind: &ConditionKind,
        func_env: &FunctionEnv<'_>,
        exp: &Exp,
        loc: &Loc,
        data: &mut FunctionTargetData,
        first_level: bool,
        preconds: &Vec<Condition>) {
        let module_env = &func_env.module_env;
        if func_env.module_env.get_node_type(exp.node_id()) == BOOL_TYPE && (!first_level || *kind == ConditionKind::RequiresSpecCheckAssert) {
            let exp_is_true = exp.clone();
            let exp_is_false = Exp::make_call_not(&module_env, exp.clone());
            let exps = vec![exp_is_true, exp_is_false];

            // Create a specification check for each boolean polarity true and false
            for exp in exps {
                let mut conds = preconds.clone();
                let error_annotation = format!("{}_{}",
                            func_env.get_name().display(func_env.symbol_pool()),
                            exp.node_id().as_usize(),
                        );
                let annotation_loc =
                    Loc::annotated(
                        func_env.get_loc().file_id(),
                        loc.span(),
                        error_annotation,
                    );

                let cond = self.make_condition(
                    loc.clone(),
                    kind.clone(),
                    exp.clone(),
                );
                conds.push(cond);
                let info = ConditionInfo {
                    message: format!("subexpression is constant: {:?}.", &exp),
                    omit_trace: true,
                    negative_cond: true,
                };

                func_env.module_env.env.set_condition_info(annotation_loc.clone(), ConditionTag::NegativeTest, info);
                let spec = Spec::new(conds, Default::default(), Default::default());
                data.add_spec_check(spec, None, annotation_loc.clone());
            }
        }
        use Exp::*;
        match exp {
            Call(_, _, exps) => {
                for exp in exps {
                    self.create_const_subexp_condition(kind, func_env, &exp, loc, data, false, preconds);
                }
            }
            Block(_, _, exp) => {
                self.create_const_subexp_condition(kind, func_env, &exp, loc, data, false, preconds);
            }
            IfElse(_, cond, then_, else_) => {
                self.create_const_subexp_condition(kind, func_env, &cond, loc, data, false, preconds);
                self.create_const_subexp_condition(kind, func_env, &then_, loc, data, false, preconds);
                self.create_const_subexp_condition(kind, func_env, &else_, loc, data, false, preconds);
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

        // Create dummy unconstrained value
        data.local_types.push(Type::Tuple(vec![]));
        let uncon_index = data.local_types.len()-1;

        // Stores the string representation of temporary variable paths.
        let mut path_strings: BTreeMap<usize, String> = BTreeMap::new();
        // Stores whether the temporary is an alias to a global or local argument.
        // If it is not in the map then it is neither.
        let mut ref_type: BTreeMap<usize, BorrowNode> = BTreeMap::new();

        // Initialize path strings and ref types for arguments
        for (index, proxy_index) in &data.ref_param_proxy_map {
            path_strings.insert(*proxy_index, format!("{}", func_env.get_local_name(*index).display(func_env.symbol_pool())));
            ref_type.insert(*proxy_index, BorrowNode::LocalRoot(*index));
        }

        // Update the path strings and ref types by traversing the bytecode
        for bytecode in &data.code {
            match bytecode {
                Bytecode::Assign(_, dst, src, _) => {
                    // update the path
                    if path_strings.contains_key(src) {
                        path_strings.insert(*dst, path_strings[src].clone());
                    }
                    // update the reference type
                    if ref_type.contains_key(src) {
                        ref_type.insert(*dst, ref_type[src].clone());
                    }
                }
                Bytecode::Call(_, dsts, op, srcs) => {
                    match op {
                        Operation::BorrowField(mid, sid, _, field_index) => {
                            // update the path
                            let mod_env = module_env
                                .env
                                .get_module(*mid);
                            let struct_env = mod_env
                                .get_struct(*sid);
                            let field_name = struct_env
                                .get_field_by_offset(*field_index)
                                .get_name();
                            if path_strings.contains_key(&srcs[0]) {
                                path_strings.insert(dsts[0], format!("{}_{}", path_strings[&srcs[0]].clone(), field_name.display(struct_env.symbol_pool())));
                            }
                            // update the reference type
                            if ref_type.contains_key(&srcs[0]) {
                                ref_type.insert(dsts[0], ref_type[&srcs[0]].clone());
                            }
                        }
                        Operation::BorrowLoc => {
                            // update the path
                            if path_strings.contains_key(&srcs[0]) {
                                path_strings.insert(dsts[0], path_strings[&srcs[0]].clone());
                            }
                            // update the reference type
                            if ref_type.contains_key(&srcs[0]) {
                                ref_type.insert(dsts[0], ref_type[&srcs[0]].clone());
                            }
                        }
                        _ => (),
                    }
                }
                _ => (),
            }
        }

        // Generate specification checks for each WriteRef
        for (i, bytecode) in data.code.clone().iter().enumerate() {
            match bytecode {
                Bytecode::Call(id, dsts, Operation::WriteRef, srcs) => {
                    // Only mutate WriteRef of &mut arguments
                    // TODO: and globals
                    if ref_type.get(&srcs[0]).is_none() {
                        continue;
                    }

                    let mut conds = preconds.clone();

                    let mut rw_code = data.code.clone();
                    let write_ref = &mut rw_code[i];
                    let mutated_srcs = vec![srcs[0], uncon_index];
                    *write_ref = Bytecode::Call(id.clone(), dsts.clone(), Operation::WriteRef, mutated_srcs);

                    // Conjunction of all postconditions
                    let exp = postconds
                        .iter()
                        .fold(Exp::make_value_bool(&module_env, true), |acc, cond| {
                            Exp::make_call_and(&module_env, acc, cond.exp.clone())
                        });
                    let not_abort_flag = Exp::make_call_not(&module_env, self.make_abort_flag_bool_exp(&module_env));
                    let exp = Exp::make_call_implies(&module_env, not_abort_flag, exp);

                    let path_str = match path_strings.get(&srcs[0]) {
                        Some(s) => s,
                        _ => panic!("Expected a path string for a local or global."),
                    };

                    let error_annotation = format!("{}_{}",
                                func_env.get_name().display(func_env.symbol_pool()),
                                path_str,
                            );
                    let annotation_loc =
                        Loc::annotated(
                            func_env.get_loc().file_id(),
                            func_env.get_loc().span(),
                            error_annotation,
                        );

                    let cond = self.make_condition(
                        annotation_loc.clone(),
                        ConditionKind::Ensures,
                        exp);
                    conds.push(cond);

                    let info = ConditionInfo {
                        message: format!("missing specification for {}", path_str),
                        omit_trace: true,
                        negative_cond: true,
                    };
                    func_env.module_env.env.set_condition_info(annotation_loc.clone(), ConditionTag::NegativeTest, info);
                    data.add_spec_check(Spec::new(conds, Default::default(), Default::default()), Some(rw_code), annotation_loc.clone());
                },
                _ => (),
            }
        }
    }

    /// Instrument function arguments with specification checks to check if every field of
    /// the specification has been specified.
    fn instrument_mutable_args(&self, func_env: &FunctionEnv<'_>, data: &mut FunctionTargetData) -> Option<()>{
        let module_env = &func_env.module_env;
        let module_name = module_env
            .get_name()
            .display(module_env.symbol_pool());

        // Get pre and post conditions
        let preconds = self.get_smoke_test_preconditions(func_env);
        let postconds = self.get_smoke_test_postconditions(func_env);

        // Create dummy reference temporary
        // NOTE: Needs to use tuple or else value is not updated; why?
        data.local_types.push(Type::Reference(true, Box::new(Type::Tuple(vec![]))));
        let tmp_index = data.local_types.len()-1;

        // Create dummy unconstrained value
        data.local_types.push(Type::Tuple(vec![]));
        let uncon_index = data.local_types.len()-1;

        // Create temporary variables to store the return values
        // at every program point where there was a return instruction
        // and return them at the return block.
        let mut ret_tmp_indices = vec![];
        for ty in func_env.get_return_types() {
            if ty.is_mutable_reference() {
                data.local_types.push(ty.clone());
            } else {
                data.local_types.push(BOOL_TYPE.clone());
            }
            ret_tmp_indices.push(data.local_types.len()-1);
        }
        for _ in 0..func_env.get_mut_ref_count() {
            // Mutable references will always be values in the procedure returns.
            data.local_types.push(BOOL_TYPE.clone());
            ret_tmp_indices.push(data.local_types.len()-1);
        }

        // Code to transform to have a single return location
        let mut single_ret_code = data.code.clone();

        // Insert the single return label at the end of the code
        let ret_label_index = self.get_next_label_index(&single_ret_code);
        let return_label = Bytecode::Label(
            AttrId::new(0),
            Label::new(ret_label_index));
        single_ret_code.push(return_label);

        // Replace return statements with assignments of the returned values
        // to the temporary return values and jump to the return block.
        // The code should now have a single program point for returns.
        single_ret_code = single_ret_code
            .into_iter()
            .fold(vec![], |mut acc, bytecode| match bytecode {
                Bytecode::Ret(_, srcs) => {
                    assert!(srcs.len() == ret_tmp_indices.len(), format!("different return arg lengths: {}, {}", srcs.len(), ret_tmp_indices.len()));
                    for i in 0..srcs.len() {
                        acc.push(Bytecode::Assign(AttrId::new(0), ret_tmp_indices[i], srcs[i], AssignKind::Move));
                    }
                    acc.push(Bytecode::Jump(AttrId::new(0), Label::new(ret_label_index)));
                    acc
                },
                _ => {
                    acc.push(bytecode);
                    acc
                },
            });

        // For each procedure return value, assign an unconstrained value and generate
        for (i, typ) in func_env.get_proc_return_types().iter().enumerate() {
            // Get the type of the mutable reference or return value
            let inner_ref_typ = match typ {
                Type::Reference(_, inner_typ) => {
                    *inner_typ.clone()
                },
                _ => typ.clone(),
            };

            match inner_ref_typ {
                Type::TypeParameter(_) |
                Type::Primitive(_) |
                Type::Vector(_) => {
                    let mut rw_code = single_ret_code.clone();

                    // tmp_index <- BorrowLoc(return_i)
                    self.add_borrow(&mut rw_code, tmp_index, ret_tmp_indices[i], &typ);
                    // tmp_index <- WriteRef(tmp_index, uncon_index)
                    rw_code.push(self.make_writeref(tmp_index, uncon_index));
                    // return_i <- WritebackToValue(tmp_index)
                    self.add_writeback(&mut rw_code, ret_tmp_indices[i], tmp_index, &typ);

                    // return
                    rw_code.push(Bytecode::Ret(AttrId::new(0), ret_tmp_indices.clone()));

                    let error_annotation = format!("{} {} $ret_{}",
                        module_name,
                        func_env.get_name().display(func_env.symbol_pool()),
                        i,
                    );

                    self.add_underspec_spec_check(func_env, error_annotation, &preconds, &postconds, rw_code, data);
                },
                Type::Struct(mid, sid, _) => {
                    let mod_env = module_env.env.get_module(mid);
                    let struct_env = mod_env.get_struct(sid);
                    for field_index in 0..struct_env.get_field_count() {
                        let field_symbol = self.get_field_symbol(module_env, mid, sid, field_index);
                        let mut rw_code = single_ret_code.clone();

                        // tmp_index <- BorrowLoc(return_i)
                        self.add_borrow(&mut rw_code, tmp_index, ret_tmp_indices[i], typ);
                        // tmp_index <- BorrowField(tmp_index, field)
                        rw_code.push(self.make_borrow_field(tmp_index, tmp_index, mod_env.get_id(), struct_env.get_id(), field_index));
                        // tmp_index <- WriteRef(tmp_index, uncon_index)
                        rw_code.push(self.make_writeref(tmp_index, uncon_index));
                        // return_i <- WritebackToValue(tmp_index)
                        self.add_writeback(&mut rw_code, ret_tmp_indices[i], tmp_index, typ);

                        // return
                        rw_code.push(Bytecode::Ret(AttrId::new(0), ret_tmp_indices.clone()));

                        let error_annotation = format!("{} {} $ret_{} {}",
                            module_name,
                            func_env.get_name().display(func_env.symbol_pool()),
                            i,
                            field_symbol.display(struct_env.symbol_pool()),
                        );

                        self.add_underspec_spec_check(func_env, error_annotation, &preconds, &postconds, rw_code, data);
                    }
                },
                _ => panic!("unimplemented underspecified postcondition check for {:?}", typ),
            }
        }
        None
    }

    /// Helper function that pushes an Assign or CopyOrMoveValue to the code
    /// given depending on the type.
    /// NOTE: Mutable references for generics are returned as references.
    fn add_borrow(&self, code: &mut Vec<Bytecode>, dst: TempIndex, src: TempIndex, typ: &Type) {
        let inner_ref_typ = match typ {
            Type::Reference(true, ty) => Some(*ty.clone()),
            _ => None,
        };
        match inner_ref_typ {
            Some(Type::TypeParameter(_)) => code.push(Bytecode::Assign(AttrId::new(0), dst, src, AssignKind::Move)),
            _ => code.push(self.make_borrow_loc(dst, src)),
        }
    }

    /// Helper function that pushes a WritebackToValue or WritebackToReference to
    /// the code given depending on the type.
    /// NOTE: Mutable references for generics are returned as references.
    fn add_writeback(&self, code :&mut Vec<Bytecode>, dst: TempIndex, src: TempIndex, typ: &Type) {
        let inner_ref_typ = match typ {
            Type::Reference(true, ty) => Some(*ty.clone()),
            _ => None,
        };
        match inner_ref_typ {
            Some(Type::TypeParameter(_))  => code.push(self.make_writeback_to_ref(dst, src)),
            _ => code.push(self.make_writeback_to_value(dst, src)),
        }
    }

    /// Helper function to add the underspecified return value specification check.
    fn add_underspec_spec_check(
        &self,
        func_env: &FunctionEnv<'_>,
        error_annotation: String,
        preconds: &Vec<Condition>,
        postconds: &Vec<Condition>,
        rw_code: Vec<Bytecode>,
        data: &mut FunctionTargetData,
        )
    {
        let module_env = &func_env.module_env;

        // Create a new annotated location for specification
        // check error reporting
        let annotation_loc = Loc::annotated(
            func_env.get_loc().file_id(),
            func_env.get_loc().span(),
            error_annotation.clone(),
        );

        // Set the condition info for error reporting
        let info = ConditionInfo {
            message: format!("missing specification for {} in {}",
                error_annotation.clone(),
                func_env.get_name().display(func_env.symbol_pool())),
            omit_trace: true,
            negative_cond: true,
        };
        func_env.module_env.env.set_condition_info(annotation_loc.clone(), ConditionTag::NegativeTest, info);

        // Conjunction of all postconditions
        let exp = postconds
            .iter()
            .fold(Exp::make_value_bool(&module_env, true), |acc, cond| {
                Exp::make_call_and(&module_env, acc, cond.exp.clone())
            });
        let not_abort_flag = Exp::make_call_not(&module_env, self.make_abort_flag_bool_exp(&module_env));
        let exp = Exp::make_call_implies(&module_env, not_abort_flag, exp);

        // Create the specification check conditions
        let post_cond = self.make_condition(
            annotation_loc.clone(),
            ConditionKind::Ensures,
            exp);
        let mut conds = preconds.clone();
        conds.push(post_cond);

        data.add_spec_check(
            Spec::new(conds, Default::default(), Default::default()),
            Some(rw_code),
            annotation_loc.clone());
    }

    /// Helper function for `BorrowLoc` bytecode instruction.
    fn make_borrow_loc(
        &self,
        dst: TempIndex,
        src: TempIndex) -> Bytecode
    {
        Bytecode::Call(
            AttrId::new(0),
            vec![dst],
            Operation::BorrowLoc,
            vec![src],
        )
    }

    /// Helper function for `BorrowField` bytecode instruction.
    fn make_borrow_field(
        &self,
        dst: TempIndex,
        src: TempIndex,
        mid: ModuleId,
        sid: StructId,
        field: usize) -> Bytecode
    {
        Bytecode::Call(
            AttrId::new(0),
            vec![dst],
            Operation::BorrowField(mid, sid, vec![], field),
            vec![src]
        )
    }

    /// Helper function for `WriteRef` bytecode instruction.
    fn make_writeref(
        &self,
        dst: TempIndex,
        src: TempIndex) -> Bytecode
    {
        Bytecode::Call(
            AttrId::new(0),
            vec![],
            Operation::WriteRef,
            vec![dst, src],
        )
    }

    /// Helper function for `WritebackToValue` bytecode instruction.
    fn make_writeback_to_value(
        &self,
        dst: TempIndex,
        src: TempIndex) -> Bytecode
    {
        Bytecode::WriteBack(
            AttrId::new(0),
            BorrowNode::LocalRoot(dst),
            src,
        )
    }

    /// Helper function for `WritebackToReference` bytecode instruction.
    fn make_writeback_to_ref(
        &self,
        dst: TempIndex,
        src: TempIndex) -> Bytecode
    {
        Bytecode::WriteBack(
            AttrId::new(0),
            BorrowNode::Reference(dst),
            src,
        )
    }

    /// Returns the `findex`-th field of the struct (`mid`, `sid`) as a symbol
    /// whose symbol name is "$ModuleName_StructName_FieldName".
    /// This is tied to the current boogie translation.
    fn get_field_symbol(&self, module_env: &ModuleEnv<'_>, mid: ModuleId, sid: StructId, findex: usize) -> Symbol {
        // Get module name
        let module_env = module_env
            .env
            .get_module(mid);
        let mod_name = module_env.get_name().display(module_env.symbol_pool()).to_string();

        // Get struct name
        let struct_env = module_env
            .get_struct(sid);
        let struct_name = struct_env.get_name().display(module_env.symbol_pool()).to_string();

        // Get field name
        let field_env = struct_env
            .get_field_by_offset(findex);
        let field_name = field_env.get_name().display(module_env.symbol_pool()).to_string();

        module_env
            .symbol_pool()
            .make(&format!("${}_{}_{}", mod_name, struct_name, field_name)[..])
    }
}


// ==============================================================================
// Helpers

impl TestInstrumenter {
    /// Helper to return the preconditions of the `func_env` as a list of
    /// `RequiresSpecCheck` conditions.
    /// These conditions are assumed at the top level `_smoke_test_` function only.
    fn get_smoke_test_preconditions(&self, func_env: &FunctionEnv<'_>) -> Vec<Condition> {
        let mut conds = vec![];
        for cond in &func_env.get_spec().conditions {
            match cond.kind {
                ConditionKind::Requires | ConditionKind::RequiresModule => {
                    let st_requires = Condition {
                        loc: cond.loc.clone(),
                        kind: ConditionKind::RequiresSpecCheck,
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

    /// Helper to find the next largest label index
    fn get_next_label_index(&self, code: &Vec<Bytecode>) -> usize {
        code.iter()
            .fold(0, |acc, bytecode| match bytecode {
                Bytecode::Label(_, l) => {
                    let l_usize = l.as_usize();
                    if acc > l_usize {
                        acc
                    } else {
                        l_usize + 1
                    }
                }
                _ => acc,
            })
    }
}

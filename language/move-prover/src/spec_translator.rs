// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module translates specification conditions to Boogie code.

use std::{cell::RefCell, collections::BTreeMap};

use spec_lang::{
    env::{FieldId, FunctionEnv, Loc, ModuleEnv, ModuleId, NodeId, SpecFunId, StructEnv, StructId},
    ty::{PrimitiveType, Type},
};

use crate::{
    boogie_helpers::{
        boogie_declare_global, boogie_field_name, boogie_local_type, boogie_spec_fun_name,
        boogie_spec_var_name, boogie_struct_name, boogie_type_value, boogie_well_formed_expr,
        WellFormedMode,
    },
    code_writer::CodeWriter,
};
use itertools::Itertools;
use spec_lang::{
    ast::{Exp, Invariant, LocalVarDecl, Operation, SpecConditionKind, Value},
    env::GlobalEnv,
    symbol::Symbol,
};
use vm::file_format::CodeOffset;

pub struct SpecTranslator<'env> {
    /// The module in which context translation happens.
    module_env: &'env ModuleEnv<'env>,
    /// The code writer.
    writer: &'env CodeWriter,
    /// Whether the translation context supports native `old`,
    supports_native_old: bool,
    /// Whether we are currently in the context of translating an `old(...)` expression.
    in_old: RefCell<bool>,
    /// The current target for invariant fields, a pair of strings for current and old value.
    invariant_target: RefCell<(String, String)>,
    /// Counter for generating fresh variables.  It's a RefCell so we can increment it without
    /// making a ton of &self arguments mutable.
    fresh_var_count: RefCell<u64>,
    /// Local variable name to local index in bytecode
    name_to_idx_map: BTreeMap<Symbol, usize>,
}

// General
// =======

impl<'env> SpecTranslator<'env> {
    /// Creates a translator.
    pub fn new(
        writer: &'env CodeWriter,
        module_env: &'env ModuleEnv<'env>,
        supports_native_old: bool,
    ) -> SpecTranslator<'env> {
        SpecTranslator {
            module_env,
            writer,
            supports_native_old,
            in_old: RefCell::new(false),
            invariant_target: RefCell::new(("".to_string(), "".to_string())),
            fresh_var_count: RefCell::new(0),
            name_to_idx_map: BTreeMap::new(),
        }
    }

    /// Creates a translator for use in translating spec blocks inside function implementation
    pub fn new_for_spec_in_impl(
        writer: &'env CodeWriter,
        func_env: &'env FunctionEnv<'env>,
        supports_native_old: bool,
    ) -> SpecTranslator<'env> {
        SpecTranslator {
            module_env: &func_env.module_env,
            writer,
            supports_native_old,
            in_old: RefCell::new(false),
            invariant_target: RefCell::new(("".to_string(), "".to_string())),
            fresh_var_count: RefCell::new(0),
            name_to_idx_map: (0..func_env.get_local_count())
                .map(|idx| (func_env.get_local_name(idx), idx))
                .collect(),
        }
    }

    pub fn error(&self, loc: &Loc, msg: &str) {
        self.module_env
            .env
            .error(loc, &format!("[boogie translator] {}", msg));
    }

    /// Sets the location of the code writer from node id.
    fn set_writer_location(&self, node_id: NodeId) {
        self.writer
            .set_location(&self.module_env.get_node_loc(node_id));
    }

    /// Sets the current invariant target.
    fn with_invariant_target<F>(&self, target: &str, old_target: &str, f: F)
    where
        F: Fn(),
    {
        *self.invariant_target.borrow_mut() = (target.to_string(), old_target.to_string());
        f();
        *self.invariant_target.borrow_mut() = ("".to_string(), "".to_string());
    }

    /// Gets the current invariant target.
    fn get_invariant_target(&self) -> String {
        let (current, old) = self.invariant_target.borrow().clone();
        let res = if *self.in_old.borrow() { old } else { current };
        assert!(!res.is_empty());
        res
    }

    /// Generate a fresh variable name
    fn fresh_var_name(&self, prefix: &str) -> String {
        let mut fvc_ref = self.fresh_var_count.borrow_mut();
        let name_str = format!("${}_{}", prefix, *fvc_ref);
        *fvc_ref += 1;
        name_str
    }

    /// Translates a sequence of items separated by `sep`.
    fn translate_seq<T, F>(&self, items: impl Iterator<Item = T>, sep: &str, f: F)
    where
        F: Fn(T),
    {
        let mut first = true;
        for item in items {
            if first {
                first = false;
            } else {
                emit!(self.writer, sep);
            }
            f(item);
        }
    }
}

// Specification Variables
// =======================

impl<'env> SpecTranslator<'env> {
    pub fn translate_spec_vars(&self) {
        emitln!(
            self.writer,
            "\n\n// ** spec vars of module {}\n",
            self.module_env
                .get_name()
                .display(self.module_env.symbol_pool())
        );
        for (_, var) in self.module_env.get_spec_vars() {
            let boogie_name = boogie_spec_var_name(&self.module_env, var.name);
            emitln!(
                self.writer,
                &boogie_declare_global(&self.module_env.env, &boogie_name, &var.type_)
            );
        }
    }
}

// Specification Functions
// =======================

impl<'env> SpecTranslator<'env> {
    pub fn translate_spec_funs(&self) {
        emitln!(
            self.writer,
            "\n\n// ** spec funs of module {}\n",
            self.module_env
                .get_name()
                .display(self.module_env.symbol_pool())
        );
        for (id, fun) in self.module_env.get_spec_funs() {
            if fun.body.is_none() {
                // This function is native and expected to be found in the prelude.
                continue;
            }
            if let Type::Tuple(..) | Type::Fun(..) = fun.result_type {
                self.error(&fun.loc, "function or tuple result type not yet supported");
                continue;
            }
            let result_type = boogie_local_type(&fun.result_type);
            let type_params = fun
                .type_params
                .iter()
                .enumerate()
                .map(|(i, _)| format!("$tv{}: TypeValue", i));
            let params = fun.params.iter().map(|(name, ty)| {
                format!(
                    "{}: {}",
                    name.display(self.module_env.symbol_pool()),
                    boogie_local_type(ty)
                )
            });
            self.writer.set_location(&fun.loc);
            emitln!(
                self.writer,
                "function {{:inline}} {}({}): {} {{",
                boogie_spec_fun_name(&self.module_env, *id),
                type_params.chain(params).join(", "),
                result_type
            );
            self.writer.indent();
            self.translate_exp(fun.body.as_ref().unwrap());
            emitln!(self.writer);
            self.writer.unindent();
            emitln!(self.writer, "}");
            emitln!(self.writer);
        }
    }
}

// Pre/Post Conditions
// ===================

impl<'env> SpecTranslator<'env> {
    // Generate boogie for asserts/assumes inside function bodies
    pub fn translate_conditions_inside_impl(
        &self,
        func_env: &'env FunctionEnv<'env>,
        offset: CodeOffset,
    ) {
        let conds = func_env.get_specification_on_impl(offset);
        if let Some(conds) = conds {
            if !conds.is_empty() {
                self.translate_seq(conds.iter(), "\n", |cond| {
                    self.writer.set_location(&cond.loc);
                    emit!(
                        self.writer,
                        if cond.kind == SpecConditionKind::Assert {
                            "assert "
                        } else {
                            "assume "
                        }
                    );
                    emit!(self.writer, "b#Boolean(");
                    self.translate_exp(&cond.exp);
                    emit!(self.writer, ");")
                });
                emitln!(self.writer);
            }
        }
    }

    /// Generates boogie for pre/post conditions.
    pub fn translate_conditions(&self, func_env: &'env FunctionEnv<'env>) {
        // Generate pre-conditions
        // For this transaction to be executed, it MUST have had
        // a valid signature for the sender's account. Therefore,
        // the senders account resource (which contains the pubkey)
        // must have existed! So we can assume txn_sender account
        // exists in pre-condition.
        let conds = func_env.get_specification_on_decl();
        emitln!(self.writer, "requires $ExistsTxnSenderAccount($m, $txn);");

        // Generate requires.
        let requires = conds
            .iter()
            .filter(|c| c.kind == SpecConditionKind::Requires)
            .collect_vec();
        if !requires.is_empty() {
            self.translate_seq(requires.iter(), "\n", |cond| {
                self.writer.set_location(&cond.loc);
                emit!(self.writer, "requires b#Boolean(");
                self.translate_exp(&cond.exp);
                emit!(self.writer, ");")
            });
            emitln!(self.writer);
        }

        // Generate aborts_if
        // abort_if P means function abort if P holds.
        // multiple abort_if conditions are "or"ed. If no condition holds, function does not abort.
        let aborts_if = conds
            .iter()
            .filter(|c| c.kind == SpecConditionKind::AbortsIf)
            .collect_vec();
        if !aborts_if.is_empty() {
            self.writer.set_location(&aborts_if[0].loc);
            emit!(self.writer, "ensures ");
            self.translate_seq(aborts_if.iter(), " || ", |c| {
                emit!(self.writer, "b#Boolean(old(");
                self.translate_exp_parenthesised(&c.exp);
                emit!(self.writer, "))")
            });
            emitln!(self.writer, " == $abort_flag;");
        }

        // Generate ensures
        let ensures = conds
            .iter()
            .filter(|c| c.kind == SpecConditionKind::Ensures)
            .collect_vec();
        if !ensures.is_empty() {
            self.translate_seq(ensures.iter(), "\n", |cond| {
                self.writer.set_location(&cond.loc);
                emit!(self.writer, "ensures !$abort_flag ==> (b#Boolean(");
                self.translate_exp(&cond.exp);
                emit!(self.writer, "));")
            });
            emitln!(self.writer);
        }

        // Generate implicit requires/ensures from module invariants if this is a public function.
        if func_env.is_public() {
            let invariants = func_env.module_env.get_module_invariants();
            if !invariants.is_empty() {
                self.translate_seq(invariants.iter(), "\n", |inv| {
                    self.writer.set_location(&inv.loc);
                    emit!(self.writer, "requires b#Boolean(");
                    self.translate_exp(&inv.exp);
                    emit!(self.writer, ");")
                });
                emitln!(self.writer);
                self.translate_seq(invariants.iter(), "\n", |inv| {
                    self.writer.set_location(&inv.loc);
                    emit!(self.writer, "ensures !$abort_flag ==> (b#Boolean(");
                    self.translate_exp(&inv.exp);
                    emit!(self.writer, "));")
                });
                emitln!(self.writer);
            }
        }
    }

    /// Assumes preconditions for function.
    pub fn assume_preconditions(&self, func_env: &FunctionEnv<'_>) {
        emitln!(self.writer, "assume $ExistsTxnSenderAccount($m, $txn);");

        // Explicit pre-conditions.
        let requires = func_env
            .get_specification_on_decl()
            .iter()
            .filter(|cond| cond.kind == SpecConditionKind::Requires)
            .collect_vec();
        if !requires.is_empty() {
            self.translate_seq(requires.iter(), "\n", |cond| {
                self.writer.set_location(&cond.loc);
                emit!(self.writer, "assume b#Boolean(");
                self.translate_exp(&cond.exp);
                emit!(self.writer, ");")
            });
            emitln!(self.writer);
        }

        // Implict module invariants.
        self.assume_module_invariants(func_env);
    }

    /// Assume module invariants of function.
    pub fn assume_module_invariants(&self, func_env: &FunctionEnv<'_>) {
        if func_env.is_public() {
            let invariants = func_env.module_env.get_module_invariants();
            if !invariants.is_empty() {
                self.translate_seq(invariants.iter(), "\n", |inv| {
                    self.writer.set_location(&inv.loc);
                    emit!(self.writer, "assume b#Boolean(");
                    self.translate_exp(&inv.exp);
                    emit!(self.writer, ");")
                });
                emitln!(self.writer);
            }
        }
    }
}

/// Invariants
/// ==========

impl<'env> SpecTranslator<'env> {
    /// Emitting invariant functions
    /// ----------------------------

    /// Emits functions and procedures needed for invariants.
    pub fn translate_invariant_functions(&self, struct_env: &StructEnv<'env>) {
        self.translate_assume_well_formed(struct_env);
        self.translate_before_update_invariant(struct_env);
        self.translate_after_update_invariant(struct_env);
    }

    /// Generates functions which assumes the struct to be well-formed. The first function
    /// only checks type assumptions and is called to ensure well-formedness while the struct is
    /// mutated. The second function checks both types and data invariants and is used while
    /// the struct is not mutated.
    fn translate_assume_well_formed(&self, struct_env: &StructEnv<'env>) {
        let emit_field_checks = |mode: WellFormedMode| {
            emitln!(self.writer, "is#Vector($this)");
            for field in struct_env.get_fields() {
                let select = format!("$SelectField($this, {})", boogie_field_name(&field));
                let type_check = boogie_well_formed_expr(
                    struct_env.module_env.env,
                    &select,
                    &field.get_type(),
                    mode,
                );
                if !type_check.is_empty() {
                    emitln!(self.writer, "  && {}", type_check);
                }
            }
        };
        emitln!(
            self.writer,
            "function {{:inline}} {}_is_well_formed_types($this: Value): bool {{",
            boogie_struct_name(struct_env),
        );
        self.writer.indent();
        emit_field_checks(WellFormedMode::WithoutInvariant);
        self.writer.unindent();
        emitln!(self.writer, "}");

        emitln!(
            self.writer,
            "function {{:inline}} {}_is_well_formed($this: Value): bool {{",
            boogie_struct_name(struct_env),
        );
        self.writer.indent();
        emit_field_checks(WellFormedMode::WithInvariant);
        for inv in struct_env.get_data_invariants() {
            emit!(self.writer, "  && b#Boolean(");
            self.with_invariant_target("$this", "", || self.translate_exp(&inv.exp));
            emitln!(self.writer, ")");
        }
        self.writer.unindent();
        emitln!(self.writer, "}");
        emitln!(self.writer);
    }

    /// Determines whether a before-update invariant is generated for this struct.
    ///
    /// We currently support two models for dealing with global spec var updates.
    /// If the specification has explicitly provided update invariants for spec vars, we use those.
    /// If not, we use the unpack invariants before the update, and the pack invariants
    /// after. This function is only true for the later case.
    pub fn has_before_update_invariant(struct_env: &StructEnv<'_>) -> bool {
        let no_explict_update = !struct_env
            .get_update_invariants()
            .iter()
            .any(Self::is_invariant_spec_var_update);
        let has_pack = !struct_env.get_unpack_invariants().is_empty();
        no_explict_update && has_pack
            // If any of the fields has it, it inherits to the struct.
            || struct_env.get_fields().any(|fe| {
                Self::has_before_update_invariant_ty(struct_env.module_env.env, &fe.get_type())
            })
    }

    /// Determines whether a before-update invariant is generated for this type.
    pub fn has_before_update_invariant_ty(env: &GlobalEnv, ty: &Type) -> bool {
        if let Some((struct_env, _)) = ty.get_struct(env) {
            Self::has_before_update_invariant(&struct_env)
        } else {
            // TODO: vectors
            false
        }
    }

    /// Helper to determine whether an invariant is a spec var update.
    fn is_invariant_spec_var_update(inv: &Invariant) -> bool {
        inv.target.is_some()
    }

    /// Generates a procedure which asserts the before-update invariants of the struct.
    pub fn translate_before_update_invariant(&self, struct_env: &StructEnv<'env>) {
        if !Self::has_before_update_invariant(struct_env) {
            return;
        }
        emitln!(
            self.writer,
            "procedure {{:inline 1}} {}_before_update_inv($before: Value) {{",
            boogie_struct_name(struct_env)
        );
        self.writer.indent();

        // Emit call to before update invariant procedure for all fields which have one by their own.
        for fe in struct_env.get_fields() {
            if let Some((nested_struct_env, _)) =
                fe.get_type().get_struct(struct_env.module_env.env)
            {
                if Self::has_before_update_invariant(&nested_struct_env) {
                    let field_name = boogie_field_name(&fe);
                    emitln!(
                        self.writer,
                        "call {}_before_update_inv($SelectField($before, {}));",
                        boogie_struct_name(&nested_struct_env),
                        field_name,
                    );
                }
            }
        }

        // Emit call to spec var updates via unpack invariants.
        self.emit_spec_var_updates("$before", "", struct_env.get_unpack_invariants());

        self.writer.unindent();
        emitln!(self.writer, "}");
        emitln!(self.writer);
    }

    /// Determines whether a after-update invariant is generated for this struct.
    pub fn has_after_update_invariant(struct_env: &StructEnv<'_>) -> bool {
        !struct_env.get_update_invariants().is_empty()
            || !struct_env.get_pack_invariants().is_empty()
            || !struct_env.get_data_invariants().is_empty()
            // If any of the fields has it, it inherits to the struct.
            || struct_env.get_fields().any(|fe| {
                Self::has_after_update_invariant_ty(struct_env.module_env.env, &fe.get_type())
            })
    }

    /// Determines whether a after-update invariant is generated for this type.
    pub fn has_after_update_invariant_ty(env: &GlobalEnv, ty: &Type) -> bool {
        if let Some((struct_env, _)) = ty.get_struct(env) {
            Self::has_after_update_invariant(&struct_env)
        } else {
            // TODO: vectors
            false
        }
    }

    /// Generates a procedure which asserts the after-update invariants of the struct.
    pub fn translate_after_update_invariant(&self, struct_env: &StructEnv<'env>) {
        if !Self::has_after_update_invariant(struct_env) {
            return;
        }
        emitln!(
            self.writer,
            "procedure {{:inline 1}} {}_after_update_inv($before: Value, $after: Value) {{",
            boogie_struct_name(struct_env)
        );
        self.writer.indent();

        // Emit call to after update invariant procedure for all fields which have one by their own.
        for fe in struct_env.get_fields() {
            if let Some((nested_struct_env, _)) =
                fe.get_type().get_struct(struct_env.module_env.env)
            {
                if Self::has_after_update_invariant(&nested_struct_env) {
                    let field_name = boogie_field_name(&fe);
                    emitln!(
                        self.writer,
                        "call {}_after_update_inv($SelectField($before, {}), $SelectField($after, {}));",
                        boogie_struct_name(&nested_struct_env),
                        field_name, field_name,
                    );
                }
            }
        }

        // Emit data invariants for this struct.
        self.emit_invariants_assume_or_assert(
            "$after",
            "",
            false,
            struct_env.get_data_invariants(),
        );

        // Emit update invariants for this struct.
        if struct_env
            .get_update_invariants()
            .iter()
            .any(Self::is_invariant_spec_var_update)
        {
            self.emit_spec_var_updates("$after", "$before", struct_env.get_update_invariants());
        } else {
            // Use the pack invariants to update spec vars.
            self.emit_spec_var_updates("$after", "", struct_env.get_pack_invariants());
        }
        self.emit_invariants_assume_or_assert(
            "$after",
            "$before",
            false,
            struct_env.get_update_invariants(),
        );

        self.writer.unindent();
        emitln!(self.writer, "}");
        emitln!(self.writer);
    }

    pub fn emit_pack_invariants(&self, struct_env: &StructEnv<'env>, target: &str) {
        self.emit_invariants_assume_or_assert(target, "", false, struct_env.get_data_invariants());
        self.emit_spec_var_updates(target, "", struct_env.get_pack_invariants());
    }

    /// Emits a sequence of statements which assert the unpack invariants.
    pub fn emit_unpack_invariants(&self, struct_env: &StructEnv<'env>, target: &str) {
        self.emit_spec_var_updates(target, "", struct_env.get_unpack_invariants());
    }

    /// Emits assume or assert for invariants.
    fn emit_invariants_assume_or_assert(
        &self,
        target: &str,
        old_target: &str,
        assume: bool,
        invariants: &[Invariant],
    ) {
        for inv in invariants {
            if inv.target.is_none() {
                self.writer.set_location(&inv.loc);
                if assume {
                    emit!(self.writer, "assume b#Boolean(");
                } else {
                    emit!(self.writer, "assert b#Boolean(");
                }
                self.with_invariant_target(target, old_target, || self.translate_exp(&inv.exp));
                emitln!(self.writer, ");");
            }
        }
    }

    /// Emits spec var updates for given invariants.
    fn emit_spec_var_updates(&self, target: &str, old_target: &str, invariants: &[Invariant]) {
        for inv in invariants {
            if let Some((module_id, spec_var_id)) = &inv.target {
                self.writer.set_location(&inv.loc);
                let module_env = self.module_env.env.get_module(*module_id);
                let spec_var = module_env.get_spec_var(*spec_var_id);
                emit!(
                    self.writer,
                    "{} := ",
                    boogie_spec_var_name(&self.module_env, spec_var.name),
                );
                self.with_invariant_target(target, old_target, || self.translate_exp(&inv.exp));
                emitln!(self.writer, ";")
            }
        }
    }
}

// Expressions
// ===========

impl<'env> SpecTranslator<'env> {
    fn translate_exp(&self, exp: &Exp) {
        match exp {
            Exp::Value(node_id, val) => {
                self.set_writer_location(*node_id);
                self.translate_value(*node_id, val);
            }
            Exp::LocalVar(node_id, name) => {
                self.set_writer_location(*node_id);
                self.translate_local_var(*node_id, *name);
            }
            Exp::SpecVar(node_id, module_id, var_id) => {
                self.set_writer_location(*node_id);
                let module_env = self.module_env.env.get_module(*module_id);
                let spec_var = module_env.get_spec_var(*var_id);
                emit!(
                    self.writer,
                    "{}",
                    boogie_spec_var_name(&module_env, spec_var.name)
                );
            }
            Exp::Call(node_id, oper, args) => {
                self.set_writer_location(*node_id);
                self.translate_call(*node_id, oper, args);
            }
            Exp::Invoke(node_id, ..) => self.error(
                &self.module_env.get_node_loc(*node_id),
                "Invoke not yet supported",
            ),
            Exp::Lambda(node_id, ..) => self.error(
                &self.module_env.get_node_loc(*node_id),
                "`|x|e` (lambda) currently only supported as argument for `all` or `any`",
            ),
            Exp::Block(node_id, vars, scope) => {
                self.set_writer_location(*node_id);
                self.translate_block(*node_id, vars, scope)
            }
            Exp::IfElse(node_id, cond, on_true, on_false) => {
                self.set_writer_location(*node_id);
                emit!(self.writer, "if (");
                self.translate_exp(cond);
                emit!(self.writer, ") then ");
                self.translate_exp_parenthesised(on_true);
                emit!(self.writer, " else ");
                self.translate_exp_parenthesised(on_false);
            }
            Exp::Error(_) => panic!("unexpected error expression"),
        }
    }

    fn translate_exp_parenthesised(&self, exp: &Exp) {
        emit!(self.writer, "(");
        self.translate_exp(exp);
        emit!(self.writer, ")");
    }

    fn translate_value(&self, _node_id: NodeId, val: &Value) {
        match val {
            Value::Address(addr) => emit!(self.writer, "Address({})", addr),
            Value::Number(val) => emit!(self.writer, "Integer({})", val),
            Value::Bool(val) => emit!(self.writer, "Boolean({})", val),
        }
    }

    fn translate_local_var(&self, node_id: NodeId, name: Symbol) {
        self.auto_dref(node_id, || {
            emit!(
                self.writer,
                self.module_env.symbol_pool().string(name).as_ref()
            );
        });
    }

    fn auto_dref<F>(&self, node_id: NodeId, f: F)
    where
        F: Fn(),
    {
        let ty = self.module_env.get_node_type(node_id);
        if ty.is_reference() {
            // Automatically dereference
            emit!(self.writer, "$Dereference($m, ");
        }
        f();
        if ty.is_reference() {
            emit!(self.writer, ")")
        }
    }

    fn translate_block(&self, node_id: NodeId, vars: &[LocalVarDecl], exp: &Exp) {
        if vars.is_empty() {
            return self.translate_exp(exp);
        }
        let loc = self.module_env.get_node_loc(node_id);
        if let Some((name, binding)) = self.get_decl_var(&loc, vars) {
            let name_str = self.module_env.symbol_pool().string(name);
            emit!(self.writer, "(var {} := ", name_str);
            self.translate_exp(binding.as_ref().expect("binding"));
            emit!(self.writer, "; ");
            self.translate_exp(exp);
            emit!(self.writer, ")");
        } else {
            // Error reported.
        }
    }

    fn translate_call(&self, node_id: NodeId, oper: &Operation, args: &[Exp]) {
        let loc = self.module_env.get_node_loc(node_id);
        match oper {
            Operation::Function(module_id, fun_id) => {
                self.translate_spec_fun_call(node_id, *module_id, *fun_id, args)
            }
            Operation::Pack(..) => self.error(&loc, "Pack not yet supported"),
            Operation::Tuple => self.error(&loc, "Tuple not yet supported"),
            Operation::Select(module_id, struct_id, field_id) => {
                self.translate_select(*module_id, *struct_id, *field_id, args)
            }
            Operation::Local(sym) => {
                emit!(
                    self.writer,
                    "$GetLocal($m, $frame + {})",
                    self.name_to_idx_map[sym],
                );
            }
            Operation::Result(pos) => self.auto_dref(node_id, || {
                emit!(self.writer, "$ret{}", pos);
            }),
            Operation::Index => self.translate_primitive_call("$select_vector_by_value", args),
            Operation::Slice => self.translate_primitive_call("$slice_vector", args),
            Operation::Range => self.translate_primitive_call("$Range", args),

            // Binary operators
            Operation::Add => self.translate_arith_op("+", args),
            Operation::Sub => self.translate_arith_op("-", args),
            Operation::Mul => self.translate_arith_op("*", args),
            Operation::Mod => self.translate_arith_op("mod", args),
            Operation::Div => self.translate_arith_op("div", args),
            Operation::BitOr => self.translate_arith_op("|", args),
            Operation::BitAnd => self.translate_arith_op("&", args),
            Operation::Xor => self.translate_arith_op("^", args),
            Operation::Shl => self.error(&loc, "Shl not yet supported"),
            Operation::Shr => self.error(&loc, "Shr not yet supported"),
            Operation::Implies => self.translate_logical_op("==>", args),
            Operation::And => self.translate_logical_op("&&", args),
            Operation::Or => self.translate_logical_op("||", args),
            Operation::Lt => self.translate_rel_op("<", args),
            Operation::Le => self.translate_rel_op("<=", args),
            Operation::Gt => self.translate_rel_op(">", args),
            Operation::Ge => self.translate_rel_op(">=", args),
            Operation::Eq => self.translate_eq_neq("IsEqual", args),
            Operation::Neq => self.translate_eq_neq("!IsEqual", args),

            // Unary operators
            Operation::Not => self.translate_logical_unary_op("!", args),

            // Builtin functions
            Operation::Global => self.translate_resource_access(node_id, args),
            Operation::Exists => self.translate_resource_exists(node_id, args),
            Operation::Len => self.translate_primitive_call("$vlen_value", args),
            Operation::Sender => emit!(self.writer, "$TxnSender($txn)"),
            Operation::All => self.translate_all_or_exists(&loc, true, args),
            Operation::Any => self.translate_all_or_exists(&loc, false, args),
            Operation::Update => self.translate_primitive_call("$update_vector_by_value", args),
            Operation::Old => self.translate_old(args),
            Operation::MaxU8 => emit!(self.writer, "Integer(MAX_U8)"),
            Operation::MaxU64 => emit!(self.writer, "Integer(MAX_U64)"),
            Operation::MaxU128 => emit!(self.writer, "Integer(MAX_U128)"),
        }
    }

    fn translate_spec_fun_call(
        &self,
        node_id: NodeId,
        module_id: ModuleId,
        fun_id: SpecFunId,
        args: &[Exp],
    ) {
        let instantiation = self.module_env.get_node_instantiation(node_id);
        let module_env = self.module_env.env.get_module(module_id);
        let name = boogie_spec_fun_name(&module_env, fun_id);
        emit!(self.writer, "{}(", name);
        let mut first = true;
        let mut maybe_comma = || {
            if first {
                first = false;
            } else {
                emit!(self.writer, ", ");
            }
        };
        for ty in instantiation.iter() {
            maybe_comma();
            emit!(self.writer, &boogie_type_value(self.module_env.env, ty));
        }
        for exp in args {
            maybe_comma();
            self.translate_exp(exp);
        }
        emit!(self.writer, ")");
    }

    fn translate_select(
        &self,
        module_id: ModuleId,
        struct_id: StructId,
        field_id: FieldId,
        args: &[Exp],
    ) {
        let module_env = self.module_env.env.get_module(module_id);
        let struct_env = module_env.get_struct(struct_id);
        let field_env = struct_env.get_field(field_id);
        let field_name = boogie_field_name(&field_env);
        emit!(self.writer, "$SelectField(");
        if args.is_empty() {
            // This is a field selection on an implicit invariant target.
            emit!(self.writer, &self.get_invariant_target());
        } else {
            self.translate_exp(&args[0]);
        }
        emit!(self.writer, ", {})", field_name);
    }

    fn translate_resource_access(&self, node_id: NodeId, args: &[Exp]) {
        let rty = &self.module_env.get_node_instantiation(node_id)[0];
        let type_value = boogie_type_value(self.module_env.env, rty);
        emit!(self.writer, "$ResourceValue($m, {}, ", type_value);
        self.translate_exp(&args[0]);
        emit!(self.writer, ")");
    }

    fn translate_resource_exists(&self, node_id: NodeId, args: &[Exp]) {
        let rty = &self.module_env.get_node_instantiation(node_id)[0];
        let type_value = boogie_type_value(self.module_env.env, rty);
        emit!(self.writer, "$ResourceExists($m, {}, ", type_value);
        self.translate_exp(&args[0]);
        emit!(self.writer, ")");
    }

    fn translate_all_or_exists(&self, loc: &Loc, is_all: bool, args: &[Exp]) {
        // all(v, |x| x > 0) -->
        //      (var $r := v; forall $i: int :: $InVectorRange($v, $i) ==> (var x:=$r[$i]; x > 0))
        // all(r, |x| x > 0) -->
        //      (var $r := r; forall $i: int :: $InRange($r, $i) ==> (var x:=$i; x > 0))
        // any(v, |x| x > 0) -->
        //      (var $r := v; exists $i: int :: $InVectorRange($v, $i) && (var x:=$r[$i]; x > 0))
        // any(r, |x| x > 0) -->
        //      (var $r := r; exists $i: int :: $InRange($r, $i) && (var x:=$i; x > 0))
        let quant_ty = self.module_env.get_node_type(args[0].node_id());
        let connective = if is_all { "==>" } else { "&&" };
        if let Exp::Lambda(_, vars, exp) = &args[1] {
            if let Some((var, _)) = self.get_decl_var(loc, vars) {
                let var_name = self.module_env.symbol_pool().string(var);
                let quant_var = self.fresh_var_name("i");
                let is_vector = match quant_ty {
                    Type::Vector(..) => true,
                    Type::Primitive(PrimitiveType::Range) => false,
                    Type::Reference(_, b) => {
                        if let Type::Vector(..) = *b {
                            true
                        } else {
                            panic!("unexpected type")
                        }
                    }
                    _ => panic!("unexpected type"),
                };
                let range_tmp = self.fresh_var_name("range");
                emit!(self.writer, "Boolean((var {} := ", range_tmp);
                self.translate_exp(&args[0]);
                if is_all {
                    emit!(self.writer, "; (forall {}: int :: ", quant_var);
                } else {
                    emit!(self.writer, "; (exists {}: int :: ", quant_var);
                }
                if is_vector {
                    emit!(
                        self.writer,
                        "$InVectorRange({}, {}) {} (var {} := $select_vector({}, {}); ",
                        range_tmp,
                        quant_var,
                        connective,
                        var_name,
                        range_tmp,
                        quant_var,
                    );
                } else {
                    emit!(
                        self.writer,
                        "$InRange({}, {}) {} (var {} := Integer({}); ",
                        range_tmp,
                        quant_var,
                        connective,
                        var_name,
                        quant_var,
                    );
                }
                emit!(self.writer, "b#Boolean(");
                self.translate_exp(exp.as_ref());
                emit!(self.writer, ")))))");
            } else {
                // error reported
            }
        } else {
            self.error(loc, "currently 2nd argument must be a lambda");
        }
    }

    fn get_decl_var<'a>(
        &self,
        loc: &Loc,
        vars: &'a [LocalVarDecl],
    ) -> Option<(Symbol, &'a Option<Exp>)> {
        if let [var] = vars {
            Some((var.name, &var.binding))
        } else {
            self.error(loc, "currently only single variable binding supported");
            None
        }
    }

    fn translate_old(&self, args: &[Exp]) {
        if self.supports_native_old {
            self.translate_primitive_call("old", args);
        } else {
            *self.in_old.borrow_mut() = true;
            self.translate_exp(&args[0]);
            *self.in_old.borrow_mut() = false;
        }
    }

    fn translate_eq_neq(&self, boogie_val_fun: &str, args: &[Exp]) {
        emit!(self.writer, "Boolean(");
        emit!(self.writer, "{}(", boogie_val_fun);
        self.translate_exp(&args[0]);
        emit!(self.writer, ", ");
        self.translate_exp(&args[1]);
        emit!(self.writer, ")");
        emit!(self.writer, ")");
    }

    fn translate_arith_op(&self, boogie_op: &str, args: &[Exp]) {
        emit!(self.writer, "Integer(i#Integer(");
        self.translate_exp(&args[0]);
        emit!(self.writer, ") {} i#Integer(", boogie_op);
        self.translate_exp(&args[1]);
        emit!(self.writer, "))");
    }

    fn translate_rel_op(&self, boogie_op: &str, args: &[Exp]) {
        emit!(self.writer, "Boolean(i#Integer(");
        self.translate_exp(&args[0]);
        emit!(self.writer, ") {} i#Integer(", boogie_op);
        self.translate_exp(&args[1]);
        emit!(self.writer, "))");
    }

    fn translate_logical_op(&self, boogie_op: &str, args: &[Exp]) {
        emit!(self.writer, "Boolean(b#Boolean(");
        self.translate_exp(&args[0]);
        emit!(self.writer, ") {} b#Boolean(", boogie_op);
        self.translate_exp(&args[1]);
        emit!(self.writer, "))");
    }

    fn translate_logical_unary_op(&self, boogie_op: &str, args: &[Exp]) {
        emit!(self.writer, "Boolean({}b#Boolean(", boogie_op);
        self.translate_exp(&args[0]);
        emit!(self.writer, "))");
    }

    fn translate_primitive_call(&self, fun: &str, args: &[Exp]) {
        emit!(self.writer, "{}(", fun);
        self.translate_seq(args.iter(), ", ", |e| self.translate_exp(e));
        emit!(self.writer, ")");
    }
}

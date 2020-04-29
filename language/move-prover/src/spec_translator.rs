// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module translates specification conditions to Boogie code.

use std::{cell::RefCell, collections::BTreeMap};

use spec_lang::{
    env::{FieldId, Loc, ModuleEnv, ModuleId, NodeId, SpecFunId, StructEnv, StructId},
    ty::{PrimitiveType, Type},
};

use crate::{
    boogie_helpers::{
        boogie_byte_blob, boogie_declare_global, boogie_field_name, boogie_global_declarator,
        boogie_local_type, boogie_spec_fun_name, boogie_spec_var_name, boogie_struct_name,
        boogie_type_value, boogie_well_formed_expr, WellFormedMode,
    },
    code_writer::CodeWriter,
};
use itertools::Itertools;
use spec_lang::{
    ast::{Condition, ConditionKind, Exp, LocalVarDecl, Operation, Value},
    env::GlobalEnv,
    symbol::Symbol,
};
use stackless_bytecode_generator::{
    function_target::FunctionTarget, stackless_bytecode::SpecBlockId,
};

enum SpecEnv<'env> {
    Module(&'env ModuleEnv<'env>),
    Function(&'env FunctionTarget<'env>),
}

pub struct SpecTranslator<'env> {
    /// The module in which context translation happens.
    spec_env: SpecEnv<'env>,
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
    // If we are translating in the context of a type instantiation, the type arguments.
    type_args_opt: Option<Vec<Type>>,
}

impl<'env> SpecTranslator<'env> {
    fn module_env(&self) -> &'env ModuleEnv<'env> {
        use SpecEnv::*;
        match self.spec_env {
            Module(module_env) => module_env,
            Function(func_target) => &func_target.func_env.module_env,
        }
    }

    fn function_target(&self) -> &'env FunctionTarget<'env> {
        use SpecEnv::*;
        match self.spec_env {
            Module(_) => panic!(),
            Function(func_target) => func_target,
        }
    }
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
            spec_env: SpecEnv::Module(module_env),
            writer,
            supports_native_old,
            in_old: RefCell::new(false),
            invariant_target: RefCell::new(("".to_string(), "".to_string())),
            fresh_var_count: RefCell::new(0),
            name_to_idx_map: BTreeMap::new(),
            type_args_opt: None,
        }
    }

    /// Creates a translator for use in translating spec blocks inside function implementation
    pub fn new_for_spec_in_impl(
        writer: &'env CodeWriter,
        func_target: &'env FunctionTarget<'env>,
        supports_native_old: bool,
    ) -> SpecTranslator<'env> {
        SpecTranslator {
            spec_env: SpecEnv::Function(func_target),
            writer,
            supports_native_old,
            in_old: RefCell::new(false),
            invariant_target: RefCell::new(("".to_string(), "".to_string())),
            fresh_var_count: RefCell::new(0),
            name_to_idx_map: (0..func_target.get_local_count())
                .map(|idx| (func_target.get_local_name(idx), idx))
                .collect(),
            type_args_opt: None,
        }
    }

    /// Sets type arguments in which context this translator works.
    pub fn set_type_args(mut self, type_args: Vec<Type>) -> Self {
        self.type_args_opt = Some(type_args);
        self
    }

    /// Emits a translation error.
    pub fn error(&self, loc: &Loc, msg: &str) {
        self.module_env()
            .env
            .error(loc, &format!("[boogie translator] {}", msg));
    }

    /// Sets the location of the code writer from node id.
    fn set_writer_location(&self, node_id: NodeId) {
        self.writer
            .set_location(&self.module_env().get_node_loc(node_id));
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
            self.module_env()
                .get_name()
                .display(self.module_env().symbol_pool())
        );
        for (_, var) in self.module_env().get_spec_vars() {
            let boogie_name = boogie_spec_var_name(&self.module_env(), var.name);
            emitln!(
                self.writer,
                &boogie_declare_global(
                    &self.module_env().env,
                    &boogie_name,
                    var.type_params.len(),
                    &var.type_
                )
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
            self.module_env()
                .get_name()
                .display(self.module_env().symbol_pool())
        );
        for (id, fun) in self.module_env().get_spec_funs() {
            if fun.body.is_none() {
                // This function is native and expected to be found in the prelude.
                continue;
            }
            if let Type::Tuple(..) | Type::Fun(..) = fun.result_type {
                self.error(&fun.loc, "function or tuple result type not yet supported");
                continue;
            }
            let result_type = boogie_local_type(&fun.result_type);
            let spec_var_params = fun.used_spec_vars.iter().map(|(mid, vid)| {
                let declaring_module = self.module_env().env.get_module(*mid);
                let decl = declaring_module.get_spec_var(*vid);
                let boogie_name = boogie_spec_var_name(&declaring_module, decl.name);
                boogie_global_declarator(
                    declaring_module.env,
                    &boogie_name,
                    decl.type_params.len(),
                    &decl.type_,
                )
            });
            let type_params = fun
                .type_params
                .iter()
                .enumerate()
                .map(|(i, _)| format!("$tv{}: TypeValue", i));
            let params = fun.params.iter().map(|(name, ty)| {
                format!(
                    "{}: {}",
                    name.display(self.module_env().symbol_pool()),
                    boogie_local_type(ty)
                )
            });
            let state_params = if fun.is_pure {
                vec![]
            } else {
                vec!["$m: Memory, $txn: Transaction".to_string()]
            };
            self.writer.set_location(&fun.loc);
            emitln!(
                self.writer,
                "function {{:inline}} {}({}): {} {{",
                boogie_spec_fun_name(&self.module_env(), *id),
                state_params
                    .into_iter()
                    .chain(spec_var_params)
                    .chain(type_params)
                    .chain(params)
                    .join(", "),
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
    pub fn translate_conditions_inside_impl(&self, block_id: SpecBlockId) {
        let func_target = self.function_target();
        let spec = func_target.get_spec_on_impl(block_id);
        if !spec.conditions.is_empty() {
            self.translate_seq(spec.conditions.iter(), "\n", |cond| {
                self.writer.set_location(&cond.loc);
                emit!(
                    self.writer,
                    if cond.kind == ConditionKind::Assert {
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

    /// Generates boogie for pre/post conditions.
    pub fn translate_conditions(&self) {
        // Generate pre-conditions
        // For this transaction to be executed, it MUST have had
        // a valid signature for the sender's account. Therefore,
        // the senders account resource (which contains the pubkey)
        // must have existed! So we can assume txn_sender account
        // exists in pre-condition.
        let func_target = self.function_target();
        let spec = func_target.get_spec();
        emitln!(self.writer, "requires $ExistsTxnSenderAccount($m, $txn);");

        // Generate requires.
        let requires = spec
            .filter(|c| {
                matches!(
                    c.kind,
                    ConditionKind::Requires | ConditionKind::RequiresModule
                )
            })
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
        let aborts_if = spec.filter_kind(ConditionKind::AbortsIf).collect_vec();
        if !aborts_if.is_empty() {
            self.writer.set_location(&aborts_if[0].loc);
            emit!(self.writer, "ensures (");
            self.translate_seq(aborts_if.iter(), " || ", |c| {
                emit!(self.writer, "b#Boolean(old(");
                self.translate_exp_parenthesised(&c.exp);
                emit!(self.writer, "))")
            });
            emitln!(self.writer, ") <==> $abort_flag;");
        }

        // Generate ensures
        let ensures = spec.filter_kind(ConditionKind::Ensures).collect_vec();
        if !ensures.is_empty() {
            self.translate_seq(ensures.iter(), "\n", |cond| {
                self.writer.set_location(&cond.loc);
                emit!(self.writer, "ensures !$abort_flag ==> (b#Boolean(");
                self.translate_exp(&cond.exp);
                emit!(self.writer, "));")
            });
            emitln!(self.writer);
        }
    }

    /// Assumes preconditions for function. This is used for the top-level verification
    /// entry point of a function.
    pub fn assume_preconditions(&self) {
        emitln!(self.writer, "assume $ExistsTxnSenderAccount($m, $txn);");
        let func_target = self.function_target();
        let requires = func_target
            .get_spec()
            .filter(|c| match c.kind {
                ConditionKind::Requires => true,
                ConditionKind::RequiresModule => func_target.is_public(),
                _ => false,
            })
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
    }

    /// Assume module requires of a function. This is used when the function is called from
    /// outside of a module.
    pub fn assume_module_preconditions(&self, func_target: &FunctionTarget<'_>) {
        if func_target.is_public() {
            let requires = func_target
                .get_spec()
                .filter(|c| matches!(c.kind, ConditionKind::RequiresModule))
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
            emitln!(
                self.writer,
                "  && $vlen($this) == {}",
                struct_env.get_fields().count()
            );
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
        for inv in struct_env.get_spec().filter_kind(ConditionKind::Invariant) {
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
        let spec = struct_env.get_spec();
        let no_explict_update = !spec.any(|c| matches!(c.kind, ConditionKind::VarUpdate(..)));
        let has_pack = spec.any(|c| matches!(c.kind, ConditionKind::VarPack(..)));
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

    /// Translate type parameters for given struct.
    pub fn translate_type_parameters(struct_env: &StructEnv<'_>) -> Vec<String> {
        (0..struct_env.get_type_parameters().len())
            .map(|i| format!("$tv{}: TypeValue", i))
            .collect_vec()
    }

    /// Generates a procedure which asserts the before-update invariants of the struct.
    pub fn translate_before_update_invariant(&self, struct_env: &StructEnv<'env>) {
        if !Self::has_before_update_invariant(struct_env) {
            return;
        }
        emitln!(
            self.writer,
            "procedure {{:inline 1}} {}_before_update_inv({}) {{",
            boogie_struct_name(struct_env),
            Self::translate_type_parameters(struct_env)
                .into_iter()
                .chain(vec!["$before: Value".to_string()])
                .join(", "),
        );
        self.writer.indent();

        // Emit call to before update invariant procedure for all fields which have one by their own.
        for fe in struct_env.get_fields() {
            if let Some((nested_struct_env, ty_args)) =
                fe.get_type().get_struct(struct_env.module_env.env)
            {
                if Self::has_before_update_invariant(&nested_struct_env) {
                    let field_name = boogie_field_name(&fe);
                    let args = ty_args
                        .iter()
                        .map(|ty| self.translate_type(ty))
                        .chain(vec![format!("$SelectField($before, {})", field_name)].into_iter())
                        .join(", ");
                    emitln!(
                        self.writer,
                        "call {}_before_update_inv({});",
                        boogie_struct_name(&nested_struct_env),
                        args,
                    );
                }
            }
        }

        // Emit call to spec var updates via unpack invariants.
        self.emit_spec_var_updates(
            "$before",
            "",
            struct_env
                .get_spec()
                .filter(|c| matches!(c.kind, ConditionKind::VarUnpack(..))),
        );

        self.writer.unindent();
        emitln!(self.writer, "}");
        emitln!(self.writer);
    }

    /// Determines whether a after-update invariant is generated for this struct.
    pub fn has_after_update_invariant(struct_env: &StructEnv<'_>) -> bool {
        use ConditionKind::*;
        struct_env.get_spec().any(|c| matches!(c.kind, VarUpdate(..)|VarPack(..)|Invariant))
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
            "procedure {{:inline 1}} {}_after_update_inv({}) {{",
            boogie_struct_name(struct_env),
            Self::translate_type_parameters(struct_env)
                .into_iter()
                .chain(vec!["$before: Value, $after: Value".to_string()])
                .join(", "),
        );
        self.writer.indent();

        // Emit call to after update invariant procedure for all fields which have one by their own.
        for fe in struct_env.get_fields() {
            if let Some((nested_struct_env, ty_args)) =
                fe.get_type().get_struct(struct_env.module_env.env)
            {
                if Self::has_after_update_invariant(&nested_struct_env) {
                    let field_name = boogie_field_name(&fe);
                    let args = ty_args
                        .iter()
                        .map(|ty| self.translate_type(ty))
                        .chain(
                            vec![format!(
                                "$SelectField($before, {}), $SelectField($after, {})",
                                field_name, field_name
                            )]
                            .into_iter(),
                        )
                        .join(", ");
                    emitln!(
                        self.writer,
                        "call {}_after_update_inv({});",
                        boogie_struct_name(&nested_struct_env),
                        args
                    );
                }
            }
        }

        // Emit data invariants for this struct.
        let spec = struct_env.get_spec();
        self.emit_invariants_assume_or_assert(
            "$after",
            "",
            false,
            spec.filter_kind(ConditionKind::Invariant),
        );

        // Emit update invariants for this struct.
        let var_updates = spec
            .filter(|c| matches!(c.kind, ConditionKind::VarUpdate(..)))
            .collect_vec();
        if !var_updates.is_empty() {
            self.emit_spec_var_updates("$after", "$before", var_updates.into_iter());
        } else {
            // Use the pack invariants to update spec vars.
            self.emit_spec_var_updates(
                "$after",
                "",
                spec.filter(|c| matches!(c.kind, ConditionKind::VarPack(..))),
            );
        }
        self.emit_invariants_assume_or_assert(
            "$after",
            "$before",
            false,
            spec.filter_kind(ConditionKind::InvariantUpdate),
        );

        self.writer.unindent();
        emitln!(self.writer, "}");
        emitln!(self.writer);
    }

    pub fn emit_pack_invariants(&self, struct_env: &StructEnv<'env>, target: &str) {
        let spec = struct_env.get_spec();
        self.emit_invariants_assume_or_assert(
            target,
            "",
            false,
            spec.filter_kind(ConditionKind::Invariant),
        );
        self.emit_spec_var_updates(
            target,
            "",
            spec.filter(|c| matches!(c.kind, ConditionKind::VarPack(..))),
        );
    }

    /// Emits a sequence of statements which assert the unpack invariants.
    pub fn emit_unpack_invariants(&self, struct_env: &StructEnv<'env>, target: &str) {
        self.emit_spec_var_updates(
            target,
            "",
            struct_env
                .get_spec()
                .filter(|c| matches!(c.kind, ConditionKind::VarUnpack(..))),
        );
    }

    /// Emits assume or assert for invariants.
    fn emit_invariants_assume_or_assert<'a>(
        &self,
        target: &str,
        old_target: &str,
        assume: bool,
        invariants: impl Iterator<Item = &'a Condition>,
    ) {
        for inv in invariants {
            if inv.kind.get_spec_var_target().is_none() {
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
    fn emit_spec_var_updates<'a>(
        &self,
        target: &str,
        old_target: &str,
        invariants: impl Iterator<Item = &'a Condition>,
    ) {
        for inv in invariants {
            if let Some((module_id, spec_var_id, tys)) = &inv.kind.get_spec_var_target() {
                self.writer.set_location(&inv.loc);
                let module_env = self.module_env().env.get_module(*module_id);
                let spec_var = module_env.get_spec_var(*spec_var_id);
                let var_name = boogie_spec_var_name(&self.module_env(), spec_var.name);
                if !tys.is_empty() {
                    emit!(
                        self.writer,
                        "{} := {}[{} := ",
                        var_name,
                        var_name,
                        tys.iter().map(|ty| self.translate_type(ty)).join(", ")
                    );
                } else {
                    emit!(self.writer, "{} := ", var_name);
                }
                self.with_invariant_target(target, old_target, || self.translate_exp(&inv.exp));
                if !tys.is_empty() {
                    emitln!(self.writer, "];")
                } else {
                    emitln!(self.writer, ";")
                }
            }
        }
    }
}

// Types
// ===========

impl<'env> SpecTranslator<'env> {
    /// Translates a type into a string in boogie. If the translator works with a type
    /// instantiation, this will be used to instantiate the type.
    fn translate_type(&self, ty: &Type) -> String {
        if let Some(ty_args) = &self.type_args_opt {
            boogie_type_value(self.module_env().env, &ty.instantiate(ty_args))
        } else {
            boogie_type_value(self.module_env().env, ty)
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
                let module_env = self.module_env().env.get_module(*module_id);
                let spec_var = module_env.get_spec_var(*var_id);
                let instantiation = self.module_env().get_node_instantiation(*node_id);
                let instantiation_str = if instantiation.is_empty() {
                    "".to_string()
                } else {
                    format!(
                        "[{}]",
                        instantiation
                            .iter()
                            .map(|ty| self.translate_type(ty))
                            .join(", ")
                    )
                };
                emit!(
                    self.writer,
                    "{}{}",
                    boogie_spec_var_name(&module_env, spec_var.name),
                    instantiation_str
                );
            }
            Exp::Call(node_id, oper, args) => {
                self.set_writer_location(*node_id);
                self.translate_call(*node_id, oper, args);
            }
            Exp::Invoke(node_id, ..) => self.error(
                &self.module_env().get_node_loc(*node_id),
                "Invoke not yet supported",
            ),
            Exp::Lambda(node_id, ..) => self.error(
                &self.module_env().get_node_loc(*node_id),
                "`|x|e` (lambda) currently only supported as argument for `all` or `any`",
            ),
            Exp::Block(node_id, vars, scope) => {
                self.set_writer_location(*node_id);
                self.translate_block(*node_id, vars, scope)
            }
            Exp::IfElse(node_id, cond, on_true, on_false) => {
                self.set_writer_location(*node_id);
                emit!(self.writer, "if (b#Boolean(");
                self.translate_exp(cond);
                emit!(self.writer, ")) then ");
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
            Value::ByteArray(val) => emit!(self.writer, &boogie_byte_blob(val)),
        }
    }

    fn translate_local_var(&self, node_id: NodeId, name: Symbol) {
        let mut ty = &self.module_env().get_node_type(node_id);
        // overwrite ty if func_target provides a binding for name
        if let SpecEnv::Function(func_target) = self.spec_env {
            if let Some(local_index) = func_target.get_local_index(name) {
                ty = func_target.get_local_type(*local_index);
            }
        };
        self.auto_dref(ty, || {
            emit!(
                self.writer,
                self.module_env().symbol_pool().string(name).as_ref()
            );
        });
    }

    fn auto_dref<F>(&self, ty: &Type, f: F)
    where
        F: Fn(),
    {
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
        let loc = self.module_env().get_node_loc(node_id);
        if let Some((name, binding)) = self.get_decl_var(&loc, vars) {
            let name_str = self.module_env().symbol_pool().string(name);
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
        let loc = self.module_env().get_node_loc(node_id);
        match oper {
            Operation::Function(module_id, fun_id) => {
                self.translate_spec_fun_call(node_id, *module_id, *fun_id, args)
            }
            Operation::Pack(..) => self.translate_pack(args),
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
            Operation::Result(pos) => {
                self.auto_dref(self.function_target().get_return_type(*pos), || {
                    emit!(self.writer, "$ret{}", pos);
                })
            }
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

    fn translate_pack(&self, args: &[Exp]) {
        emit!(
            self.writer,
            "Vector({}EmptyValueArray",
            "ExtendValueArray(".repeat(args.len())
        );
        for arg in args.iter() {
            emit!(self.writer, ", ");
            self.translate_exp(arg);
            emit!(self.writer, ")");
        }
        emit!(self.writer, ")"); // A closing bracket for Vector
    }

    fn translate_spec_fun_call(
        &self,
        node_id: NodeId,
        module_id: ModuleId,
        fun_id: SpecFunId,
        args: &[Exp],
    ) {
        let instantiation = self.module_env().get_node_instantiation(node_id);
        let module_env = self.module_env().env.get_module(module_id);
        let fun_decl = module_env.get_spec_fun(fun_id);
        let name = boogie_spec_fun_name(&module_env, fun_id);
        emit!(self.writer, "{}(", name);
        let mut first = if !fun_decl.is_pure {
            emit!(self.writer, "$m, $txn");
            false
        } else {
            true
        };
        let mut maybe_comma = || {
            if first {
                first = false;
            } else {
                emit!(self.writer, ", ");
            }
        };
        for (mid, vid) in &fun_decl.used_spec_vars {
            maybe_comma();
            let declaring_module = self.module_env().env.get_module(*mid);
            let var_decl = declaring_module.get_spec_var(*vid);
            emit!(
                self.writer,
                &boogie_spec_var_name(&declaring_module, var_decl.name)
            );
        }
        for ty in instantiation.iter() {
            maybe_comma();
            emit!(self.writer, &self.translate_type(ty));
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
        let module_env = self.module_env().env.get_module(module_id);
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
        let rty = &self.module_env().get_node_instantiation(node_id)[0];
        let type_value = self.translate_type(rty);
        emit!(self.writer, "$ResourceValue($m, {}, ", type_value);
        self.translate_exp(&args[0]);
        emit!(self.writer, ")");
    }

    fn translate_resource_exists(&self, node_id: NodeId, args: &[Exp]) {
        let rty = &self.module_env().get_node_instantiation(node_id)[0];
        let type_value = self.translate_type(rty);
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
        let quant_ty = self.module_env().get_node_type(args[0].node_id());
        let connective = if is_all { "==>" } else { "&&" };
        if let Exp::Lambda(_, vars, exp) = &args[1] {
            if let Some((var, _)) = self.get_decl_var(loc, vars) {
                let var_name = self.module_env().symbol_pool().string(var);
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

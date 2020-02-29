// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module translates specification conditions to Boogie code.

use std::{
    cell::RefCell,
    collections::{BTreeMap, LinkedList},
};

use itertools::Itertools;
use num::{BigInt, Num};

use libra_types::account_address::AccountAddress;
use move_ir_types::{
    ast::{BinOp, CopyableVal_, Field_, QualifiedStructIdent, Type},
    location::{Loc, Spanned},
    spec_language_ast::{Condition_, FieldOrIndex, SpecExp, StorageLocation},
};

use crate::{
    boogie_helpers::{
        boogie_field_name, boogie_struct_name, boogie_struct_type_value, boogie_synthetic_name,
        boogie_type_check_expr, boogie_type_value,
    },
    code_writer::CodeWriter,
    env::{
        FunctionEnv, GlobalType, ModuleEnv, ModuleIndex, StructEnv, TypeParameter, ERROR_TYPE,
        UNKNOWN_TYPE,
    },
};

pub struct SpecTranslator<'env> {
    /// The module in which context translation happens.
    module_env: &'env ModuleEnv<'env>,
    /// The code writer.
    writer: &'env CodeWriter,
    // Location for reporting type checking errors
    current_loc: RefCell<Loc>,
    /// A symbol table for looking up names. The first element in the list contains the most
    /// inner scope.
    symtab: RefCell<LinkedList<SymTabScope>>,
    /// Whether we are currently in the context of translating an `old(...)` expression.
    in_old: RefCell<bool>,
    /// Type parameters for resolving AST types in expressions.
    type_parameters: Vec<TypeParameter>,
    /// Whether the translation context supports the `old` expressions (i.e. is a Boogie
    /// pre/post condition)
    supports_native_old: bool,
}

/// A key for the symbol table. Consists of a name and whether the name lookup appears within an
/// `old(...)` context.
type SymTabKey = (String, bool);

/// A scope for the symbol table.
type SymTabScope = BTreeMap<SymTabKey, BoogieExpr>;

// General
// =======

impl<'env> SpecTranslator<'env> {
    /// Creates a translator.
    pub fn new(
        writer: &'env CodeWriter,
        module_env: &'env ModuleEnv<'env>,
        type_parameters: Vec<TypeParameter>,
        supports_native_old: bool,
    ) -> SpecTranslator<'env> {
        SpecTranslator {
            module_env,
            writer,
            current_loc: RefCell::new(Spanned::unsafe_no_loc(()).loc),
            symtab: RefCell::new(LinkedList::new()),
            in_old: RefCell::new(false),
            type_parameters,
            supports_native_old,
        }
    }

    fn current_loc(&self) -> Loc {
        *self.current_loc.borrow()
    }

    /// Reports a type checking error.
    fn error<T>(&self, msg: &str, pass_through: T) -> T {
        self.module_env.error(self.current_loc(), msg, pass_through)
    }

    /// Returns an error expression.
    fn error_exp(&self) -> BoogieExpr {
        BoogieExpr("<error>".to_string(), ERROR_TYPE)
    }

    /// Helper to update the current location. This is both used for informing the `error` function
    /// above for type checking errors and the CodeWriter for being able to map boogie errors
    /// back to the source.
    fn update_location(&self, loc: Loc) -> (ModuleIndex, Loc) {
        self.current_loc.replace(loc);
        self.writer
            .set_location(self.module_env.get_module_idx(), loc)
    }

    /// Restore the saved location from a previous saved one.
    fn restore_saved_location(&self, saved: (ModuleIndex, Loc)) {
        // This is only for the writer.
        self.writer.set_location(saved.0, saved.1);
    }

    /// Enters a new scope in the symbol table.
    fn enter_scope(&self) {
        self.symtab.borrow_mut().push_front(SymTabScope::new());
    }

    /// Exists the most inner scope of the symbol table.
    fn exit_scope(&self) {
        self.symtab.borrow_mut().pop_front();
    }

    /// Defines a symbol in the most inner scope. This produces an error
    /// if the name already exists.
    fn define_symbol(&self, name: &str, for_old: bool, def: BoogieExpr) {
        if self
            .symtab
            .borrow_mut()
            .front_mut()
            .expect("symbol table empty")
            .insert((name.to_string(), for_old), def)
            .is_some()
        {
            self.error(&format!("duplicate declaration of `{}`", name), ())
        }
    }

    /// Looks up a name in the symbol table. This produces an error and returns an error expression
    /// if the name is not defined. Iterates the scopes of the symbol table inner-most out.
    fn lookup_symbol(&self, name: &str, for_old: bool) -> BoogieExpr {
        let key = (name.to_string(), for_old);
        for scope in &*self.symtab.borrow() {
            if let Some(exp) = scope.get(&key) {
                return exp.clone();
            }
        }
        if for_old {
            // Try to lookup the plain symbol as no old version of it exists.
            self.lookup_symbol(name, false)
        } else {
            self.error(&format!("undeclared name `{}`", name), self.error_exp())
        }
    }

    /// Populates the symbol table with the synthetics of the module.
    fn define_synthetics(&self) {
        for (syn, ty) in self.module_env.get_synthetics() {
            self.define_symbol(
                syn.value.name.as_str(),
                false,
                BoogieExpr(
                    boogie_synthetic_name(self.module_env, syn.value.name.as_str()),
                    ty.clone(),
                ),
            );
        }
    }
}

// Pre/Post Conditions
// ===================

impl<'env> SpecTranslator<'env> {
    /// Generates boogie for pre/post conditions. The SpecTranslator must be
    /// targeted for conditions.
    pub fn translate_conditions(&self, func_env: &'env FunctionEnv<'env>) {
        // Populate the symbol table.
        self.enter_scope();
        self.define_synthetics();
        self.enter_scope(); // Parameter names shadow synthetic names
        for param in &func_env.get_parameters() {
            self.define_symbol(
                param.0.as_str(),
                false,
                BoogieExpr(param.0.to_string(), param.1.clone()),
            );
        }
        for (i, ty) in func_env.get_return_types().iter().enumerate() {
            let name = &format!("__ret{}", i);
            self.define_symbol(name, false, BoogieExpr(name.clone(), ty.clone()));
        }

        // Generate pre-conditions
        // For this transaction to be executed, it MUST have had
        // a valid signature for the sender's account. Therefore,
        // the senders account resource (which contains the pubkey)
        // must have existed! So we can assume txn_sender account
        // exists in pre-condition.
        let conds = func_env.get_specification();
        for cond in conds {
            if let Condition_::Requires(expr) = &cond.value {
                self.update_location(cond.loc);
                emitln!(
                    self.writer,
                    "requires b#Boolean({});",
                    self.translate_expr(expr).result()
                );
            }
        }
        emitln!(self.writer, "requires ExistsTxnSenderAccount(__m, __txn);");

        // Generate succeeds_if and aborts_if setup (for post conditions)

        // When all succeeds_if conditions hold, function must not abort.
        let mut succeeds_if_string = conds
            .iter()
            .filter_map(|c| match &c.value {
                Condition_::SucceedsIf(expr) => {
                    self.update_location(c.loc);
                    Some(format!("b#Boolean({})", self.translate_expr(expr).result()))
                }
                _ => None,
            })
            .join(" && ");

        // abort_if P means function MUST abort if P holds.
        // multiple abort_if conditions are "or"ed.
        let aborts_if_string = conds
            .iter()
            .filter_map(|c| match &c.value {
                Condition_::AbortsIf(expr) => {
                    self.update_location(c.loc);
                    Some(format!("b#Boolean({})", self.translate_expr(expr).result()))
                }
                _ => None,
            })
            .join(" || ");

        // negation of abort_if condition is an implicit succeeds_if condition.
        // So, if no explicit succeeds_if specifications exist, the function must
        // succeed whenever the aborts_if condition does not hold.
        // If there are explicit succeeds_if conditions, there may be cases where
        // no aborts_if holds and not all succeeds_if conditions hold.  In that case,
        // the function may or may not abort. If it does not abort, it must meet all
        // explicit ensures conditions.
        // NOTE: It's not yet clear that succeeds_if is useful, or if this is the most
        // useful interpretation.
        if !aborts_if_string.is_empty() {
            if !succeeds_if_string.is_empty() {
                succeeds_if_string = format!("!({}) && ({})", aborts_if_string, succeeds_if_string);
            } else {
                succeeds_if_string = format!("!({})", aborts_if_string);
            }
        }

        // Generate explicit ensures conditions
        for cond in conds {
            if let Condition_::Ensures(expr) = &cond.value {
                // FIXME: Do we really need to check whether succeeds_if & aborts_if are
                // empty, below?
                self.update_location(cond.loc);
                emitln!(
                    self.writer,
                    "ensures {}b#Boolean({});",
                    if !succeeds_if_string.is_empty() || !aborts_if_string.is_empty() {
                        // Guard the condition to be only effective if not aborted.
                        "!__abort_flag ==> "
                    } else {
                        ""
                    },
                    self.translate_expr(expr).result()
                );
            }
        }

        // emit the Boogie ensures condition for succeeds_if
        if !succeeds_if_string.is_empty() {
            // If succeeds_if condition is met, function must NOT abort.
            emitln!(
                self.writer,
                "ensures old({}) ==> !__abort_flag;",
                succeeds_if_string
            );
        }

        // emit Boogie ensures condition for aborts_if
        if !aborts_if_string.is_empty() {
            emitln!(
                self.writer,
                "ensures old({}) ==> __abort_flag;\n",
                aborts_if_string
            );
        }
    }
}

/// Invariants
/// ==========

impl<'env> SpecTranslator<'env> {
    /// Emitting invariant functions
    /// ----------------------------

    /// Emits functions amd procedures for needed for invariants.
    pub fn translate_invariant_functions(&self, struct_env: &StructEnv<'env>) {
        self.enter_scope();
        self.define_synthetics();
        self.translate_assume_well_formed(struct_env);
        self.translate_update_invariant(struct_env);
        self.exit_scope();
    }

    /// Separates elements in vector, dropping empty ones.
    /// Generates a function which assumes the struct to be well-formed. This generates
    /// both type assumptions for each field and assumptions of the data invariants.
    fn translate_assume_well_formed(&self, struct_env: &StructEnv<'env>) {
        emitln!(
            self.writer,
            "function {{:inline 1}} ${}_is_well_formed(__this: Value): bool {{",
            boogie_struct_name(struct_env),
        );
        self.writer.indent();
        self.enter_scope();
        self.define_invariant_symbols(struct_env, "__this", false);
        let mut assumptions = vec![];

        // Emit type assumptions.
        assumptions.push("is#Vector(__this)".to_string());
        for field in struct_env.get_fields() {
            let select = format!("SelectField(__this, {})", boogie_field_name(&field));
            let type_check =
                boogie_type_check_expr(struct_env.module_env.env, &select, &field.get_type());
            if !type_check.is_empty() {
                assumptions.push(type_check);
            }
        }

        // Emit invariant assumptions.
        for inv in struct_env.get_data_invariants() {
            if inv.value.target.is_some() {
                self.error("a data invariant cannot have a synthetic assignment", ());
            }
            let cond = self.translate_expr_require_type(&inv.value.exp, &GlobalType::Bool);
            assumptions.push(format!("b#Boolean({})", cond));
        }

        emitln!(self.writer, &assumptions.iter().join("\n    && "));
        self.exit_scope();
        self.writer.unindent();
        emitln!(self.writer, "}");
        emitln!(self.writer);
    }

    /// Generates a procedure which asserts the update invariants of the struct.
    ///
    pub fn translate_update_invariant(&self, struct_env: &StructEnv<'env>) {
        if struct_env.get_update_invariants().is_empty() {
            return;
        }
        emitln!(
            self.writer,
            "procedure {{:inline 1}} ${}_update_inv(__before: Value, __after: Value) {{",
            boogie_struct_name(struct_env)
        );
        self.writer.indent();

        // Emit update assertions.
        self.enter_scope();
        self.define_invariant_symbols(struct_env, "__before", true);
        self.define_invariant_symbols(struct_env, "__after", false);
        for inv in struct_env.get_update_invariants() {
            let saved = self.update_location(inv.loc);
            if let Some(syn_name) = &inv.value.target {
                let syn_ty = self.get_synthetic_type(struct_env, syn_name);
                // Assignment to a synthetic variable.
                let value = self.translate_expr_require_type(&inv.value.exp, &syn_ty);
                emitln!(
                    self.writer,
                    "{} := {};",
                    boogie_synthetic_name(&struct_env.module_env, syn_name),
                    value
                );
            } else {
                // Delta invariant
                let cond = self.translate_expr_require_type(&inv.value.exp, &GlobalType::Bool);
                emitln!(self.writer, &format!("assert b#Boolean({});", cond));
            }
            self.restore_saved_location(saved);
        }
        self.exit_scope();

        // Emmit data assertions.
        self.enter_scope();
        self.define_invariant_symbols(struct_env, "__after", false);
        for inv in struct_env.get_data_invariants() {
            let saved = self.update_location(inv.loc);
            if inv.value.target.is_some() {
                self.error("a data invariant cannot have a synthetic assignment", ());
            }
            let cond = self.translate_expr_require_type(&inv.value.exp, &GlobalType::Bool);
            emitln!(self.writer, &format!("assert b#Boolean({});", cond));
            self.restore_saved_location(saved);
        }
        self.exit_scope();

        self.writer.unindent();
        emitln!(self.writer, "}");
        emitln!(self.writer);
    }

    /// Emits a sequence of statements which assert the pack invariants,
    pub fn emit_pack_invariants(&self, struct_env: &StructEnv<'env>, target: &str) {
        self.enter_scope();
        self.define_synthetics();
        self.enter_scope();
        self.define_invariant_symbols(struct_env, target, false);

        // TODO: should we assume type assumptions here instead of on callee side?

        // Emmit data assertions.
        for inv in struct_env.get_data_invariants() {
            let saved = self.update_location(inv.loc);
            if inv.value.target.is_some() {
                self.error("a data invariant cannot have a synthetic assignment", ());
            }
            let cond = self.translate_expr_require_type(&inv.value.exp, &GlobalType::Bool);
            emitln!(self.writer, &format!("assert b#Boolean({});", cond));
            self.restore_saved_location(saved);
        }

        // Emit synthetic variable updates.
        for inv in struct_env.get_pack_invariants() {
            self.update_location(inv.loc);
            if let Some(syn_name) = &inv.value.target {
                let syn_ty = self.get_synthetic_type(struct_env, syn_name);
                let value = self.translate_expr_require_type(&inv.value.exp, &syn_ty);
                emitln!(
                    self.writer,
                    "{} := {};",
                    boogie_synthetic_name(&struct_env.module_env, syn_name),
                    value
                );
            } else {
                self.error("a pack invariant must be assignment to synthetic", ());
            }
        }
        self.exit_scope();
        self.exit_scope();
    }

    /// Emits a sequence of statements which assert the unpack invariants.
    pub fn emit_unpack_invariants(&self, struct_env: &StructEnv<'env>, target: &str) {
        self.enter_scope();
        self.define_synthetics();
        self.enter_scope();
        self.define_invariant_symbols(struct_env, target, false);
        for inv in struct_env.get_unpack_invariants() {
            let saved = self.update_location(inv.loc);
            if let Some(syn_name) = &inv.value.target {
                let syn_ty = self.get_synthetic_type(struct_env, syn_name);
                let value = self.translate_expr_require_type(&inv.value.exp, &syn_ty);
                emitln!(
                    self.writer,
                    "{} := {};",
                    boogie_synthetic_name(&struct_env.module_env, syn_name),
                    value
                );
            } else {
                self.error("an unpack invariant must be assignment to synthetic", ());
            }
            self.restore_saved_location(saved);
        }
        self.exit_scope();
        self.exit_scope();
    }

    /// Looks up a synthetic and returns it type, or generates an error if not found.
    fn get_synthetic_type(&self, struct_env: &StructEnv<'_>, name: &str) -> GlobalType {
        if let Some((_, ty)) = struct_env.module_env.find_synthetic(name) {
            ty.clone()
        } else {
            self.error(
                &format!(
                    "synthetic `{}` not declared in module `{}`",
                    name,
                    struct_env.module_env.get_id().name()
                ),
                ERROR_TYPE,
            )
        }
    }

    /// Defines symbols in the symbol table for invariant expression translation.
    fn define_invariant_symbols(&self, struct_env: &StructEnv<'_>, value: &str, for_old: bool) {
        for field in struct_env.get_fields() {
            let name = field.get_name().as_str();
            let ty = &field.get_type();
            self.define_symbol(
                name,
                for_old,
                BoogieExpr(
                    format!("SelectField({}, {})", value, boogie_field_name(&field)),
                    ty.clone(),
                ),
            );
        }
    }

    /// Emitting invariant checks
    /// -------------------------

    /// Emits an update invariant check. `reaching_root_types` is the set of root types the
    /// updated reference can have. A valid over-approximation is the the set of all structs
    /// with update invariants in a program, but the update check will be more efficient for
    /// larger programs if `reaching_root_types` is more specific than that.
    pub fn emit_update_invariant_check(
        &self,
        target: &str,
        before_value: &str,
        after_value: &str,
        reaching_root_types: &[GlobalType],
    ) {
        // Determine structs with invariants from the provided root types.
        let structs = self.get_structs_which(reaching_root_types, |s| {
            !s.get_update_invariants().is_empty()
        });

        for struct_env in structs {
            // Generate:
            //
            //   if (RootType($target) == <struct1_type>) {
            //      <struct1_update_invariant>($old_value, $value);
            //   }
            //   if (RootType($target) == <struct2_type>) {
            //      <struct1_update_invariant>($old_value, $value);
            //   }
            //   ...
            let boogie_type = boogie_struct_type_value(
                self.module_env.env,
                struct_env.module_env.get_module_idx(),
                struct_env.get_def_idx(),
                &[],
            );
            emitln!(
                self.writer,
                "if (RootReferenceType({}) == {}) {{",
                target,
                boogie_type
            );
            self.writer.indent();
            emitln!(
                self.writer,
                "call ${}_update_inv({}, {});",
                boogie_struct_name(&struct_env),
                before_value,
                after_value
            );
            self.writer.unindent();
            emitln!(self.writer, "}");
        }
    }

    /// Determines whether an update invariant check is needed for the reaching root types.
    pub fn needs_update_invariant_check(&self, reaching_root_types: &[GlobalType]) -> bool {
        // TODO: may want to avoid collecting the structs first
        let structs = self.get_structs_which(reaching_root_types, |s| {
            !s.get_update_invariants().is_empty()
        });
        !structs.is_empty()
    }

    /// Gets the subset of structs in `cands` which satisfy a predicate.
    fn get_structs_which<P>(&'env self, cands: &[GlobalType], p: P) -> Vec<StructEnv<'env>>
    where
        P: Fn(&StructEnv<'_>) -> bool,
    {
        cands
            .iter()
            .filter_map(|ty| {
                if let GlobalType::Struct(module_idx, struct_idx, _) = ty {
                    let struct_env = self
                        .module_env
                        .env
                        .get_module(*module_idx)
                        .into_get_struct(*struct_idx);
                    if p(&struct_env) {
                        Some(struct_env)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect_vec()
    }
}

// Expressions
// ===========

/// Represents a boogie expression as a string and its type. The type is used to access
/// necessary context information for generating boogie expressions, as well as for type
/// checking.
#[derive(Debug, Clone)]
struct BoogieExpr(String, GlobalType);

impl BoogieExpr {
    fn result(self) -> String {
        self.0
    }
}

impl<'env> SpecTranslator<'env> {
    /// Translates a specification expression into boogie.
    ///
    /// This returns a boogie expression of type Value or Reference.
    fn translate_expr(&self, expr: &SpecExp) -> BoogieExpr {
        match expr {
            SpecExp::Constant(val) => self.translate_constant(val),
            SpecExp::StorageLocation(loc) => self.translate_location_as_value(loc),
            SpecExp::GlobalExists {
                type_,
                type_actuals,
                address,
            } => {
                let BoogieExpr(a, at) = self.translate_location_as_value(address);
                let _ = self.require_type(at, &GlobalType::Address);
                BoogieExpr(
                    format!(
                        "ExistsResource(__m, {}, a#Address({}))",
                        self.translate_resource_type(type_, type_actuals).0,
                        a
                    ),
                    GlobalType::Bool,
                )
            }
            SpecExp::Dereference(loc) => self.translate_dref(loc),
            SpecExp::Reference(loc) => self.translate_location_as_reference(loc),
            SpecExp::Not(expr) => {
                let BoogieExpr(s, t) = self.translate_expr(expr);
                BoogieExpr(
                    format!("Boolean(!(b#Boolean({})))", s),
                    self.require_type(t, &GlobalType::Bool),
                )
            }
            SpecExp::Binop(left, op, right) => {
                let left = self.translate_expr(left);
                let right = self.translate_expr(right);
                self.translate_binop(op, left, right)
            }
            SpecExp::Update(_, _) => self.error(
                "Vector update operator (:=) should not be used here",
                self.error_exp(),
            ),
            SpecExp::Old(expr) => {
                if *self.in_old.borrow() {
                    self.error("cannot nest `old(_)`", self.error_exp())
                } else {
                    *self.in_old.borrow_mut() = true;
                    let BoogieExpr(s, t) = self.translate_expr(expr);
                    *self.in_old.borrow_mut() = false;
                    if self.supports_native_old {
                        BoogieExpr(format!("old({})", s), t)
                    } else {
                        BoogieExpr(s, t)
                    }
                }
            }
            SpecExp::Call(name, exprs) => match name.as_str() {
                "len" => BoogieExpr(
                    format!(
                        "Integer(vlen({}))",
                        exprs.iter().map(|e| self.translate_expr(e).0).join(", ")
                    ),
                    GlobalType::U64,
                ),
                _ => BoogieExpr(
                    format!(
                        "{}({})",
                        name,
                        exprs.iter().map(|e| self.translate_expr(e).0).join(", ")
                    ),
                    UNKNOWN_TYPE,
                ),
            },
        }
    }

    /// Translate expression and expect it to be of given type.
    fn translate_expr_require_type(&self, expr: &SpecExp, expected_type: &GlobalType) -> String {
        let BoogieExpr(s, ty) = self.translate_expr(expr);
        let _ = self.require_type(ty, expected_type);
        s
    }

    /// Translate a dereference.
    fn translate_dref(&self, loc: &StorageLocation) -> BoogieExpr {
        let BoogieExpr(s, t) = self.translate_location_as_reference(loc);
        if let GlobalType::Reference(sig) | GlobalType::MutableReference(sig) = t {
            BoogieExpr(format!("Dereference(__m, {})", s), *sig)
        } else {
            self.error(
                &format!(
                    "expected reference type, found `{}`",
                    boogie_type_value(self.module_env.env, &t)
                ),
                BoogieExpr("<deref>".to_string(), ERROR_TYPE),
            )
        }
    }

    /// Translates a binary operator.
    fn translate_binop(&self, op: &BinOp, left: BoogieExpr, right: BoogieExpr) -> BoogieExpr {
        let operand_type = left.1.clone();
        match op {
            // u64
            BinOp::Add => {
                self.translate_op_helper("+", &operand_type, operand_type.clone(), left, right)
            }
            BinOp::Sub => {
                self.translate_op_helper("-", &operand_type, operand_type.clone(), left, right)
            }
            BinOp::Mul => {
                self.translate_op_helper("*", &operand_type, operand_type.clone(), left, right)
            }
            BinOp::Mod => {
                self.translate_op_helper("mod", &operand_type, operand_type.clone(), left, right)
            }
            BinOp::Div => {
                self.translate_op_helper("div", &operand_type, operand_type.clone(), left, right)
            }
            BinOp::BitAnd => {
                self.translate_op_helper("&", &operand_type, operand_type.clone(), left, right)
            }
            BinOp::BitOr => {
                self.translate_op_helper("|", &operand_type, operand_type.clone(), left, right)
            }
            BinOp::Xor => {
                self.translate_op_helper("^", &operand_type, operand_type.clone(), left, right)
            }
            BinOp::Shl => unimplemented!(),
            BinOp::Shr => unimplemented!(),

            // bool
            BinOp::And => {
                self.translate_op_helper("&&", &GlobalType::Bool, GlobalType::Bool, left, right)
            }
            BinOp::Or => {
                self.translate_op_helper("||", &GlobalType::Bool, GlobalType::Bool, left, right)
            }

            // generic equality
            BinOp::Eq => BoogieExpr(
                if left.1.is_reference() || right.1.is_reference() {
                    format!("Boolean(({}) == ({}))", left.0, right.0)
                } else {
                    format!("Boolean(IsEqual({}, {}))", left.0, right.0)
                },
                GlobalType::Bool,
            ),
            BinOp::Neq => BoogieExpr(
                if left.1.is_reference() || right.1.is_reference() {
                    format!("Boolean(({}) != ({}))", left.0, right.0)
                } else {
                    format!("Boolean(!IsEqual({}, {}))", left.0, right.0)
                },
                GlobalType::Bool,
            ),

            // Ordering
            // TODO: is this defined also for non-integer types?
            BinOp::Lt => {
                self.translate_op_helper("<", &operand_type, GlobalType::Bool, left, right)
            }
            BinOp::Gt => {
                self.translate_op_helper(">", &operand_type, GlobalType::Bool, left, right)
            }
            BinOp::Le => {
                self.translate_op_helper("<=", &operand_type, GlobalType::Bool, left, right)
            }
            BinOp::Ge => {
                self.translate_op_helper(">=", &operand_type, GlobalType::Bool, left, right)
            }
            BinOp::Subrange => {
                self.translate_op_helper("..", &GlobalType::U64, GlobalType::Subrange, left, right)
            }
        }
    }

    /// Helper for translating a binary op with type check of arguments and unwrapping/wrapping
    /// Boogie values.
    fn translate_op_helper(
        &self,
        op: &str,
        expected_operand_type: &GlobalType,
        result_type: GlobalType,
        BoogieExpr(l, lt): BoogieExpr,
        BoogieExpr(r, rt): BoogieExpr,
    ) -> BoogieExpr {
        let _ = self.require_type(rt, expected_operand_type);
        let _ = self.require_type(lt, expected_operand_type);
        let expr = match expected_operand_type {
            GlobalType::U8 | GlobalType::U64 | GlobalType::U128 => {
                format!("i#Integer({}) {} i#Integer({})", l, op, r)
            }
            GlobalType::Bool => format!("b#Boolean({}) {} b#Boolean({})", l, op, r),
            &UNKNOWN_TYPE => self.error(
                "unknown result type of helper function; cannot use in operation",
                "<error>".to_string(),
            ),
            &ERROR_TYPE => "<error>".to_string(),
            _ => panic!("unexpected type: {:?}", expected_operand_type),
        };
        match result_type {
            GlobalType::U8 | GlobalType::U64 | GlobalType::U128 => {
                BoogieExpr(format!("Integer({})", expr), result_type)
            }
            GlobalType::Bool => BoogieExpr(format!("Boolean({})", expr), result_type),
            UNKNOWN_TYPE => self.error(
                "unknown result type of helper function; cannot use in operation",
                BoogieExpr("<error>".to_string(), ERROR_TYPE),
            ),
            ERROR_TYPE => BoogieExpr("<error>".to_string(), ERROR_TYPE),
            _ => panic!("unexpected type"),
        }
    }

    /// Translates a constant.
    fn translate_constant(&self, val: &CopyableVal_) -> BoogieExpr {
        match val {
            CopyableVal_::Address(addr) => BoogieExpr(
                format!("Address({})", self.translate_account_address(addr)),
                GlobalType::Address,
            ),
            CopyableVal_::U8(val) => BoogieExpr(format!("Integer({})", val), GlobalType::U8),
            CopyableVal_::U64(val) => BoogieExpr(format!("Integer({})", val), GlobalType::U64),
            CopyableVal_::U128(val) => BoogieExpr(format!("Integer({})", val), GlobalType::U128),
            CopyableVal_::Bool(val) => BoogieExpr(format!("Boolean({})", val), GlobalType::Bool),
            // TODO: byte arrays
            CopyableVal_::ByteArray(_arr) => BoogieExpr(
                self.error("ByteArray not implemented", "<bytearray>".to_string()),
                GlobalType::ByteArray,
            ),
        }
    }

    /// Translates a location into a boogie expression of type Value.
    ///
    /// The StorageLocation AST can represent different types in the Boogie model,
    /// therefore we need to interpret in context. See also `translate_location_as_reference`.
    fn translate_location_as_value(&self, loc: &StorageLocation) -> BoogieExpr {
        match loc {
            StorageLocation::Formal(name) => self.lookup_symbol(name, *self.in_old.borrow()),
            StorageLocation::Ret(index) => self.lookup_symbol(&format!("__ret{}", index), false),
            StorageLocation::TxnSenderAddress => BoogieExpr(
                "Address(TxnSenderAddress(__txn))".to_string(),
                GlobalType::Address,
            ),
            StorageLocation::Address(addr) => BoogieExpr(
                format!("Address({})", self.translate_account_address(addr)),
                GlobalType::Address,
            ),
            StorageLocation::AccessPath {
                base,
                fields_and_indices,
            } => {
                let BoogieExpr(mut res, mut t) = self.translate_location_as_value(base);
                // If the type of the location is a reference, dref it now.
                if let GlobalType::Reference(vt) | GlobalType::MutableReference(vt) = t {
                    res = format!("Dereference(__m, {})", res);
                    t = *vt;
                }

                for f_or_i in fields_and_indices {
                    match f_or_i {
                        FieldOrIndex::Field(f) => {
                            let (s, tf) = self.translate_field_access(&t, f);
                            res = format!("SelectField({}, {})", res, s);
                            t = tf;
                        }
                        FieldOrIndex::Index(i) => {
                            match i {
                                SpecExp::Update(ind, val) => {
                                    let BoogieExpr(idx, _t_ind) = self.translate_expr(&ind);
                                    let BoogieExpr(val, _t_val) = self.translate_expr(&val);
                                    // TODO: raise an error if t_ind is not an integer type (u8, u64, u128)
                                    // TODO: raise an error if t_val is not an Element type
                                    res = format!(
                                        "update_vector({}, i#Integer({}), {})",
                                        res, idx, val
                                    );
                                    // t does not change.
                                }
                                SpecExp::Binop(left, BinOp::Subrange, right) => {
                                    let BoogieExpr(left, _t_left) = self.translate_expr(&left);
                                    let BoogieExpr(right, _t_right) = self.translate_expr(&right);
                                    // TODO: raise an error if t_left is not an integer type (u8, u64, u128)
                                    // TODO: raise an error if t_right is not an integer type (u8, u64, u128)
                                    res = format!(
                                        "slice_vector({}, i#Integer({}), i#Integer({}))",
                                        res, left, right
                                    );
                                }
                                _ => {
                                    let BoogieExpr(s, _ti) = self.translate_expr(&i);
                                    // TODO: raise an error if _ti is not an integer type (u8, u64, u128)
                                    res = format!("select_vector({}, i#Integer({}))", res, s);
                                    if let GlobalType::Struct(_, _, vt) = &mut t {
                                        // vt should be a vector with a single element.
                                        t = match vt.pop() {
                                            Some(vt) => vt,
                                            None => t, // TODO: raise an error if None
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                BoogieExpr(res, t)
            }
            _ => {
                // Interpret as reference which we immediately dref.
                self.translate_dref(loc)
            }
        }
    }

    /// Translate a location as Reference.
    fn translate_location_as_reference(&self, loc: &StorageLocation) -> BoogieExpr {
        match loc {
            StorageLocation::Formal(name) => {
                let BoogieExpr(s, t) = self.lookup_symbol(name, *self.in_old.borrow());
                if let GlobalType::Reference(d) | GlobalType::MutableReference(d) = t {
                    BoogieExpr(s, GlobalType::Reference(Box::new(*d)))
                } else {
                    self.error(
                        &format!("`{}` expected to be a reference", name),
                        BoogieExpr("<ref>".to_string(), ERROR_TYPE),
                    )
                }
            }
            StorageLocation::GlobalResource {
                type_,
                type_actuals,
                address,
            } => {
                let (s, t) = self.translate_resource_type(type_, type_actuals);
                let BoogieExpr(a, at) = self.translate_location_as_value(address);
                let _ = self.require_type(at, &GlobalType::Address);
                BoogieExpr(
                    format!("GetResourceReference({}, a#Address({}))", s, a),
                    GlobalType::Reference(Box::new(t)),
                )
            }
            StorageLocation::AccessPath {
                base,
                fields_and_indices,
            } => {
                let BoogieExpr(mut res, mut t) = self.translate_location_as_reference(base);
                for f_or_i in fields_and_indices {
                    match f_or_i {
                        FieldOrIndex::Field(f) => {
                            let (s, tf) = self.translate_field_access(&t, f);
                            res = format!("SelectFieldFromRef({}, {})", res, s);
                            t = tf;
                        }
                        FieldOrIndex::Index(_i) => {
                            // TODO: Generate code for vector indexing
                            res = format!("Unimplemented vector indexing. AST = {:?}", f_or_i);
                        }
                    }
                }
                BoogieExpr(res, t)
            }
            _ => self.error(
                &format!("cannot translate as reference: {:?}", loc),
                BoogieExpr("<ref>".to_string(), ERROR_TYPE),
            ),
        }
    }

    /// Checks for an expected type.
    fn require_type(&self, t: GlobalType, expected: &GlobalType) -> GlobalType {
        if t != ERROR_TYPE && t != UNKNOWN_TYPE && t != *expected {
            self.error(
                &format!(
                    "incompatible types: expected `{}`, found `{}`",
                    boogie_type_value(self.module_env.env, expected),
                    boogie_type_value(self.module_env.env, &t)
                ),
                expected.clone(),
            )
        } else {
            t
        }
    }

    /// Translate an account address literal.
    fn translate_account_address(&self, addr: &AccountAddress) -> String {
        format!("{}", BigInt::from_str_radix(&addr.to_string(), 16).unwrap())
    }

    /// Translates a resource name with type actuals
    fn translate_resource_type(
        &self,
        id: &QualifiedStructIdent,
        type_actuals: &[Type],
    ) -> (String, GlobalType) {
        let resource_type = self.module_env.translate_struct_ast_type(
            self.current_loc(),
            id,
            type_actuals,
            &self.type_parameters,
        );
        // Verify the type is actually a resource.
        if let GlobalType::Struct(module_idx, struct_idx, _) = &resource_type {
            let module_env = self.module_env.env.get_module(*module_idx);
            let struct_env = module_env.get_struct(*struct_idx);
            if !struct_env.is_resource() {
                self.error(
                    &format!("type `{}` is not a resource", struct_env.get_name()),
                    (),
                );
            }
        }
        (
            boogie_type_value(self.module_env.env, &resource_type),
            resource_type,
        )
    }

    /// Translates a field name, where `sig` is the type from which the field is selected.
    /// Returns boogie field name and type.
    fn translate_field_access(&self, mut sig: &GlobalType, field: &Field_) -> (String, GlobalType) {
        // If this is a reference, use the underlying type. This function works with both
        // references and non-references.
        let is_ref = if let GlobalType::Reference(s) = sig {
            sig = &*s;
            true
        } else {
            false
        };
        if *sig == ERROR_TYPE {
            return ("<field>".to_string(), ERROR_TYPE);
        }
        if *sig == UNKNOWN_TYPE {
            return self.error(
                "unknown result type of helper function; cannot select field",
                ("<field>".to_string(), ERROR_TYPE),
            );
        }
        if let GlobalType::Struct(module_index, struct_index, _actuals) = sig {
            let module_env = self.module_env.env.get_module(*module_index);
            let struct_env = module_env.get_struct(*struct_index);
            if let Some(field_env) = struct_env.find_field(field.name()) {
                (
                    boogie_field_name(&field_env),
                    if is_ref {
                        GlobalType::Reference(Box::new(field_env.get_type()))
                    } else {
                        field_env.get_type()
                    },
                )
            } else {
                self.error(
                    &format!(
                        "struct `{}` does not have field `{}`",
                        struct_env.get_name(),
                        field
                    ),
                    ("<field>".to_string(), ERROR_TYPE),
                )
            }
        } else {
            self.error(
                &format!(
                    "expected Struct but found `{}`",
                    boogie_type_value(self.module_env.env, sig)
                ),
                ("<field>".to_string(), ERROR_TYPE),
            )
        }
    }
}

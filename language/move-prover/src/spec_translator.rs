// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module translates specification conditions to Boogie code.

use std::{cell::RefCell, rc::Rc};

use spec_lang::{
    env::{FieldId, Loc, ModuleEnv, ModuleId, NodeId, SpecFunId, StructEnv, StructId},
    ty::{PrimitiveType, Type},
};

#[allow(unused_imports)]
use log::{debug, info, warn};

use crate::{
    boogie_helpers::{
        boogie_byte_blob, boogie_caller_resource_memory_domain_name, boogie_declare_global,
        boogie_field_name, boogie_global_declarator, boogie_local_type,
        boogie_resource_memory_name, boogie_saved_resource_memory_name,
        boogie_self_resource_memory_domain_name, boogie_spec_fun_name, boogie_spec_var_name,
        boogie_struct_name, boogie_type_value, boogie_type_value_array,
        boogie_type_value_array_from_strings, boogie_well_formed_expr, WellFormedMode,
    },
    cli::Options,
};
use itertools::Itertools;
use spec_lang::{
    ast::{Condition, ConditionKind, Exp, LocalVarDecl, Operation, Value},
    code_writer::CodeWriter,
    emit, emitln,
    env::{
        ConditionInfo, ConditionTag, GlobalEnv, GlobalId, QualifiedId, SpecVarId,
        ABORTS_IF_IS_PARTIAL_PRAGMA, ABORTS_IF_IS_STRICT_PRAGMA, CONDITION_ABORT_ASSERT_PROP,
        CONDITION_ABORT_ASSUME_PROP, CONDITION_ABSTRACT_PROP, CONDITION_CONCRETE_PROP,
        CONDITION_EXPORT_PROP, CONDITION_INJECTED_PROP, CONDITION_ON_UPDATE_PROP,
        EXPORT_ENSURES_PRAGMA, OPAQUE_PRAGMA, REQUIRES_IF_ABORTS,
    },
    symbol::Symbol,
    ty::TypeDisplayContext,
};
use stackless_bytecode_generator::{
    function_target::FunctionTarget, function_target_pipeline::FunctionTargetsHolder,
    stackless_bytecode::SpecBlockId, usage_analysis::TransitiveUsage,
};
use std::collections::BTreeSet;

const REQUIRES_FAILS_MESSAGE: &str = "precondition does not hold at this call";
const ENSURES_FAILS_MESSAGE: &str = "post-condition does not hold";
const ABORTS_IF_FAILS_MESSAGE: &str = "function does not abort under this condition";
const ABORTS_NOT_COVERED: &str = "abort not covered by any of the `aborts_if` clauses";
const WRONG_ABORTS_CODE: &str = "function does not abort with any of the expected codes";
const SUCCEEDS_IF_FAILS_MESSAGE: &str = "function does not succeed under this condition";
const INVARIANT_FAILS_MESSAGE: &str = "data invariant does not hold";
const GLOBAL_INVARIANT_FAILS_MESSAGE: &str = "global memory invariant does not hold";
const MODIFY_TARGET_FAILS_MESSAGE: &str = "caller does not have permission for this modify target";

pub enum SpecEnv<'env> {
    Module(ModuleEnv<'env>),
    Function(FunctionTarget<'env>),
    Struct(StructEnv<'env>),
}

impl<'env> Into<SpecEnv<'env>> for FunctionTarget<'env> {
    fn into(self) -> SpecEnv<'env> {
        SpecEnv::Function(self)
    }
}

impl<'env> Into<SpecEnv<'env>> for StructEnv<'env> {
    fn into(self) -> SpecEnv<'env> {
        SpecEnv::Struct(self)
    }
}

impl<'env> Into<SpecEnv<'env>> for ModuleEnv<'env> {
    fn into(self) -> SpecEnv<'env> {
        SpecEnv::Module(self)
    }
}

/// Different kinds of function entry points.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FunctionEntryPoint {
    /// Definition without pre/post conditions. Used by the others below.
    /// Functions called from here use the Indirect stub.
    Definition,
    /// Definition without pre/post condition for verification. Differs from the
    /// above in that DirectInter/IntraModule is used for called functions.
    VerificationDefinition,
    /// Inlined or opaque stub for calls to this function from the currently verified function,
    /// if this function is in another module. Preconditions are asserted for such calls as long as
    /// they do not stem from injected conditions (module invariants) or are exported.
    DirectInterModule,
    /// Inlined or opaque stub for calls to this function from the currently verified function,
    /// if this function is in the same module. Asserts all preconditions, explicit or injected.
    DirectIntraModule,
    /// Stub for indirect calls, that is functions which are called from functions which are
    /// not subject of verification.
    Indirect,
    /// Variant used for verification.
    Verification,
}

impl FunctionEntryPoint {
    pub fn suffix(self) -> &'static str {
        use FunctionEntryPoint::*;
        match self {
            Definition => "_$def",
            VerificationDefinition => "_$def_verify",
            DirectInterModule => "_$direct_inter",
            DirectIntraModule => "_$direct_intra",
            Indirect => "",
            Verification => "_$verify",
        }
    }
}

pub struct SpecTranslator<'env> {
    /// The environment in which context translation happens.
    spec_env: SpecEnv<'env>,
    /// Options passed into the translator.
    options: &'env Options,
    /// The code writer.
    writer: &'env CodeWriter,
    /// A reference to the function targets holder.
    targets: &'env FunctionTargetsHolder,
    /// Whether the translation context supports native `old`
    supports_native_old: bool,
    /// Whether we are currently in the context of translating an `old(...)` expression.
    in_old: RefCell<bool>,
    /// Whether we are currently translating an assert or assume specification.
    in_assert_or_assume: RefCell<bool>,
    /// Whether we are currently translating an ensures specification.
    in_ensures: RefCell<bool>,
    /// The current target for invariant fields, a pair of strings for current and old value.
    invariant_target: RefCell<(String, String)>,
    /// Counter for generating fresh variables.
    fresh_var_count: RefCell<usize>,
    /// If we are translating in the context of a type instantiation, the type arguments.
    type_args_opt: Option<Vec<Type>>,
    /// Set of items which have been already traced. This is used to avoid redundant tracing
    /// of expressions whose value has been already tracked.
    traced_items: RefCell<BTreeSet<TraceItem>>,
}

/// A item which is traced for printing in diagnosis. The boolean indicates whether the item is
/// traced inside old context or not.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum TraceItem {
    // Automatically traced items when `options.prover.debug_trace_exp` is on.
    Local(bool, Symbol),
    SpecVar(bool, ModuleId, SpecVarId, Vec<Type>),
    Exp,
    // Explicitly traced item via user level trace function.
    Explicit,
}

impl<'env> SpecTranslator<'env> {
    fn module_env(&'env self) -> &'env ModuleEnv<'env> {
        use SpecEnv::*;
        match &self.spec_env {
            Module(module_env) => module_env,
            Function(func_target) => &func_target.func_env.module_env,
            Struct(struct_env) => &struct_env.module_env,
        }
    }

    fn global_env(&'env self) -> &'env GlobalEnv {
        self.module_env().env
    }

    pub fn function_target(&'env self) -> &'env FunctionTarget<'env> {
        use SpecEnv::*;
        match &self.spec_env {
            Module(_) | Struct(_) => panic!(),
            Function(func_target) => func_target,
        }
    }

    fn struct_env(&'env self) -> &'env StructEnv<'env> {
        use SpecEnv::*;
        match &self.spec_env {
            Module(_) | Function(_) => panic!(),
            Struct(struct_env) => struct_env,
        }
    }
}

// General
// =======

impl<'env> SpecTranslator<'env> {
    /// Creates a translator.
    pub fn new<E>(
        writer: &'env CodeWriter,
        env: E,
        targets: &'env FunctionTargetsHolder,
        options: &'env Options,
        supports_native_old: bool,
    ) -> SpecTranslator<'env>
    where
        E: Into<SpecEnv<'env>>,
    {
        SpecTranslator {
            spec_env: env.into(),
            options,
            writer,
            targets,
            supports_native_old,
            in_old: RefCell::new(false),
            in_assert_or_assume: RefCell::new(false),
            in_ensures: RefCell::new(false),
            invariant_target: RefCell::new(("".to_string(), "".to_string())),
            fresh_var_count: RefCell::new(0),
            type_args_opt: None,
            traced_items: Default::default(),
        }
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
                    &self.global_env(),
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
            if fun.body.is_none() && !fun.uninterpreted {
                // This function is native and expected to be found in the prelude.
                continue;
            }
            if fun.is_move_fun && fun.is_native {
                // This function is a native move function and its spec version is
                // expected to be found in the prelude.
                continue;
            }
            if fun.is_move_fun && !self.module_env().spec_fun_is_used(*id) {
                // This function is a pure move function but is never used,
                // so we don't need to translate it.
                continue;
            }
            if let Type::Tuple(..) | Type::Fun(..) = fun.result_type {
                self.error(&fun.loc, "function or tuple result type not yet supported");
                continue;
            }
            let result_type = boogie_local_type(&fun.result_type);
            let spec_var_params = fun.used_spec_vars.iter().map(
                |QualifiedId {
                     module_id: mid,
                     id: vid,
                 }| {
                    let declaring_module = self.global_env().get_module(*mid);
                    let decl = declaring_module.get_spec_var(*vid);
                    let boogie_name = boogie_spec_var_name(&declaring_module, decl.name);
                    boogie_global_declarator(
                        declaring_module.env,
                        &boogie_name,
                        decl.type_params.len(),
                        &decl.type_,
                    )
                },
            );
            let mem_params = fun.used_memory.iter().map(|memory| {
                format!(
                    "{}: $Memory",
                    boogie_resource_memory_name(self.global_env(), *memory)
                )
            });
            let type_params = fun
                .type_params
                .iter()
                .enumerate()
                .map(|(i, _)| format!("$tv{}: $TypeValue", i));
            let params = fun.params.iter().map(|(name, ty)| {
                format!(
                    "{}: {}",
                    name.display(self.module_env().symbol_pool()),
                    boogie_local_type(ty)
                )
            });
            self.writer.set_location(&fun.loc);
            let boogie_name = boogie_spec_fun_name(&self.module_env(), *id);
            let param_list = mem_params
                .chain(spec_var_params)
                .chain(type_params)
                .chain(params)
                .join(", ");
            emit!(
                self.writer,
                "function {{:inline}} {}({}): {}",
                boogie_name,
                param_list,
                result_type
            );
            if fun.uninterpreted {
                // Uninterpreted function has no body.
                emitln!(self.writer, ";");
                // Emit axiom about return type.
                let call = format!(
                    "{}({})",
                    boogie_name,
                    fun.type_params
                        .iter()
                        .enumerate()
                        .map(|(i, _)| format!("$tv{}", i))
                        .chain(fun.params.iter().map(|(n, _)| {
                            format!("{}", n.display(self.module_env().symbol_pool()))
                        }))
                        .join(", ")
                );
                let type_check = boogie_well_formed_expr(
                    self.global_env(),
                    &call,
                    &fun.result_type,
                    WellFormedMode::WithInvariant,
                );
                if !param_list.is_empty() {
                    emitln!(
                        self.writer,
                        "axiom (forall {} :: {});",
                        param_list,
                        type_check
                    );
                } else {
                    emitln!(self.writer, "axiom {};", type_check);
                }
            } else {
                emitln!(self.writer, " {");
                self.writer.indent();
                self.translate_exp(fun.body.as_ref().unwrap());
                emitln!(self.writer);
                self.writer.unindent();
                emitln!(self.writer, "}");
                emitln!(self.writer);
            }
        }
    }
}

// Pre/Post Conditions
// ===================

/// A data structure which describe the distribution of conditions how they are mapped to a
/// Boogie entrypoint function. The different function entry points have different mappings.
///
#[derive(Debug, Default)]
pub struct ConditionDistribution<'a> {
    // Conditions which are emitted as boogie requires. This list can contain aborts_if
    // conditions (stemming from the [assert] property) which are interpreted to be
    // negated.
    requires: Vec<&'a Condition>,
    // Conditions which are emitted as boogie ensures. If this list contains aborts_if
    // or succeeds_if conditions, they are treated as propagated aborts.
    ensures: Vec<&'a Condition>,
    /// Conditions which are emitted as assumptions on entry. This list can contain aborts_if
    /// conditions (stemming from the [assume] property) which are interpreted to be
    /// negated.
    entry_assumes: Vec<&'a Condition>,
}

impl<'a> ConditionDistribution<'a> {
    /// Adds a condition to this distribution. This determines various properties of the
    /// condition, and then branches over the cross-product of entry point kind and
    /// condition kind, considering the properties, to determine where the condition
    /// will be placed in the distribution.
    ///
    /// This is source of truth in the code defining how conditions are evaluated on various
    /// function entry points, in a relative declarative way.
    fn add(
        &mut self,
        tr: &SpecTranslator<'a>,
        entry_point: FunctionEntryPoint,
        cond: &'a Condition,
    ) {
        let func_target = tr.function_target();
        let env = tr.global_env();

        let opaque = func_target.is_pragma_true(OPAQUE_PRAGMA, || false);
        let export_ensures = func_target.is_pragma_true(EXPORT_ENSURES_PRAGMA, || false);

        let get_prop = |name: &str| {
            env.is_property_true(&cond.properties, name)
                .unwrap_or(false)
        };
        let injected = get_prop(CONDITION_INJECTED_PROP);
        let exported = get_prop(CONDITION_EXPORT_PROP);
        let asserted = get_prop(CONDITION_ABORT_ASSERT_PROP);
        let assumed = get_prop(CONDITION_ABORT_ASSUME_PROP);
        let abstract_ = get_prop(CONDITION_ABSTRACT_PROP);
        let concrete = get_prop(CONDITION_CONCRETE_PROP);

        let external = (!injected || exported) && !concrete;

        match entry_point {
            FunctionEntryPoint::DirectInterModule if opaque => {
                use ConditionKind::*;
                match &cond.kind {
                    Requires if external => self.requires.push(cond),
                    Ensures if external => self.ensures.push(cond),
                    AbortsIf | AbortsWith | SucceedsIf if external && asserted => {
                        self.requires.push(cond)
                    }
                    AbortsIf | AbortsWith | SucceedsIf if external && !assumed => {
                        self.ensures.push(cond)
                    }
                    _ => {}
                }
            }
            FunctionEntryPoint::DirectInterModule if !opaque => {
                use ConditionKind::*;
                match &cond.kind {
                    Requires if external => self.requires.push(cond),
                    Ensures if external || export_ensures => self.ensures.push(cond),
                    AbortsIf | SucceedsIf if external && asserted => self.requires.push(cond),
                    AbortsIf | AbortsWith | SucceedsIf if external && assumed => {
                        self.entry_assumes.push(cond)
                    }
                    AbortsIf | AbortsWith | SucceedsIf if external || export_ensures => {
                        self.ensures.push(cond)
                    }
                    _ => {}
                }
            }
            FunctionEntryPoint::DirectIntraModule if opaque => {
                use ConditionKind::*;
                match &cond.kind {
                    Requires | RequiresModule => self.requires.push(cond),
                    Ensures => self.ensures.push(cond),
                    AbortsIf | AbortsWith | SucceedsIf if asserted => self.requires.push(cond),
                    AbortsIf | AbortsWith | SucceedsIf if !assumed => self.ensures.push(cond),
                    _ => {}
                }
            }
            FunctionEntryPoint::DirectIntraModule if !opaque => {
                use ConditionKind::*;
                match &cond.kind {
                    Requires | RequiresModule => self.requires.push(cond),
                    Ensures if export_ensures => self.ensures.push(cond),
                    AbortsIf | AbortsWith | SucceedsIf if asserted => self.requires.push(cond),
                    AbortsIf | AbortsWith | SucceedsIf if assumed => self.entry_assumes.push(cond),
                    AbortsIf | AbortsWith | SucceedsIf if export_ensures => self.ensures.push(cond),
                    _ => {}
                }
            }
            FunctionEntryPoint::Indirect if opaque => {
                use ConditionKind::*;
                match &cond.kind {
                    Ensures if external || export_ensures => self.ensures.push(cond),
                    AbortsIf | AbortsWith | SucceedsIf
                        if !assumed && !asserted && (external || export_ensures) =>
                    {
                        self.ensures.push(cond)
                    }
                    _ => {}
                }
            }
            FunctionEntryPoint::Indirect if !opaque => {
                use ConditionKind::*;
                match &cond.kind {
                    Ensures if export_ensures => self.ensures.push(cond),
                    AbortsIf | AbortsWith | SucceedsIf if assumed || asserted => {
                        self.entry_assumes.push(cond)
                    }
                    AbortsIf | AbortsWith | SucceedsIf if export_ensures => self.ensures.push(cond),
                    _ => {}
                }
            }
            FunctionEntryPoint::Verification => {
                use ConditionKind::*;
                if !abstract_ {
                    match &cond.kind {
                        Requires | RequiresModule => self.entry_assumes.push(cond),
                        Ensures => self.ensures.push(cond),
                        AbortsIf | AbortsWith | SucceedsIf => self.ensures.push(cond),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

impl<'env> SpecTranslator<'env> {
    // Generate boogie for asserts/assumes inside function bodies
    pub fn translate_conditions_inside_impl(&self, block_id: SpecBlockId) {
        let func_target = self.function_target();
        let spec = func_target.get_spec_on_impl(block_id);
        if !spec.conditions.is_empty() {
            *self.in_assert_or_assume.borrow_mut() = true;
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
                emit!(self.writer, "b#$Boolean(");
                self.translate_exp(&cond.exp);
                emit!(self.writer, ");")
            });
            *self.in_assert_or_assume.borrow_mut() = false;
            emitln!(self.writer);
        }
    }

    /// Generates boogie for pre/post conditions of a procedure. Conditions are generated depending
    /// on the kind of entry point. Conditions which are assumptions are not translated
    /// by this function, but by calling `translate_entry_point_assumptions` with
    /// the data structure returned from here. This allows splitting the translation in
    /// some outputs attached to the signature and some to the function body.
    pub fn translate_conditions(
        &self,
        for_entry_point: FunctionEntryPoint,
    ) -> ConditionDistribution<'_> {
        use FunctionEntryPoint::*;
        assert!(!matches!(
            for_entry_point,
            Definition | VerificationDefinition
        ));
        let func_target = self.function_target();
        let opaque = func_target.is_pragma_true(OPAQUE_PRAGMA, || false);
        let spec = func_target.get_spec();

        // Create distribution of the conditions of this entry point.
        let mut distribution = ConditionDistribution::default();
        for cond in &spec.conditions {
            distribution.add(self, for_entry_point, cond);
        }

        // emit preconditions for modifies
        // TODO: implement optimization to make sure that modifies checking is done once and not repeatedly
        for (ty, targets) in func_target.get_modify_targets() {
            let ty_name = boogie_caller_resource_memory_domain_name(func_target.global_env(), *ty);
            for target in targets {
                let loc = self.module_env().get_node_loc(target.node_id());
                self.writer.set_location(&loc);
                self.set_condition_info(&loc, ConditionTag::Requires, MODIFY_TARGET_FAILS_MESSAGE);
                emit!(self.writer, "requires ");
                let node_id = target.node_id();
                let args = target.call_args();
                let rty = &self.module_env().get_node_instantiation(node_id)[0];
                let (_, _, targs) = rty.require_struct();
                let env = self.global_env();
                let type_args = boogie_type_value_array(env, targs);
                emit!(self.writer, "{}[{}, a#$Address(", ty_name, type_args);
                self.translate_exp(&args[0]);
                emit!(self.writer, ")];");
                emitln!(self.writer);
            }
        }

        // Helper to filter conditions.
        let kind_filter = |k: ConditionKind| move |c: &&&Condition| c.kind == k;

        // Get all aborts_if conditions.
        let aborts_if = distribution
            .ensures
            .iter()
            .filter(kind_filter(ConditionKind::AbortsIf))
            .copied()
            .collect_vec();

        // Generate requires.
        if !distribution.requires.is_empty() {
            self.emit_requires(false, &aborts_if, &distribution.requires);
            emitln!(self.writer);
        }

        // Generate aborts_if. Logically, if we have abort conditions P1..Pn, we have
        // (P1 || .. || Pn) <==> abort_flag. However, we generate different code to get
        // better error positions. We also need to respect the pragma `aborts_if_is_partial`
        // which changes the iff above into an implies.
        let aborts_with = distribution
            .ensures
            .iter()
            .filter(kind_filter(ConditionKind::AbortsWith))
            .copied()
            .collect_vec();
        let aborts_if_is_partial =
            func_target.is_pragma_true(ABORTS_IF_IS_PARTIAL_PRAGMA, || false);
        if aborts_if.is_empty() && aborts_with.is_empty() {
            if !aborts_if_is_partial
                && func_target.is_pragma_true(ABORTS_IF_IS_STRICT_PRAGMA, || false)
                && for_entry_point == Verification
            {
                // No user provided aborts_if and pragma is set for handling this
                // as s.t. the function must never abort.
                self.writer.set_location(&func_target.get_loc());
                self.set_condition_info(
                    &func_target.get_loc(),
                    ConditionTag::Ensures,
                    ABORTS_NOT_COVERED,
                );
                emitln!(self.writer, "ensures !$abort_flag;");
            }
        } else {
            // Emit `ensures P1 ==> abort_flag; ... ensures PN ==> abort_flag;`. This gives us
            // good error positions which Pi is expected to cause failure but doesn't. (Boogie
            // reports positions only back per entire ensures, not individual sub-expression.)
            for c in &aborts_if {
                self.writer.set_location(&c.loc);
                self.set_condition_info(&c.loc, ConditionTag::Ensures, ABORTS_IF_FAILS_MESSAGE);
                let (exp, _) = c.exp.extract_cond_and_aborts_code();
                emit!(self.writer, "ensures b#$Boolean(old(");
                self.translate_exp(exp);
                emitln!(self.writer, ")) ==> $abort_flag;");
            }

            // If aborts_if is configured to be total,
            // emit `ensures abort_flag => (P1 || .. || Pn)`. If the error
            // is reported on this condition, we catch the case where the function aborts but no
            // conditions covers it. We use as a position for the ensures the function itself,
            // because reporting on (non-covering) aborts_if conditions is misleading.
            if !aborts_if_is_partial && !aborts_if.is_empty() {
                self.writer.set_location(&func_target.get_loc());
                self.set_condition_info(
                    &func_target.get_loc(),
                    ConditionTag::Ensures,
                    ABORTS_NOT_COVERED,
                );
                emit!(self.writer, "ensures $abort_flag ==> (");
                self.translate_seq(aborts_if.iter().copied(), "\n    || ", |c: &Condition| {
                    let (exp, _) = c.exp.extract_cond_and_aborts_code();
                    emit!(self.writer, "b#$Boolean(old(");
                    self.translate_exp(exp);
                    emit!(self.writer, "))")
                });
                emitln!(self.writer, ");");
            }

            // Create the abort code condition. Let (Pi, Ci?), i in 1..n, be the aborts_if
            // conditions with optional code Ci. Let UCj, j in 1..m, be the abort codes which have
            // been specified via an aborts_with. We generate:
            //
            //   ensures $aborts_flag ==>
            //      Pi [ && $aborts_code == Ci ]
            //    | ..
            //    | Pn [ && $aborts_code == Cn ]
            //    | $aborts_code == UCi | .. | $aborts_code == UCm
            //
            // Notice that an aborts_if without an aborts code will allow an arbitrary code.
            // The codes from the aborts_with only apply if none of the aborts
            // conditions are true. This seems to be the most consistent encoding, but this
            // aspect may need to be revisited.
            let aborts_if_has_codes = aborts_if
                .iter()
                .any(|c| c.exp.extract_cond_and_aborts_code().1.is_some());
            if aborts_if_has_codes || !aborts_with.is_empty() {
                // TODO(wrwg): we need a location for the spec block of this function.
                //   The conditions don't give usa a good indication because via
                //   schemas, they can come from anywhere. For now we use the
                //   function start location to not clash with the full function location
                //   used in the above ensures.
                let abort_code_loc = func_target.get_loc().at_start();
                if aborts_if_is_partial && aborts_with.is_empty() {
                    // If the aborts spec is partial but there are no aborts_with, the
                    // aborts code specification is meaningless, because the unspecified
                    // aborts conditions can produce arbitrary codes. We better report an
                    // error in this case instead of silently ignoring the aborts code spec.
                    self.global_env().error(
                        &abort_code_loc,
                        "`aborts_if_is_partial` is set but \
                            there are no abort codes specified with `aborts_with` to cover \
                            the codes of the unspecified abort cases",
                    );
                }
                self.writer.set_location(&abort_code_loc);
                self.set_condition_info(&abort_code_loc, ConditionTag::Ensures, WRONG_ABORTS_CODE);
                emit!(self.writer, "ensures $abort_flag ==> (");
                self.translate_seq(aborts_if.iter().copied(), "\n    ||", |c: &Condition| {
                    let (exp, code_opt) = c.exp.extract_cond_and_aborts_code();
                    emit!(self.writer, "(b#$Boolean(old(");
                    self.translate_exp(exp);
                    if let Some(code) = code_opt {
                        emit!(self.writer, ")) &&\n       $abort_code == i#$Integer(");
                        self.translate_exp(code);
                        emit!(self.writer, "))");
                    } else {
                        emit!(self.writer, ")))");
                    }
                });
                if !aborts_if.is_empty() && !aborts_with.is_empty() {
                    emit!(self.writer, "\n    ||");
                }
                self.translate_seq(aborts_with.iter().copied(), "\n    ||", |c: &Condition| {
                    let codes = c.exp.extract_abort_codes();
                    self.translate_seq(codes.iter(), " || ", |code| {
                        emit!(self.writer, "$abort_code == i#$Integer(");
                        self.translate_exp(code);
                        emit!(self.writer, ")");
                    });
                });
                emitln!(self.writer, ");");
            }
        }

        // Generate succeeds_if.
        // Emits `ensures S1 ==> !abort_flag; ... ensures Sn ==> !abort_flag;`.
        let succeeds_if = distribution
            .ensures
            .iter()
            .filter(kind_filter(ConditionKind::SucceedsIf))
            .copied()
            .collect_vec();
        for c in succeeds_if {
            self.writer.set_location(&c.loc);
            self.set_condition_info(&c.loc, ConditionTag::Ensures, SUCCEEDS_IF_FAILS_MESSAGE);
            emit!(self.writer, "ensures b#$Boolean(old(");
            self.translate_exp(&c.exp);
            emitln!(self.writer, ")) ==> !$abort_flag;")
        }

        // Generate ensures
        let ensures = distribution
            .ensures
            .iter()
            .filter(kind_filter(ConditionKind::Ensures))
            .copied()
            .collect_vec();
        if !ensures.is_empty() {
            *self.in_ensures.borrow_mut() = true;
            self.translate_seq(ensures.iter(), "\n", |cond| {
                self.writer.set_location(&cond.loc);
                self.set_condition_info(&cond.loc, ConditionTag::Ensures, ENSURES_FAILS_MESSAGE);
                emit!(self.writer, "ensures !$abort_flag ==> (b#$Boolean(");
                self.translate_exp(&cond.exp);
                emit!(self.writer, "));")
            });
            *self.in_ensures.borrow_mut() = false;
            emitln!(self.writer);
        }

        // If this is an opaque function, also generate ensures for type assumptions.
        if opaque
            && matches!(
                for_entry_point,
                FunctionEntryPoint::Indirect
                    | FunctionEntryPoint::DirectIntraModule
                    | FunctionEntryPoint::DirectInterModule
            )
        {
            for (i, ty) in func_target.get_return_types().iter().enumerate() {
                let result_name = format!("$ret{}", i);
                let check = boogie_well_formed_expr(
                    self.global_env(),
                    &result_name,
                    ty,
                    WellFormedMode::Default,
                );
                if !check.is_empty() {
                    emitln!(self.writer, "ensures {};", check)
                }
            }
        }
        distribution
    }

    /// Emit either assume or requires for preconditions. For a regular requires,
    /// 'or' them with the aborts conditions. If the condition is an aborts_if or a succeeds_if,
    /// treat it as an [assert] or an [assume] which has been lifted as a precondition.
    fn emit_requires(&self, assume: bool, aborts_if: &[&Condition], requires: &[&Condition]) {
        let func_target = self.function_target();
        self.translate_seq(requires.iter().copied(), "\n", |cond: &Condition| {
            self.writer.set_location(&cond.loc);
            self.set_condition_info(&cond.loc, ConditionTag::Requires, REQUIRES_FAILS_MESSAGE);
            emit!(
                self.writer,
                "{} {}b#$Boolean(",
                if assume { "assume" } else { "requires" },
                if cond.kind == ConditionKind::AbortsIf {
                    // If an aborts_if appears in precondition position, it must be negated.
                    "!"
                } else {
                    ""
                },
            );
            let (exp, _) = cond.exp.extract_cond_and_aborts_code();
            self.translate_exp(exp);
            emit!(self.writer, ")");
            if !matches!(
                cond.kind,
                ConditionKind::AbortsIf | ConditionKind::SucceedsIf
            ) && func_target.is_pragma_true(REQUIRES_IF_ABORTS, || false)
            {
                for aborts in aborts_if {
                    let (exp, _) = aborts.exp.extract_cond_and_aborts_code();
                    emit!(self.writer, "\n    || b#$Boolean(");
                    self.translate_exp(exp);
                    emit!(self.writer, ")")
                }
            }
            emit!(self.writer, ";")
        });
    }

    /// Generates assumptions to make at function entry points.
    pub fn translate_entry_point_assumptions(
        &self,
        for_entry_point: FunctionEntryPoint,
        distribution: ConditionDistribution,
    ) {
        use FunctionEntryPoint::*;
        assert!(!matches!(
            for_entry_point,
            Definition | VerificationDefinition
        ));
        let func_target = self.function_target();
        if for_entry_point == Verification {
            // Generate assumes for top-level verification entry

            // (a) init prelude specific stuff.
            emitln!(self.writer, "call $InitVerification();");

            // (b) assume reference parameters to be based on the Param(i) Location, ensuring
            // they are disjoint from all other references. This prevents aliasing and is justified as
            // follows:
            // - for mutual references, by their exclusive access in Move.
            // - for immutable references, by that mutation is not possible, and they are equivalent
            //   to some given but arbitrary value.
            for i in 0..func_target.get_parameter_count() {
                let ty = func_target.get_local_type(i);
                if ty.is_reference() {
                    let name = func_target
                        .symbol_pool()
                        .string(func_target.get_local_name(i));
                    emitln!(
                        self.writer,
                        "assume l#$Mutation({}) == $Param({});",
                        name,
                        i
                    );
                    emitln!(self.writer, "assume size#Path(p#$Mutation({})) == 0;", name);
                }
            }

            // (c) assume invariants.
            self.assume_invariants_for_verify();
        }
        if !distribution.entry_assumes.is_empty() {
            // assume preconditions which have been converted into assumptions.
            // TODO(wrwg): investigate soundness of emitting those assumptions without
            // or-ing with aborts conditions.
            self.emit_requires(true, &[], &distribution.entry_assumes);
            emitln!(self.writer);
        }
    }

    pub fn assume_invariants_for_verify(&self) {
        let env = self.global_env();
        let func_target = self.function_target();
        let usage = TransitiveUsage::default();
        let used_mem = usage.get_used_memory(
            self.global_env(),
            self.targets,
            func_target
                .module_env()
                .get_id()
                .qualified(func_target.get_id()),
        );
        let mut invariants: BTreeSet<GlobalId> = BTreeSet::new();
        for mem in used_mem {
            // Emit type well-formedness invariant.
            let struct_env = env.get_module(mem.module_id).into_struct(mem.id);
            emit!(self.writer, "assume ");
            let memory_name = boogie_resource_memory_name(func_target.global_env(), mem);
            emit!(self.writer, "(forall $inv_addr: int");
            let mut type_args = vec![];
            for i in 0..struct_env.get_type_parameters().len() {
                emit!(self.writer, ", $inv_tv{}: $TypeValue", i);
                type_args.push(format!("$inv_tv{}", i));
            }
            let get_resource = format!(
                "contents#$Memory({})[{}, $inv_addr]",
                memory_name,
                boogie_type_value_array_from_strings(&type_args)
            );
            emitln!(self.writer, " :: {{{}}}", get_resource);
            self.writer.indent();
            emitln!(
                self.writer,
                "{}_is_well_formed({})",
                boogie_struct_name(&struct_env),
                get_resource,
            );
            self.writer.unindent();
            emitln!(self.writer, ");");

            // Collect global invariants.
            invariants.extend(env.get_global_invariants_for_memory(mem).into_iter());
        }

        // Now emit assume of global invariants which touch the used memory.
        self.emit_global_invariants(
            true,
            invariants
                .into_iter()
                .filter(|id| {
                    !env.get_global_invariant(*id)
                        .and_then(|inv| {
                            env.is_property_true(&inv.properties, CONDITION_ON_UPDATE_PROP)
                        })
                        .unwrap_or(false)
                })
                .collect_vec(),
        )
    }

    /// Generate initialization of modifies permissions of a function
    pub fn emit_modifies_initialization(&self) {
        let func_target = self.function_target();
        for (ty, targets) in func_target.get_modify_targets() {
            emit!(
                self.writer,
                "{} := {}",
                boogie_self_resource_memory_domain_name(func_target.global_env(), *ty),
                "$ConstMemoryDomain(false)"
            );
            for target in targets {
                let node_id = target.node_id();
                let args = target.call_args();
                let rty = &self.module_env().get_node_instantiation(node_id)[0];
                let (_, _, targs) = rty.require_struct();
                let env = func_target.global_env();
                let type_args = boogie_type_value_array(env, targs);
                emit!(self.writer, "[{}, a#$Address(", type_args);
                self.translate_exp(&args[0]);
                emit!(self.writer, ") := true]");
            }
            emitln!(self.writer, ";");
        }
    }

    /// Translate modify targets to constrain havocs at opaque calls
    pub fn translate_modify_targets(&self) {
        let func_target = self.function_target();
        let usage = TransitiveUsage::default();
        let modified_types = usage.get_modified_memory(
            self.global_env(),
            self.targets,
            func_target
                .module_env()
                .get_id()
                .qualified(func_target.get_id()),
        );
        for type_name in modified_types {
            let memory_name = self.get_memory_name(type_name);
            emitln!(self.writer, "modifies {};", memory_name);
            let type_name_targets = func_target.get_modify_targets_for_type(&type_name);
            if let Some(type_name_targets) = type_name_targets {
                let generate_ensures = |bpl_map| {
                    emit!(self.writer, "ensures {} == old({})", bpl_map, bpl_map);
                    for target in type_name_targets {
                        let node_id = target.node_id();
                        let args = target.call_args();
                        let rty = &self.module_env().get_node_instantiation(node_id)[0];
                        let (_, _, targs) = rty.require_struct();
                        let env = self.global_env();
                        let type_args = boogie_type_value_array(env, targs);
                        emit!(self.writer, "[{}, a#$Address(", type_args);
                        self.translate_exp(&args[0]);
                        emit!(self.writer, ") := {}", bpl_map);
                        emit!(self.writer, "[{}, a#$Address(", type_args);
                        self.translate_exp(&args[0]);
                        emit!(self.writer, ")]]");
                    }
                    emitln!(self.writer, ";");
                };
                generate_ensures(format!("contents#$Memory({})", memory_name));
                generate_ensures(format!("domain#$Memory({})", memory_name));
            }
        }
    }

    /// Sets info for verification condition so it can be later retrieved by the boogie wrapper.
    fn set_condition_info(&self, loc: &Loc, tag: ConditionTag, message: &str) {
        self.global_env()
            .set_condition_info(loc.clone(), tag, ConditionInfo::for_message(message));
    }
}

/// Invariants
/// ==========

impl<'env> SpecTranslator<'env> {
    /// Emits functions and procedures needed for invariants.
    pub fn translate_invariant_functions(&self) {
        self.translate_assume_well_formed();
        self.translate_before_update_invariant();
        self.translate_after_update_invariant();
    }

    /// Generates functions which assumes the struct to be well-formed. The first function
    /// only checks type assumptions and is called to ensure well-formedness while the struct is
    /// mutated. The second function checks both types and data invariants and is used while
    /// the struct is not mutated.
    fn translate_assume_well_formed(&self) {
        let struct_env = self.struct_env();
        let emit_field_checks = |mode: WellFormedMode| {
            emitln!(self.writer, "$Vector_is_well_formed($this)");
            emitln!(
                self.writer,
                "&& $vlen($this) == {}",
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
            "function {{:inline}} {}_is_well_formed_types($this: $Value): bool {{",
            boogie_struct_name(struct_env),
        );
        self.writer.indent();
        emit_field_checks(WellFormedMode::WithoutInvariant);
        self.writer.unindent();
        emitln!(self.writer, "}");

        emitln!(
            self.writer,
            "function {{:inline}} {}_is_well_formed($this: $Value): bool {{",
            boogie_struct_name(struct_env),
        );
        self.writer.indent();
        emit_field_checks(WellFormedMode::WithInvariant);
        for inv in struct_env.get_spec().filter_kind(ConditionKind::Invariant) {
            emit!(self.writer, "  && b#$Boolean(");
            self.with_invariant_target("$this", "", || self.translate_exp(&inv.exp));
            emitln!(self.writer, ")");
        }
        self.writer.unindent();
        emitln!(self.writer, "}");
        emitln!(self.writer);
    }

    /// Determines whether a before-update invariant is generated for this struct.
    pub fn has_before_update_invariant(struct_env: &StructEnv<'_>) -> bool {
        use ConditionKind::*;
        struct_env.get_spec().any(|c| matches!(c.kind, VarUpdate(..)|VarUnpack(..)|Invariant))
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
            .map(|i| format!("$tv{}: $TypeValue", i))
            .collect_vec()
    }

    /// Generates a procedure which asserts the before-update invariants of the struct.
    pub fn translate_before_update_invariant(&self) {
        let struct_env = self.struct_env();
        if !Self::has_before_update_invariant(struct_env) {
            return;
        }
        emitln!(
            self.writer,
            "procedure {{:inline 1}} {}_before_update_inv({}) {{",
            boogie_struct_name(struct_env),
            Self::translate_type_parameters(struct_env)
                .into_iter()
                .chain(vec!["$before: $Value".to_string()])
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

        // Emit data invariants for this struct.
        let spec = struct_env.get_spec();
        self.emit_invariants_assume_or_assert(
            "$before",
            "",
            true,
            spec.filter_kind(ConditionKind::Invariant),
        );

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
    pub fn translate_after_update_invariant(&self) {
        let struct_env = self.struct_env();
        if !Self::has_after_update_invariant(struct_env) {
            return;
        }
        emitln!(
            self.writer,
            "procedure {{:inline 1}} {}_after_update_inv({}) {{",
            boogie_struct_name(struct_env),
            Self::translate_type_parameters(struct_env)
                .into_iter()
                .chain(vec!["$after: $Value".to_string()])
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
                        .chain(vec![format!("$SelectField($after, {})", field_name)].into_iter())
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

    pub fn emit_pack_invariants(&self, target: &str) {
        let struct_env = self.struct_env();
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
    pub fn emit_unpack_invariants(&self, target: &str) {
        let struct_env = self.struct_env();
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
                    emit!(self.writer, "assume b#$Boolean(");
                } else {
                    self.set_condition_info(
                        &inv.loc,
                        ConditionTag::Ensures,
                        INVARIANT_FAILS_MESSAGE,
                    );
                    emit!(self.writer, "assert b#$Boolean(");
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
                let module_env = self.global_env().get_module(*module_id);
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

    pub fn save_memory_for_update_invariants(&self, memory: QualifiedId<StructId>) {
        let env = self.global_env();
        let mut memory_to_save = BTreeSet::new();
        // Collect all update invariants.
        for id in env.get_global_invariants_for_memory(memory) {
            let inv = env.get_global_invariant(id).unwrap();
            if inv.kind == ConditionKind::InvariantUpdate {
                memory_to_save.extend(inv.mem_usage.iter());
            }
        }
        // Save their memory.
        for used_memory in memory_to_save {
            let name = boogie_resource_memory_name(env, used_memory);
            let saved_name = boogie_saved_resource_memory_name(env, used_memory);
            emitln!(self.writer, "{} := {};", saved_name, name);
        }
    }

    pub fn emit_on_update_global_invariant_assumes(&self, memory: QualifiedId<StructId>) {
        let env = self.global_env();
        self.emit_global_invariants(
            true,
            env.get_global_invariants_for_memory(memory)
                .into_iter()
                .filter(|id| {
                    env.get_global_invariant(*id)
                        .and_then(|inv| {
                            env.is_property_true(&inv.properties, CONDITION_ON_UPDATE_PROP)
                        })
                        .unwrap_or(false)
                })
                .collect_vec(),
        );
    }

    pub fn emit_global_invariants_for_memory(&self, assume: bool, memory: QualifiedId<StructId>) {
        let env = self.global_env();
        self.emit_global_invariants(assume, env.get_global_invariants_for_memory(memory))
    }

    fn emit_global_invariants(&self, assume: bool, invariants: Vec<GlobalId>) {
        let env = self.global_env();
        for inv in invariants
            .into_iter()
            .map(|id| env.get_global_invariant(id).unwrap())
        {
            self.writer.set_location(&inv.loc);
            if assume && inv.kind == ConditionKind::InvariantUpdate {
                // Update invariants are never assumed.
                continue;
            }
            if assume {
                emit!(self.writer, "assume b#$Boolean(");
            } else {
                self.set_condition_info(
                    &inv.loc,
                    ConditionTag::Ensures,
                    GLOBAL_INVARIANT_FAILS_MESSAGE,
                );
                emit!(self.writer, "assert b#$Boolean(");
            }
            // We need to use a translator for the module which declared the invariant, which is not
            // necessarily the one which is emitting the invariants, such that node_id annotations
            // of the expression are right.
            let translator_for_exp = SpecTranslator::new(
                self.writer,
                env.get_module(inv.declaring_module),
                self.targets,
                self.options,
                self.supports_native_old,
            );
            translator_for_exp.translate_exp(&inv.cond);
            emitln!(self.writer, ");")
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
            boogie_type_value(self.global_env(), &ty.instantiate(ty_args))
        } else {
            boogie_type_value(self.global_env(), ty)
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
                let instantiation = &self.module_env().get_node_instantiation(*node_id);
                self.trace_value(
                    *node_id,
                    TraceItem::SpecVar(
                        *self.in_old.borrow(),
                        *module_id,
                        *var_id,
                        instantiation.clone(),
                    ),
                    || {
                        self.set_writer_location(*node_id);
                        let module_env = self.global_env().get_module(*module_id);
                        let spec_var = module_env.get_spec_var(*var_id);
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
                    },
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
                emit!(self.writer, "if (b#$Boolean(");
                self.translate_exp(cond);
                emit!(self.writer, ")) then ");
                self.translate_exp_parenthesised(on_true);
                emit!(self.writer, " else ");
                self.translate_exp_parenthesised(on_false);
            }
            Exp::Error(_) => panic!("unexpected error expression"),
        }
    }

    fn trace_value<F>(&self, node_id: NodeId, item: TraceItem, f: F)
    where
        F: Fn(),
    {
        let go = if item == TraceItem::Explicit {
            // User called TRACE function, always do it.
            true
        } else if self.options.prover.debug_trace {
            // Option for automatic tracing has been enabled
            if item == TraceItem::Exp {
                // Some arbitrary exp
                true
            } else {
                // Some named item, like a spec var or local. Only trace again if it has not
                // been done yet. This avoids redundant noise.
                self.traced_items.borrow_mut().insert(item)
            }
        } else {
            false
        };
        if go {
            let module_env = self.module_env();
            emit!(
                self.writer,
                "$DebugTrackExp({}, {}, ",
                module_env.get_id().to_usize(),
                node_id.as_usize(),
            );
            f();
            emit!(self.writer, ")");
        } else {
            f();
        }
    }

    fn translate_exp_parenthesised(&self, exp: &Exp) {
        emit!(self.writer, "(");
        self.translate_exp(exp);
        emit!(self.writer, ")");
    }

    fn translate_value(&self, _node_id: NodeId, val: &Value) {
        match val {
            Value::Address(addr) => emit!(self.writer, "$Address({})", addr),
            Value::Number(val) => emit!(self.writer, "$Integer({})", val),
            Value::Bool(val) => emit!(self.writer, "$Boolean({})", val),
            Value::ByteArray(val) => emit!(self.writer, &boogie_byte_blob(self.options, val)),
        }
    }

    fn translate_local_var(&self, node_id: NodeId, name: Symbol) {
        self.trace_value(
            node_id,
            TraceItem::Local(*self.in_old.borrow(), name),
            || {
                let mut ty = &self.module_env().get_node_type(node_id);
                let mut var_name = self.module_env().symbol_pool().string(name);
                if let SpecEnv::Function(func_target) = &self.spec_env {
                    // overwrite ty and var_name if func_target provides a binding for name
                    // TODO(wrwg): this interferes with name scoping rules in Move/spec lang
                    // and needs to be fixed.
                    if let Some(local_index) = func_target.get_local_index(name) {
                        if *self.in_assert_or_assume.borrow() {
                            if let Some(proxy_index) = if ty.is_reference() {
                                func_target.get_ref_proxy_index(*local_index)
                            } else {
                                func_target.get_proxy_index(*local_index)
                            } {
                                var_name = func_target
                                    .symbol_pool()
                                    .string(func_target.get_local_name(*proxy_index));
                                ty = func_target.get_local_type(*proxy_index);
                            } else {
                                ty = func_target.get_local_type(*local_index);
                            }
                        } else if *self.in_ensures.borrow() && !*self.in_old.borrow() {
                            if let Some(return_index) = func_target.get_return_index(*local_index) {
                                var_name = Rc::new(format!("$ret{}", return_index));
                                ty = func_target.get_return_type(*return_index);
                            } else {
                                ty = func_target.get_local_type(*local_index);
                            }
                        } else {
                            ty = func_target.get_local_type(*local_index);
                        }
                    }
                };
                self.auto_dref(ty, || {
                    emit!(self.writer, var_name.as_ref());
                });
            },
        );
    }

    fn auto_dref<F>(&self, ty: &Type, f: F)
    where
        F: Fn(),
    {
        if ty.is_reference() {
            // Automatically dereference
            emit!(self.writer, "$Dereference(");
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
            Operation::ModifyTargets | Operation::AbortCodes | Operation::CondWithAbortCode => {
                panic!("unexpected virtual operator")
            }
            Operation::Function(module_id, fun_id) => {
                self.translate_spec_fun_call(node_id, *module_id, *fun_id, args)
            }
            Operation::Pack(..) => self.translate_pack(args),
            Operation::Tuple => self.error(&loc, "Tuple not yet supported"),
            Operation::Select(module_id, struct_id, field_id) => {
                self.translate_select(*module_id, *struct_id, *field_id, args)
            }
            Operation::Local(sym) => {
                self.translate_local_var(node_id, *sym);
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
            Operation::Shl => self.translate_primitive_call("$shl", args),
            Operation::Shr => self.translate_primitive_call("$shr", args),
            Operation::Implies => self.translate_logical_op("==>", args),
            Operation::And => self.translate_logical_op("&&", args),
            Operation::Or => self.translate_logical_op("||", args),
            Operation::Lt => self.translate_rel_op("<", args),
            Operation::Le => self.translate_rel_op("<=", args),
            Operation::Gt => self.translate_rel_op(">", args),
            Operation::Ge => self.translate_rel_op(">=", args),
            Operation::Eq => self.translate_eq_neq("$IsEqual", args),
            Operation::Neq => self.translate_eq_neq("!$IsEqual", args),

            // Unary operators
            Operation::Not => self.translate_logical_unary_op("!", args),

            // Builtin functions
            Operation::Global => self.translate_resource_access(node_id, args),
            Operation::Exists => self.translate_resource_exists(node_id, args),
            Operation::Len => self.translate_primitive_call("$vlen_value", args),
            Operation::All => self.translate_all_or_exists(&loc, true, args),
            Operation::Any => self.translate_all_or_exists(&loc, false, args),
            Operation::TypeValue => self.translate_type_value(node_id),
            Operation::TypeDomain => self.error(
                &loc,
                "the `domain<T>()` function can only be used as the 1st \
                 parameter of `all` or `any`",
            ),
            Operation::Update => self.translate_primitive_call("$update_vector_by_value", args),
            Operation::Concat => self.translate_primitive_call("$append_vector", args),
            Operation::Empty => self.translate_primitive_call("$mk_vector", args),
            Operation::Single => self.translate_primitive_call("$single_vector", args),
            Operation::Old => self.translate_old(args),
            Operation::Trace => self.trace_value(node_id, TraceItem::Explicit, || {
                self.translate_exp(&args[0])
            }),
            Operation::MaxU8 => emit!(self.writer, "$Integer($MAX_U8)"),
            Operation::MaxU64 => emit!(self.writer, "$Integer($MAX_U64)"),
            Operation::MaxU128 => emit!(self.writer, "$Integer($MAX_U128)"),
            Operation::NoOp => { /* do nothing. */ }
        }
    }

    fn translate_pack(&self, args: &[Exp]) {
        emit!(
            self.writer,
            "$Vector({}$EmptyValueArray()",
            "$ExtendValueArray(".repeat(args.len())
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
        let module_env = self.global_env().get_module(module_id);
        let fun_decl = module_env.get_spec_fun(fun_id);
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
        for memory in &fun_decl.used_memory {
            maybe_comma();
            let memory = boogie_resource_memory_name(self.global_env(), *memory);
            emit!(self.writer, &memory);
        }
        for QualifiedId {
            module_id: mid,
            id: vid,
        } in &fun_decl.used_spec_vars
        {
            maybe_comma();
            let declaring_module = self.global_env().get_module(*mid);
            let var_decl = declaring_module.get_spec_var(*vid);
            emit!(
                self.writer,
                &boogie_spec_var_name(&declaring_module, var_decl.name)
            );
        }
        for ty in instantiation.iter() {
            maybe_comma();
            assert!(!ty.is_incomplete());
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
        let module_env = self.global_env().get_module(module_id);
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

    fn translate_type_value(&self, node_id: NodeId) {
        let ty = &self.module_env().get_node_instantiation(node_id)[0];
        let type_value = self.translate_type(ty);
        emit!(self.writer, "$Type({})", type_value);
    }

    fn translate_resource_access(&self, node_id: NodeId, args: &[Exp]) {
        self.trace_value(node_id, TraceItem::Exp, || {
            let rty = &self.module_env().get_node_instantiation(node_id)[0];
            let (mid, sid, targs) = rty.require_struct();
            let env = self.global_env();
            emit!(
                self.writer,
                "$ResourceValue({}, {}, ",
                self.get_memory_name(mid.qualified(sid)),
                boogie_type_value_array(env, targs)
            );
            self.translate_exp(&args[0]);
            emit!(self.writer, ")");
        });
    }

    fn get_memory_name(&self, memory: QualifiedId<StructId>) -> String {
        if self.supports_native_old || !*self.in_old.borrow() {
            boogie_resource_memory_name(self.global_env(), memory)
        } else {
            boogie_saved_resource_memory_name(self.global_env(), memory)
        }
    }

    fn translate_resource_exists(&self, node_id: NodeId, args: &[Exp]) {
        self.trace_value(node_id, TraceItem::Exp, || {
            let rty = &self.module_env().get_node_instantiation(node_id)[0];
            let (mid, sid, targs) = rty.require_struct();
            let env = self.global_env();
            emit!(
                self.writer,
                "$ResourceExists({}, {}, ",
                self.get_memory_name(mid.qualified(sid)),
                boogie_type_value_array(env, targs)
            );
            self.translate_exp(&args[0]);
            emit!(self.writer, ")");
        });
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
        // all(domain<T>(), |a| P(a)) -->
        //      (forall $a: Value :: is#T($a) ==> P($a))
        // any(domain<T>(), |a| P(a)) -->
        //      (exists $a: Value :: is#T($a) && P($a))
        let quant_ty = self.module_env().get_node_type(args[0].node_id());
        let connective = if is_all { "==>" } else { "&&" };
        if let Exp::Lambda(_, vars, exp) = &args[1] {
            if let Some((var, _)) = self.get_decl_var(loc, vars) {
                let var_name = self.module_env().symbol_pool().string(var);
                let quant_var = self.fresh_var_name("i");
                let mut is_vector = false;
                let mut is_domain: Option<Type> = None;
                match quant_ty {
                    Type::Vector(..) => is_vector = true,
                    Type::TypeDomain(t) => is_domain = Some(t.as_ref().clone()),
                    Type::Primitive(PrimitiveType::Range) => (),
                    Type::Reference(_, b) => {
                        if let Type::Vector(..) = *b {
                            is_vector = true
                        } else {
                            panic!("unexpected type")
                        }
                    }
                    _ => panic!("unexpected type"),
                };
                if let Some(domain_ty) = is_domain {
                    let type_check = boogie_well_formed_expr(
                        self.global_env(),
                        &var_name,
                        &domain_ty,
                        WellFormedMode::Default,
                    );
                    if type_check.is_empty() {
                        let tctx = TypeDisplayContext::WithEnv {
                            env: self.global_env(),
                            type_param_names: None,
                        };
                        self.error(
                            loc,
                            &format!(
                                "cannot quantify over `{}` because the type is not concrete",
                                Type::TypeDomain(Box::new(domain_ty)).display(&tctx)
                            ),
                        );
                    } else {
                        emit!(
                            self.writer,
                            "$Boolean(({} {}: $Value :: {} {} ",
                            if is_all { "forall" } else { "exists" },
                            var_name,
                            type_check,
                            connective
                        );
                        emit!(self.writer, "b#$Boolean(");
                        self.translate_exp(exp.as_ref());
                        emit!(self.writer, ")))");
                    }
                } else {
                    let range_tmp = self.fresh_var_name("range");
                    emit!(self.writer, "$Boolean((var {} := ", range_tmp);
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
                            "$InRange({}, {}) {} (var {} := $Integer({}); ",
                            range_tmp,
                            quant_var,
                            connective,
                            var_name,
                            quant_var,
                        );
                    }
                    emit!(self.writer, "b#$Boolean(");
                    self.translate_exp(exp.as_ref());
                    emit!(self.writer, ")))))");
                }
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
            *self.in_old.borrow_mut() = true;
            self.translate_primitive_call("old", args);
            *self.in_old.borrow_mut() = false;
        } else {
            *self.in_old.borrow_mut() = true;
            self.translate_exp(&args[0]);
            *self.in_old.borrow_mut() = false;
        }
    }

    fn translate_eq_neq(&self, boogie_val_fun: &str, args: &[Exp]) {
        emit!(self.writer, "$Boolean(");
        emit!(self.writer, "{}(", boogie_val_fun);
        self.translate_exp(&args[0]);
        emit!(self.writer, ", ");
        self.translate_exp(&args[1]);
        emit!(self.writer, ")");
        emit!(self.writer, ")");
    }

    fn translate_arith_op(&self, boogie_op: &str, args: &[Exp]) {
        emit!(self.writer, "$Integer(i#$Integer(");
        self.translate_exp(&args[0]);
        emit!(self.writer, ") {} i#$Integer(", boogie_op);
        self.translate_exp(&args[1]);
        emit!(self.writer, "))");
    }

    fn translate_rel_op(&self, boogie_op: &str, args: &[Exp]) {
        emit!(self.writer, "$Boolean(i#$Integer(");
        self.translate_exp(&args[0]);
        emit!(self.writer, ") {} i#$Integer(", boogie_op);
        self.translate_exp(&args[1]);
        emit!(self.writer, "))");
    }

    fn translate_logical_op(&self, boogie_op: &str, args: &[Exp]) {
        emit!(self.writer, "$Boolean(b#$Boolean(");
        self.translate_exp(&args[0]);
        emit!(self.writer, ") {} b#$Boolean(", boogie_op);
        self.translate_exp(&args[1]);
        emit!(self.writer, "))");
    }

    fn translate_logical_unary_op(&self, boogie_op: &str, args: &[Exp]) {
        emit!(self.writer, "$Boolean({}b#$Boolean(", boogie_op);
        self.translate_exp(&args[0]);
        emit!(self.writer, "))");
    }

    fn translate_primitive_call(&self, fun: &str, args: &[Exp]) {
        emit!(self.writer, "{}(", fun);
        self.translate_seq(args.iter(), ", ", |e| self.translate_exp(e));
        emit!(self.writer, ")");
    }
}

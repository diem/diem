// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module translates specification conditions to Boogie code.

use std::collections::{BTreeSet, HashMap};

use itertools::Itertools;
#[allow(unused_imports)]
use log::{debug, info, warn};

use move_model::{
    ast::{Exp, LocalVarDecl, Operation, Value},
    code_writer::CodeWriter,
    emit, emitln,
    model::{
        FieldId, GlobalEnv, Loc, ModuleEnv, ModuleId, NodeId, QualifiedId, SpecFunId, SpecVarId,
        StructEnv, StructId,
    },
    symbol::Symbol,
    ty::{PrimitiveType, Type, TypeDisplayContext},
};

use crate::{
    boogie_helpers::{
        boogie_byte_blob, boogie_declare_global, boogie_field_name, boogie_global_declarator,
        boogie_local_type, boogie_resource_memory_name, boogie_spec_fun_name, boogie_spec_var_name,
        boogie_struct_name, boogie_type_value, boogie_type_value_array, boogie_well_formed_expr,
    },
    options::BoogieOptions,
};
use move_model::ast::{MemoryLabel, QuantKind};
use std::cell::RefCell;

pub struct SpecTranslator<'env> {
    /// The global environment.
    env: &'env GlobalEnv,
    /// Options passed into the translator.
    options: &'env BoogieOptions,
    /// The code writer.
    writer: &'env CodeWriter,
    /// If we are translating in the context of a type instantiation, the type arguments.
    type_args_opt: Option<Vec<Type>>,
    /// Counter for creating new variables.
    fresh_var_count: RefCell<usize>,
    /// Set of items which have been already traced. This is used to avoid redundant tracing
    /// of expressions whose value has been already tracked.
    traced_items: RefCell<BTreeSet<TraceItem>>,
}

/// A item which is traced for printing in diagnosis. The boolean indicates whether the item is
/// traced inside old context or not.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum TraceItem {
    // Automatically traced items when `options.prover.debug_trace_exp` is on.
    Local(Symbol),
    SpecVar(ModuleId, SpecVarId, Vec<Type>, Option<MemoryLabel>),
    Exp,
    // Explicitly traced item via user level trace function.
    Explicit,
}

impl<'env> SpecTranslator<'env> {
    /// Creates a translator.
    pub fn new(
        writer: &'env CodeWriter,
        env: &'env GlobalEnv,
        options: &'env BoogieOptions,
    ) -> Self {
        Self {
            env,
            options,
            writer,
            type_args_opt: None,
            fresh_var_count: Default::default(),
            traced_items: Default::default(),
        }
    }

    /// Emits a translation error.
    pub fn error(&self, loc: &Loc, msg: &str) {
        self.env.error(loc, &format!("[boogie translator] {}", msg));
    }

    /// Sets the location of the code writer from node id.
    fn set_writer_location(&self, node_id: NodeId) {
        self.writer.set_location(&self.env.get_node_loc(node_id));
    }

    /// Generates a fresh variable name.
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
    pub fn translate_spec_vars(&self, module_env: &ModuleEnv<'_>) {
        emitln!(
            self.writer,
            "\n\n// ** spec vars of module {}\n",
            module_env.get_name().display(module_env.symbol_pool())
        );
        for (_, var) in module_env.get_spec_vars() {
            let boogie_name = boogie_spec_var_name(module_env, var.name, &None);
            emitln!(
                self.writer,
                &boogie_declare_global(self.env, &boogie_name, var.type_params.len(), &var.type_)
            );
        }
    }
}

// Specification Functions
// =======================

impl<'env> SpecTranslator<'env> {
    pub fn translate_spec_funs(&self, module_env: &ModuleEnv<'_>) {
        emitln!(
            self.writer,
            "\n\n// ** spec funs of module {}\n",
            module_env.get_name().display(module_env.symbol_pool())
        );
        for (id, fun) in module_env.get_spec_funs() {
            if fun.body.is_none() && !fun.uninterpreted {
                // This function is native and expected to be found in the prelude.
                continue;
            }
            if fun.is_move_fun && fun.is_native {
                // This function is a native Move function and its spec version is
                // expected to be found in the prelude.
                continue;
            }
            if fun.is_move_fun && !module_env.spec_fun_is_used(*id) {
                // This function is a pure Move function but is never used,
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
                    let declaring_module = self.env.get_module(*mid);
                    let decl = declaring_module.get_spec_var(*vid);
                    let boogie_name = boogie_spec_var_name(&declaring_module, decl.name, &None);
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
                    boogie_resource_memory_name(self.env, *memory, &None)
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
                    name.display(module_env.symbol_pool()),
                    boogie_local_type(ty)
                )
            });
            self.writer.set_location(&fun.loc);
            let boogie_name = boogie_spec_fun_name(&module_env, *id);
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
                let call =
                    format!(
                        "{}({})",
                        boogie_name,
                        fun.type_params
                            .iter()
                            .enumerate()
                            .map(|(i, _)| format!("$tv{}", i))
                            .chain(fun.params.iter().map(|(n, _)| {
                                format!("{}", n.display(module_env.symbol_pool()))
                            }))
                            .join(", ")
                    );
                let type_check = boogie_well_formed_expr(self.env, &call, &fun.result_type);
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

/// Well Formedness function
/// ========================

impl<'env> SpecTranslator<'env> {
    /// Generates a function which assumes well formedness (type correcness) of a struct.
    pub fn translate_assume_well_formed(&self, struct_env: &StructEnv<'_>) {
        emitln!(
            self.writer,
            "function {{:inline}} {}_$is_well_formed($this: $Value): bool {{",
            boogie_struct_name(struct_env),
        );
        self.writer.indent();
        emitln!(self.writer, "$Vector_$is_well_formed($this)");
        emitln!(
            self.writer,
            "&& $vlen($this) == {}",
            struct_env.get_fields().count()
        );
        for field in struct_env.get_fields() {
            let select = format!("$SelectField($this, {})", boogie_field_name(&field));
            let check =
                boogie_well_formed_expr(struct_env.module_env.env, &select, &field.get_type());
            if !check.is_empty() {
                emit!(self.writer, "  && ");
                emitln!(self.writer, "{}", check);
            }
        }
        self.writer.unindent();
        emitln!(self.writer, "}");
    }
}

// Types
// ===========

impl<'env> SpecTranslator<'env> {
    /// Translates a type into a string in boogie. If the translator works with a type
    /// instantiation, this will be used to instantiate the type.
    fn translate_type(&self, ty: &Type) -> String {
        if let Some(ty_args) = &self.type_args_opt {
            boogie_type_value(self.env, &ty.instantiate(ty_args))
        } else {
            boogie_type_value(self.env, ty)
        }
    }
}

// Expressions
// ===========

impl<'env> SpecTranslator<'env> {
    pub(crate) fn translate_exp(&self, exp: &Exp) {
        match exp {
            Exp::Value(node_id, val) => {
                self.set_writer_location(*node_id);
                self.translate_value(*node_id, val);
            }
            Exp::LocalVar(node_id, name) => {
                self.set_writer_location(*node_id);
                self.translate_local_var(*node_id, *name);
            }
            Exp::SpecVar(node_id, module_id, var_id, mem_label) => {
                let instantiation = &self.env.get_node_instantiation(*node_id);
                self.trace_value(
                    *node_id,
                    TraceItem::SpecVar(*module_id, *var_id, instantiation.clone(), *mem_label),
                    || {
                        self.set_writer_location(*node_id);
                        let module_env = self.env.get_module(*module_id);
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
                            boogie_spec_var_name(&module_env, spec_var.name, mem_label),
                            instantiation_str
                        );
                    },
                );
            }
            Exp::Call(node_id, oper, args) => {
                self.set_writer_location(*node_id);
                self.translate_call(*node_id, oper, args);
            }
            Exp::Invoke(node_id, ..) => {
                self.error(&self.env.get_node_loc(*node_id), "Invoke not yet supported")
            }
            Exp::Lambda(node_id, ..) => self.error(
                &self.env.get_node_loc(*node_id),
                "`|x|e` (lambda) currently only supported as argument for `all` or `any`",
            ),
            Exp::Quant(node_id, kind, ranges, condition, exp) => {
                self.set_writer_location(*node_id);
                self.translate_quant(*node_id, *kind, ranges, condition, exp)
            }
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
            Exp::Invalid(_) => panic!("unexpected error expression"),
        }
    }

    #[allow(clippy::if_same_then_else)]
    fn trace_value<F>(&self, _node_id: NodeId, item: TraceItem, f: F)
    where
        F: Fn(),
    {
        let go = if item == TraceItem::Explicit {
            // User called TRACE function, always do it.
            true
        } else if self.options.debug_trace {
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
            // TODO(refactoring): make this work without need for module
            /*
            let module_env = self.module_env();
            emit!(
                self.writer,
                "$DebugTrackExp({}, {}, ",
                module_env.get_id().to_usize(),
                node_id.as_usize(),
            );
            f();
            emit!(self.writer, ")");
             */
            f();
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
        self.trace_value(node_id, TraceItem::Local(name), || {
            emit!(self.writer, "{}", name.display(self.env.symbol_pool()));
        });
    }

    fn translate_block(&self, node_id: NodeId, vars: &[LocalVarDecl], exp: &Exp) {
        if vars.is_empty() {
            return self.translate_exp(exp);
        }
        let loc = self.env.get_node_loc(node_id);
        if let [var] = vars {
            let name_str = self.env.symbol_pool().string(var.name);
            emit!(self.writer, "(var {} := ", name_str);
            self.translate_exp(var.binding.as_ref().expect("binding"));
            emit!(self.writer, "; ");
            self.translate_exp(exp);
            emit!(self.writer, ")");
        } else {
            self.error(&loc, "currently only single variable binding supported");
        }
    }

    fn translate_call(&self, node_id: NodeId, oper: &Operation, args: &[Exp]) {
        let loc = self.env.get_node_loc(node_id);
        match oper {
            Operation::Function(module_id, fun_id, memory_labels) => {
                self.translate_spec_fun_call(node_id, *module_id, *fun_id, args, memory_labels)
            }
            Operation::Pack(..) => self.translate_pack(args),
            Operation::Tuple => self.error(&loc, "Tuple not yet supported"),
            Operation::Select(module_id, struct_id, field_id) => {
                self.translate_select(*module_id, *struct_id, *field_id, args)
            }
            Operation::UpdateField(module_id, struct_id, field_id) => {
                self.translate_update_field(*module_id, *struct_id, *field_id, args)
            }
            Operation::Local(sym) => {
                self.translate_local_var(node_id, *sym);
            }
            Operation::Result(pos) => {
                emit!(self.writer, "$ret{}", pos);
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
            Operation::Global(memory_label) => {
                self.translate_resource_access(node_id, args, memory_label)
            }
            Operation::Exists(memory_label) => {
                self.translate_resource_exists(node_id, args, memory_label)
            }
            Operation::Len => self.translate_primitive_call("$vlen_value", args),
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
            Operation::Old => panic!("old(..) expression unexpected"),
            Operation::Trace => self.trace_value(node_id, TraceItem::Explicit, || {
                self.translate_exp(&args[0])
            }),
            Operation::MaxU8 => emit!(self.writer, "$Integer($MAX_U8)"),
            Operation::MaxU64 => emit!(self.writer, "$Integer($MAX_U64)"),
            Operation::MaxU128 => emit!(self.writer, "$Integer($MAX_U128)"),
            Operation::AbortCode | Operation::AbortFlag => unimplemented!(),
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
        memory_labels: &Option<Vec<MemoryLabel>>,
    ) {
        let instantiation = self.env.get_node_instantiation(node_id);
        let module_env = self.env.get_module(module_id);
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
        let label_at = |i| {
            if let Some(labels) = memory_labels {
                Some(labels[i])
            } else {
                None
            }
        };
        let mut i = 0;
        for memory in &fun_decl.used_memory {
            maybe_comma();
            let memory = boogie_resource_memory_name(self.env, *memory, &label_at(i));
            emit!(self.writer, &memory);
            i += 1;
        }
        for QualifiedId {
            module_id: mid,
            id: vid,
        } in &fun_decl.used_spec_vars
        {
            maybe_comma();
            let declaring_module = self.env.get_module(*mid);
            let var_decl = declaring_module.get_spec_var(*vid);
            emit!(
                self.writer,
                &boogie_spec_var_name(&declaring_module, var_decl.name, &label_at(i))
            );
            i += 1;
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
        let module_env = self.env.get_module(module_id);
        let struct_env = module_env.get_struct(struct_id);
        let field_env = struct_env.get_field(field_id);
        let field_name = boogie_field_name(&field_env);
        emit!(self.writer, "$SelectField(");
        self.translate_exp(&args[0]);
        emit!(self.writer, ", {})", field_name);
    }

    fn translate_update_field(
        &self,
        module_id: ModuleId,
        struct_id: StructId,
        field_id: FieldId,
        args: &[Exp],
    ) {
        let module_env = self.env.get_module(module_id);
        let struct_env = module_env.get_struct(struct_id);
        let field_env = struct_env.get_field(field_id);
        let field_name = boogie_field_name(&field_env);
        emit!(self.writer, "$UpdateField(");
        self.translate_exp(&args[0]);
        emit!(self.writer, ", {}, ", field_name);
        self.translate_exp(&args[1]);
        emit!(self.writer, ")");
    }

    fn translate_type_value(&self, node_id: NodeId) {
        let ty = &self.env.get_node_instantiation(node_id)[0];
        let type_value = self.translate_type(ty);
        emit!(self.writer, "$Type({})", type_value);
    }

    fn translate_resource_access(
        &self,
        node_id: NodeId,
        args: &[Exp],
        memory_label: &Option<MemoryLabel>,
    ) {
        self.trace_value(node_id, TraceItem::Exp, || {
            let rty = &self.env.get_node_instantiation(node_id)[0];
            let (mid, sid, targs) = rty.require_struct();
            let env = self.env;
            emit!(
                self.writer,
                "$ResourceValue({}, {}, ",
                boogie_resource_memory_name(self.env, mid.qualified(sid), memory_label),
                boogie_type_value_array(env, targs)
            );
            self.translate_exp(&args[0]);
            emit!(self.writer, ")");
        });
    }

    fn translate_resource_exists(
        &self,
        node_id: NodeId,
        args: &[Exp],
        memory_label: &Option<MemoryLabel>,
    ) {
        self.trace_value(node_id, TraceItem::Exp, || {
            let rty = &self.env.get_node_instantiation(node_id)[0];
            let (mid, sid, targs) = rty.require_struct();
            let env = self.env;
            emit!(
                self.writer,
                "$ResourceExists({}, {}, ",
                boogie_resource_memory_name(self.env, mid.qualified(sid), memory_label),
                boogie_type_value_array(env, targs)
            );
            self.translate_exp(&args[0]);
            emit!(self.writer, ")");
        });
    }

    fn translate_quant(
        &self,
        node_id: NodeId,
        kind: QuantKind,
        ranges: &[(LocalVarDecl, Exp)],
        condition: &Option<Box<Exp>>,
        body: &Exp,
    ) {
        let loc = self.env.get_node_loc(node_id);
        // Translate range expressions.
        let mut range_tmps = HashMap::new();
        for (var, range) in ranges {
            let quant_ty = self.env.get_node_type(range.node_id());
            if matches!(
                quant_ty.skip_reference(),
                Type::Vector(..) | Type::Primitive(PrimitiveType::Range)
            ) {
                let var_name = self.env.symbol_pool().string(var.name);
                let range_tmp = self.fresh_var_name("range");
                emit!(self.writer, "(var {} := ", range_tmp);
                self.translate_exp(&range);
                emit!(self.writer, "; ");
                range_tmps.insert(var_name, range_tmp);
            }
        }
        // Translate quantified variables.
        emit!(self.writer, "$Boolean(({} ", kind);
        let mut quant_vars = HashMap::new();
        let mut comma = "";
        for (var, range) in ranges {
            let var_name = self.env.symbol_pool().string(var.name);
            let quant_ty = self.env.get_node_type(range.node_id());
            match quant_ty.skip_reference() {
                Type::TypeDomain(_) => {
                    emit!(self.writer, "{}{}: $Value", comma, var_name);
                }
                _ => {
                    let quant_var = self.fresh_var_name("i");
                    emit!(self.writer, "{}{}: int", comma, quant_var);
                    quant_vars.insert(var_name, quant_var);
                }
            }
            comma = ", ";
        }
        emit!(self.writer, " :: ");
        // Translate range constraints.
        let connective = match kind {
            QuantKind::Forall => " ==> ",
            QuantKind::Exists => " && ",
        };
        let mut separator = "";
        for (var, range) in ranges {
            let var_name = self.env.symbol_pool().string(var.name);
            let quant_ty = self.env.get_node_type(range.node_id());
            match quant_ty.skip_reference() {
                Type::TypeDomain(domain_ty) => {
                    let type_check = boogie_well_formed_expr(self.env, &var_name, &domain_ty);
                    if type_check.is_empty() {
                        let tctx = TypeDisplayContext::WithEnv {
                            env: self.env,
                            type_param_names: None,
                        };
                        self.error(
                            &loc,
                            &format!(
                                "cannot quantify over `{}` because the type is not concrete",
                                Type::TypeDomain(domain_ty.clone()).display(&tctx)
                            ),
                        );
                    } else {
                        emit!(self.writer, "{}{}", separator, type_check);
                    }
                }
                Type::Vector(..) => {
                    let range_tmp = range_tmps.get(&var_name).unwrap();
                    let quant_var = quant_vars.get(&var_name).unwrap();
                    emit!(
                        self.writer,
                        "{}$InVectorRange({}, {})",
                        separator,
                        range_tmp,
                        quant_var,
                    );
                }
                Type::Primitive(PrimitiveType::Range) => {
                    let range_tmp = range_tmps.get(&var_name).unwrap();
                    let quant_var = quant_vars.get(&var_name).unwrap();
                    emit!(
                        self.writer,
                        "{}$InRange({}, {})",
                        separator,
                        range_tmp,
                        quant_var,
                    );
                }
                _ => panic!("unexpected type"),
            }
            separator = connective;
        }
        emit!(self.writer, "{}", connective);
        // Translate range selectors.
        for (var, range) in ranges {
            let var_name = self.env.symbol_pool().string(var.name);
            let quant_ty = self.env.get_node_type(range.node_id());
            match quant_ty.skip_reference() {
                Type::Vector(..) => {
                    let range_tmp = range_tmps.get(&var_name).unwrap();
                    let quant_var = quant_vars.get(&var_name).unwrap();
                    emit!(
                        self.writer,
                        "(var {} := $select_vector({}, {}); ",
                        var_name,
                        range_tmp,
                        quant_var,
                    );
                }
                Type::Primitive(PrimitiveType::Range) => {
                    let quant_var = quant_vars.get(&var_name).unwrap();
                    emit!(
                        self.writer,
                        "(var {} := $Integer({}); ",
                        var_name,
                        quant_var
                    );
                }
                _ => (),
            }
        }
        // Translate body and "where" condition.
        if let Some(cond) = condition {
            emit!(self.writer, "b#$Boolean(");
            self.translate_exp(cond);
            emit!(self.writer, ") {}", connective);
        }
        emit!(self.writer, "b#$Boolean(");
        self.translate_exp(body);
        emit!(
            self.writer,
            &std::iter::repeat(")")
                .take(3 + 2 * range_tmps.len())
                .collect::<String>()
        );
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

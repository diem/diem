// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module translates specification conditions to Boogie code.

use std::collections::HashMap;

use itertools::Itertools;
#[allow(unused_imports)]
use log::{debug, info, warn};

use move_model::{
    ast::{Exp, LocalVarDecl, Operation, Value},
    code_writer::CodeWriter,
    emit, emitln,
    model::{FieldId, GlobalEnv, Loc, ModuleEnv, ModuleId, NodeId, SpecFunId, StructId},
    symbol::Symbol,
    ty::{PrimitiveType, Type},
};

use crate::{
    boogie_helpers::{
        boogie_byte_blob, boogie_declare_global, boogie_field_name, boogie_global_declarator,
        boogie_local_type, boogie_modifies_memory_name, boogie_resource_memory_name,
        boogie_spec_fun_name, boogie_spec_var_name, boogie_type_suffix, boogie_type_value,
        boogie_type_value_array, boogie_type_value_array_from_strings, boogie_well_formed_expr,
        MAX_MAKE_VEC_ARGS,
    },
    options::BoogieOptions,
};
use move_model::{
    ast::{ConditionKind, MemoryLabel, QuantKind, TempIndex},
    model::QualifiedId,
};
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
        *fvc_ref = usize::saturating_add(*fvc_ref, 1);
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

// Specification Functions and Axioms
// ==================================

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
            let spec_var_params = fun
                .used_spec_vars
                .iter()
                .map(|id| id.to_qualified_id())
                .unique()
                .map(
                    |QualifiedId {
                         module_id: ref mid,
                         id: ref vid,
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
            let mem_params = fun
                .used_memory
                .iter()
                .map(|id| id.to_qualified_id())
                .unique()
                .map(|memory| {
                    format!(
                        "{}: $Memory",
                        boogie_resource_memory_name(self.env, memory, &None)
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
                let type_check = boogie_well_formed_expr(self.env, "$$res", &fun.result_type);
                if !type_check.is_empty() {
                    if !param_list.is_empty() {
                        emitln!(
                            self.writer,
                            "axiom (forall {} ::\n(var $$res := {};\n{}));",
                            param_list,
                            call,
                            type_check
                        );
                    } else {
                        emitln!(
                            self.writer,
                            "axiom (var $$res := {};\n{});",
                            call,
                            type_check
                        );
                    }
                }
                emitln!(self.writer);
            } else {
                emitln!(self.writer, " {");
                self.writer.indent();
                self.translate_unboxed(fun.body.as_ref().unwrap());
                emitln!(self.writer);
                self.writer.unindent();
                emitln!(self.writer, "}");
                emitln!(self.writer);
            }
        }
    }

    pub fn translate_axioms(&self, module_env: &ModuleEnv) {
        for axiom in module_env.get_spec().filter_kind(ConditionKind::Axiom) {
            emitln!(self.writer, "// axiom {}", axiom.loc.display(self.env));
            emit!(self.writer, "axiom ");
            self.translate_unboxed(&axiom.exp);
            emitln!(self.writer, ";\n");
        }
    }
}

// Types
// =====

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

// Boxing/Unboxing
// ===============

impl<'env> SpecTranslator<'env> {
    /// Transformation of an expression which introduces BoxValue/UnboxValue calls depending
    /// on whether the underlying Boogie implementation works with $Value or with the unboxed
    /// version. This optimizes `UnboxValue(BoxValue(e))` into `e` where possible. It is run
    /// before the actual translation which relies on that boxing has been made explicit.
    ///
    /// One design principle here is that we expect each expression to deliver the boxed
    /// representation of a value. We then unbox that value as is determined by expression
    /// context, possibly reducing it via the above rewriting rule.
    fn box_unbox(&self, exp: Exp) -> Exp {
        use Exp::*;

        match exp {
            LocalVar(id, name) => self.box_value(LocalVar(id, name)),
            Temporary(id, idx) => self.box_value(Temporary(id, idx)),
            Value(id, val) => self.box_value(Value(id, val)),
            Call(id, oper, args) => self.box_unbox_call(id, oper, args),
            Invoke(id, fun, args) => {
                Invoke(id, Box::new(self.box_unbox(*fun)), self.box_unbox_vec(args))
            }
            Lambda(id, decls, body) => Lambda(
                id,
                decls
                    .into_iter()
                    .map(|d| LocalVarDecl {
                        id: d.id,
                        name: d.name,
                        binding: d.binding.map(|e| self.box_unbox(e)),
                    })
                    .collect(),
                Box::new(self.box_unbox(*body)),
            ),
            Block(id, decls, body) => Block(
                id,
                decls
                    .into_iter()
                    .map(|d| LocalVarDecl {
                        id: d.id,
                        name: d.name,
                        binding: d.binding.map(|e| self.unbox_value(self.box_unbox(e))),
                    })
                    .collect(),
                Box::new(self.box_unbox(*body)),
            ),
            Quant(id, kind, ranges, triggers, constr, body) => self.box_value(Quant(
                id,
                kind,
                ranges
                    .into_iter()
                    .map(|(d, e)| {
                        (
                            LocalVarDecl {
                                id: d.id,
                                name: d.name,
                                binding: d.binding.map(|e| self.box_unbox(e)),
                            },
                            self.unbox_value(self.box_unbox(e)),
                        )
                    })
                    .collect_vec(),
                triggers
                    .into_iter()
                    .map(|ts| self.box_unbox_vec_unbox_value(ts))
                    .collect(),
                constr.map(|e| Box::new(self.unbox_value(self.box_unbox(*e)))),
                Box::new(self.unbox_value(self.box_unbox(*body))),
            )),
            IfElse(id, i, t, e) => IfElse(
                id,
                Box::new(self.unbox_value(self.box_unbox(*i))),
                Box::new(self.box_unbox(*t)),
                Box::new(self.box_unbox(*e)),
            ),
            _ => exp,
        }
    }

    fn box_unbox_vec(&self, vec: Vec<Exp>) -> Vec<Exp> {
        vec.into_iter().map(|e| self.box_unbox(e)).collect()
    }

    fn box_unbox_vec_unbox_value(&self, vec: Vec<Exp>) -> Vec<Exp> {
        vec.into_iter()
            .map(|e| self.box_unbox(e))
            .map(|e| self.unbox_value(e))
            .collect()
    }

    fn box_unbox_call(&self, id: NodeId, oper: Operation, mut args: Vec<Exp>) -> Exp {
        use Exp::*;
        use Operation::*;
        match &oper {
            // Function `f<T>` requires boxing of parameters of type `T` and unboxing of
            // results of type `T`.
            Function(mid, fid, _) => {
                let module_env = self.env.get_module(*mid);
                let decl = module_env.get_spec_fun(*fid);
                let new_args = args
                    .into_iter()
                    .enumerate()
                    .map(|(i, exp)| {
                        if decl.params[i].1.skip_reference().is_type_parameter() {
                            self.box_unbox(exp)
                        } else {
                            self.unbox_value(self.box_unbox(exp))
                        }
                    })
                    .collect_vec();
                let new_call = Call(id, oper, new_args);
                if decl.result_type.skip_reference().is_type_parameter() {
                    new_call
                } else {
                    self.box_value(new_call)
                }
            }
            // Irregular operators which have some arguments boxed others none
            UpdateField(..) => {
                let arg2 = args.pop().unwrap();
                let arg1 = args.pop().unwrap();
                self.box_value(Call(
                    id,
                    oper,
                    vec![self.unbox_value(self.box_unbox(arg1)), self.box_unbox(arg2)],
                ))
            }
            Update => {
                let arg3 = args.pop().unwrap();
                let arg2 = args.pop().unwrap();
                let arg1 = args.pop().unwrap();
                self.box_value(Call(
                    id,
                    oper,
                    vec![
                        self.unbox_value(self.box_unbox(arg1)),
                        self.unbox_value(self.box_unbox(arg2)),
                        self.box_unbox(arg3),
                    ],
                ))
            }

            // Operators which take boxed arguments and deliver unboxed ones
            Pack(..) => self.box_value(Call(id, oper, self.box_unbox_vec(args))),

            // Operators which take unboxed arguments and deliver boxed ones
            Index | Select(..) => Call(id, oper, self.box_unbox_vec_unbox_value(args)),

            // Everything else which take unboxed arguments and delivers unboxed
            _ => self.box_value(Call(id, oper, self.box_unbox_vec_unbox_value(args))),
        }
    }

    fn box_value(&self, e: Exp) -> Exp {
        Exp::Call(e.node_id(), Operation::BoxValue, vec![e])
    }

    fn unbox_value(&self, e: Exp) -> Exp {
        use Exp::*;
        use Operation::*;
        match e {
            Call(_, BoxValue, args) => args.into_iter().next().unwrap(),
            Block(id, decls, body) => Block(id, decls, Box::new(self.unbox_value(*body))),
            IfElse(id, cond, if_e, else_e) => IfElse(
                id,
                cond,
                Box::new(self.unbox_value(*if_e)),
                Box::new(self.unbox_value(*else_e)),
            ),
            _ => Exp::Call(e.node_id(), Operation::UnboxValue, vec![e]),
        }
    }
}

// Expressions
// ===========

impl<'env> SpecTranslator<'env> {
    // Translate the expression.
    pub(crate) fn translate(&self, exp: &Exp) {
        let exp1 = self.box_unbox(exp.clone());
        self.translate_exp(&exp1)
    }

    // Translate the expression and deliver an unboxed (Boogie native) result.
    pub(crate) fn translate_unboxed(&self, exp: &Exp) {
        let exp1 = self.unbox_value(self.box_unbox(exp.clone()));
        self.translate_exp(&exp1)
    }

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
            Exp::Temporary(node_id, idx) => {
                self.set_writer_location(*node_id);
                self.translate_temporary(*node_id, *idx);
            }
            Exp::SpecVar(node_id, module_id, var_id, mem_label) => {
                let instantiation = &self.env.get_node_instantiation(*node_id);
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
            Exp::Quant(node_id, kind, ranges, triggers, condition, exp) => {
                self.set_writer_location(*node_id);
                self.translate_quant(*node_id, *kind, ranges, triggers, condition, exp)
            }
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
            Exp::Invalid(_) => panic!("unexpected error expression"),
        }
    }

    fn translate_exp_parenthesised(&self, exp: &Exp) {
        emit!(self.writer, "(");
        self.translate_exp(exp);
        emit!(self.writer, ")");
    }

    fn translate_value(&self, _node_id: NodeId, val: &Value) {
        match val {
            Value::Address(addr) => emit!(self.writer, "{}", addr),
            Value::Number(val) => emit!(self.writer, "{}", val),
            Value::Bool(val) => emit!(self.writer, "{}", val),
            Value::ByteArray(val) => emit!(self.writer, &boogie_byte_blob(self.options, val)),
        }
    }

    fn translate_local_var(&self, _node_id: NodeId, name: Symbol) {
        emit!(self.writer, "{}", name.display(self.env.symbol_pool()));
    }

    fn translate_temporary(&self, node_id: NodeId, idx: TempIndex) {
        let ty = self.env.get_node_type(node_id);
        let mut_ref = ty.is_mutable_reference();
        if mut_ref {
            let suffix = boogie_type_suffix(ty.skip_reference());
            emit!(self.writer, "$Unbox{}($Dereference(", suffix);
        }
        emit!(self.writer, "$t{}", idx);
        if mut_ref {
            emit!(self.writer, "))")
        }
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
            // Operators we introduced in the top level public entry `SpecTranslator::translate`,
            // mapping between Boogies single value domain and our typed world.
            Operation::BoxValue => self.translate_box_value(node_id, &args[0]),
            Operation::UnboxValue => self.translate_unbox_value(node_id, &args[0]),

            // Internal operators for event stores.
            Operation::EmptyEventStore => emit!(self.writer, "$EmptyEventStore"),
            Operation::ExtendEventStore => self.translate_extend_event_store(args),
            Operation::EventStoreIncludes => self.translate_event_store_includes(args),
            Operation::EventStoreIncludedIn => self.translate_event_store_included_in(args),

            // Regular expressions
            Operation::Function(module_id, fun_id, memory_labels) => {
                self.translate_spec_fun_call(node_id, *module_id, *fun_id, args, memory_labels)
            }
            Operation::Pack(mid, sid) => self.translate_pack(*mid, *sid, args),
            Operation::Tuple => self.error(&loc, "Tuple not yet supported"),
            Operation::Select(module_id, struct_id, field_id) => {
                self.translate_select(*module_id, *struct_id, *field_id, args)
            }
            Operation::UpdateField(module_id, struct_id, field_id) => {
                self.translate_update_field(*module_id, *struct_id, *field_id, args)
            }
            Operation::Result(pos) => {
                emit!(self.writer, "$ret{}", pos);
            }
            Operation::Index => self.translate_primitive_call("ReadVec", args),
            Operation::Slice => self.translate_primitive_call("$SliceVecByRange", args),
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
            Operation::Identical => self.translate_identical(args),
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
            Operation::CanModify => self.translate_can_modify(node_id, args),
            Operation::Len => self.translate_primitive_call("LenVec", args),
            Operation::TypeValue => self.translate_type_value(node_id),
            Operation::TypeDomain | Operation::ResourceDomain => self.error(
                &loc,
                "domain functions can only be used as the range of a quantifier",
            ),
            Operation::Update => self.translate_primitive_call("UpdateVec", args),
            Operation::Concat => self.translate_primitive_call("ConcatVec", args),
            Operation::Empty => self.translate_primitive_call("EmptyVec", args),
            Operation::Single => self.translate_primitive_call("MakeVec1", args),
            Operation::IndexOf => self.translate_primitive_call("$IndexOfVec", args),
            Operation::Contains => self.translate_primitive_call("$ContainsVec", args),
            Operation::InRange => self.translate_in_range(args),
            Operation::Old => panic!("old(..) expression unexpected"),
            Operation::Trace => self.translate_exp(&args[0]),
            Operation::MaxU8 => emit!(self.writer, "$MAX_U8"),
            Operation::MaxU64 => emit!(self.writer, "$MAX_U64"),
            Operation::MaxU128 => emit!(self.writer, "$MAX_U128"),
            Operation::WellFormed => self.translate_well_formed(&args[0]),
            Operation::AbortCode => emit!(self.writer, "$abort_code"),
            Operation::AbortFlag => emit!(self.writer, "$abort_flag"),
            Operation::NoOp => { /* do nothing. */ }
        }
    }

    fn translate_event_store_includes(&self, args: &[Exp]) {
        emit!(
            self.writer,
            "(var actual := $EventStore__subtract($es, old($es)); "
        );
        emit!(self.writer, "(var expected := ");
        self.translate_exp(&args[0]);
        emit!(self.writer, "; $EventStore__is_subset(expected, actual)))");
    }

    fn translate_event_store_included_in(&self, args: &[Exp]) {
        emit!(
            self.writer,
            "(var actual := $EventStore__subtract($es, old($es)); "
        );
        emit!(self.writer, "(var expected := ");
        self.translate_exp(&args[0]);
        emit!(self.writer, "; $EventStore__is_subset(actual, expected)))");
    }

    fn translate_extend_event_store(&self, args: &[Exp]) {
        let with_cond = args.len() == 4;
        if with_cond {
            emit!(self.writer, "$CondExtendEventStore(")
        } else {
            emit!(self.writer, "$ExtendEventStore(")
        }
        self.translate_exp(&args[0]); // event store
        emit!(self.writer, ", ");
        // Next expected argument is the guid.
        emit!(self.writer, "$GetEventHandleGuid(");
        self.translate_exp(&args[2]);
        emit!(self.writer, "), ");
        // Next comes the event.
        let suffix = boogie_type_suffix(&self.env.get_node_type(args[1].node_id()));
        emit!(self.writer, "$Box{}(", suffix);
        self.translate_exp(&args[1]);
        emit!(self.writer, ")");
        // Next comes the optional condition
        if with_cond {
            emit!(self.writer, ", ");
            self.translate_exp(&args[3]);
        }
        emit!(self.writer, ")");
    }

    fn translate_pack(&self, mid: ModuleId, sid: StructId, args: &[Exp]) {
        let struct_env = self.env.get_module(mid).into_struct(sid);
        let field_count = struct_env.get_field_count();
        let direct_make_count = usize::min(field_count, MAX_MAKE_VEC_ARGS);
        for _ in direct_make_count..field_count {
            emit!(self.writer, "ExtendVec(")
        }
        emit!(self.writer, "MakeVec{}(", direct_make_count);
        for (i, arg) in args[0..direct_make_count].iter().enumerate() {
            if i > 0 {
                emit!(self.writer, ", ");
            }
            self.translate_exp(arg);
        }
        emit!(self.writer, ")");
        for arg in &args[direct_make_count..] {
            emit!(self.writer, ", ");
            self.translate_exp(arg);
            emit!(self.writer, ")");
        }
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
        for memory in fun_decl
            .used_memory
            .iter()
            .map(|id| id.to_qualified_id())
            .unique()
        {
            maybe_comma();
            let memory = boogie_resource_memory_name(self.env, memory, &label_at(i));
            emit!(self.writer, &memory);
            i = usize::saturating_add(i, 1);
        }
        for QualifiedId {
            module_id: ref mid,
            id: ref vid,
        } in fun_decl
            .used_spec_vars
            .iter()
            .map(|id| id.to_qualified_id())
            .unique()
        {
            maybe_comma();
            let declaring_module = self.env.get_module(*mid);
            let var_decl = declaring_module.get_spec_var(*vid);
            emit!(
                self.writer,
                &boogie_spec_var_name(&declaring_module, var_decl.name, &label_at(i))
            );
            i = usize::saturating_add(i, 1);
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
        emit!(self.writer, "ReadVec(");
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
        emit!(self.writer, "UpdateVec(");
        self.translate_exp(&args[0]);
        emit!(self.writer, ", {}, ", field_name);
        self.translate_exp(&args[1]);
        emit!(self.writer, ")");
    }

    fn translate_type_value(&self, node_id: NodeId) {
        let ty = &self.env.get_node_instantiation(node_id)[0];
        let type_value = self.translate_type(ty);
        emit!(self.writer, "{}", type_value);
    }

    fn translate_resource_access(
        &self,
        node_id: NodeId,
        args: &[Exp],
        memory_label: &Option<MemoryLabel>,
    ) {
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
    }

    fn translate_resource_exists(
        &self,
        node_id: NodeId,
        args: &[Exp],
        memory_label: &Option<MemoryLabel>,
    ) {
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
    }

    fn translate_can_modify(&self, node_id: NodeId, args: &[Exp]) {
        let env = self.env;
        let rty = &self.env.get_node_instantiation(node_id)[0];
        let (mid, sid, targs) = rty.require_struct();
        let resource_name = boogie_modifies_memory_name(self.env, mid.qualified(sid));
        let type_args = boogie_type_value_array(env, targs);
        emit!(self.writer, "{}[{}, ", resource_name, type_args);
        self.translate_exp(&args[0]);
        emit!(self.writer, "]");
    }

    fn with_range_selector_assignments<F>(
        &self,
        ranges: &[(LocalVarDecl, Exp)],
        range_tmps: &HashMap<Symbol, String>,
        quant_vars: &HashMap<Symbol, String>,
        resource_vars: &HashMap<Symbol, (String, Vec<String>)>,
        f: F,
    ) where
        F: Fn(),
    {
        // Translate range selectors.
        for (var, range) in ranges {
            let var_name = self.env.symbol_pool().string(var.name);
            let quant_ty = self.env.get_node_type(range.node_id());
            match quant_ty.skip_reference() {
                Type::Vector(elem_ty) => {
                    let range_tmp = range_tmps.get(&var.name).unwrap();
                    let quant_var = quant_vars.get(&var.name).unwrap();
                    let suffix = boogie_type_suffix(&elem_ty);
                    emit!(
                        self.writer,
                        "(var {} := $Unbox{}(ReadVec({}, {}));\n",
                        var_name,
                        suffix,
                        range_tmp,
                        quant_var,
                    );
                }
                Type::Primitive(PrimitiveType::Range) => {
                    let quant_var = quant_vars.get(&var.name).unwrap();
                    emit!(self.writer, "(var {} := {};\n", var_name, quant_var);
                }
                Type::ResourceDomain(mid, sid, inst_opt) => {
                    let (addr_var, type_vars) = resource_vars.get(&var.name).unwrap();
                    let resource_name =
                        boogie_resource_memory_name(self.env, mid.qualified(*sid), &None);
                    let type_array = if let Some(inst) = inst_opt {
                        boogie_type_value_array(self.env, inst)
                    } else {
                        boogie_type_value_array_from_strings(type_vars)
                    };
                    emit!(
                        self.writer,
                        "(var {} := $ResourceValue({}, {}, {});\n",
                        var_name,
                        resource_name,
                        type_array,
                        addr_var
                    );
                }
                _ => (),
            }
        }
        f();
        emit!(
            self.writer,
            &std::iter::repeat(")")
                .take(usize::checked_add(range_tmps.len(), resource_vars.len()).unwrap())
                .collect::<String>()
        );
    }

    fn translate_quant(
        &self,
        _node_id: NodeId,
        kind: QuantKind,
        ranges: &[(LocalVarDecl, Exp)],
        triggers: &[Vec<Exp>],
        condition: &Option<Box<Exp>>,
        body: &Exp,
    ) {
        assert!(!kind.is_choice());
        // Translate range expressions.
        let mut range_tmps = HashMap::new();
        for (var, range) in ranges {
            let quant_ty = self.env.get_node_type(range.node_id());
            if matches!(
                quant_ty.skip_reference(),
                Type::Vector(..) | Type::Primitive(PrimitiveType::Range)
            ) {
                let range_tmp = self.fresh_var_name("range");
                emit!(self.writer, "(var {} := ", range_tmp);
                self.translate_exp(&range);
                emit!(self.writer, "; ");
                range_tmps.insert(var.name, range_tmp);
            }
        }
        // Translate quantified variables.
        emit!(self.writer, "({} ", kind);
        let mut quant_vars = HashMap::new();
        let mut resource_vars = HashMap::new();
        let mut comma = "";
        for (var, range) in ranges {
            let var_name = self.env.symbol_pool().string(var.name);
            let quant_ty = self.env.get_node_type(range.node_id());
            match quant_ty.skip_reference() {
                Type::TypeDomain(ty) => {
                    emit!(
                        self.writer,
                        "{}{}: {}",
                        comma,
                        var_name,
                        boogie_local_type(&ty)
                    );
                }
                Type::ResourceDomain(mid, sid, inst_opt) => {
                    let struct_env = self.env.get_module(*mid).into_struct(*sid);
                    let addr_quant_var = self.fresh_var_name("a");
                    let type_quant_vars = if inst_opt.is_some() {
                        vec![]
                    } else {
                        (0..struct_env.get_type_parameters().len())
                            .map(|i| self.fresh_var_name(&format!("ty{}", i)))
                            .collect_vec()
                    };
                    emit!(self.writer, "{}{}: int", comma, addr_quant_var);
                    comma = ", ";
                    for tvar in &type_quant_vars {
                        emit!(self.writer, "{}{}: $TypeValue", comma, tvar);
                    }
                    resource_vars.insert(var.name, (addr_quant_var, type_quant_vars));
                }
                _ => {
                    let quant_var = self.fresh_var_name("i");
                    emit!(self.writer, "{}{}: int", comma, quant_var);
                    quant_vars.insert(var.name, quant_var);
                }
            }
            comma = ", ";
        }
        emit!(self.writer, " :: ");
        // Translate triggers.
        if !triggers.is_empty() {
            for trigger in triggers {
                emit!(self.writer, "{");
                let mut comma = "";
                for p in trigger {
                    emit!(self.writer, "{}", comma);
                    self.with_range_selector_assignments(
                        &ranges,
                        &range_tmps,
                        &quant_vars,
                        &resource_vars,
                        || {
                            self.translate_exp(p);
                        },
                    );
                    comma = ",";
                }
                emit!(self.writer, "}");
            }
        } else {
            // Implicit triggers from ResourceDomain range.
            for (var, range) in ranges {
                let quant_ty = self.env.get_node_type(range.node_id());
                if let Type::ResourceDomain(mid, sid, inst_opt) = quant_ty.skip_reference() {
                    let (addr_var, type_vars) = resource_vars.get(&var.name).unwrap();
                    let resource_name =
                        boogie_resource_memory_name(self.env, mid.qualified(*sid), &None);
                    let type_array = if let Some(inst) = inst_opt {
                        boogie_type_value_array(self.env, inst)
                    } else {
                        boogie_type_value_array_from_strings(type_vars)
                    };
                    let resource_value = format!(
                        "$ResourceValue({}, {}, {})",
                        resource_name, type_array, addr_var
                    );
                    emit!(self.writer, "{{{}}}", resource_value);
                }
            }
        }
        // Translate range constraints.
        let connective = match kind {
            QuantKind::Forall => " ==> ",
            QuantKind::Exists => " && ",
            _ => unreachable!(),
        };
        let mut separator = "";
        for (var, range) in ranges {
            let var_name = self.env.symbol_pool().string(var.name);
            let quant_ty = self.env.get_node_type(range.node_id());
            match quant_ty.skip_reference() {
                Type::TypeDomain(domain_ty) => {
                    let mut type_check = boogie_well_formed_expr(self.env, &var_name, &domain_ty);
                    if type_check.is_empty() {
                        type_check = "true".to_string();
                    }
                    emit!(self.writer, "{}{}", separator, type_check);
                }
                Type::ResourceDomain(..) => {
                    // currently does not generate a constraint
                    continue;
                }
                Type::Vector(..) => {
                    let range_tmp = range_tmps.get(&var.name).unwrap();
                    let quant_var = quant_vars.get(&var.name).unwrap();
                    emit!(
                        self.writer,
                        "{}$InRangeVec({}, {})",
                        separator,
                        range_tmp,
                        quant_var,
                    );
                }
                Type::Primitive(PrimitiveType::Range) => {
                    let range_tmp = range_tmps.get(&var.name).unwrap();
                    let quant_var = quant_vars.get(&var.name).unwrap();
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
        emit!(self.writer, "{}", separator);
        self.with_range_selector_assignments(
            &ranges,
            &range_tmps,
            &quant_vars,
            &resource_vars,
            || {
                // Translate body and "where" condition.
                if let Some(cond) = condition {
                    emit!(self.writer, "(");
                    self.translate_exp(cond);
                    emit!(self.writer, ") {}", connective);
                }
                emit!(self.writer, "(");
                self.translate_exp(body);
                emit!(self.writer, ")");
            },
        );
        emit!(
            self.writer,
            &std::iter::repeat(")")
                .take(range_tmps.len().checked_add(1).unwrap())
                .collect::<String>()
        );
    }

    fn translate_eq_neq(&self, boogie_val_fun: &str, args: &[Exp]) {
        let suffix =
            boogie_type_suffix(&self.env.get_node_type(args[0].node_id()).skip_reference());
        emit!(self.writer, "{}{}(", boogie_val_fun, suffix);
        self.translate_exp(&args[0]);
        emit!(self.writer, ", ");
        self.translate_exp(&args[1]);
        emit!(self.writer, ")");
    }

    fn translate_identical(&self, args: &[Exp]) {
        use Exp::*;
        // If both arguments are &mut temporaries, we just directly make them equal. This allows
        // a more efficient representation of equality between $Mutation objects. Otherwise
        // we translate it the default way with automatic reference removal.
        match (&args[0], &args[1]) {
            (Temporary(_, idx1), Temporary(_, idx2)) => {
                emit!(self.writer, "$t{} == $t{}", idx1, idx2);
            }
            _ => self.translate_rel_op("==", args),
        }
    }

    fn translate_arith_op(&self, boogie_op: &str, args: &[Exp]) {
        emit!(self.writer, "(");
        self.translate_exp(&args[0]);
        emit!(self.writer, " {} ", boogie_op);
        self.translate_exp(&args[1]);
        emit!(self.writer, ")");
    }

    fn translate_rel_op(&self, boogie_op: &str, args: &[Exp]) {
        emit!(self.writer, "(");
        self.translate_exp(&args[0]);
        emit!(self.writer, " {} ", boogie_op);
        self.translate_exp(&args[1]);
        emit!(self.writer, ")");
    }

    fn translate_logical_op(&self, boogie_op: &str, args: &[Exp]) {
        emit!(self.writer, "(");
        self.translate_exp(&args[0]);
        emit!(self.writer, " {} ", boogie_op);
        self.translate_exp(&args[1]);
        emit!(self.writer, ")");
    }

    fn translate_logical_unary_op(&self, boogie_op: &str, args: &[Exp]) {
        emit!(self.writer, "{}", boogie_op);
        self.translate_exp(&args[0]);
    }

    fn translate_primitive_call(&self, fun: &str, args: &[Exp]) {
        emit!(self.writer, "{}(", fun);
        self.translate_seq(args.iter(), ", ", |e| self.translate_exp(e));
        emit!(self.writer, ")");
    }

    fn translate_in_range(&self, args: &[Exp]) {
        // Only difference to primitive call is swapped argument order.
        emit!(self.writer, "InRangeVec(");
        self.translate_exp(&args[1]);
        emit!(self.writer, ", ");
        self.translate_exp(&args[0]);
        emit!(self.writer, ")");
    }

    fn translate_well_formed(&self, exp: &Exp) {
        let ty = self.env.get_node_type(exp.node_id());
        match exp {
            Exp::Temporary(_, idx) => {
                // For the special case of a temporary which can represent a
                // &mut, skip the normal translation of `exp` which would do automatic
                // dereferencing. Instead let boogie_well_formed_expr handle the
                // the dereferencing as part of its logic.
                let check = boogie_well_formed_expr(self.env, &format!("$t{}", idx), &ty);
                if !check.is_empty() {
                    emit!(self.writer, &check);
                } else {
                    emit!(self.writer, "true");
                }
            }
            _ => {
                let check = boogie_well_formed_expr(self.env, "$val", ty.skip_reference());
                if !check.is_empty() {
                    emit!(self.writer, "(var $val := ");
                    self.translate_exp(exp);
                    emit!(self.writer, "; {})", check);
                } else {
                    emit!(self.writer, "true");
                }
            }
        }
    }

    fn translate_unbox_value(&self, id: NodeId, exp: &Exp) {
        let ty = self.env.get_node_type(id);
        let suffix = boogie_type_suffix(&ty);
        emit!(self.writer, "$Unbox{}(", suffix);
        self.translate_exp(exp);
        emit!(self.writer, ")");
    }

    fn translate_box_value(&self, id: NodeId, exp: &Exp) {
        let ty = self.env.get_node_type(id);
        let suffix = boogie_type_suffix(&ty);
        emit!(self.writer, "$Box{}(", suffix);
        self.translate_exp(exp);
        emit!(self.writer, ")");
    }
}

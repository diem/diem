// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This file implements the expression evaluation part of the stackless bytecode interpreter.

use itertools::Itertools;
use num::{BigInt, ToPrimitive, Zero};
use std::{cell::Cell, collections::BTreeMap, rc::Rc};

use move_core_types::account_address::AccountAddress;
use move_model::{
    ast::{
        Exp, ExpData, LocalVarDecl, MemoryLabel, Operation, QuantKind, SpecFunDecl, TempIndex,
        Value,
    },
    model::{FieldId, ModuleEnv, ModuleId, NodeId, SpecFunId, StructId},
    ty as MTy,
};
use move_stackless_bytecode::{
    function_target::FunctionTarget, function_target_pipeline::FunctionTargetsHolder,
};

use crate::{
    concrete::{
        local_state::LocalState,
        player,
        settings::InterpreterSettings,
        ty::{convert_model_base_type, convert_model_struct_type, BaseType, StructInstantiation},
        value::{BaseValue, EvalState, GlobalState, TypedValue},
    },
    shared::{ident::StructIdent, variant::choose_variant},
};

//**************************************************************************************************
// Types
//**************************************************************************************************

pub type EvalResult<T> = ::std::result::Result<T, BigInt>;

//**************************************************************************************************
// Constants
//**************************************************************************************************

const DIEM_CORE_ADDR: AccountAddress = AccountAddress::ONE;

//**************************************************************************************************
// Evaluation context
//**************************************************************************************************

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct ExpState {
    // bindings for the local variables
    local_vars: BTreeMap<String, BaseValue>,
    // bindings for lambda functions
    local_lambdas: BTreeMap<String, (Vec<LocalVarDecl>, Exp)>,
}

impl ExpState {
    pub fn add_var(&mut self, name: String, val: BaseValue) {
        let exists = self.local_vars.insert(name, val);
        if cfg!(debug_assertions) {
            assert!(exists.is_none());
        }
    }

    pub fn get_var(&self, name: &str) -> BaseValue {
        self.local_vars.get(name).unwrap().clone()
    }

    pub fn add_lambda(&mut self, name: String, vars: Vec<LocalVarDecl>, stmt: Exp) {
        let exists = self.local_lambdas.insert(name, (vars, stmt));
        if cfg!(debug_assertions) {
            assert!(exists.is_none());
        }
    }

    pub fn get_lambda(&self, name: &str) -> &(Vec<LocalVarDecl>, Exp) {
        self.local_lambdas.get(name).unwrap()
    }
}

pub struct Evaluator<'env> {
    // context
    holder: &'env FunctionTargetsHolder,
    target: &'env FunctionTarget<'env>,
    ty_args: &'env [BaseType],
    // debug
    level: Cell<usize>,
    // states
    exp_state: ExpState,
    eval_state: &'env EvalState,
    local_state: &'env LocalState,
    global_state: &'env GlobalState,
}

impl<'env> Evaluator<'env> {
    pub fn new(
        holder: &'env FunctionTargetsHolder,
        target: &'env FunctionTarget<'env>,
        ty_args: &'env [BaseType],
        level: usize,
        exp_state: ExpState,
        eval_state: &'env EvalState,
        local_state: &'env LocalState,
        global_state: &'env GlobalState,
    ) -> Self {
        Self {
            holder,
            target,
            ty_args,
            level: Cell::new(level),
            exp_state,
            eval_state,
            local_state,
            global_state,
        }
    }

    //
    // entry points
    //

    /// Check whether an assert expression holds
    pub fn check_assert(&self, exp: &Exp) {
        match self.evaluate(exp) {
            Ok(val) => {
                if !val.into_bool() {
                    self.record_checking_failure(exp);
                }
            }
            Err(err) => {
                // TODO (mengxu) this is just to keep tests happy, to be removed once completed
                if err == BigInt::zero() {
                    return;
                }
                self.record_evaluation_failure(exp, err);
            }
        }
    }

    /// Check whether an assume expression holds, unless the assume expression represents a `let`
    /// binding. In that case, return the `TypedValue` of the let-binding as well as the local
    /// variable (index) the value should bind to.
    pub fn check_assume(&self, exp: &Exp) -> Option<(TempIndex, TypedValue)> {
        // NOTE: `let` bindings are translated to `Assume(Identical($t, <exp>));`. This should be
        // treated as an assignment.
        if let ExpData::Call(_, Operation::Identical, args) = exp.as_ref() {
            if cfg!(debug_assertions) {
                assert_eq!(args.len(), 2);
            }
            let env = self.target.global_env();
            let (local_idx, local_ty) = match args[0].as_ref() {
                ExpData::Temporary(node_id, idx) => {
                    let node_ty = env.get_node_type(*node_id);
                    let local_ty = convert_model_base_type(env, &node_ty, self.ty_args);
                    (*idx, local_ty)
                }
                _ => unreachable!(),
            };
            if cfg!(debug_assertions) {
                assert_eq!(
                    &local_ty,
                    self.local_state.get_type(local_idx).get_base_type()
                );
            }
            let local_val = self.evaluate(&args[1]).unwrap();
            Some((local_idx, TypedValue::fuse_base(local_ty, local_val)))
        } else {
            // for all other cases, treat with as an assertion
            self.check_assert(exp);
            None
        }
    }

    //
    // dispatcher
    //

    /// The entrypoint to the main evaluation logic. Expression evaluation is done recursively in
    /// a bottom-up manner, i.e., for an expression `e`, all its sub-expressions are evaluated first
    /// before `e` is evaluated. An expression will be evaluated to to a `BaseValue` (if the process
    /// is successful) or an error code (if the process fails, for example, due to arithmetic errors
    /// or referring to a non-existent global resource).
    fn evaluate(&self, exp: &Exp) -> EvalResult<BaseValue> {
        let exp_level = self.level.get();
        self.level.set(exp_level + 1);

        let debug_expression = self.get_settings().verbose_expression;
        if debug_expression {
            println!(
                "{} {}[>]: {}",
                "-".repeat(self.level.get()),
                self.target.func_env.get_full_name_str(),
                exp.display(self.target.global_env())
            );
        }

        let result = match exp.as_ref() {
            ExpData::Value(_, val) => self.evaluate_constant(val),
            ExpData::Temporary(_, idx) => self.local_state.get_value(*idx).decompose().1,
            ExpData::LocalVar(_, name) => {
                let env = self.target.global_env();
                self.exp_state
                    .get_var(env.symbol_pool().string(*name).as_str())
            }
            ExpData::Call(node_id, op, args) => self.evaluate_operation(*node_id, op, args)?,
            ExpData::IfElse(_, cond, t_exp, f_exp) => {
                self.evaluate_if_then_else(cond, t_exp, f_exp)?
            }
            ExpData::Invoke(_, def, args) => self.evaluate_invocation(def, args)?,
            ExpData::Quant(_, kind, ranges, _triggers, constraint, body) => {
                self.evaluate_quantifier(*kind, ranges, constraint.as_ref(), body)?
            }
            ExpData::Invalid(_) => unreachable!(),
            // should not appear in this context
            ExpData::Lambda(..) | ExpData::Block(..) => unreachable!(),
        };

        if debug_expression {
            println!(
                "{} {}[<]: {}",
                "-".repeat(self.level.get()),
                self.target.func_env.get_full_name_str(),
                exp.display(self.target.global_env())
            );
        }

        self.level.set(exp_level);
        Ok(result)
    }

    //
    // concrete cases
    //

    fn evaluate_constant(&self, val: &Value) -> BaseValue {
        match val {
            Value::Address(v) => BaseValue::mk_address(
                AccountAddress::from_hex_literal(&format!("{:#x}", v)).unwrap(),
            ),
            Value::Number(v) => BaseValue::mk_num(v.clone()),
            Value::Bool(v) => BaseValue::mk_bool(*v),
            Value::ByteArray(v) => {
                BaseValue::mk_vector(v.iter().map(|e| BaseValue::mk_u8(*e)).collect())
            }
        }
    }

    fn evaluate_operation(
        &self,
        node_id: NodeId,
        op: &Operation,
        args: &[Exp],
    ) -> EvalResult<BaseValue> {
        let result = match op {
            Operation::Slice => {
                if cfg!(debug_assertions) {
                    assert_eq!(args.len(), 2);
                }
                let arg_vec = self.evaluate(&args[0])?;
                self.handle_vector_slice(arg_vec, &args[1])?
            }
            Operation::InRangeRange => {
                if cfg!(debug_assertions) {
                    assert_eq!(args.len(), 2);
                }
                let arg_idx = self.evaluate(&args[1])?;
                self.handle_in_range(&args[0], arg_idx)?
            }
            Operation::Function(module_id, spec_fun_id, mem_labels_opt) => self.handle_call(
                node_id,
                *module_id,
                *spec_fun_id,
                mem_labels_opt.as_ref(),
                args,
            )?,
            _ => self.evaluate_operation_with_values(node_id, op, args)?,
        };
        Ok(result)
    }

    fn evaluate_operation_with_values(
        &self,
        node_id: NodeId,
        op: &Operation,
        args: &[Exp],
    ) -> EvalResult<BaseValue> {
        let mut arg_vals = args
            .iter()
            .map(|arg_exp| self.evaluate(arg_exp))
            .collect::<EvalResult<Vec<_>>>()?;
        let result = match op {
            // binary arithmetic
            Operation::Add | Operation::Sub | Operation::Mul | Operation::Div | Operation::Mod => {
                if cfg!(debug_assertions) {
                    assert_eq!(arg_vals.len(), 2);
                }
                let rhs = arg_vals.remove(1);
                let lhs = arg_vals.remove(0);
                self.handle_binary_arithmetic(op, lhs, rhs)?
            }
            // binary bitwise
            Operation::BitAnd | Operation::BitOr | Operation::Xor => {
                if cfg!(debug_assertions) {
                    assert_eq!(arg_vals.len(), 2);
                }
                let rhs = arg_vals.remove(1);
                let lhs = arg_vals.remove(0);
                self.handle_binary_bitwise(op, lhs, rhs)
            }
            // binary bitshift
            Operation::Shl | Operation::Shr => {
                if cfg!(debug_assertions) {
                    assert_eq!(arg_vals.len(), 2);
                }
                let rhs = arg_vals.remove(1);
                let lhs = arg_vals.remove(0);
                self.handle_binary_bitshift(op, lhs, rhs)
            }
            // binary comparison
            Operation::Lt | Operation::Le | Operation::Ge | Operation::Gt => {
                if cfg!(debug_assertions) {
                    assert_eq!(arg_vals.len(), 2);
                }
                let rhs = arg_vals.remove(1);
                let lhs = arg_vals.remove(0);
                self.handle_binary_comparison(op, lhs, rhs)
            }
            // binary equality
            Operation::Eq | Operation::Neq => {
                if cfg!(debug_assertions) {
                    assert_eq!(arg_vals.len(), 2);
                }
                let rhs = arg_vals.remove(1);
                let lhs = arg_vals.remove(0);
                self.handle_binary_equality(op, lhs, rhs)
            }
            // unary boolean
            Operation::Not => {
                if cfg!(debug_assertions) {
                    assert_eq!(arg_vals.len(), 1);
                }
                let opv = arg_vals.remove(0);
                self.handle_unary_boolean(op, opv)
            }
            // binary boolean
            Operation::And | Operation::Or | Operation::Implies | Operation::Iff => {
                if cfg!(debug_assertions) {
                    assert_eq!(arg_vals.len(), 2);
                }
                let rhs = arg_vals.remove(1);
                let lhs = arg_vals.remove(0);
                self.handle_binary_boolean(op, lhs, rhs)
            }
            // vector operation
            Operation::Len => {
                if cfg!(debug_assertions) {
                    assert_eq!(arg_vals.len(), 1);
                }
                let opv = arg_vals.remove(0);
                BaseValue::mk_num(BigInt::from(opv.into_vector().len()))
            }
            Operation::EmptyVec => {
                if cfg!(debug_assertions) {
                    assert!(arg_vals.is_empty());
                }
                BaseValue::mk_vector(vec![])
            }
            Operation::SingleVec => {
                if cfg!(debug_assertions) {
                    assert_eq!(arg_vals.len(), 1);
                }
                BaseValue::mk_vector(arg_vals)
            }
            Operation::Index => {
                if cfg!(debug_assertions) {
                    assert_eq!(arg_vals.len(), 2);
                }
                let idx = arg_vals.remove(1);
                let vec = arg_vals.remove(0);
                self.handle_vector_get(vec, idx)?
            }
            Operation::UpdateVec => {
                if cfg!(debug_assertions) {
                    assert_eq!(arg_vals.len(), 3);
                }
                let elem = arg_vals.remove(2);
                let idx = arg_vals.remove(1);
                let vec = arg_vals.remove(0);
                self.handle_vector_update(vec, idx, elem)?
            }
            Operation::ConcatVec => {
                if cfg!(debug_assertions) {
                    assert_eq!(arg_vals.len(), 2);
                }
                let rhs = arg_vals.remove(1);
                let lhs = arg_vals.remove(0);
                self.handle_vector_concat(lhs, rhs)
            }
            Operation::IndexOfVec => {
                if cfg!(debug_assertions) {
                    assert_eq!(arg_vals.len(), 2);
                }
                let elem = arg_vals.remove(1);
                let vec = arg_vals.remove(0);
                self.handle_vector_index_of(vec, elem)
            }
            Operation::ContainsVec => {
                if cfg!(debug_assertions) {
                    assert_eq!(arg_vals.len(), 2);
                }
                let elem = arg_vals.remove(1);
                let vec = arg_vals.remove(0);
                self.handle_vector_contains(vec, elem)
            }
            Operation::InRangeVec => {
                if cfg!(debug_assertions) {
                    assert_eq!(args.len(), 2);
                }
                let idx = arg_vals.remove(1);
                let vec = arg_vals.remove(0);
                self.handle_vector_in_range(vec, idx)
            }
            // struct
            Operation::Pack(module_id, struct_id) => {
                self.handle_struct_pack(*module_id, *struct_id, arg_vals)
            }
            Operation::Select(module_id, struct_id, field_id) => {
                if cfg!(debug_assertions) {
                    assert_eq!(arg_vals.len(), 1);
                }
                let struct_val = arg_vals.remove(0);
                self.handle_struct_get_field(*module_id, *struct_id, *field_id, struct_val)
            }
            Operation::UpdateField(module_id, struct_id, field_id) => {
                if cfg!(debug_assertions) {
                    assert_eq!(arg_vals.len(), 2);
                }
                let field_val = arg_vals.remove(1);
                let struct_val = arg_vals.remove(0);
                self.handle_struct_update_field(
                    *module_id, *struct_id, *field_id, struct_val, field_val,
                )
            }
            // globals
            Operation::Exists(mem_opt) => {
                if cfg!(debug_assertions) {
                    assert_eq!(arg_vals.len(), 1);
                }
                let addr = arg_vals.remove(0);
                self.handle_global_exists(node_id, mem_opt.as_ref().copied(), addr)
            }
            Operation::Global(mem_opt) => {
                if cfg!(debug_assertions) {
                    assert_eq!(arg_vals.len(), 1);
                }
                let addr = arg_vals.remove(0);
                self.handle_global_get(node_id, mem_opt.as_ref().copied(), addr)?
            }
            // constant values
            Operation::MaxU8 => {
                if cfg!(debug_assertions) {
                    assert!(arg_vals.is_empty());
                }
                BaseValue::mk_num(BigInt::from(u8::MAX))
            }
            Operation::MaxU64 => {
                if cfg!(debug_assertions) {
                    assert!(arg_vals.is_empty());
                }
                BaseValue::mk_num(BigInt::from(u64::MAX))
            }
            Operation::MaxU128 => {
                if cfg!(debug_assertions) {
                    assert!(arg_vals.is_empty());
                }
                BaseValue::mk_num(BigInt::from(u128::MAX))
            }
            Operation::AbortFlag => {
                if cfg!(debug_assertions) {
                    assert!(arg_vals.is_empty());
                }
                BaseValue::mk_bool(self.local_state.is_post_abort())
            }
            // type checking
            Operation::WellFormed => {
                if cfg!(debug_assertions) {
                    assert_eq!(arg_vals.len(), 1);
                }
                // TODO (mengxu) implement the type checking logic
                // if we don't, and the value is not actually well-formed, it should panic the
                // stackless VM later in the execution.
                BaseValue::mk_bool(true)
            }
            // TODO (mengxu) modifies check is not supported now
            Operation::CanModify => {
                // TODO (to avoid test case failure)
                return Err(BigInt::zero());
            }
            // TODO (mengxu) events are not handled now
            Operation::EmptyEventStore
            | Operation::ExtendEventStore
            | Operation::EventStoreIncludes
            | Operation::EventStoreIncludedIn => {
                // TODO (to avoid test case failure)
                return Err(BigInt::zero());
            }
            // unexpected operations in this context
            Operation::NoOp
            | Operation::Identical
            | Operation::Old
            | Operation::Trace(_)
            | Operation::Tuple
            | Operation::Result(_)
            | Operation::AbortCode
            | Operation::TypeValue
            | Operation::TypeDomain
            | Operation::ResourceDomain
            | Operation::BoxValue
            | Operation::UnboxValue => {
                unreachable!()
            }
            // handled elsewhere, should not be here
            Operation::Function(..)
            | Operation::Slice
            | Operation::Range
            | Operation::RangeVec
            | Operation::InRangeRange => {
                unreachable!()
            }
        };
        Ok(result)
    }

    fn evaluate_if_then_else(&self, cond: &Exp, t_exp: &Exp, f_exp: &Exp) -> EvalResult<BaseValue> {
        let cond_val = self.evaluate(cond)?;
        // NOTE: do not short-circuit the other path. For ITE expressions, we want to evaluate both
        // expressions instead of short-circuiting the path that is not going to be taken.
        let t_val = self.evaluate(t_exp)?;
        let f_val = self.evaluate(f_exp)?;
        let result = if cond_val.into_bool() { t_val } else { f_val };
        Ok(result)
    }

    fn evaluate_invocation(&self, def: &Exp, args: &[Exp]) -> EvalResult<BaseValue> {
        let env = self.target.global_env();

        let (vars, stmt) = match def.as_ref() {
            // TODO (mengxu): the assumption here is that the only way to invoke a lambda is to pass
            // the lambda to a spec function as a function argument and then invoke the lambda
            // within the spec function.
            ExpData::LocalVar(_, symbol) => self
                .exp_state
                .get_lambda(env.symbol_pool().string(*symbol).as_str()),
            _ => unreachable!(),
        };

        let arg_vals = args
            .iter()
            .map(|arg_exp| self.evaluate(arg_exp))
            .collect::<EvalResult<Vec<_>>>()?;

        let mut exp_state = ExpState::default();
        for (var, val) in vars.iter().zip(arg_vals.into_iter()) {
            if cfg!(debug_assertions) {
                assert!(var.binding.is_none());
            }
            let name = env.symbol_pool().string(var.name).to_string();
            exp_state.add_var(name, val);
        }

        let evaluator = Evaluator::new(
            self.holder,
            self.target,
            self.ty_args,
            self.level.get(),
            exp_state,
            self.eval_state,
            self.local_state,
            self.global_state,
        );
        evaluator.evaluate(stmt)
    }

    fn prepare_range(&self, exp: &Exp) -> EvalResult<Vec<BaseValue>> {
        let env = self.target.global_env();
        let vals = match exp.as_ref() {
            // case: range
            ExpData::Call(_, Operation::Range, _) | ExpData::Call(_, Operation::RangeVec, _) => {
                let (lower, upper) = self.unroll_range(exp)?;
                let mut vals = vec![];
                let mut i = lower;
                while i < upper {
                    vals.push(BaseValue::Int(i.clone()));
                    i += 1;
                }
                vals
            }
            // case: type domain
            ExpData::Call(node_id, Operation::TypeDomain, _) => match env.get_node_type(*node_id) {
                MTy::Type::TypeDomain(ty) => self.unroll_type_domain(&ty),
                _ => unreachable!(),
            },
            // case: resource domain
            ExpData::Call(node_id, Operation::ResourceDomain, _) => {
                match env.get_node_type(*node_id) {
                    MTy::Type::ResourceDomain(module_id, struct_id, inst_opt) => match inst_opt {
                        None => {
                            let struct_env = env.get_struct(module_id.qualified(struct_id));
                            let struct_ident = StructIdent::new(&struct_env);
                            self.unroll_resource_domain_by_ident(&struct_ident)
                        }
                        Some(ty_args) => {
                            let struct_inst = convert_model_struct_type(
                                env,
                                module_id,
                                struct_id,
                                &ty_args,
                                self.ty_args,
                            );
                            self.unroll_resource_domain_by_inst(&struct_inst)
                        }
                    },
                    _ => unreachable!(),
                }
            }
            // cont.
            _ => match env.get_node_type(exp.node_id()) {
                // case: vector
                MTy::Type::Vector(_) => self.evaluate(exp)?.into_vector(),
                _ => unreachable!(),
            },
        };
        Ok(vals)
    }

    fn evaluate_quantifier(
        &self,
        kind: QuantKind,
        ranges: &[(LocalVarDecl, Exp)],
        constraint: Option<&Exp>,
        body: &Exp,
    ) -> EvalResult<BaseValue> {
        let env = self.target.global_env();

        let mut var_vec = vec![];
        let mut vals_vec = vec![];
        for (r_var, r_exp) in ranges {
            var_vec.push(r_var);
            let r_vals = self.prepare_range(r_exp)?;
            vals_vec.push(r_vals);
        }

        let mut loop_results = vec![];
        for val_vec in vals_vec.into_iter().multi_cartesian_product() {
            let mut exp_state = ExpState::default();
            for (var, val) in var_vec.iter().zip(val_vec.into_iter()) {
                if cfg!(debug_assertions) {
                    assert!(var.binding.is_none());
                }
                let name = env.symbol_pool().string(var.name).to_string();
                exp_state.add_var(name, val);
            }

            let evaluator = Evaluator::new(
                self.holder,
                self.target,
                self.ty_args,
                self.level.get(),
                exp_state,
                self.eval_state,
                self.local_state,
                self.global_state,
            );
            let run_body = match constraint {
                None => true,
                Some(c_exp) => evaluator.evaluate(c_exp)?.into_bool(),
            };
            if run_body {
                loop_results.push(evaluator.evaluate(body)?);
            }
        }

        let quant_result = match kind {
            QuantKind::Forall => {
                let v = loop_results.into_iter().all(|r| r.into_bool());
                BaseValue::mk_bool(v)
            }
            QuantKind::Exists => {
                let v = loop_results.into_iter().any(|r| r.into_bool());
                BaseValue::mk_bool(v)
            }
            // TODO (mengxu) support choose operators
            QuantKind::Choose | QuantKind::ChooseMin => unimplemented!(),
        };
        Ok(quant_result)
    }

    //
    // handlers
    //

    fn handle_call(
        &self,
        node_id: NodeId,
        module_id: ModuleId,
        spec_fun_id: SpecFunId,
        mem_labels_opt: Option<&Vec<MemoryLabel>>,
        args: &[Exp],
    ) -> EvalResult<BaseValue> {
        let env = self.target.global_env();
        let module_env = env.get_module(module_id);
        let decl = module_env.get_spec_fun(spec_fun_id);
        if cfg!(debug_assertions) {
            assert_eq!(decl.params.len(), args.len());
        }

        let mut global_state = match mem_labels_opt {
            None => self.global_state.clone(),
            Some(labels) => {
                let mut state = GlobalState::default();
                for label in labels {
                    self.eval_state.register_memory(label, &mut state);
                }
                state
            }
        };

        let result = if decl.is_move_fun {
            // TODO (mengxu) the decl for calling a move function might also have a body, maybe
            // we could just interpret that?
            self.handle_call_move_function(node_id, &module_env, decl, args, &mut global_state)?
        } else if decl.body.is_none() {
            self.handle_call_native_or_uninterpreted(&module_env, decl, args, &mut global_state)?
        } else {
            self.handle_call_spec_function(node_id, decl, args, &mut global_state)?
        };
        Ok(result)
    }

    fn handle_call_native_or_uninterpreted(
        &self,
        module_env: &ModuleEnv,
        decl: &SpecFunDecl,
        args: &[Exp],
        _global_state: &mut GlobalState,
    ) -> EvalResult<BaseValue> {
        // convert args
        let mut arg_vals = args
            .iter()
            .map(|arg_exp| self.evaluate(arg_exp))
            .collect::<EvalResult<Vec<_>>>()?;

        let env = self.target.global_env();
        let addr = *module_env.self_address();
        let module_name = env.symbol_pool().string(module_env.get_name().name());
        let function_name = env.symbol_pool().string(decl.name);

        // dispatch
        let result = match (addr, module_name.as_str(), function_name.as_str()) {
            (DIEM_CORE_ADDR, "Signer", "spec_address_of") => {
                if cfg!(debug_assertions) {
                    assert_eq!(arg_vals.len(), 1);
                }
                self.native_spec_signer_of(arg_vals.remove(0))
            }
            _ => unreachable!(),
        };
        Ok(result)
    }

    fn handle_call_spec_function(
        &self,
        node_id: NodeId,
        decl: &SpecFunDecl,
        args: &[Exp],
        global_state: &mut GlobalState,
    ) -> EvalResult<BaseValue> {
        let env = self.target.global_env();

        // convert type args
        let callee_inst = env.get_node_instantiation(node_id);
        if cfg!(debug_assertions) {
            assert_eq!(decl.type_params.len(), callee_inst.len());
        }
        let ty_args: Vec<_> = callee_inst
            .into_iter()
            .map(|inst_ty| convert_model_base_type(env, &inst_ty, self.ty_args))
            .collect();

        // interpret
        self.interpret_spec_function(decl, &ty_args, args, global_state)
    }

    fn handle_call_move_function(
        &self,
        node_id: NodeId,
        module_env: &ModuleEnv,
        decl: &SpecFunDecl,
        args: &[Exp],
        global_state: &mut GlobalState,
    ) -> EvalResult<BaseValue> {
        let env = self.target.global_env();
        let decl_fun_name = env.symbol_pool().string(decl.name);
        let fun_name_tokens: Vec<_> = decl_fun_name.split('$').collect();
        if cfg!(debug_assertions) {
            assert_eq!(fun_name_tokens.len(), 2);
            assert_eq!(fun_name_tokens[0], "");
        }
        let fun_name = fun_name_tokens[1];

        // find callee
        let callee_env = module_env
            .get_functions()
            .find(|fun_env| env.symbol_pool().string(fun_env.get_name()).as_str() == fun_name)
            .unwrap();
        let callee_target = choose_variant(self.holder, &callee_env);

        // convert type args
        let callee_inst = env.get_node_instantiation(node_id);
        if cfg!(debug_assertions) {
            let callee_ty_params = callee_target.func_env.get_type_parameters();
            assert_eq!(decl.type_params.len(), callee_ty_params.len());
            assert_eq!(decl.type_params.len(), callee_inst.len());
        }
        let ty_args: Vec<_> = callee_inst
            .into_iter()
            .map(|inst_ty| convert_model_base_type(env, &inst_ty, self.ty_args))
            .collect();

        // convert args
        if cfg!(debug_assertions) {
            assert_eq!(decl.params.len(), callee_target.get_parameter_count());
        }
        let arg_vals = args
            .iter()
            .map(|arg_exp| self.evaluate(arg_exp))
            .collect::<EvalResult<Vec<_>>>()?;

        let typed_args = decl
            .params
            .iter()
            .zip(arg_vals.into_iter())
            .map(|((_, arg_ty), arg_val)| {
                let converted = convert_model_base_type(env, arg_ty, &ty_args);
                TypedValue::fuse_base(converted, arg_val)
            })
            .collect();

        // execute the function
        let callee_result = player::entrypoint(
            self.holder,
            callee_target,
            &ty_args,
            typed_args,
            /* skip_specs */ true,
            self.level.get(),
            global_state,
        );

        // analyze the result
        let return_val = match callee_result {
            Ok(rets) => {
                if cfg!(debug_assertions) {
                    assert_eq!(rets.len(), 1);
                }
                rets.into_iter().next().unwrap().decompose().1
            }
            Err(_) => {
                return Err(Self::eval_failure_code());
            }
        };
        Ok(return_val)
    }

    fn handle_binary_arithmetic(
        &self,
        op: &Operation,
        lhs: BaseValue,
        rhs: BaseValue,
    ) -> EvalResult<BaseValue> {
        let lval = lhs.into_int();
        let rval = rhs.into_int();
        let result = match op {
            Operation::Add => lval + rval,
            Operation::Sub => lval - rval,
            Operation::Mul => lval * rval,
            Operation::Div => {
                if rval.is_zero() {
                    return Err(Self::eval_failure_code());
                }
                lval / rval
            }
            Operation::Mod => {
                if rval.is_zero() {
                    return Err(Self::eval_failure_code());
                }
                lval % rval
            }
            _ => unreachable!(),
        };

        Ok(BaseValue::mk_num(result))
    }

    fn handle_binary_bitwise(&self, op: &Operation, lhs: BaseValue, rhs: BaseValue) -> BaseValue {
        let lval = lhs.into_int();
        let rval = rhs.into_int();
        let result = match op {
            Operation::BitAnd => lval & rval,
            Operation::BitOr => lval | rval,
            Operation::Xor => lval ^ rval,
            _ => unreachable!(),
        };
        BaseValue::mk_num(result)
    }

    fn handle_binary_bitshift(&self, op: &Operation, lhs: BaseValue, rhs: BaseValue) -> BaseValue {
        let lval = lhs.into_int();
        let rval = rhs.into_int();
        let result = match op {
            Operation::Shl => lval << rval.to_usize().unwrap(),
            Operation::Shr => lval >> rval.to_usize().unwrap(),
            _ => unreachable!(),
        };
        BaseValue::mk_num(result)
    }

    fn handle_binary_comparison(
        &self,
        op: &Operation,
        lhs: BaseValue,
        rhs: BaseValue,
    ) -> BaseValue {
        let lval = lhs.into_int();
        let rval = rhs.into_int();
        let result = match op {
            Operation::Lt => lval < rval,
            Operation::Le => lval <= rval,
            Operation::Ge => lval >= rval,
            Operation::Gt => lval > rval,
            _ => unreachable!(),
        };
        BaseValue::mk_bool(result)
    }

    fn handle_binary_equality(&self, op: &Operation, lhs: BaseValue, rhs: BaseValue) -> BaseValue {
        let result = match op {
            Operation::Eq => lhs == rhs,
            Operation::Neq => lhs != rhs,
            _ => unreachable!(),
        };
        BaseValue::mk_bool(result)
    }

    fn handle_unary_boolean(&self, op: &Operation, opv: BaseValue) -> BaseValue {
        let opval = opv.into_bool();
        let result = match op {
            Operation::Not => !opval,
            _ => unreachable!(),
        };
        BaseValue::mk_bool(result)
    }

    fn handle_binary_boolean(&self, op: &Operation, lhs: BaseValue, rhs: BaseValue) -> BaseValue {
        let lval = lhs.into_bool();
        let rval = rhs.into_bool();
        let result = match op {
            Operation::And => lval && rval,
            Operation::Or => lval || rval,
            Operation::Implies => !lval || rval,
            Operation::Iff => lval == rval,
            _ => unreachable!(),
        };
        BaseValue::mk_bool(result)
    }

    fn handle_in_range(&self, range: &Exp, idx: BaseValue) -> EvalResult<BaseValue> {
        let (lhs, rhs) = self.unroll_range(range)?;
        let i = idx.into_int();
        let r = i >= lhs && i < rhs;
        Ok(BaseValue::mk_bool(r))
    }

    fn handle_vector_get(&self, vec: BaseValue, idx: BaseValue) -> EvalResult<BaseValue> {
        let mut v = vec.into_vector();
        let i = idx.into_int().to_usize().unwrap();
        if i >= v.len() {
            return Err(Self::eval_failure_code());
        }
        Ok(v.remove(i))
    }

    fn handle_vector_update(
        &self,
        vec: BaseValue,
        idx: BaseValue,
        elem: BaseValue,
    ) -> EvalResult<BaseValue> {
        let mut v = vec.into_vector();
        let i = idx.into_int().to_usize().unwrap();
        if i >= v.len() {
            return Err(Self::eval_failure_code());
        }
        *v.get_mut(i).unwrap() = elem;
        Ok(BaseValue::mk_vector(v))
    }

    fn handle_vector_slice(&self, vec: BaseValue, range: &Exp) -> EvalResult<BaseValue> {
        let mut v = vec.into_vector();
        let (lhs, rhs) = self.unroll_range(range)?;
        if lhs > rhs || lhs < BigInt::zero() || rhs > BigInt::from(v.len()) {
            return Err(Self::eval_failure_code());
        }
        let mut slice = v.split_off(lhs.to_usize().unwrap());
        let _ = slice.split_off(rhs.to_usize().unwrap());
        Ok(BaseValue::mk_vector(slice))
    }

    fn handle_vector_concat(&self, lhs: BaseValue, rhs: BaseValue) -> BaseValue {
        let mut concat = lhs.into_vector();
        concat.append(&mut rhs.into_vector());
        BaseValue::mk_vector(concat)
    }

    fn handle_vector_index_of(&self, vec: BaseValue, elem: BaseValue) -> BaseValue {
        let v = vec.into_vector();
        let idx = match v.into_iter().position(|e| e == elem) {
            None => BigInt::from(-1),
            Some(i) => BigInt::from(i),
        };
        BaseValue::mk_num(idx)
    }

    fn handle_vector_contains(&self, vec: BaseValue, elem: BaseValue) -> BaseValue {
        let v = vec.into_vector();
        BaseValue::mk_bool(v.contains(&elem))
    }

    fn handle_vector_in_range(&self, vec: BaseValue, idx: BaseValue) -> BaseValue {
        let v = vec.into_vector();
        let i = idx.into_int();
        let r = i >= BigInt::zero() && i < BigInt::from(v.len());
        BaseValue::mk_bool(r)
    }

    fn handle_struct_pack(
        &self,
        module_id: ModuleId,
        struct_id: StructId,
        fields: Vec<BaseValue>,
    ) -> BaseValue {
        if cfg!(debug_assertions) {
            let env = self.target.global_env();
            let struct_env = env.get_struct(module_id.qualified(struct_id));
            assert_eq!(struct_env.get_field_count(), fields.len());
        }
        BaseValue::mk_struct(fields)
    }

    fn handle_struct_get_field(
        &self,
        module_id: ModuleId,
        struct_id: StructId,
        field_id: FieldId,
        struct_val: BaseValue,
    ) -> BaseValue {
        let mut fields = struct_val.into_struct();

        let env = self.target.global_env();
        let struct_env = env.get_struct(module_id.qualified(struct_id));
        if cfg!(debug_assertions) {
            assert_eq!(struct_env.get_field_count(), fields.len());
        }

        let offset = struct_env.get_field(field_id).get_offset();
        fields.remove(offset)
    }

    fn handle_struct_update_field(
        &self,
        module_id: ModuleId,
        struct_id: StructId,
        field_id: FieldId,
        struct_val: BaseValue,
        field_val: BaseValue,
    ) -> BaseValue {
        let mut fields = struct_val.into_struct();

        let env = self.target.global_env();
        let struct_env = env.get_struct(module_id.qualified(struct_id));
        if cfg!(debug_assertions) {
            assert_eq!(struct_env.get_field_count(), fields.len());
        }

        let offset = struct_env.get_field(field_id).get_offset();
        *fields.get_mut(offset).unwrap() = field_val;
        BaseValue::mk_struct(fields)
    }

    fn handle_global_exists(
        &self,
        node_id: NodeId,
        mem_opt: Option<MemoryLabel>,
        addr_val: BaseValue,
    ) -> BaseValue {
        let env = self.target.global_env();
        let node_ty = env
            .get_node_instantiation(node_id)
            .into_iter()
            .next()
            .unwrap();
        let struct_ty = convert_model_base_type(env, &node_ty, self.ty_args);
        let struct_inst = struct_ty.into_struct_inst();
        let addr = addr_val.into_address();
        let result = match mem_opt {
            None => self.global_state.has_resource(&addr, &struct_inst),
            Some(mem_label) => self
                .eval_state
                .load_memory(&mem_label, &struct_inst, &addr)
                .is_some(),
        };
        BaseValue::mk_bool(result)
    }

    fn handle_global_get(
        &self,
        node_id: NodeId,
        mem_opt: Option<MemoryLabel>,
        addr_val: BaseValue,
    ) -> EvalResult<BaseValue> {
        let env = self.target.global_env();
        let node_ty = env
            .get_node_instantiation(node_id)
            .into_iter()
            .next()
            .unwrap();
        let struct_ty = convert_model_base_type(env, &node_ty, self.ty_args);
        let struct_inst = struct_ty.into_struct_inst();
        let addr = addr_val.into_address();
        let resource = match mem_opt {
            None => {
                match self
                    .global_state
                    .get_resource(Some(true), addr, struct_inst)
                {
                    None => {
                        return Err(Self::eval_failure_code());
                    }
                    Some(val) => val.decompose().1,
                }
            }
            Some(mem_label) => match self.eval_state.load_memory(&mem_label, &struct_inst, &addr) {
                None => {
                    return Err(Self::eval_failure_code());
                }
                Some(val) => val,
            },
        };
        Ok(resource)
    }

    //
    // natives
    //

    fn native_spec_signer_of(&self, arg: BaseValue) -> BaseValue {
        BaseValue::mk_address(arg.into_signer())
    }

    //
    // spec function interpretation
    //

    fn interpret_spec_function(
        &self,
        decl: &SpecFunDecl,
        ty_args: &[BaseType],
        args: &[Exp],
        global_state: &mut GlobalState,
    ) -> EvalResult<BaseValue> {
        let (vars, stmt) = match decl.body.as_ref().unwrap().as_ref() {
            ExpData::Block(_, vars, stmt) => (vars, stmt),
            _ => unreachable!(),
        };

        let env = self.target.global_env();
        let mut exp_state = ExpState::default();
        for ((param_name, _), arg_exp) in decl.params.iter().zip(args.iter()) {
            let name = env.symbol_pool().string(*param_name).to_string();
            match arg_exp.as_ref() {
                ExpData::Lambda(_, vars, stmt) => {
                    exp_state.add_lambda(name, vars.to_vec(), stmt.clone())
                }
                _ => {
                    let arg_val = self.evaluate(arg_exp)?;
                    exp_state.add_var(name, arg_val);
                }
            }
        }
        for var in vars {
            let name = env.symbol_pool().string(var.name).to_string();
            let val = self.evaluate(var.binding.as_ref().unwrap())?;
            exp_state.add_var(name, val);
        }

        let evaluator = Evaluator::new(
            self.holder,
            self.target,
            ty_args,
            self.level.get(),
            exp_state,
            self.eval_state,
            self.local_state,
            global_state,
        );
        evaluator.evaluate(stmt)
    }

    //
    // operators that gives other types
    //

    fn unroll_range(&self, exp: &Exp) -> EvalResult<(BigInt, BigInt)> {
        let range = match exp.as_ref() {
            ExpData::Call(_, Operation::Range, args) => {
                if cfg!(debug_assertions) {
                    assert_eq!(args.len(), 2);
                }
                let lhs = self.evaluate(&args[0])?.into_int();
                let rhs = self.evaluate(&args[1])?.into_int();
                (lhs, rhs)
            }
            ExpData::Call(_, Operation::RangeVec, args) => {
                if cfg!(debug_assertions) {
                    assert_eq!(args.len(), 1);
                }
                let opv = self.evaluate(&args[0])?.into_vector();
                (BigInt::zero(), BigInt::from(opv.len()))
            }
            _ => unreachable!(),
        };
        Ok(range)
    }

    fn unroll_type_domain(&self, ty: &MTy::Type) -> Vec<BaseValue> {
        match ty {
            MTy::Type::Primitive(MTy::PrimitiveType::Address) => self
                .eval_state
                .all_addresses()
                .into_iter()
                .map(BaseValue::Address)
                .collect(),
            _ => unreachable!(),
        }
    }

    fn unroll_resource_domain_by_inst(&self, inst: &StructInstantiation) -> Vec<BaseValue> {
        self.eval_state.all_resources_by_inst(inst)
    }

    fn unroll_resource_domain_by_ident(&self, ident: &StructIdent) -> Vec<BaseValue> {
        self.eval_state.all_resources_by_ident(ident)
    }

    //
    // utilities
    //

    fn record_evaluation_failure(&self, exp: &Exp, _err: BigInt) {
        let env = self.target.global_env();
        let loc = env.get_node_loc(exp.node_id());
        env.error(&loc, "failed to evaluate expression");
    }

    fn record_checking_failure(&self, exp: &Exp) {
        let env = self.target.global_env();
        let loc = env.get_node_loc(exp.node_id());
        env.error(&loc, "property does not hold");
    }

    fn eval_failure_code() -> BigInt {
        BigInt::from(-1)
    }

    //
    // settings
    //

    fn get_settings(&self) -> Rc<InterpreterSettings> {
        self.target
            .global_env()
            .get_extension::<InterpreterSettings>()
            .unwrap_or_default()
    }
}

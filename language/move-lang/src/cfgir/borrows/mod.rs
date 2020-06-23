// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod state;

use super::absint::*;
use crate::{
    errors::*,
    hlir::ast::*,
    parser::ast::{BinOp_, StructName, Var},
    shared::unique_map::UniqueMap,
};
use move_ir_types::location::*;
use state::*;
use std::collections::BTreeMap;

//**************************************************************************************************
// Entry and trait bindings
//**************************************************************************************************

struct BorrowSafety {
    local_numbers: UniqueMap<Var, usize>,
}

impl BorrowSafety {
    fn new<T>(local_types: &UniqueMap<Var, T>) -> Self {
        let mut local_numbers = UniqueMap::new();
        for (idx, (v, _)) in local_types.iter().enumerate() {
            local_numbers.add(v, idx).unwrap();
        }
        Self { local_numbers }
    }
}

struct Context<'a, 'b> {
    local_numbers: &'a UniqueMap<Var, usize>,
    borrow_state: &'b mut BorrowState,
    errors: Errors,
}

impl<'a, 'b> Context<'a, 'b> {
    fn new(safety: &'a BorrowSafety, borrow_state: &'b mut BorrowState) -> Self {
        let local_numbers = &safety.local_numbers;
        Self {
            local_numbers,
            borrow_state,
            errors: vec![],
        }
    }

    fn get_errors(self) -> Errors {
        self.errors
    }

    fn add_errors(&mut self, mut additional: Errors) {
        self.errors.append(&mut additional);
    }
}

impl TransferFunctions for BorrowSafety {
    type State = BorrowState;

    fn execute(
        &mut self,
        pre: &mut Self::State,
        _lbl: Label,
        _idx: usize,
        cmd: &Command,
    ) -> Errors {
        let mut context = Context::new(self, pre);
        command(&mut context, cmd);
        context
            .borrow_state
            .canonicalize_locals(&context.local_numbers);
        context.get_errors()
    }
}

impl AbstractInterpreter for BorrowSafety {}

pub fn verify(
    errors: &mut Errors,
    signature: &FunctionSignature,
    acquires: &BTreeMap<StructName, Loc>,
    locals: &UniqueMap<Var, SingleType>,
    cfg: &super::cfg::BlockCFG,
) -> BTreeMap<Label, BorrowState> {
    let mut initial_state = BorrowState::initial(locals, acquires.clone(), !errors.is_empty());
    initial_state.bind_arguments(&signature.parameters);
    let mut safety = BorrowSafety::new(locals);
    initial_state.canonicalize_locals(&safety.local_numbers);
    let (final_state, mut es) = safety.analyze_function(cfg, initial_state);
    errors.append(&mut es);
    final_state
}

//**************************************************************************************************
// Command
//**************************************************************************************************

fn command(context: &mut Context, sp!(loc, cmd_): &Command) {
    use Command_ as C;
    match cmd_ {
        C::Assign(ls, e) => {
            let values = exp(context, e);
            lvalues(context, ls, values);
        }
        C::Mutate(el, er) => {
            let value = assert_single_value(exp(context, er));
            assert!(!value.is_ref());
            let lvalue = assert_single_value(exp(context, el));
            let errors = context.borrow_state.mutate(*loc, lvalue);
            context.add_errors(errors);
        }
        C::JumpIf { cond: e, .. } => {
            let value = assert_single_value(exp(context, e));
            assert!(!value.is_ref());
        }
        C::IgnoreAndPop { exp: e, .. } => {
            let values = exp(context, e);
            context.borrow_state.release_values(values);
        }

        C::Return(e) => {
            let values = exp(context, e);
            let errors = context.borrow_state.return_(*loc, values);
            context.add_errors(errors);
        }
        C::Abort(e) => {
            let value = assert_single_value(exp(context, e));
            assert!(!value.is_ref());
            context.borrow_state.abort()
        }
        C::Jump(_) => (),
        C::Break | C::Continue => panic!("ICE break/continue not translated to jumps"),
    }
}

fn lvalues(context: &mut Context, ls: &[LValue], values: Values) {
    assert!(ls.len() == values.len());
    ls.iter()
        .zip(values)
        .for_each(|(l, value)| lvalue(context, l, value))
}

fn lvalue(context: &mut Context, sp!(loc, l_): &LValue, value: Value) {
    use LValue_ as L;
    match l_ {
        L::Ignore => {
            context.borrow_state.release_value(value);
        }
        L::Var(v, _) => {
            let errors = context.borrow_state.assign_local(*loc, v, value);
            context.add_errors(errors)
        }
        L::Unpack(_, _, fields) => {
            assert!(!value.is_ref());
            fields
                .iter()
                .for_each(|(_, l)| lvalue(context, l, Value::NonRef))
        }
    }
}

fn exp(context: &mut Context, parent_e: &Exp) -> Values {
    use UnannotatedExp_ as E;
    let eloc = &parent_e.exp.loc;
    let svalue = || vec![Value::NonRef];
    match &parent_e.exp.value {
        E::Move { var, .. } => {
            let (errors, value) = context.borrow_state.move_local(*eloc, var);
            context.add_errors(errors);
            vec![value]
        }
        E::Copy { var, .. } => {
            let (errors, value) = context.borrow_state.copy_local(*eloc, var);
            context.add_errors(errors);
            vec![value]
        }
        E::BorrowLocal(mut_, var) => {
            let (errors, value) = context.borrow_state.borrow_local(*eloc, *mut_, var);
            context.add_errors(errors);
            assert!(value.is_ref());
            vec![value]
        }
        E::Freeze(e) => {
            let evalue = assert_single_value(exp(context, e));
            let (errors, value) = context.borrow_state.freeze(*eloc, evalue);
            context.add_errors(errors);
            vec![value]
        }
        E::Dereference(e) => {
            let evalue = assert_single_value(exp(context, e));
            let (errors, value) = context.borrow_state.dereference(*eloc, evalue);
            context.add_errors(errors);
            vec![value]
        }
        E::Borrow(mut_, e, f) => {
            let evalue = assert_single_value(exp(context, e));
            let (errors, value) = context.borrow_state.borrow_field(*eloc, *mut_, evalue, f);
            context.add_errors(errors);
            vec![value]
        }

        E::Builtin(b, e) => {
            let evalues = exp(context, e);
            let b: &BuiltinFunction = b;
            match b {
                sp!(_, BuiltinFunction_::BorrowGlobal(mut_, t)) => {
                    assert!(!assert_single_value(evalues).is_ref());
                    let (errors, value) = context.borrow_state.borrow_global(*eloc, *mut_, t);
                    context.add_errors(errors);
                    vec![value]
                }
                sp!(_, BuiltinFunction_::MoveFrom(t)) => {
                    assert!(!assert_single_value(evalues).is_ref());
                    let (errors, value) = context.borrow_state.move_from(*eloc, t);
                    assert!(!value.is_ref());
                    context.add_errors(errors);
                    vec![value]
                }
                _ => {
                    let ret_ty = &parent_e.ty;
                    let (errors, values) =
                        context
                            .borrow_state
                            .call(*eloc, evalues, &BTreeMap::new(), ret_ty);
                    context.add_errors(errors);
                    values
                }
            }
        }

        E::ModuleCall(mcall) => {
            let evalues = exp(context, &mcall.arguments);
            let ret_ty = &parent_e.ty;
            let (errors, values) =
                context
                    .borrow_state
                    .call(*eloc, evalues, &mcall.acquires, ret_ty);
            context.add_errors(errors);
            values
        }

        E::Unit { .. } | E::Value(_) | E::Constant(_) | E::Spec(_, _) | E::UnresolvedError => {
            svalue()
        }
        E::Cast(e, _) | E::UnaryExp(_, e) => {
            let v = exp(context, e);
            assert!(!assert_single_value(v).is_ref());
            svalue()
        }
        E::BinopExp(e1, sp!(_, BinOp_::Eq), e2) | E::BinopExp(e1, sp!(_, BinOp_::Neq), e2) => {
            let v1 = assert_single_value(exp(context, e1));
            let v2 = assert_single_value(exp(context, e2));
            if v1.is_ref() {
                // derefrence releases the id and checks that it is readable
                assert!(v2.is_ref());
                let (errors, _) = context.borrow_state.dereference(e1.exp.loc, v1);
                assert!(errors.is_empty(), "ICE eq freezing failed");
                let (errors, _) = context.borrow_state.dereference(e1.exp.loc, v2);
                assert!(errors.is_empty(), "ICE eq freezing failed");
            }
            svalue()
        }
        E::BinopExp(e1, _, e2) => {
            let v1 = assert_single_value(exp(context, e1));
            let v2 = assert_single_value(exp(context, e2));
            assert!(!v1.is_ref());
            assert!(!v2.is_ref());
            svalue()
        }
        E::Pack(_, _, fields) => {
            fields.iter().for_each(|(_, _, e)| {
                let arg = exp(context, e);
                assert!(!assert_single_value(arg).is_ref());
            });
            svalue()
        }

        E::ExpList(es) => es
            .iter()
            .flat_map(|item| exp_list_item(context, item))
            .collect(),

        E::Unreachable => panic!("ICE should not analyze dead code"),
    }
}

fn exp_list_item(context: &mut Context, item: &ExpListItem) -> Values {
    match item {
        ExpListItem::Single(e, _) | ExpListItem::Splat(_, e, _) => exp(context, e),
    }
}

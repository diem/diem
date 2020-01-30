// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::core::Context;
use crate::{
    naming::ast::{self as N, BaseType, BaseType_, TypeName_},
    parser::ast::StructName,
    shared::*,
    typing::ast as T,
};
use std::collections::{BTreeSet, HashMap};

pub type Seen = BTreeSet<StructName>;

//**************************************************************************************************
// Functions
//**************************************************************************************************

pub fn function_body_(
    context: &mut Context,
    annotated_acquires: &BTreeSet<StructName>,
    b_: &T::FunctionBody_,
) {
    let mut seen = Seen::new();
    match b_ {
        T::FunctionBody_::Native => (),
        T::FunctionBody_::Defined(es) => sequence(context, annotated_acquires, &mut seen, es),
    }

    for annotated_acquire in annotated_acquires {
        if !seen.contains(&annotated_acquire) {
            let msg = format!(
                "Invalid 'acquires' list. The resource '{}::{}' was never acquired by '{}', '{}', \
                 '{}', or a transitive call",
                context.current_module.as_ref().unwrap(),
                annotated_acquire,
                N::BuiltinFunction_::MOVE_FROM,
                N::BuiltinFunction_::BORROW_GLOBAL,
                N::BuiltinFunction_::BORROW_GLOBAL_MUT
            );
            context.error(vec![(annotated_acquire.loc(), msg)])
        }
    }
}

//**************************************************************************************************
// Expressions
//**************************************************************************************************

fn sequence(
    context: &mut Context,
    annotated_acquires: &BTreeSet<StructName>,
    seen: &mut Seen,
    seq: &T::Sequence,
) {
    for item in seq {
        sequence_item(context, annotated_acquires, seen, item)
    }
}

fn sequence_item(
    context: &mut Context,
    annotated_acquires: &BTreeSet<StructName>,
    seen: &mut Seen,
    item: &T::SequenceItem,
) {
    use T::SequenceItem_ as S;
    match &item.value {
        S::Bind(_, _, te) | S::Seq(te) => exp(context, annotated_acquires, seen, te),

        S::Declare(_) => (),
    }
}

fn exp(
    context: &mut Context,
    annotated_acquires: &BTreeSet<StructName>,
    seen: &mut Seen,
    e: &T::Exp,
) {
    use T::UnannotatedExp_ as E;
    match &e.exp.value {
        E::Use(_) => panic!("ICE should have been expanded"),

        E::Unit
        | E::Value(_)
        | E::Move { .. }
        | E::Copy { .. }
        | E::BorrowLocal(_, _)
        | E::Break
        | E::Continue
        | E::UnresolvedError => (),

        E::ModuleCall(call) => {
            let loc = e.exp.loc;
            let acquires = call
                .acquires
                .iter()
                .filter(|a| valid_acquires_annot(context, loc, a))
                .collect::<BTreeSet<_>>();
            let msg = || format!("Invalid call to '{}::{}'", &call.module, &call.name);
            for sn in acquires {
                check_acquire_listed(context, annotated_acquires, loc, msg, sn);
                seen.insert(sn.clone());
            }

            exp(context, annotated_acquires, seen, &call.arguments);
        }
        E::Builtin(b, args) => {
            builtin_function(context, annotated_acquires, seen, &e.exp.loc, b);
            exp(context, annotated_acquires, seen, args);
        }

        E::IfElse(eb, et, ef) => {
            exp(context, annotated_acquires, seen, eb);
            exp(context, annotated_acquires, seen, et);
            exp(context, annotated_acquires, seen, ef);
        }
        E::While(eb, eloop) => {
            exp(context, annotated_acquires, seen, eb);
            exp(context, annotated_acquires, seen, eloop);
        }
        E::Loop { body: eloop, .. } => exp(context, annotated_acquires, seen, eloop),
        E::Block(seq) => sequence(context, annotated_acquires, seen, seq),
        E::Assign(_, _, er) => {
            exp(context, annotated_acquires, seen, er);
        }

        E::Return(er)
        | E::Abort(er)
        | E::Dereference(er)
        | E::UnaryExp(_, er)
        | E::Borrow(_, er, _)
        | E::TempBorrow(_, er) => exp(context, annotated_acquires, seen, er),
        E::Mutate(el, er) | E::BinopExp(el, _, _, er) => {
            exp(context, annotated_acquires, seen, el);
            exp(context, annotated_acquires, seen, er)
        }

        E::Pack(_, _, _, fields) => {
            for (_, (_, (_, fe))) in fields.iter() {
                exp(context, annotated_acquires, seen, fe)
            }
        }
        E::ExpList(el) => exp_list(context, annotated_acquires, seen, el),

        E::Annotate(e, _) => exp(context, annotated_acquires, seen, e),
    }
}

fn exp_list(
    context: &mut Context,
    annotated_acquires: &BTreeSet<StructName>,
    seen: &mut Seen,
    items: &[T::ExpListItem],
) {
    for item in items {
        exp_list_item(context, annotated_acquires, seen, item)
    }
}

fn exp_list_item(
    context: &mut Context,
    annotated_acquires: &BTreeSet<StructName>,
    seen: &mut Seen,
    item: &T::ExpListItem,
) {
    use T::ExpListItem as I;
    match item {
        I::Single(e, _) | I::Splat(_, e, _) => {
            exp(context, annotated_acquires, seen, e);
        }
    }
}

fn builtin_function(
    context: &mut Context,
    annotated_acquires: &BTreeSet<StructName>,
    seen: &mut Seen,
    loc: &Loc,
    b: &T::BuiltinFunction,
) {
    use T::BuiltinFunction_ as B;
    let mk_msg = |s| move || format!("Invalid call to {}.", s);
    match &b.value {
        B::MoveFrom(bt) => {
            let msg = mk_msg(N::BuiltinFunction_::MOVE_FROM);
            if let Some(sn) = check_global_access(context, loc, msg, bt) {
                check_acquire_listed(context, annotated_acquires, *loc, msg, sn);
                seen.insert(sn.clone());
            }
        }
        B::BorrowGlobal(mut_, bt) => {
            let name = if *mut_ {
                N::BuiltinFunction_::BORROW_GLOBAL_MUT
            } else {
                N::BuiltinFunction_::BORROW_GLOBAL
            };
            let msg = mk_msg(name);
            if let Some(sn) = check_global_access(context, loc, msg, bt) {
                check_acquire_listed(context, annotated_acquires, *loc, msg, sn);
                seen.insert(sn.clone());
            }
        }
        B::MoveToSender(bt) => {
            let msg = mk_msg(N::BuiltinFunction_::MOVE_TO_SENDER);
            check_global_access(context, loc, msg, bt);
        }
        B::Exists(bt) => {
            let msg = mk_msg(N::BuiltinFunction_::EXISTS);
            check_global_access(context, loc, msg, bt);
        }
        B::Freeze(_) => (),
    }
}

//**************************************************************************************************
// Checks
//**************************************************************************************************

fn check_acquire_listed<F>(
    context: &mut Context,
    annotated_acquires: &BTreeSet<StructName>,
    loc: Loc,
    msg: F,
    global_type: &StructName,
) where
    F: Fn() -> String,
{
    if !annotated_acquires.contains(global_type) {
        let tmsg = format!(
            "The call acquires '{}::{}', but the 'acquires' list for the current function does not \
            contain this type. It must be present in the calling context's acquires list",
            context.current_module.as_ref().unwrap(),
            global_type
        );
        context.error(vec![(loc, msg()), (global_type.loc(), tmsg)]);
    }
}

pub fn check_global_access<'a, F>(
    context: &mut Context,
    loc: &Loc,
    msg: F,
    global_type: &'a BaseType,
) -> Option<&'a StructName>
where
    F: Fn() -> String,
{
    check_global_access_(context, loc, msg, global_type)
}

fn valid_acquires_annot(_context: &mut Context, _loc: Loc, _global_type: &StructName) -> bool {
    // let msg = || panic!("ICE should not have recorded errors");
    // check_global_access_(context, loc, msg, global_type, false)
    true
}

fn check_global_access_<'a, F>(
    context: &mut Context,
    loc: &Loc,
    msg: F,
    global_type: &'a BaseType,
) -> Option<&'a StructName>
where
    F: Fn() -> String,
{
    use BaseType_ as B;
    use TypeName_ as TN;
    let tloc = &global_type.loc;
    let (def_loc, declared_module, sn, resource_opt) = match &global_type.value {
        B::Var(_) => panic!("ICE type expansion failed"),
        B::Anything => {
            assert!(context.has_errors());
            return None;
        }
        B::Param(_) | B::Apply(_, sp!(_, TN::Builtin(_)), _) => {
            let ty_debug = global_type.value.subst_format(&HashMap::new());
            let tmsg = format!("Expected a nominal resource. Found the type: {}", ty_debug);

            context.error(vec![(*loc, msg()), (*tloc, tmsg)]);
            return None;
        }
        B::Apply(_, sp!(_, TN::ModuleType(m, s)), _args) => {
            let def_loc = context.struct_declared_loc(m, s);
            let resource_opt = context.resource_opt(m, s);
            (def_loc, m.clone(), s, resource_opt)
        }
    };

    match &context.current_module {
        Some(current_module) if current_module != &declared_module => {
            let ty_debug = global_type.value.subst_format(&HashMap::new());
            let tmsg = format!(
                "The type '{}' was not declared in the current module. Global storage access is \
                 internal to the module'",
                ty_debug
            );
            context.error(vec![(*loc, msg()), (*tloc, tmsg)]);
            return None;
        }
        _ => (),
    }

    if resource_opt.is_none() {
        let ty_debug = global_type.value.subst_format(&HashMap::new());
        let tmsg = format!("Expected a nominal resource. Found the type: {}", ty_debug);

        context.error(vec![
            (*loc, msg()),
            (*tloc, tmsg),
            (def_loc, "Declared as a normal struct here".into()),
        ]);
        return None;
    }

    Some(sn)
}

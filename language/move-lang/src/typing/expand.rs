// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::core::{self, Context};
use crate::{
    naming::ast::{
        BaseType, BaseType_, FunctionSignature, SingleType, SingleType_, TParam, Type, Type_,
    },
    parser::ast::{Kind, Kind_},
    shared::*,
    typing::ast as T,
};

//**************************************************************************************************
// Functions
//**************************************************************************************************

pub fn function_body_(context: &mut Context, b_: &mut T::FunctionBody_) {
    match b_ {
        T::FunctionBody_::Native => (),
        T::FunctionBody_::Defined(es) => sequence(context, es),
    }
}

pub fn function_signature(context: &mut Context, sig: &mut FunctionSignature) {
    for (_, st) in &mut sig.parameters {
        single_type(context, st);
    }
    type_(context, &mut sig.return_type)
}

//**************************************************************************************************
// Types
//**************************************************************************************************

fn type_(context: &mut Context, ty: &mut Type) {
    use Type_::*;
    match &mut ty.value {
        Unit => (),
        Multiple(ss) => single_types(context, ss),
        Single(s) => single_type(context, s),
    }
}

fn expected_types(context: &mut Context, ss: &mut Vec<Option<SingleType>>) {
    for st_opt in ss {
        if let Some(ss) = st_opt {
            single_type(context, ss)
        }
    }
}

fn single_types(context: &mut Context, ss: &mut Vec<SingleType>) {
    for st in ss {
        single_type(context, st)
    }
}

fn single_type(context: &mut Context, st: &mut SingleType) {
    use SingleType_::*;
    match &mut st.value {
        Ref(_, b) | Base(b) => base_type(context, b),
    }
}

fn base_types(context: &mut Context, bs: &mut Vec<BaseType>) {
    for bt in bs {
        base_type(context, bt)
    }
}

pub fn base_type(context: &mut Context, bt: &mut BaseType) {
    use BaseType_ as B;
    match &mut bt.value {
        B::Var(tvar) => {
            let btvar = sp(bt.loc, B::Var(*tvar));
            let replacement = core::unfold_type_base(&context.subst, btvar);
            match &replacement {
                sp!(_, B::Var(_)) => panic!("ICE unfold_type_base failed to expand"),
                sp!(_, B::Anything) => {
                    context.error(
                        // TODO maybe try to point to which type parameter this tvar is for
                        vec![(
                            bt.loc,
                            "Could not infer this type. Try adding an annotation",
                        )],
                    )
                }
                _ => (),
            }
            *bt = replacement;
            base_type(context, bt)
        }
        B::Apply(Some(_), _, bs) => base_types(context, bs),
        B::Apply(_, _, _) => {
            let kind = core::infer_kind_base(&context, &context.subst, bt.clone()).unwrap();
            match &mut bt.value {
                B::Apply(k_opt, _, bs) => {
                    *k_opt = Some(kind);
                    base_types(context, bs);
                }
                _ => panic!("ICE impossible. tapply switched to nontapply"),
            }
        }
        B::Param(_) => (),
        // TODO might want to add a flag to Anything for reporting errors or not
        B::Anything => (),
    }
}

fn get_kind(s: &SingleType) -> Kind {
    use SingleType_ as S;
    match &s.value {
        S::Ref(_, _) => sp(s.loc, Kind_::Unrestricted),
        S::Base(b) => get_kind_base(b),
    }
}

fn get_kind_base(b: &BaseType) -> Kind {
    use BaseType_ as B;
    match b {
        sp!(_, B::Var(_)) => panic!("ICE unexpanded type"),
        sp!(loc, B::Anything) => sp(*loc, Kind_::Unrestricted),
        sp!(_, B::Param(TParam { kind, .. })) => kind.clone(),
        sp!(_, B::Apply(Some(kind), _, _)) => kind.clone(),
        sp!(_, B::Apply(None, _, _)) => panic!("ICE unexpanded type"),
    }
}

//**************************************************************************************************
// Expressions
//**************************************************************************************************

fn sequence(context: &mut Context, seq: &mut T::Sequence) {
    for item in seq {
        sequence_item(context, item)
    }
}

fn sequence_item(context: &mut Context, item: &mut T::SequenceItem) {
    use T::SequenceItem_ as S;
    match &mut item.value {
        S::Seq(te) => exp(context, te),

        S::Declare(tbind) => bind_list(context, tbind),
        S::Bind(tbind, tys, te) => {
            bind_list(context, tbind);
            expected_types(context, tys);
            exp(context, te)
        }
    }
}

fn exp(context: &mut Context, e: &mut T::Exp) {
    use T::UnannotatedExp_ as E;
    // dont expand the type for return or abort
    match &e.exp.value {
        E::Break | E::Continue | E::Return(_) | E::Abort(_) => match &e.ty.value {
            Type_::Single(sp!(_, SingleType_::Base(sp!(_, BaseType_::Anything)))) => (),
            _ => {
                let t = &mut e.ty;
                type_(context, t);
                *t = Type_::anything(t.loc);
            }
        },
        _ => type_(context, &mut e.ty),
    }
    match &mut e.exp.value {
        E::Use(v) => {
            let st = match &e.ty.value {
                Type_::Unit | Type_::Multiple(_) => {
                    panic!("ICE vars cannot have unit/multiple types")
                }
                Type_::Single(st) => st,
            };
            let from_user = false;
            let var = v.clone();
            e.exp.value = match get_kind(st).value {
                Kind_::Unrestricted => E::Copy { from_user, var },
                Kind_::Unknown | Kind_::Affine | Kind_::Resource => E::Move { from_user, var },
            }
        }

        E::Unit
        | E::Value(_)
        | E::Move { .. }
        | E::Copy { .. }
        | E::BorrowLocal(_, _)
        | E::Break
        | E::Continue
        | E::UnresolvedError => (),

        E::ModuleCall(call) => module_call(context, call),
        E::Builtin(b, args) => {
            builtin_function(context, b);
            exp(context, args);
        }

        E::IfElse(eb, et, ef) => {
            exp(context, eb);
            exp(context, et);
            exp(context, ef);
        }
        E::While(eb, eloop) => {
            exp(context, eb);
            exp(context, eloop);
        }
        E::Loop { body: eloop, .. } => exp(context, eloop),
        E::Block(seq) => sequence(context, seq),
        E::Assign(assigns, tys, er) => {
            assign_list(context, assigns);
            expected_types(context, tys);
            exp(context, er);
        }

        E::Return(er)
        | E::Abort(er)
        | E::Dereference(er)
        | E::UnaryExp(_, er)
        | E::Borrow(_, er, _)
        | E::TempBorrow(_, er) => exp(context, er),
        E::Mutate(el, er) | E::BinopExp(el, _, er) => {
            exp(context, el);
            exp(context, er)
        }

        E::Pack(_, _, bs, fields) => {
            base_types(context, bs);
            for (_, (_, (bt, fe))) in fields.iter_mut() {
                base_type(context, bt);
                exp(context, fe)
            }
        }
        E::ExpList(el) => exp_list(context, el),
    }
}

fn bind_list(context: &mut Context, binds: &mut T::BindList) {
    for b in &mut binds.value {
        bind(context, b)
    }
}

fn bind(context: &mut Context, b: &mut T::Bind) {
    use T::Bind_ as B;
    match &mut b.value {
        B::Ignore => (),
        B::Var(v, None) => {
            let msg = format!(
                "Unused local '{0}'. Consider removing or prefixing with an underscore: '_{0}'",
                v
            );
            context.error(vec![(b.loc, msg)]);
            b.value = B::Ignore
        }
        B::Var(_, Some(st)) => single_type(context, st),
        B::BorrowUnpack(_, _, _, bts, fields) | B::Unpack(_, _, bts, fields) => {
            base_types(context, bts);
            for (_, (_, (bt, innerb))) in fields.iter_mut() {
                base_type(context, bt);
                bind(context, innerb)
            }
        }
    }
}

fn assign_list(context: &mut Context, assigns: &mut T::AssignList) {
    for a in &mut assigns.value {
        assign(context, a)
    }
}

fn assign(context: &mut Context, a: &mut T::Assign) {
    use T::Assign_ as A;
    match &mut a.value {
        A::Ignore => (),
        A::Var(_, st) => single_type(context, st),
        A::Unpack(_, _, bts, fields) | A::BorrowUnpack(_, _, _, bts, fields) => {
            base_types(context, bts);
            for (_, (_, (bt, innera))) in fields.iter_mut() {
                base_type(context, bt);
                assign(context, innera)
            }
        }
    }
}

fn module_call(context: &mut Context, call: &mut T::ModuleCall) {
    base_types(context, &mut call.type_arguments);
    exp(context, &mut call.arguments);
    single_types(context, &mut call.parameter_types)
}

fn builtin_function(context: &mut Context, b: &mut T::BuiltinFunction) {
    use T::BuiltinFunction_ as B;
    match &mut b.value {
        B::MoveToSender(bt)
        | B::MoveFrom(bt)
        | B::BorrowGlobal(_, bt)
        | B::Exists(bt)
        | B::Freeze(bt) => base_type(context, bt),
    }
}

fn exp_list(context: &mut Context, items: &mut Vec<T::ExpListItem>) {
    for item in items {
        exp_list_item(context, item)
    }
}

fn exp_list_item(context: &mut Context, item: &mut T::ExpListItem) {
    use T::ExpListItem as I;
    match item {
        I::Single(e, st) => {
            exp(context, e);
            single_type(context, st);
        }
        I::Splat(_, e, ss) => {
            exp(context, e);
            single_types(context, ss);
        }
    }
}

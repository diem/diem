// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::{
    core::{self, Context, LocalStatus, Subst},
    expand, globals,
};
use crate::shared::unique_map::UniqueMap;
use crate::{
    errors::Errors,
    expansion::ast::Fields,
    naming::ast::{
        self as N, BaseType, BaseType_, BuiltinTypeName_, SingleType, SingleType_, TVar, Type,
        TypeName_, Type_,
    },
    parser::ast::{BinOp_, Field, FunctionName, Kind_, ModuleIdent, StructName, UnaryOp_, Var},
    shared::*,
    typing::ast as T,
};
use std::collections::{BTreeSet, VecDeque};

//**************************************************************************************************
// Entry
//**************************************************************************************************

pub fn program(prog: N::Program, errors: Errors) -> (T::Program, Errors) {
    let mut context = Context::new(&prog, errors);
    let modules = modules(&mut context, prog.modules);
    let main = main_function(&mut context, prog.main);

    assert!(context.constraints.is_empty());
    (T::Program { modules, main }, context.get_errors())
}

fn modules(
    context: &mut Context,
    modules: UniqueMap<ModuleIdent, N::ModuleDefinition>,
) -> UniqueMap<ModuleIdent, T::ModuleDefinition> {
    modules.map(|ident, mdef| module(context, ident, mdef))
}

fn module(
    context: &mut Context,
    ident: ModuleIdent,
    mdef: N::ModuleDefinition,
) -> T::ModuleDefinition {
    context.current_module = Some(ident);
    let N::ModuleDefinition {
        is_source_module,
        mut structs,
        functions: n_functions,
        ..
    } = mdef;
    structs
        .iter_mut()
        .for_each(|(name, s)| struct_def(context, name, s));
    let functions = n_functions.map(|name, f| function(context, name, f));
    assert!(context.constraints.is_empty());
    T::ModuleDefinition {
        is_source_module,
        structs,
        functions,
    }
}

fn main_function(
    context: &mut Context,
    main: Option<(Address, FunctionName, N::Function)>,
) -> Option<(Address, FunctionName, T::Function)> {
    context.current_module = None;
    main.map(|(addr, name, f)| (addr, name.clone(), function(context, name, f)))
}

//**************************************************************************************************
// Functions
//**************************************************************************************************

fn function(context: &mut Context, _name: FunctionName, f: N::Function) -> T::Function {
    let N::Function {
        visibility,
        mut signature,
        body: n_body,
        acquires,
    } = f;
    assert!(context.constraints.is_empty());
    context.reset_for_module_item();

    function_signature(context, &signature);
    expand::function_signature(context, &mut signature);

    let body = function_body(context, &acquires, n_body);

    T::Function {
        visibility,
        signature,
        acquires,
        body,
    }
}

fn function_signature(context: &mut Context, sig: &N::FunctionSignature) {
    assert!(context.constraints.is_empty());

    let mut declared = UniqueMap::new();
    for (param, param_ty) in &sig.parameters {
        let param_ty = core::instantiate_single(context, param_ty.clone());
        if let Err(prev_loc) = declared.add(param.clone(), ()) {
            context.error(vec![
                (
                    param.loc(),
                    format!("Duplicate parameter with name '{}'", param),
                ),
                (prev_loc, "Previously declared here".into()),
            ]);
        }
        context.declare_local(param.clone(), Some(param_ty));
    }
    context.return_type = Some(core::instantiate(context, sig.return_type.clone()));
    core::solve_constraints(context);
}

fn function_body(
    context: &mut Context,
    acquires: &BTreeSet<StructName>,
    sp!(loc, nb_): N::FunctionBody,
) -> T::FunctionBody {
    assert!(context.constraints.is_empty());

    let mut b_ = match nb_ {
        N::FunctionBody_::Native => T::FunctionBody_::Native,
        N::FunctionBody_::Defined(es) => {
            let seq = sequence(context, es);
            let ety = sequence_type(&seq);
            let ret_ty = context.return_type.clone().unwrap();
            let sloc = seq.back().unwrap().loc;
            subtype(context, sloc, || "Invalid return expression", &ety, &ret_ty);
            T::FunctionBody_::Defined(seq)
        }
    };
    core::solve_constraints(context);
    expand::function_body_(context, &mut b_);
    globals::function_body_(context, &acquires, &b_);
    // freeze::function_body_(context, &mut b_);
    sp(loc, b_)
}

//**************************************************************************************************
// Structs
//**************************************************************************************************

fn struct_def(context: &mut Context, _name: StructName, s: &mut N::StructDefinition) {
    assert!(context.constraints.is_empty());
    context.reset_for_module_item();

    let field_map = match &mut s.fields {
        N::StructFields::Native(_) => return,
        N::StructFields::Defined(m) => m,
    };

    for (_field, idx_ty) in field_map.iter() {
        core::instantiate_base(context, idx_ty.1.clone());
    }
    core::solve_constraints(context);

    for (_field, idx_ty) in field_map.iter_mut() {
        expand::base_type(context, &mut idx_ty.1);
    }
}

//**************************************************************************************************
// Types
//**************************************************************************************************

macro_rules! TApply {
    ($k:pat, $n:pat, $args:pat) => {
        Type_::Single(sp!(
            _,
            SingleType_::Base(sp!(_, BaseType_::Apply($k, $n, $args)))
        ))
    };
}

fn typing_error<T: Into<String>, F: FnOnce() -> T>(
    context: &mut Context,
    loc: Loc,
    msg: F,
    e: core::TypingError,
) {
    use super::core::TypingError::*;
    let subst = &context.subst;
    let error = match e {
        SubtypeError(st1, st2) => vec![
            (loc, msg().into()),
            (
                st1.loc,
                format!("The type: '{}'", st1.value.subst_format(subst)),
            ),
            (
                st2.loc,
                format!("Is not a subtype of: '{}'", st2.value.subst_format(subst)),
            ),
        ],
        Incompatible(t1, t2) => vec![
            (loc, msg().into()),
            (
                t1.loc,
                format!("The type: '{}'", t1.value.subst_format(subst)),
            ),
            (
                t2.loc,
                format!("Is not compatible with: '{}'", t2.value.subst_format(subst)),
            ),
        ],
        RecursiveType(rloc) => vec![
            (loc, msg().into()),
            (
                rloc,
                "Unable to infer the type. Recursive type found.".into(),
            ),
        ],
    };
    context.error(error);
}

fn subtype<T: Into<String>, F: FnOnce() -> T>(
    context: &mut Context,
    loc: Loc,
    msg: F,
    lhs: &Type,
    rhs: &Type,
) -> Type {
    let subst = std::mem::replace(&mut context.subst, Subst::new());
    match core::subtype(subst.clone(), lhs, rhs) {
        Err(e) => {
            context.subst = subst;
            typing_error(context, loc, msg, e);
            Type_::anything(loc)
        }
        Ok((next_subst, ty)) => {
            context.subst = next_subst;
            ty
        }
    }
}

fn subtype_single<T: Into<String>, F: FnOnce() -> T>(
    context: &mut Context,
    loc: Loc,
    msg: F,
    lhs: &SingleType,
    rhs: &SingleType,
) -> SingleType {
    let subst = std::mem::replace(&mut context.subst, Subst::new());
    match core::subtype_single(subst.clone(), lhs, rhs) {
        Err(e) => {
            context.subst = subst;
            typing_error(context, loc, msg, e);
            SingleType_::anything(loc)
        }
        Ok((next_subst, ty)) => {
            context.subst = next_subst;
            ty
        }
    }
}

fn join<T: Into<String>, F: FnOnce() -> T>(
    context: &mut Context,
    loc: Loc,
    msg: F,
    t1: &Type,
    t2: &Type,
) -> Type {
    let subst = std::mem::replace(&mut context.subst, Subst::new());
    match core::join(subst.clone(), t1, t2) {
        Err(e) => {
            context.subst = subst;
            typing_error(context, loc, msg, e);
            Type_::anything(loc)
        }
        Ok((next_subst, ty)) => {
            context.subst = next_subst;
            ty
        }
    }
}

#[allow(dead_code)]
pub fn join_single<T: Into<String>, F: FnOnce() -> T>(
    context: &mut Context,
    loc: Loc,
    msg: F,
    t1: &SingleType,
    t2: &SingleType,
) -> SingleType {
    let subst = std::mem::replace(&mut context.subst, Subst::new());
    match core::join_single(subst.clone(), t1, t2) {
        Err(e) => {
            context.subst = subst;
            typing_error(context, loc, msg, e);
            SingleType_::anything(loc)
        }
        Ok((next_subst, ty)) => {
            context.subst = next_subst;
            ty
        }
    }
}

fn join_base<T: Into<String>, F: FnOnce() -> T>(
    context: &mut Context,
    loc: Loc,
    msg: F,
    t1: &BaseType,
    t2: &BaseType,
) -> BaseType {
    let subst = std::mem::replace(&mut context.subst, Subst::new());
    match core::join_base_type(subst.clone(), t1, t2) {
        Err(e) => {
            context.subst = subst;
            typing_error(context, loc, msg, e);
            BaseType_::anything(loc)
        }
        Ok((next_subst, ty)) => {
            context.subst = next_subst;
            ty
        }
    }
}

fn expect_builtin_ty<T: Into<String>, F: FnOnce() -> T>(
    context: &mut Context,
    builtin_set: BTreeSet<BuiltinTypeName_>,
    loc: Loc,
    msg: F,
    ty: &Type,
) -> Type {
    use BaseType_::*;
    use SingleType_::*;
    use TypeName_::*;
    use Type_::*;
    let sp!(_, t_) = core::unfold_type(&context.subst, ty.clone());
    let tmsg = || {
        let set_msg = if builtin_set.is_empty() {
            "the operation is not yet supported on any type".to_string()
        } else {
            format!(
                "expected: {}",
                format_comma(builtin_set.iter().map(|b| format!("'{}'", b)))
            )
        };
        format!(
            "Found: '{}'. But {}",
            t_.subst_format(&Subst::new()),
            set_msg
        )
    };
    match &t_ {
        TApply!(k, sp!(_, Builtin(sp!(_, b))), args) if builtin_set.contains(b) => {
            if let Some(sp!(_, Kind_::Resource)) = k {
                panic!("ICE assumes this type is being consumed so shouldn't be a resource");
            }
            assert!(args.is_empty());
            ty.clone()
        }
        Single(sp!(_, Base(sp!(_, Var(_))))) => panic!("ICE unfold_type failed"),
        Single(sp!(_, Base(sp!(_, Anything)))) => {
            context.error(vec![
                (loc, format!("{}. Try annotating this type.", msg().into())),
                (ty.loc, tmsg()),
            ]);
            Type_::anything(loc)
        }
        _ => {
            context.error(vec![(loc, msg().into()), (ty.loc, tmsg())]);
            Type_::anything(loc)
        }
    }
}

//**************************************************************************************************
// Expressions
//**************************************************************************************************

enum SeqCase {
    Seq(Loc, Box<T::Exp>),
    Declare {
        old_locals: UniqueMap<Var, LocalStatus>,
        declared: UniqueMap<Var, ()>,
        loc: Loc,
        b: T::BindList,
    },
    Bind {
        old_locals: UniqueMap<Var, LocalStatus>,
        declared: UniqueMap<Var, ()>,
        loc: Loc,
        b: T::BindList,
        e: Box<T::Exp>,
    },
}

fn sequence(context: &mut Context, seq: N::Sequence) -> T::Sequence {
    use N::SequenceItem_ as NS;
    use T::SequenceItem_ as TS;

    let mut work_queue = VecDeque::new();
    let mut resulting_sequence = T::Sequence::new();

    let len = seq.len();
    for (idx, sp!(loc, ns_)) in seq.into_iter().enumerate() {
        match ns_ {
            NS::Seq(ne) => {
                let e = exp_(context, ne);
                // If it is not the last element
                if idx < len - 1 {
                    let mut add_constraint = |st| {
                        context.add_copyable_constraint(
                            loc,
                            "Cannot ignore resource values. The value must be used",
                            st,
                        )
                    };
                    match &e.ty.value {
                        Type_::Unit => (),
                        Type_::Multiple(ss) => ss.iter().for_each(|st| add_constraint(st.clone())),
                        Type_::Single(st) => add_constraint(st.clone()),
                    }
                }
                work_queue.push_front(SeqCase::Seq(loc, Box::new(e)));
            }
            NS::Declare(nbind, ty_opt) => {
                let old_locals = context.locals.clone();
                let instantiated_ty_op = ty_opt.map(|t| core::instantiate(context, t));
                let (declared, b) = bind_list(context, nbind, instantiated_ty_op);
                work_queue.push_front(SeqCase::Declare {
                    old_locals,
                    declared,
                    loc,
                    b,
                });
            }
            NS::Bind(nbind, nr) => {
                let e = exp_(context, nr);
                let old_locals = context.locals.clone();
                let (declared, b) = bind_list(context, nbind, Some(e.ty.clone()));
                work_queue.push_front(SeqCase::Bind {
                    old_locals,
                    declared,
                    loc,
                    b,
                    e: Box::new(e),
                });
            }
        }
    }

    for case in work_queue {
        match case {
            SeqCase::Seq(loc, e) => resulting_sequence.push_front(sp(loc, TS::Seq(e))),
            SeqCase::Declare {
                old_locals,
                declared,
                loc,
                mut b,
            } => {
                close_scope(context, old_locals, declared, &mut b);
                resulting_sequence.push_front(sp(loc, TS::Declare(b)))
            }
            SeqCase::Bind {
                old_locals,
                declared,
                loc,
                mut b,
                e,
            } => {
                close_scope(context, old_locals, declared, &mut b);
                let lvalue_ty = binds_expected_types(context, &b);
                resulting_sequence.push_front(sp(loc, TS::Bind(b, lvalue_ty, e)))
            }
        }
    }

    resulting_sequence
}

fn sequence_type(seq: &T::Sequence) -> &Type {
    use T::SequenceItem_ as TS;
    match seq.back().unwrap() {
        sp!(_, TS::Bind(_, _, _)) | sp!(_, TS::Declare(_)) => {
            panic!("ICE unit should have been inserted past bind/decl")
        }
        sp!(_, TS::Seq(last_e)) => &last_e.ty,
    }
}

fn exp_vec(context: &mut Context, es: Vec<N::Exp>) -> Vec<T::Exp> {
    es.into_iter().map(|e| exp_(context, e)).collect()
}

fn exp(context: &mut Context, ne: N::Exp) -> Box<T::Exp> {
    Box::new(exp_(context, ne))
}

fn exp_(context: &mut Context, sp!(eloc, ne_): N::Exp) -> T::Exp {
    use N::Exp_ as NE;
    use T::UnannotatedExp_ as TE;
    let (ty, e_) = match ne_ {
        NE::Unit => (sp(eloc, Type_::Unit), TE::Unit),
        NE::Value(sp!(vloc, v)) => (Type_::base(v.type_(vloc)), TE::Value(sp(vloc, v))),

        NE::Move(var) => {
            let st = get_local(context, eloc, "move", &var);
            let from_user = true;
            (Type_::single(st), TE::Move { var, from_user })
        }
        NE::Copy(var) => {
            let st = get_local(context, eloc, "copy", &var);
            context.add_copyable_constraint(
                eloc,
                "Invalid 'copy' of owned resource value",
                st.clone(),
            );
            let from_user = true;
            (Type_::single(st), TE::Copy { var, from_user })
        }
        NE::Use(var) => {
            let st = get_local(context, eloc, "local usage", &var);
            (Type_::single(st), TE::Use(var))
        }

        NE::ModuleCall(m, f, ty_args_opt, nargs) => {
            let args = exp_(context, *nargs);
            module_call(context, eloc, m, f, ty_args_opt, args)
        }
        NE::Builtin(b, nargs) => {
            let args = exp_(context, *nargs);
            builtin_call(context, eloc, b, args)
        }

        NE::IfElse(nb, nt, nf) => {
            let eb = exp(context, *nb);
            let bloc = eb.exp.loc;
            subtype(
                context,
                bloc,
                || "Invalid if condition",
                &eb.ty,
                &Type_::bool(bloc),
            );
            let et = exp(context, *nt);
            let ef = exp(context, *nf);
            let ty = join(context, eloc, || "Incompatible branches", &et.ty, &ef.ty);
            (ty, TE::IfElse(eb, et, ef))
        }
        NE::While(nb, nloop) => {
            let eb = exp(context, *nb);
            let bloc = eb.exp.loc;
            subtype(
                context,
                bloc,
                || "Invalid while condition",
                &eb.ty,
                &Type_::bool(bloc),
            );
            let (_has_break, ty, body) = loop_body(context, eloc, false, *nloop);
            (sp(eloc, ty.value), TE::While(eb, body))
        }
        NE::Loop(nloop) => {
            let (has_break, ty, body) = loop_body(context, eloc, true, *nloop);
            let eloop = TE::Loop { has_break, body };
            (sp(eloc, ty.value), eloop)
        }
        NE::Block(nseq) => {
            let seq = sequence(context, nseq);
            (sequence_type(&seq).clone(), TE::Block(seq))
        }

        NE::Assign(na, nr) => {
            let er = exp(context, *nr);
            let a = assign_list(context, na, er.ty.clone());
            let lvalue_ty = assigns_expected_types(context, &a);
            (sp(eloc, Type_::Unit), TE::Assign(a, lvalue_ty, er))
        }

        NE::Mutate(nl, nr) => {
            let el = exp(context, *nl);
            let er = exp(context, *nr);
            let mut rvalue_tys = checked_rvalue_tys(context, eloc, "mutation", 1, er.ty.clone());
            assert!(rvalue_tys.len() == 1);
            let rvalue_ty = rvalue_tys.pop().unwrap();
            check_mutation(context, el.exp.loc, el.ty.clone(), rvalue_ty);
            (sp(eloc, Type_::Unit), TE::Mutate(el, er))
        }

        NE::FieldMutate(ndotted, nr) => {
            let er = exp(context, *nr);
            let mut rvalue_tys = checked_rvalue_tys(context, eloc, "mutation", 1, er.ty.clone());
            assert!(rvalue_tys.len() == 1);
            let rvalue_ty = rvalue_tys.pop().unwrap();
            match exp_dotted(context, "mutation", true, ndotted) {
                None => {
                    assert!(context.has_errors());
                    (Type_::anything(eloc), TE::UnresolvedError)
                }
                Some((edotted, _)) => {
                    let eborrow = exp_dotted_to_borrow(context, eloc, true, edotted);
                    check_mutation(context, eborrow.exp.loc, eborrow.ty.clone(), rvalue_ty);
                    (sp(eloc, Type_::Unit), TE::Mutate(Box::new(eborrow), er))
                }
            }
        }

        NE::Return(nret) => {
            let eret = exp(context, *nret);
            let ret_ty = context.return_type.clone().unwrap();
            subtype(context, eloc, || "Invalid return", &eret.ty, &ret_ty);
            (Type_::anything(eloc), TE::Return(eret))
        }
        NE::Abort(ncode) => {
            let ecode = exp(context, *ncode);
            let code_ty = Type_::u64(eloc);
            subtype(context, eloc, || "Invalid abort", &ecode.ty, &code_ty);
            (Type_::anything(eloc), TE::Abort(ecode))
        }
        NE::Break => {
            if !context.in_loop {
                context.error(vec![(
                    eloc,
                    "Invalid usage of 'break'. 'break' can only be used inside a loop body",
                )]);
            }
            let current_break_ty = sp(eloc, Type_::Unit);
            let break_ty = match &context.break_type {
                None => current_break_ty,
                Some(t) => {
                    let t = t.clone();
                    join(context, eloc, || "Invalid break.", &current_break_ty, &t)
                }
            };
            context.break_type = Some(break_ty);
            (Type_::anything(eloc), TE::Break)
        }
        NE::Continue => {
            if !context.in_loop {
                context.error(vec![(
                    eloc,
                    "Invalid usage of 'continue'. 'continue' can only be used inside a loop body",
                )]);
            }
            (Type_::anything(eloc), TE::Continue)
        }

        NE::Dereference(nref) => {
            let eref = exp(context, *nref);
            let inner = sp(eloc, BaseType_::Var(TVar::next()));
            let ref_ty = sp(eloc, SingleType_::Ref(false, inner.clone()));
            subtype(
                context,
                eloc,
                || "Invalid dereference.",
                &eref.ty,
                &Type_::single(ref_ty),
            );
            context.add_copyable_constraint(
                eloc,
                "Invalid dereference. Can only dereference references to copyable types",
                SingleType_::base(inner.clone()),
            );
            (Type_::base(inner), TE::Dereference(eref))
        }
        NE::UnaryExp(uop, nr) => {
            use BuiltinTypeName_ as BT;
            use UnaryOp_::*;
            let msg = || format!("Invalid argument to '{}'", &uop);
            let er = exp(context, *nr);
            let ty = match &uop.value {
                Not => {
                    let rloc = er.exp.loc;
                    subtype(context, rloc, msg, &er.ty, &Type_::bool(rloc));
                    Type_::bool(eloc)
                }
                Neg => expect_builtin_ty(context, BT::signed(), er.exp.loc, msg, &er.ty),
            };
            (ty, TE::UnaryExp(uop, er))
        }
        NE::BinopExp(nl, bop, nr) => {
            use BinOp_::*;
            use BuiltinTypeName_ as BT;
            let msg = || format!("Invalid argument to '{}'", &bop);
            let el = exp(context, *nl);
            let er = exp(context, *nr);
            let ty = match &bop.value {
                Sub | Add | Mul | Mod | Div => {
                    let ty = expect_builtin_ty(context, BT::numeric(), el.exp.loc, msg, &el.ty);
                    subtype(context, er.exp.loc, msg, &er.ty, &ty);
                    ty
                }

                BitOr | BitAnd | Xor => {
                    let ty = expect_builtin_ty(context, BT::bits(), el.exp.loc, msg, &el.ty);
                    subtype(context, er.exp.loc, msg, &er.ty, &ty);
                    ty
                }

                Lt | Gt | Le | Ge => {
                    let ty = expect_builtin_ty(context, BT::ordered(), el.exp.loc, msg, &el.ty);
                    join(context, er.exp.loc, msg, &er.ty, &ty);
                    Type_::bool(eloc)
                }

                Eq | Neq => {
                    let ty = join(context, er.exp.loc, msg, &el.ty, &er.ty);
                    let st = match ty {
                        sp!(tloc, t@Type_::Unit) | sp!(tloc, t@Type_::Multiple(_)) => {
                            let subst_t = t.subst_format(&context.subst);
                            context.error(vec![
                            (eloc, format!("Invalid use of '{}'. Expected an expression of a single type but got an expression of the list: {}", &bop, subst_t)),
                            (tloc, "Type found here".into()),
                        ]);
                            SingleType_::anything(eloc)
                        }
                        sp!(_, Type_::Single(t)) => t,
                    };
                    context.add_copyable_constraint(
                        eloc,
                        format!("Cannot use '{}' on resource values. This would destroy the resource. Try borrowing the values with '&' first.'", &bop),
                        st
                    );

                    Type_::bool(eloc)
                }

                And | Or => {
                    let lloc = el.exp.loc;
                    subtype(context, lloc, msg, &el.ty, &Type_::bool(lloc));
                    let rloc = er.exp.loc;
                    subtype(context, rloc, msg, &er.ty, &Type_::bool(lloc));
                    Type_::bool(eloc)
                }
            };
            (ty, TE::BinopExp(el, bop, er))
        }

        NE::ExpList(nes) => {
            assert!(!nes.is_empty());
            let es = exp_vec(context, nes);
            let tys = es.iter().map(|e| {
                use Type_::*;
                match &e.ty {
                    sp!(tloc, t@Unit) |
                    sp!(tloc, t@Multiple(_)) => {
                        let subst_t =  t.subst_format(&context.subst);
                        context.error(vec![
                            (e.exp.loc, format!("Invalid result list item. Expected an expression of a single type but got an expression of the list: {}", subst_t)),
                            (*tloc, "Type found here".into()),
                        ]);
                        SingleType_::anything(e.exp.loc)
                    }
                    sp!(_, Single(t)) => t.clone()
                }
            }).collect();
            let items_opt = es
                .into_iter()
                .map(T::single_item_opt)
                .collect::<Option<Vec<_>>>();
            match items_opt {
                Some(items) => {
                    let ty = sp(eloc, Type_::Multiple(tys));
                    (ty, TE::ExpList(items))
                }
                None => {
                    assert!(context.has_errors());
                    (Type_::anything(eloc), TE::UnresolvedError)
                }
            }
        }
        NE::Pack(m, n, ty_args_opt, nfields) => {
            let current_module = context.current_module.clone().unwrap();
            let (bt, targs) = core::make_struct_type(context, eloc, &m, &n, ty_args_opt);
            let typed_nfields =
                add_field_types(context, eloc, "argument", &m, &n, targs.clone(), nfields);

            let tfields = typed_nfields.map(|f, (idx, (fty, narg))| {
                let arg = exp_(context, narg);
                subtype(
                    context,
                    arg.exp.loc,
                    || format!("Invalid argument for field '{}' for '{}::{}'", f, &m, &n),
                    &arg.ty,
                    &Type_::base(fty.clone()),
                );
                (idx, (fty, arg))
            });
            if m != current_module {
                context.error(
                    vec![
                        (eloc, format!("Invalid instantiation of '{}::{}'", &m, &n)),
                        (current_module.loc(), "Currently, all structs can only be instantiated in module that they were declared".into())
                    ]
                )
            }
            (Type_::base(bt), TE::Pack(m, n, targs, tfields))
        }

        NE::Borrow(mut_, sp!(_, N::ExpDotted_::Exp(ner))) => {
            let er = exp_(context, *ner);
            let inner = match &er.ty {
                sp!(_, Type_::Single(sp!(_, SingleType_::Base(b)))) => b.clone(),
                sp!(tloc, t) => {
                    // TODO we could just collapse this so &&x == &x
                    let s = t.subst_format(&context.subst);
                    let tmsg = format!("Expected a single non-reference type, but got: {}", s);
                    context.error(vec![(eloc, "Invalid borrow".into()), (*tloc, tmsg)]);
                    BaseType_::anything(eloc)
                }
            };
            let ty = Type_::single(sp(eloc, SingleType_::Ref(mut_, inner)));
            let eborrow = match er.exp {
                sp!(_, TE::Use(v)) => TE::BorrowLocal(mut_, v),
                erexp => TE::TempBorrow(mut_, Box::new(T::exp(er.ty, erexp))),
            };
            (ty, eborrow)
        }

        NE::Borrow(mut_, ndotted) => match exp_dotted(context, "borrow", true, ndotted) {
            None => {
                assert!(context.has_errors());
                (Type_::anything(eloc), TE::UnresolvedError)
            }
            Some((edotted, _)) => {
                let eborrow = exp_dotted_to_borrow(context, eloc, mut_, edotted);
                (eborrow.ty, eborrow.exp.value)
            }
        },

        NE::DerefBorrow(ndotted) => {
            assert!(match ndotted {
                sp!(_, N::ExpDotted_::Exp(_)) => false,
                _ => true,
            });
            match exp_dotted(context, "dot access", true, ndotted) {
                None => {
                    assert!(context.has_errors());
                    (Type_::anything(eloc), TE::UnresolvedError)
                }
                Some((edotted, inner_ty)) => {
                    let ederefborrow = exp_dotted_to_owned_value(context, eloc, edotted, inner_ty);
                    (ederefborrow.ty, ederefborrow.exp.value)
                }
            }
        }

        NE::Annotate(nl, ty_annot) => {
            let el = exp(context, *nl);
            let annot_loc = ty_annot.loc;
            let rhs = core::instantiate(context, ty_annot);
            subtype(
                context,
                annot_loc,
                || "Invalid type annotation",
                &el.ty,
                &rhs,
            );
            (rhs, el.exp.value)
        }
        NE::UnresolvedError => {
            assert!(context.has_errors());
            (Type_::anything(eloc), TE::UnresolvedError)
        }
    };
    T::exp(ty, sp(eloc, e_))
}

fn loop_body(
    context: &mut Context,
    eloc: Loc,
    is_loop: bool,
    nloop: N::Exp,
) -> (bool, Type, Box<T::Exp>) {
    let old_in_loop = std::mem::replace(&mut context.in_loop, true);
    let old_break_type = std::mem::replace(&mut context.break_type, None);
    let eloop = exp(context, nloop);
    context.in_loop = old_in_loop;
    let break_type = std::mem::replace(&mut context.break_type, old_break_type);

    let lloc = eloop.exp.loc;
    subtype(
        context,
        lloc,
        || "Invalid loop body",
        &eloop.ty,
        &sp(lloc, Type_::Unit),
    );
    let has_break = break_type.is_some();
    let ty = if is_loop && !has_break {
        Type_::anything(lloc)
    } else {
        break_type.unwrap_or_else(|| sp(eloc, Type_::Unit))
    };
    (has_break, ty, eloop)
}

//**************************************************************************************************
// Locals and LValues
//**************************************************************************************************

fn get_local(context: &mut Context, loc: Loc, verb: &str, var: &Var) -> SingleType {
    match context.locals.get(var).cloned() {
        None => {
            context.error(vec![(
                loc,
                format!("Invalid {}. Unbound local '{}'", verb, var),
            )]);
            SingleType_::anything(loc)
        }
        Some(LocalStatus::Declared(dloc)) => {
            context.error(vec![
                (
                    loc,
                    format!(
                        "Invalid {}. Local '{}' was declared but could not infer the type. Try annotating the type here",
                        verb, var
                    ),
                ),
                (dloc, "Local declared but not assigned here".into()),
            ]);
            SingleType_::anything(loc)
        }
        Some(LocalStatus::Typed(st)) => st,
    }
}

fn binds_expected_types(
    context: &mut Context,
    sp!(_loc, bs_): &T::BindList,
) -> Vec<Option<N::SingleType>> {
    bs_.iter().map(|b| bind_expected_type(context, b)).collect()
}

fn bind_expected_type(_context: &mut Context, sp!(loc, b_): &T::Bind) -> Option<N::SingleType> {
    use N::{BaseType_ as B, SingleType_ as S};
    use T::Bind_ as TB;
    let loc = *loc;
    match b_ {
        TB::Ignore => None,
        TB::Var(_, ty_opt) => ty_opt.clone(),
        TB::BorrowUnpack(mut_, m, s, tys, _) => {
            let tn = sp(loc, N::TypeName_::ModuleType(m.clone(), s.clone()));
            Some(sp(
                loc,
                S::Ref(*mut_, sp(loc, B::Apply(None, tn, tys.clone()))),
            ))
        }
        TB::Unpack(m, s, tys, _) => {
            let tn = sp(loc, N::TypeName_::ModuleType(m.clone(), s.clone()));
            Some(S::base(sp(loc, B::Apply(None, tn, tys.clone()))))
        }
    }
}

fn close_scope(
    context: &mut Context,
    old_locals: UniqueMap<Var, LocalStatus>,
    declared: UniqueMap<Var, ()>,
    sp!(_, bs_): &mut T::BindList,
) {
    bs_.iter_mut().for_each(|b| close_scope_bind(context, b));

    // remove new locals from inner scope
    for (new_local, _) in declared.iter().filter(|(v, _)| !old_locals.contains_key(v)) {
        context.locals.remove(&new_local);
    }

    // return old status
    let shadowed = old_locals
        .into_iter()
        .filter(|(k, _)| declared.contains_key(k));
    for (var, shadowed_status) in shadowed {
        context.locals.remove(&var);
        context.locals.add(var, shadowed_status).unwrap();
    }
}

fn close_scope_bind(context: &mut Context, sp!(_, b_): &mut T::Bind) {
    use T::Bind_ as TB;
    match b_ {
        TB::Ignore => (),
        TB::Var(var, ty_opt) => {
            assert!(ty_opt.is_none());
            match context.locals.get(var).unwrap() {
                // Error reported in Expand
                LocalStatus::Declared(_loc) => (),
                LocalStatus::Typed(ty) => *ty_opt = Some(ty.clone()),
            };
        }
        TB::BorrowUnpack(_, _, _, _, fields) | TB::Unpack(_, _, _, fields) => fields
            .iter_mut()
            .for_each(|(_, (_, (_, fb)))| close_scope_bind(context, fb)),
    };
}

fn bind_list(
    context: &mut Context,
    sp!(loc, nbinds): N::BindList,
    ty_opt: Option<Type>,
) -> (UniqueMap<Var, ()>, T::BindList) {
    use Type_::*;
    let num_binds = nbinds.len();
    let mut mk =
        |tloc, args| make_list_types(context, loc, "list-binding", num_binds, tloc, args, None);
    let tys = match ty_opt {
        None => nbinds.iter().map(|_| None).collect(),
        Some(sp!(tloc, Unit)) => mk(tloc, vec![]),
        Some(sp!(tloc, Single(t))) => mk(tloc, vec![Some(t)]),
        Some(sp!(tloc, Multiple(ts))) => mk(tloc, ts.into_iter().map(Some).collect()),
    };
    assert!(tys.len() == num_binds);
    let mut declared: UniqueMap<Var, ()> = UniqueMap::new();
    let tbinds = nbinds
        .into_iter()
        .zip(tys)
        .map(|(b, t)| bind(context, &mut declared, b, t))
        .collect();
    (declared, sp(loc, tbinds))
}

fn bind(
    context: &mut Context,
    declared: &mut UniqueMap<Var, ()>,
    sp!(loc, nbind_): N::Bind,
    ty_opt: Option<SingleType>,
) -> T::Bind {
    use N::Bind_ as NB;
    use T::Bind_ as TB;
    let tbind_ = match nbind_ {
        NB::Ignore => {
            if let Some(st) = ty_opt {
                context.add_copyable_constraint(
                    loc,
                    "Cannot ignore resource values. The value must be used",
                    st,
                )
            }
            TB::Ignore
        }
        NB::Var(var) => {
            context.declare_local(var.clone(), ty_opt);
            if let Err(prev_loc) = declared.add(var.clone(), ()) {
                context.error(vec![
                    (
                        var.loc(),
                        format!(
                            "Duplicate declaration for local '{}' in a given 'let'",
                            &var
                        ),
                    ),
                    (prev_loc, "Previously declared here".into()),
                ]);
            }
            TB::Var(var, None)
        }
        NB::Unpack(m, n, ty_args_opt, fields) => {
            let current_module = &context.current_module.clone().unwrap();
            let (bt, targs) = core::make_struct_type(context, loc, &m, &n, ty_args_opt);
            let ref_mut: Option<bool>;
            if let Some(ty) = ty_opt {
                let inner = match ty.value {
                    SingleType_::Ref(mut_, inner) => {
                        ref_mut = Some(mut_);
                        inner
                    }
                    SingleType_::Base(inner) => {
                        ref_mut = None;
                        inner
                    }
                };
                let lhs = &bt;
                let rhs = &inner;
                join_base(context, loc, || "Invalid deconstruction binding", lhs, rhs);
            } else {
                ref_mut = None;
            }
            let typed_fields =
                add_field_types(context, loc, "binding", &m, &n, targs.clone(), fields);
            let tfields = typed_fields.map(|f, (idx, (fty, nb))| {
                let nb_ty = match ref_mut {
                    None => SingleType_::base(fty.clone()),
                    Some(mut_) => sp(f.loc(), SingleType_::Ref(mut_, fty.clone())),
                };
                let tb = bind(context, declared, nb, Some(nb_ty));
                (idx, (fty, tb))
            });
            if &m != current_module {
                context.error(
                    vec![
                        (loc, format!("Invalid deconstruction binding of '{}::{}'", &m, &n)),
                        (current_module.loc(), "Currently, all structs can only be deconstructed in module that they were declared".into())
                    ]
                )
            }
            match ref_mut {
                None => TB::Unpack(m, n, targs, tfields),
                Some(mut_) => TB::BorrowUnpack(mut_, m, n, targs, tfields),
            }
        }
    };
    sp(loc, tbind_)
}

fn assigns_expected_types(
    context: &mut Context,
    sp!(_loc, as_): &T::AssignList,
) -> Vec<Option<N::SingleType>> {
    as_.iter()
        .map(|a| assign_expected_type(context, a))
        .collect()
}

fn assign_expected_type(_context: &mut Context, sp!(loc, a_): &T::Assign) -> Option<N::SingleType> {
    use N::{BaseType_ as B, SingleType_ as S};
    use T::Assign_ as TA;
    let loc = *loc;
    match a_ {
        TA::Ignore => None,
        TA::Var(_, st) => Some(st.clone()),
        TA::BorrowUnpack(mut_, m, s, tys, _) => {
            let tn = sp(loc, N::TypeName_::ModuleType(m.clone(), s.clone()));
            Some(sp(
                loc,
                S::Ref(*mut_, sp(loc, B::Apply(None, tn, tys.clone()))),
            ))
        }
        TA::Unpack(m, s, tys, _) => {
            let tn = sp(loc, N::TypeName_::ModuleType(m.clone(), s.clone()));
            Some(S::base(sp(loc, B::Apply(None, tn, tys.clone()))))
        }
    }
}

fn checked_rvalue_tys(
    context: &mut Context,
    loc: Loc,
    verb: &str,
    arity: usize,
    rvalue_ty: Type,
) -> Vec<SingleType> {
    use Type_::*;
    let mut mk = |tloc: Loc, args: Vec<SingleType>| {
        make_list_types(
            context,
            loc,
            verb,
            arity,
            tloc,
            args,
            SingleType_::anything(loc),
        )
    };
    match rvalue_ty {
        sp!(tloc, Unit) => mk(tloc, vec![]),
        sp!(tloc, Single(t)) => mk(tloc, vec![t]),
        sp!(tloc, Multiple(ts)) => mk(tloc, ts),
    }
}

fn assign_list(
    context: &mut Context,
    sp!(loc, nassigns): N::AssignList,
    rvalue_ty: Type,
) -> T::AssignList {
    let num_assigns = nassigns.len();
    let tys = checked_rvalue_tys(context, loc, "list-assignment", num_assigns, rvalue_ty);
    assert!(tys.len() == num_assigns);
    let mut assigned: UniqueMap<Var, ()> = UniqueMap::new();
    let tassigns = nassigns
        .into_iter()
        .zip(tys)
        .map(|(a, t)| assign(context, &mut assigned, a, t))
        .collect();
    sp(loc, tassigns)
}

fn assign(
    context: &mut Context,
    assigned: &mut UniqueMap<Var, ()>,
    sp!(aloc, na_): N::Assign,
    rvalue_ty: SingleType,
) -> T::Assign {
    use N::Assign_ as NA;
    use T::Assign_ as TA;
    let ta_ = match na_ {
        NA::Ignore => {
            context.add_copyable_constraint(
                aloc,
                "Cannot ignore resource values. The value must be used",
                rvalue_ty,
            );
            TA::Ignore
        }
        NA::Var(var) => {
            let vty = match context.locals.get(&var).cloned() {
                None => {
                    context.error(vec![(
                        aloc,
                        format!("Invalid assignment. Unbound local '{}'", &var),
                    )]);
                    SingleType_::anything(aloc)
                }
                Some(LocalStatus::Declared(_)) => {
                    context.declare_local(var.clone(), Some(rvalue_ty.clone()));
                    rvalue_ty
                }
                Some(LocalStatus::Typed(var_ty)) => subtype_single(
                    context,
                    aloc,
                    || format!("Invalid assignment to local '{}'", &var),
                    &rvalue_ty,
                    &var_ty,
                ),
            };
            if let Err(prev_loc) = assigned.add(var.clone(), ()) {
                context.error(vec![
                    (
                        var.loc(),
                        format!("Duplicate usage of local '{}' in a given assignment", &var),
                    ),
                    (prev_loc, "Previously assigned here".into()),
                ]);
            }
            TA::Var(var, vty)
        }

        NA::Unpack(m, n, ty_args_opt, fields) => {
            let current_module = &context.current_module.clone().unwrap();
            let (bt, targs) =
                core::make_struct_type(context, aloc, current_module, &n, ty_args_opt);
            let (lhs, ref_mut) = match &rvalue_ty.value {
                SingleType_::Ref(mut_, _) => (
                    Type_::single(sp(aloc, SingleType_::Ref(*mut_, bt))),
                    Some(*mut_),
                ),
                SingleType_::Base(_) => (Type_::base(bt), None),
            };
            let rhs = &Type_::single(rvalue_ty);
            subtype(
                context,
                aloc,
                || "Invalid deconstruction assignment",
                &lhs,
                rhs,
            );
            let typed_fields = add_field_types(
                context,
                aloc,
                "assignment",
                current_module,
                &n,
                targs.clone(),
                fields,
            );
            let tfields = typed_fields.map(|_, (idx, (fty, na))| {
                let fixed_fty = match ref_mut {
                    None => SingleType_::base(fty.clone()),
                    Some(mut_) => sp(aloc, SingleType_::Ref(mut_, fty.clone())),
                };
                let tb = assign(context, assigned, na, fixed_fty);
                (idx, (fty, tb))
            });
            if &m != current_module {
                context.error(
                    vec![
                        (aloc, format!("Invalid deconstruction assignment of '{}::{}'", &m, &n)),
                        (current_module.loc(), "Currently, all structs can only be deconstructed in module that they were declared".into())
                    ]
                )
            }
            match ref_mut {
                None => TA::Unpack(m, n, targs, tfields),
                Some(mut_) => TA::BorrowUnpack(mut_, m, n, targs, tfields),
            }
        }
    };
    sp(aloc, ta_)
}

fn check_mutation(
    context: &mut Context,
    loc: Loc,
    maybe_given_ref: Type,
    rvalue_ty: SingleType,
) -> SingleType {
    let inner = sp(loc, BaseType_::Var(TVar::next()));
    let ref_ty = sp(loc, SingleType_::Ref(true, inner.clone()));
    let res_ty = subtype(
        context,
        loc,
        || "Invalid mutation. Expected a mutable reference",
        &maybe_given_ref,
        &Type_::single(ref_ty),
    );
    let res_st = match res_ty.value {
        Type_::Single(st) => st,
        _ => panic!("ICE join with a single should give a single"),
    };
    subtype_single(
        context,
        loc,
        || "Invalid mutation. New value is not valid for the reference",
        &rvalue_ty,
        &SingleType_::base(inner.clone()),
    );
    context.add_copyable_constraint(
        loc,
        "Invalid mutation. Can only assign to references of a copyable type",
        SingleType_::base(inner),
    );
    res_st
}

fn make_list_types<T: Clone>(
    context: &mut Context,
    loc: Loc,
    verb: &str,
    arity: usize,
    gloc: Loc,
    mut given: Vec<T>,
    default: T,
) -> Vec<T> {
    let given_len = given.len();
    if given_len != arity {
        context.error(vec![
            (
                loc,
                format!(
                    "Invalid {0}. The {0} expected {1} argument(s) but got {2}",
                    verb, arity, given_len
                ),
            ),
            (gloc, format!("Found {} arguments here", given_len)),
        ])
    }
    if given_len < arity {
        while given.len() < arity {
            given.push(default.clone())
        }
    } else {
        while given.len() > arity {
            given.pop();
        }
    }
    given
}

//**************************************************************************************************
// Fields
//**************************************************************************************************

fn resolve_field(context: &mut Context, loc: Loc, ty: BaseType, field: &Field) -> BaseType {
    use BaseType_::*;
    use TypeName_::*;
    let msg = || format!("Unbound field '{}'", field);
    match core::unfold_type_base(&context.subst, ty) {
        sp!(tloc, Anything) => {
            context.error(vec![
                (loc, msg()),
                (tloc, "Could not infer the type. Try annotating here".into()),
            ]);

            BaseType_::anything(loc)
        }
        sp!(_, Apply(_, sp!(_, ModuleType(m, n)), targs)) => {
            let current_module = context.current_module.clone().unwrap();
            if m != current_module {
                context.error(vec![
                    (loc, format!("Invalid access of field '{}' on '{}::{}'. Fields can only be accessed inside the struct's module", field, &m, &n)),

                ])
            }
            core::make_field_type(context, loc, &m, &n, targs, field)
        }
        sp!(tloc, t) => {
            context.error(vec![
                (loc, msg()),
                (
                    tloc,
                    format!(
                        "Expected a struct type in the current module but got: {}",
                        t.subst_format(&context.subst)
                    ),
                ),
            ]);
            BaseType_::anything(loc)
        }
    }
}

fn add_field_types<T>(
    context: &mut Context,
    loc: Loc,
    verb: &str,
    m: &ModuleIdent,
    n: &StructName,
    targs: Vec<BaseType>,
    fields: Fields<T>,
) -> Fields<(BaseType, T)> {
    let maybe_fields_ty = core::make_field_types(context, loc, m, n, targs);
    let mut fields_ty = match maybe_fields_ty {
        N::StructFields::Defined(m) => m,
        N::StructFields::Native(nloc) => {
            context.error(vec![
                (loc, format!("Invalid {} usage for native struct '{}::{}'. Native structs cannot be directly constructed/deconstructd, and their fields cannot be dirctly accessed", verb, m, n)),
                (nloc, "Declared 'native' here".into())
            ]);
            return fields.map(|f, (idx, x)| (idx, (sp(f.loc(), BaseType_::Anything), x)));
        }
    };
    for (f, _) in fields_ty.iter() {
        if fields.get(&f).is_none() {
            context.error(vec![(
                loc,
                format!("Missing {} for field '{}' in '{}::{}'", verb, f, m, n),
            )])
        }
    }
    fields.map(|f, (idx, x)| {
        let fty = match fields_ty.remove(&f) {
            None => {
                context.error(vec![(
                    loc,
                    format!("Unbound field '{}' in '{}::{}'", &f, m, n),
                )]);
                sp(f.loc(), BaseType_::Anything)
            }
            Some((_, fty)) => fty,
        };
        (idx, (fty, x))
    })
}

enum ExpDotted_ {
    Exp(Box<T::Exp>),
    TmpBorrow(Box<T::Exp>, Box<BaseType>),
    Dot(Box<ExpDotted>, Field, Box<BaseType>),
}
type ExpDotted = Spanned<ExpDotted_>;

fn exp_dotted(
    context: &mut Context,
    verb: &str,
    borrow_root: bool,
    sp!(dloc, ndot_): N::ExpDotted,
) -> Option<(ExpDotted, BaseType)> {
    use N::ExpDotted_ as NE;
    let (edot_, bt) = match ndot_ {
        NE::Exp(ne) => {
            use SingleType_::*;
            use Type_::*;
            let e = exp(context, *ne);
            let (borrow_needed, bt) = match &e.ty {
                sp!(tloc, t@Unit) | sp!(tloc, t@Multiple(_)) => {
                    let subst_t = t.subst_format(&context.subst);
                    context.error(vec![
                        (e.exp.loc, format!("Invalid {}. Expected an expression of a single type but got an expression of type: '{}'", verb, subst_t)),
                        (*tloc, "Type found here".into()),
                    ]);
                    return None;
                }
                sp!(_, Single(t @ sp!(_, Ref(_, _)))) => {
                    let inner = sp(dloc, BaseType_::Var(TVar::next()));
                    let ref_ty = sp(dloc, SingleType_::Ref(false, inner.clone()));
                    subtype_single(
                        context,
                        dloc,
                        || "ICE any ref should be a subtype of &_",
                        t,
                        &ref_ty,
                    );
                    (false, inner)
                }
                sp!(_, Single(sp!(_, Base(bt)))) => (true, bt.clone()),
            };
            let edot_ = if borrow_root && borrow_needed {
                ExpDotted_::TmpBorrow(e, Box::new(bt.clone()))
            } else {
                ExpDotted_::Exp(e)
            };
            (edot_, bt)
        }
        NE::Dot(nlhs, field) => {
            let (lhs, inner) = exp_dotted(context, "dot access", true, *nlhs)?;
            let field_ty = resolve_field(context, dloc, inner, &field);
            (
                ExpDotted_::Dot(Box::new(lhs), field, Box::new(field_ty.clone())),
                field_ty,
            )
        }
    };
    Some((sp(dloc, edot_), bt))
}

fn exp_dotted_to_borrow(
    context: &mut Context,
    eloc: Loc,
    mut_: bool,
    sp!(loc, dot_): ExpDotted,
) -> T::Exp {
    use SingleType_ as S;
    use T::UnannotatedExp_ as TE;
    match dot_ {
        ExpDotted_::Exp(e) => *e,
        ExpDotted_::TmpBorrow(eb, bt) => {
            let eb_ty = eb.ty;
            let sp!(ebloc, eb_) = eb.exp;
            let e_ = match eb_ {
                TE::Use(v) => TE::BorrowLocal(mut_, v),
                eb_ => {
                    match &eb_ {
                        TE::Move { from_user, .. } | TE::Copy { from_user, .. } => {
                            assert!(*from_user);
                        }
                        _ => (),
                    }
                    TE::TempBorrow(mut_, Box::new(T::exp(eb_ty, sp(ebloc, eb_))))
                }
            };
            let ty = Type_::single(sp(eloc, SingleType_::Ref(mut_, *bt)));
            T::exp(ty, sp(loc, e_))
        }
        ExpDotted_::Dot(lhs, field, field_ty) => {
            let lhs_borrow = exp_dotted_to_borrow(context, eloc, mut_, *lhs);
            let lhs_mut = match &lhs_borrow.ty.value {
                Type_::Single(sp!(_, S::Ref(lhs_mut, _))) => *lhs_mut,
                _ => panic!("ICE expected a ref from exp_dotted borrow, otherwise should have gotten a TmpBorrow"),
            };
            // lhs is immutable and current borrow is mutable
            if !lhs_mut && mut_ {
                context.error(vec![
                    (eloc, "Invalid mutable borrow from an immutable reference"),
                    (lhs_borrow.ty.loc, "Immutable because of this position"),
                ])
            }
            let e_ = TE::Borrow(mut_, Box::new(lhs_borrow), field);
            let ty = Type_::single(sp(eloc, SingleType_::Ref(mut_, *field_ty)));
            T::exp(ty, sp(loc, e_))
        }
    }
}

fn exp_dotted_to_owned_value(
    context: &mut Context,
    eloc: Loc,
    edot: ExpDotted,
    inner_ty: BaseType,
) -> T::Exp {
    use T::UnannotatedExp_ as TE;
    match edot {
        sp!(_, ExpDotted_::Exp(lhs)) => *lhs,
        edot => {
            let name = match &edot {
                sp!(_, ExpDotted_::Exp(_)) => panic!("ICE covered above"),
                sp!(_, ExpDotted_::TmpBorrow(_, _)) => panic!("ICE why is this here?"),
                sp!(_, ExpDotted_::Dot(_, name, _)) => name.clone(),
            };
            let eborrow = exp_dotted_to_borrow(context, eloc, false, edot);
            context.add_implicit_copyable_constraint(
                eloc,
                format!("Invalid implicit copy of field '{}'.", name),
                inner_ty.clone(),
                "Try adding '*&' to the front of the field access",
            );
            T::exp(
                Type_::base(inner_ty),
                sp(eloc, TE::Dereference(Box::new(eborrow))),
            )
        }
    }
}

//**************************************************************************************************
// Calls
//**************************************************************************************************

fn module_call(
    context: &mut Context,
    loc: Loc,
    m: ModuleIdent,
    f: FunctionName,
    ty_args_opt: Option<Vec<BaseType>>,
    args: T::Exp,
) -> (Type, T::UnannotatedExp_) {
    let (_, ty_args, parameters, acquires, ret_ty) =
        core::make_function_type(context, loc, &m, &f, ty_args_opt);
    let (aloc, arg_tys) = match &args.ty {
        sp!(_, Type_::Unit) => (args.exp.loc, vec![]),
        sp!(_, Type_::Single(s)) => (args.exp.loc, vec![s.clone()]),
        sp!(_, Type_::Multiple(ss)) => (args.exp.loc, ss.clone()),
    };
    let tany = SingleType_::anything(loc);
    let cstr = format!("call of '{}::{}'", &m, &f);
    let arg_tys = make_list_types(context, loc, &cstr, parameters.len(), aloc, arg_tys, tany);
    assert!(arg_tys.len() == parameters.len());
    for ((param, param_ty), arg_ty) in parameters.iter().zip(&arg_tys) {
        subtype_single(
            context,
            loc,
            || {
                format!(
                    "Invalid call of '{}::{}'. Invalid argument for parameter '{}'",
                    &m, &f, param
                )
            },
            arg_ty,
            param_ty,
        );
    }
    let params_ty_list = parameters.into_iter().map(|(_, ty)| ty).collect();
    let call = T::ModuleCall {
        module: m,
        name: f,
        type_arguments: ty_args,
        arguments: Box::new(args),
        parameter_types: params_ty_list,
        acquires,
    };
    (ret_ty, T::UnannotatedExp_::ModuleCall(Box::new(call)))
}

fn builtin_call(
    context: &mut Context,
    loc: Loc,
    sp!(bloc, nb_): N::BuiltinFunction,
    args: T::Exp,
) -> (Type, T::UnannotatedExp_) {
    use N::BuiltinFunction_ as NB;
    use T::BuiltinFunction_ as TB;
    let new_tvar = || sp(bloc, BaseType_::Var(TVar::next()));
    let mut mk_ty_arg = |ty_arg_opt| match ty_arg_opt {
        None => new_tvar(),
        Some(ty_arg) => core::instantiate_base(context, ty_arg),
    };
    let (b_, params_ty, ret_ty);
    match nb_ {
        NB::MoveToSender(ty_arg_opt) => {
            let ty_arg = mk_ty_arg(ty_arg_opt);
            b_ = TB::MoveToSender(ty_arg.clone());
            params_ty = vec![SingleType_::base(ty_arg)];
            ret_ty = sp(loc, Type_::Unit);
        }
        NB::MoveFrom(ty_arg_opt) => {
            let ty_arg = mk_ty_arg(ty_arg_opt);
            b_ = TB::MoveFrom(ty_arg.clone());
            params_ty = vec![SingleType_::address(bloc)];
            ret_ty = Type_::base(ty_arg);
        }
        NB::BorrowGlobal(mut_, ty_arg_opt) => {
            let ty_arg = mk_ty_arg(ty_arg_opt);
            b_ = TB::BorrowGlobal(mut_, ty_arg.clone());
            params_ty = vec![SingleType_::address(bloc)];
            ret_ty = Type_::single(sp(loc, SingleType_::Ref(mut_, ty_arg)));
        }
        NB::Exists(ty_arg_opt) => {
            let ty_arg = mk_ty_arg(ty_arg_opt);
            b_ = TB::Exists(ty_arg);
            params_ty = vec![SingleType_::address(bloc)];
            ret_ty = Type_::bool(loc);
        }
        NB::Freeze(ty_arg_opt) => {
            let ty_arg = mk_ty_arg(ty_arg_opt);
            b_ = TB::Freeze(ty_arg.clone());
            params_ty = vec![sp(bloc, SingleType_::Ref(true, ty_arg.clone()))];
            ret_ty = Type_::single(sp(loc, SingleType_::Ref(false, ty_arg)));
        }
    };
    let (aloc, arg_tys) = match &args.ty {
        sp!(_, Type_::Unit) => (args.exp.loc, vec![]),
        sp!(_, Type_::Single(s)) => (args.exp.loc, vec![s.clone()]),
        sp!(_, Type_::Multiple(ss)) => (args.exp.loc, ss.clone()),
    };
    let tany = SingleType_::anything(loc);
    let cstr = format!("call of '{}'", &b_);
    let arg_tys = make_list_types(context, loc, &cstr, params_ty.len(), aloc, arg_tys, tany);
    assert!(arg_tys.len() == params_ty.len());
    for ((idx, param_ty), arg_ty) in params_ty.into_iter().enumerate().zip(arg_tys) {
        subtype_single(
            context,
            loc,
            || {
                format!(
                    "Invalid call of '{}'. Invalid argument for parameter '{}'",
                    &b_, idx
                )
            },
            &arg_ty,
            &param_ty,
        );
    }
    (
        ret_ty,
        T::UnannotatedExp_::Builtin(Box::new(sp(bloc, b_)), Box::new(args)),
    )
}

// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::shared::unique_map::UniqueMap;
use crate::{
    errors::*,
    expansion::ast as E,
    naming::ast as N,
    parser::ast::{Field, FunctionName, Kind, Kind_, ModuleIdent, ResourceLoc, StructName, Var},
    shared::*,
};
use std::collections::{BTreeMap, BTreeSet, HashMap};

//**************************************************************************************************
// Context
//**************************************************************************************************

struct Context {
    errors: Errors,
    current_module: Option<ModuleIdent>,
    tparams: UniqueMap<Name, N::TParam>,
    structs: UniqueMap<ModuleIdent, UniqueMap<StructName, ResourceLoc>>,
    functions: UniqueMap<ModuleIdent, UniqueMap<FunctionName, ()>>,
}
impl Context {
    fn new(prog: &E::Program, errors: Errors) -> Self {
        Self {
            errors,
            current_module: None,
            tparams: UniqueMap::new(),
            structs: prog
                .modules
                .ref_map(|_, m| m.structs.ref_map(|_, s| s.resource_opt)),
            functions: prog.modules.ref_map(|_, m| m.functions.ref_map(|_, _| ())),
        }
    }

    fn error(&mut self, e: Vec<(Loc, impl Into<String>)>) {
        self.errors
            .push(e.into_iter().map(|(loc, msg)| (loc, msg.into())).collect())
    }

    fn get_errors(self) -> Errors {
        self.errors
    }

    fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    #[allow(clippy::option_option)]
    fn resolve_struct(
        &mut self,
        loc: Loc,
        m: &ModuleIdent,
        s: &StructName,
    ) -> Option<Option<Kind>> {
        let mstructs = match self.structs.get(m) {
            None => {
                self.error(vec![(loc, format!("Unbound module '{}'", m,))]);
                return None;
            }
            Some(mstructs) => mstructs,
        };
        match mstructs.get(s).cloned() {
            None => {
                self.error(vec![(
                    loc,
                    format!("Unbound struct or resource '{}' in module '{}'", s, m),
                )]);
                None
            }
            Some(Some(resource_loc)) => Some(Some(sp(resource_loc, Kind_::Resource))),
            Some(None) => Some(None),
        }
    }

    fn maybe_resolve_local_struct_name(&mut self, n: Name) -> Option<(ModuleIdent, StructName)> {
        match &self.current_module {
            None => None,
            Some(mident) => {
                let sn = StructName(n);
                if self.structs.get(mident)?.contains_key(&sn) {
                    Some((mident.clone(), sn))
                } else {
                    None
                }
            }
        }
    }

    fn resolve_struct_name(
        &mut self,
        sp!(loc, tn_): E::TypeName,
    ) -> Option<(ModuleIdent, StructName)> {
        match tn_ {
            E::TypeName_::Name(n) => match &self.current_module {
                None => {
                    self.error(vec![(
                        loc,
                        format!("Unbound struct '{}'. Not currently in a module", n),
                    )]);
                    None
                }
                Some(mident) => {
                    let m = mident.clone();
                    let sn = StructName(n);
                    self.resolve_struct(loc, &m, &sn)?;
                    Some((m, sn))
                }
            },
            E::TypeName_::ModuleType(m, sn) => {
                self.resolve_struct(loc, &m, &sn)?;
                Some((m, sn))
            }
        }
    }

    fn maybe_resolve_function_name(&mut self, n: &Name) -> Option<(ModuleIdent, FunctionName)> {
        match &self.current_module {
            None => None,
            Some(mident) => {
                let m = mident.clone();
                let f = FunctionName(n.clone());
                if self
                    .functions
                    .get(&m)
                    .expect("ICE unbound current module")
                    .contains_key(&f)
                {
                    Some((m, f))
                } else {
                    None
                }
            }
        }
    }
}

//**************************************************************************************************
// Entry
//**************************************************************************************************

pub fn program(prog: E::Program, errors: Errors) -> (N::Program, Errors) {
    let mut context = Context::new(&prog, errors);
    let modules = modules(&mut context, prog.modules);
    let main = main_function(&mut context, prog.main);
    (N::Program { modules, main }, context.get_errors())
}

fn modules(
    context: &mut Context,
    modules: UniqueMap<ModuleIdent, E::ModuleDefinition>,
) -> UniqueMap<ModuleIdent, N::ModuleDefinition> {
    modules.map(|ident, mdef| module(context, ident, mdef))
}

fn module(
    context: &mut Context,
    ident: ModuleIdent,
    mdef: E::ModuleDefinition,
) -> N::ModuleDefinition {
    context.current_module = Some(ident);
    let is_source_module = mdef.is_source_module;
    let structs = mdef.structs.map(|name, s| struct_def(context, name, s));
    let functions = mdef.functions.map(|name, f| function(context, name, f));
    N::ModuleDefinition {
        is_source_module,
        structs,
        functions,
    }
}

fn main_function(
    context: &mut Context,
    main: Option<(Address, FunctionName, E::Function)>,
) -> Option<(Address, FunctionName, N::Function)> {
    match main {
        None => None,
        Some((addr, name, f)) => {
            if let Some((tparam, _)) = f.signature.type_parameters.get(0) {
                context.error(
                    vec![(tparam.loc, format!("Invalid '{}' declaration. Found type parameter '{}'. The main function cannot have type parameters", &name, tparam))]
                );
            }
            Some((addr, name.clone(), function(context, name, f)))
        }
    }
}

//**************************************************************************************************
// Functions
//**************************************************************************************************

fn function(context: &mut Context, _name: FunctionName, f: E::Function) -> N::Function {
    context.tparams = UniqueMap::new();
    let visibility = f.visibility;
    let signature = function_signature(context, f.signature);
    let acquires = function_acquires(context, f.acquires);
    let body = function_body(context, f.body);
    N::Function {
        visibility,
        signature,
        acquires,
        body,
    }
}

fn function_signature(context: &mut Context, sig: E::FunctionSignature) -> N::FunctionSignature {
    let type_parameters = type_parameters(context, sig.type_parameters);
    let parameters = sig
        .parameters
        .into_iter()
        .map(|(v, ty)| (v, single_type(context, ty)))
        .collect();
    let return_type = type_(context, sig.return_type);
    N::FunctionSignature {
        type_parameters,
        parameters,
        return_type,
    }
}

fn function_body(context: &mut Context, sp!(loc, b_): E::FunctionBody) -> N::FunctionBody {
    match b_ {
        E::FunctionBody_::Native => sp(loc, N::FunctionBody_::Native),
        E::FunctionBody_::Defined(es) => sp(loc, N::FunctionBody_::Defined(sequence(context, es))),
    }
}

fn function_acquires(
    context: &mut Context,
    eacquires: Vec<E::SingleType>,
) -> BTreeSet<N::BaseType> {
    let mut acquires = BTreeMap::new();
    for eacquire in eacquires {
        let bt = base_type(context, eacquire);
        let new_loc = bt.loc;
        if let Some(old_loc) = acquires.insert(bt, new_loc) {
            context.error(vec![
                (new_loc, "Duplicate acquires item"),
                (old_loc, "Previously listed here"),
            ])
        }
    }

    acquires.into_iter().map(|(b, _)| b).collect()
}

//**************************************************************************************************
// Structs
//**************************************************************************************************

fn struct_def(
    context: &mut Context,
    name: StructName,
    sdef: E::StructDefinition,
) -> N::StructDefinition {
    context.tparams = UniqueMap::new();
    let resource_opt = sdef.resource_opt;
    let type_parameters = type_parameters(context, sdef.type_parameters);
    let fields = struct_fields(context, sdef.fields);
    match (&resource_opt, &fields) {
        (Some(_), _) | (_, N::StructFields::Native(_)) => (),
        (None, N::StructFields::Defined(fields)) => {
            for (field, idx_ty) in fields.iter() {
                check_no_nominal_resources(context, &name, &field, &idx_ty.1);
            }
        }
    }
    N::StructDefinition {
        resource_opt,
        type_parameters,
        fields,
    }
}

fn struct_fields(context: &mut Context, efields: E::StructFields) -> N::StructFields {
    match efields {
        E::StructFields::Native(loc) => N::StructFields::Native(loc),
        E::StructFields::Defined(em) => {
            N::StructFields::Defined(em.map(|_f, (idx, t)| (idx, base_type(context, t))))
        }
    }
}

fn check_no_nominal_resources(
    context: &mut Context,
    s: &StructName,
    field: &Field,
    ty: &N::BaseType,
) {
    use N::BaseType_ as T;
    match ty {
        sp!(tloc, T::Apply(Some(sp!(kloc, Kind_::Resource)), _, _)) => {
            context.error(vec![
                (field.loc(), format!("Invalid resource field '{}' for struct '{}'. Structs cannot contain resource types, except through type parameters", field, s)),
                (*tloc, format!("Field '{}' is a resource due to the type: '{}'", field, ty.value.subst_format(&HashMap::new()))),
                (*kloc, format!("Type '{}' was declared as a resource here", ty.value.subst_format(&HashMap::new()))),
                (s.loc(), format!("'{}' declared as a `struct` here", s)),
            ])
        }
        sp!(_, T::Apply(None, _, tyl)) => {
            tyl.iter().for_each(|t| check_no_nominal_resources(context, s, field, t))
        }
        _ => ()
    }
}

//**************************************************************************************************
// Types
//**************************************************************************************************

fn type_parameters(context: &mut Context, type_parameters: Vec<(Name, Kind)>) -> Vec<N::TParam> {
    type_parameters
        .into_iter()
        .map(|(name, kind)| {
            let id = N::TParamID::next();
            let debug = name.clone();
            let tp = N::TParam { id, debug, kind };
            if let Err(old_loc) = context.tparams.add(name.clone(), tp.clone()) {
                context.error(vec![
                    (
                        name.loc,
                        format!("Duplicate type parameter declared with name '{}'", name),
                    ),
                    (old_loc, "Previously defined here".to_string()),
                ])
            }
            tp
        })
        .collect()
}

fn base_types(context: &mut Context, tys: Vec<E::SingleType>) -> Vec<N::BaseType> {
    tys.into_iter().map(|t| base_type(context, t)).collect()
}

fn base_type(context: &mut Context, ty: E::SingleType) -> N::BaseType {
    use E::{SingleType_ as ES, TypeName_ as EN};
    use N::{BaseType_ as NB, TypeName_ as NN};
    let tyloc = ty.loc;
    let n_ty_ = match ty.value {
        ES::UnresolvedError => {
            assert!(context.has_errors());
            NB::Anything
        }
        ES::Apply(sp!(loc, EN::Name(n)), tys) => {
            if let Some(tp) = context.tparams.get(&n) {
                if !tys.is_empty() {
                    context.error(vec![(
                        tyloc,
                        "Generic type parameters cannot take type arguments",
                    )]);
                    NB::Anything
                } else {
                    NB::Param(tp.clone())
                }
            } else if let Some((m, s)) = context.maybe_resolve_local_struct_name(n.clone()) {
                match context.resolve_struct(loc, &m, &s) {
                    None => {
                        assert!(context.has_errors());
                        NB::Anything
                    }
                    Some(kind) => {
                        let tn = sp(loc, NN::ModuleType(m, s));
                        NB::Apply(kind, tn, base_types(context, tys))
                    }
                }
            } else if let Some(builtin_) = N::BuiltinTypeName_::resolve(&n.value) {
                let kind = sp(n.loc, builtin_.kind());
                let bn = sp(n.loc, builtin_);
                let tn = sp(loc, N::TypeName_::Builtin(bn));
                NB::Apply(Some(kind), tn, base_types(context, tys))
            } else {
                context.error(vec![(loc, format!("Could not resolve type name: '{}'", n))]);
                NB::Anything
            }
        }
        ES::Apply(sp!(loc, EN::ModuleType(m, s)), tys) => {
            match context.resolve_struct(loc, &m, &s) {
                None => {
                    assert!(context.has_errors());
                    NB::Anything
                }
                Some(resource_opt) => {
                    let tn = sp(loc, NN::ModuleType(m, s));
                    NB::Apply(resource_opt, tn, base_types(context, tys))
                }
            }
        }
        t @ ES::Ref(_, _) => {
            context
                    .error(vec![(tyloc, format!("Invalid usage of reference type: '{}'. Reference types cannot be used as type arguments.", t))]);
            NB::Anything
        }
    };
    sp(tyloc, n_ty_)
}

fn single_type(context: &mut Context, ty: E::SingleType) -> N::SingleType {
    use E::SingleType_ as ES;
    use N::SingleType_ as NS;
    let tyloc = ty.loc;
    let n_ty_ = match ty.value {
        ES::Ref(mut_, inner) => NS::Ref(mut_, base_type(context, *inner)),
        _ => NS::Base(base_type(context, ty)),
    };
    sp(tyloc, n_ty_)
}

fn type_(context: &mut Context, sp!(loc, ty_): E::Type) -> N::Type {
    use E::Type_ as ET;
    use N::Type_ as NT;
    match ty_ {
        ET::Unit => sp(loc, NT::Unit),
        ET::Single(t) => sp(loc, NT::Single(single_type(context, t))),
        ET::Multiple(tys) => sp(
            loc,
            NT::Multiple(tys.into_iter().map(|t| single_type(context, t)).collect()),
        ),
    }
}

//**************************************************************************************************
// Exp
//**************************************************************************************************

fn sequence(context: &mut Context, seq: E::Sequence) -> N::Sequence {
    seq.into_iter().map(|s| sequence_item(context, s)).collect()
}

fn sequence_item(context: &mut Context, sp!(loc, ns_): E::SequenceItem) -> N::SequenceItem {
    use E::SequenceItem_ as ES;
    use N::SequenceItem_ as NS;

    let s_ = match ns_ {
        ES::Seq(e) => NS::Seq(exp_(context, e)),
        ES::Declare(b, ty_opt) => {
            let bind_opt = bind_list(context, b);
            let tys = ty_opt.map(|t| type_(context, t));
            match bind_opt {
                None => {
                    assert!(context.has_errors());
                    NS::Seq(sp(loc, N::Exp_::UnresolvedError))
                }
                Some(bind) => NS::Declare(bind, tys),
            }
        }
        ES::Bind(b, e) => {
            let bind_opt = bind_list(context, b);
            let e = exp_(context, e);
            match bind_opt {
                None => {
                    assert!(context.has_errors());
                    NS::Seq(sp(loc, N::Exp_::UnresolvedError))
                }
                Some(bind) => NS::Bind(bind, e),
            }
        }
    };
    sp(loc, s_)
}

fn exp_vec(context: &mut Context, es: Vec<E::Exp>) -> Vec<N::Exp> {
    es.into_iter().map(|e| exp_(context, e)).collect()
}

// Macthes the LHS and field of a single dot
macro_rules! dot {
    ($lhs:pat, $field:pat) => {
        sp!(_, E::ExpDotted_::Dot($lhs, $field))
    };
}

// Macthes an root expression (no field access) in a spot where a dot chain was possibly expected
macro_rules! dexp {
    ($e:pat) => {
        sp!(_, E::ExpDotted_::Exp($e))
    };
}

fn exp(context: &mut Context, e: E::Exp) -> Box<N::Exp> {
    Box::new(exp_(context, e))
}

fn exp_(context: &mut Context, e: E::Exp) -> N::Exp {
    use E::Exp_ as EE;
    use N::Exp_ as NE;
    let sp!(eloc, e_) = e;
    let ne_ = match e_ {
        EE::Unit => NE::Unit,
        EE::Value(val) => NE::Value(val),
        EE::Move(v) => NE::Move(v),
        EE::Copy(v) => NE::Copy(v),
        EE::MName(n) => {
            context.error(vec![(
                n.loc,
                format!("Unexpected module or type identifier: '{}'", n),
            )]);
            NE::UnresolvedError
        }
        EE::Name(v) => NE::Use(Var(v)),

        EE::IfElse(eb, et, ef) => {
            NE::IfElse(exp(context, *eb), exp(context, *et), exp(context, *ef))
        }
        EE::While(eb, el) => NE::While(exp(context, *eb), exp(context, *el)),
        EE::Loop(el) => NE::Loop(exp(context, *el)),
        EE::Block(seq) => NE::Block(sequence(context, seq)),

        EE::Assign(a, e) => {
            let na_opt = assign_list(context, a);
            let ne = exp(context, *e);
            match na_opt {
                None => {
                    assert!(context.has_errors());
                    NE::UnresolvedError
                }
                Some(na) => NE::Assign(na, ne),
            }
        }
        EE::FieldMutate(edotted, er) => {
            let ndot_opt = dotted(context, *edotted);
            let ner = exp(context, *er);
            match ndot_opt {
                None => {
                    assert!(context.has_errors());
                    NE::UnresolvedError
                }
                Some(ndot) => NE::FieldMutate(ndot, ner),
            }
        }
        EE::Mutate(el, er) => {
            let nel = exp(context, *el);
            let ner = exp(context, *er);
            NE::Mutate(nel, ner)
        }

        EE::Return(es) => NE::Return(exp(context, *es)),
        EE::Abort(es) => NE::Abort(exp(context, *es)),
        EE::Break => NE::Break,
        EE::Continue => NE::Continue,

        EE::Dereference(e) => NE::Dereference(exp(context, *e)),
        EE::UnaryExp(uop, e) => NE::UnaryExp(uop, exp(context, *e)),
        EE::BinopExp(e1, bop, e2) => NE::BinopExp(exp(context, *e1), bop, exp(context, *e2)),

        EE::Pack(tn, tys_opt, efields) => match context.resolve_struct_name(tn) {
            None => {
                assert!(context.has_errors());
                NE::UnresolvedError
            }
            Some((m, sn)) => NE::Pack(
                m,
                sn,
                tys_opt.map(|tys| base_types(context, tys)),
                efields.map(|_, (idx, e)| (idx, exp_(context, e))),
            ),
        },
        EE::ExpList(es) => NE::ExpList(exp_vec(context, es)),

        EE::Borrow(mut_, inner) => match *inner {
            sp!(_, EE::ExpDotted(edot)) => match dotted(context, *edot) {
                None => {
                    assert!(context.has_errors());
                    NE::UnresolvedError
                }
                Some(d) => NE::Borrow(mut_, d),
            },
            e => {
                let ne = exp(context, e);
                NE::Borrow(mut_, sp(ne.loc, N::ExpDotted_::Exp(ne)))
            }
        },

        EE::ExpDotted(edot) => match dotted(context, *edot) {
            None => {
                assert!(context.has_errors());
                NE::UnresolvedError
            }
            Some(d) => NE::DerefBorrow(d),
        },

        EE::Annotate(e, t) => NE::Annotate(exp(context, *e), type_(context, t)),

        EE::GlobalCall(lhs, tys_opt, es) => {
            let ty_args = tys_opt.map(|tys| base_types(context, tys));
            let nes = exp(context, *es);
            match *lhs {
                sp!(_, EE::Name(n)) => match resolve_builtin_function(context, eloc, &n, ty_args) {
                    None => {
                        assert!(context.has_errors());
                        NE::UnresolvedError
                    }
                    Some(b) => NE::Builtin(sp(n.loc, b), nes),
                },
                sp!(loc, _) => {
                    context.error(vec![
                        (eloc, "Invalid global function call"),
                        (loc, "Expected: a name for a builtin function"),
                    ]);
                    NE::UnresolvedError
                }
            }
        }
        EE::Call(lhs, tys_opt, es) => {
            let ty_args = tys_opt.map(|tys| base_types(context, tys));
            let nes = exp(context, *es);
            match *lhs {
                sp!(_, EE::Name(n)) => match context.maybe_resolve_function_name(&n) {
                    None => match resolve_builtin_function(context, eloc, &n, ty_args) {
                        None => {
                            assert!(context.has_errors());
                            NE::UnresolvedError
                        }
                        Some(b) => NE::Builtin(sp(n.loc, b), nes),
                    },
                    Some((m, f)) => NE::ModuleCall(m, f, ty_args, nes),
                },
                sp!(_, EE::ExpDotted(ed)) => match *ed {
                    dexp!(_) => panic!("ICE stand alone expdotted"),
                    dot!(inner, f) => match *inner {
                        dexp!(sp!(_, EE::ModuleIdent(m))) => {
                            if !context.structs.contains_key(&m) {
                                context.error(vec![
                                    (eloc, "Invalid function call".into()),
                                    (m.loc(), format!("Unbound module '{}'", m,)),
                                ]);
                                NE::UnresolvedError
                            } else {
                                NE::ModuleCall(m, FunctionName(f), ty_args, nes)
                            }
                        }
                        edot => match dotted(context, edot) {
                            None => {
                                assert!(context.has_errors());
                                NE::UnresolvedError
                            }
                            Some(ndot) => NE::MethodCall(ndot, FunctionName(f), ty_args, nes),
                        },
                    },
                },
                sp!(loc, _) => {
                    context.error(vec![
                        (eloc, "Invalid function call"),
                        (loc, "Expected: a name, a dotted Module access, or dotted access of an expression")
                    ]);
                    NE::UnresolvedError
                }
            }
        }
        EE::ModuleIdent(n) => {
            context.error(vec![
                    (eloc, format!("Unexpected module identifier: '{}'", n)),
                    (n.loc(), "Modules can only be used in expressions as the left hand side of a '.' for a function call".to_string())
                ]);
            NE::UnresolvedError
        }

        EE::UnresolvedError => {
            assert!(context.has_errors());
            NE::UnresolvedError
        }
    };
    sp(eloc, ne_)
}

fn dotted(context: &mut Context, edot: E::ExpDotted) -> Option<N::ExpDotted> {
    let sp!(loc, edot_) = edot;
    let nedot_ = match edot_ {
        E::ExpDotted_::Exp(e) => {
            let ne = exp(context, e);
            match &ne.value {
                N::Exp_::UnresolvedError => return None,
                _ => N::ExpDotted_::Exp(ne),
            }
        }
        E::ExpDotted_::Dot(d, f) => N::ExpDotted_::Dot(Box::new(dotted(context, *d)?), Field(f)),
    };
    Some(sp(loc, nedot_))
}

fn bind(context: &mut Context, sp!(loc, b_): E::Bind) -> Option<N::Bind> {
    use E::Bind_ as EB;
    use N::Bind_ as NB;
    let nb_ = match b_ {
        EB::Var(v) => {
            if v.starts_with_underscore() {
                NB::Ignore
            } else {
                NB::Var(v)
            }
        }
        EB::Unpack(tn, tys_opt, efields) => {
            let (m, sn) = context.resolve_struct_name(tn)?;
            let nfields = UniqueMap::maybe_from_opt_iter(
                efields
                    .into_iter()
                    .map(|(k, (idx, inner))| Some((k, (idx, bind(context, inner)?)))),
            )?;
            NB::Unpack(
                m,
                sn,
                tys_opt.map(|tys| base_types(context, tys)),
                nfields.expect("ICE fields were already unique"),
            )
        }
    };
    Some(sp(loc, nb_))
}

fn bind_list(context: &mut Context, sp!(loc, b_): E::BindList) -> Option<N::BindList> {
    Some(sp(
        loc,
        b_.into_iter()
            .map(|inner| bind(context, inner))
            .collect::<Option<_>>()?,
    ))
}

fn assign(context: &mut Context, sp!(loc, a_): E::Assign) -> Option<N::Assign> {
    use E::Assign_ as EA;
    use N::Assign_ as NA;
    let na_ = match a_ {
        EA::Var(v) => {
            if v.starts_with_underscore() {
                NA::Ignore
            } else {
                NA::Var(v)
            }
        }
        EA::Unpack(tn, tys_opt, efields) => {
            let (m, sn) = context.resolve_struct_name(tn)?;
            let nfields = UniqueMap::maybe_from_opt_iter(
                efields
                    .into_iter()
                    .map(|(k, (idx, inner))| Some((k, (idx, assign(context, inner)?)))),
            )?;
            NA::Unpack(
                m,
                sn,
                tys_opt.map(|tys| base_types(context, tys)),
                nfields.expect("ICE fields were already unique"),
            )
        }
    };
    Some(sp(loc, na_))
}

fn assign_list(context: &mut Context, sp!(loc, a_): E::AssignList) -> Option<N::AssignList> {
    Some(sp(
        loc,
        a_.into_iter()
            .map(|inner| assign(context, inner))
            .collect::<Option<_>>()?,
    ))
}

fn resolve_builtin_function(
    context: &mut Context,
    loc: Loc,
    b: &Name,
    ty_args: Option<Vec<N::BaseType>>,
) -> Option<N::BuiltinFunction_> {
    use N::{BuiltinFunction_ as B, BuiltinFunction_::*};
    Some(match b.value.as_str() {
        B::MOVE_TO_SENDER => MoveToSender(check_builtin_ty_arg(context, loc, b, ty_args)),
        B::MOVE_FROM => MoveFrom(check_builtin_ty_arg(context, loc, b, ty_args)),
        B::BORROW_GLOBAL => BorrowGlobal(false, check_builtin_ty_arg(context, loc, b, ty_args)),
        B::BORROW_GLOBAL_MUT => BorrowGlobal(true, check_builtin_ty_arg(context, loc, b, ty_args)),
        B::EXISTS => Exists(check_builtin_ty_arg(context, loc, b, ty_args)),
        B::FREEZE => Freeze(check_builtin_ty_arg(context, loc, b, ty_args)),
        _ => {
            context.error(vec![(b.loc, format!("Unbound function: '{}'", b))]);
            return None;
        }
    })
}

fn check_builtin_ty_arg(
    context: &mut Context,
    loc: Loc,
    b: &Name,
    ty_args: Option<Vec<N::BaseType>>,
) -> Option<N::BaseType> {
    let res = check_builtin_ty_args(context, loc, b, 1, ty_args);
    res.map(|mut v| {
        assert!(v.len() == 1);
        v.pop().unwrap()
    })
}

fn check_builtin_ty_args(
    context: &mut Context,
    loc: Loc,
    b: &Name,
    arity: usize,
    ty_args: Option<Vec<N::BaseType>>,
) -> Option<Vec<N::BaseType>> {
    ty_args.map(|mut args| {
        let len = args.len();
        if len != arity {
            context.error(vec![
                (b.loc, format!("Invalid call to builtin function: '{}'", b)),
                (
                    loc,
                    format!("Expected {} type arguments but got {}", arity, len),
                ),
            ]);
        }

        while args.len() > arity {
            args.pop();
        }

        while args.len() < arity {
            args.push(N::BaseType_::anything(loc));
        }

        args
    })
}

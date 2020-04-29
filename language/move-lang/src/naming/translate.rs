// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    errors::*,
    expansion::ast as E,
    naming::ast as N,
    parser::ast::{Field, FunctionName, Kind, Kind_, ModuleIdent, StructName, Var},
    shared::{unique_map::UniqueMap, *},
    typing::core::{self, Subst},
};
use move_ir_types::location::*;
use std::collections::{BTreeMap, BTreeSet};

//**************************************************************************************************
// Context
//**************************************************************************************************

#[derive(Debug, Clone)]
enum ResolvedType {
    Struct(Loc, ModuleIdent, Option<Kind>),
    TParam(Loc, N::TParam),
    BuiltinType,
}

#[derive(Debug, Clone)]
enum ResolvedFunction {
    Function(Loc, ModuleIdent),
    BuiltinFunction,
}

impl ResolvedType {
    fn error_msg(&self, n: &Name) -> (Loc, String) {
        match self {
            ResolvedType::Struct(loc, m, _) => (
                *loc,
                format!("But '{}::{}' was declared as a struct here", m, n),
            ),
            ResolvedType::TParam(loc, _) => (
                *loc,
                format!("But '{}' was declared as a type parameter here", n),
            ),
            ResolvedType::BuiltinType => (n.loc, format!("But '{}' is a builtin type", n)),
        }
    }
}

impl ResolvedFunction {
    #[allow(dead_code)]
    fn error_msg(&self, n: &Name) -> (Loc, String) {
        match self {
            ResolvedFunction::Function(loc, m) => (
                *loc,
                format!("But '{}::{}' was declared as a function here", m, n),
            ),
            ResolvedFunction::BuiltinFunction => {
                (n.loc, format!("But '{}' is a builtin function", n))
            }
        }
    }
}

struct Context {
    errors: Errors,
    modules: BTreeSet<ModuleIdent>,
    current_module: Option<ModuleIdent>,
    scoped_types: BTreeMap<ModuleIdent, BTreeMap<String, (Loc, ModuleIdent, Option<Kind>)>>,
    unscoped_types: BTreeMap<String, ResolvedType>,
    scoped_functions: BTreeMap<ModuleIdent, BTreeMap<String, Loc>>,
    unscoped_functions: BTreeMap<String, ResolvedFunction>,
}

impl Context {
    fn new(prog: &E::Program, errors: Errors) -> Self {
        use ResolvedFunction as RF;
        use ResolvedType as RT;
        let modules = prog.modules.iter().map(|(mident, _)| mident).collect();
        let scoped_types = prog
            .modules
            .iter()
            .map(|(mident, mdef)| {
                let mems = mdef
                    .structs
                    .iter()
                    .map(|(s, sdef)| {
                        let kopt = sdef.resource_opt.map(|l| sp(l, Kind_::Resource));
                        (s.value().to_string(), (s.loc(), mident.clone(), kopt))
                    })
                    .collect();
                (mident, mems)
            })
            .collect();
        let scoped_functions = prog
            .modules
            .iter()
            .map(|(mident, mdef)| {
                let mems = mdef
                    .functions
                    .iter()
                    .map(|(n, _)| (n.value().to_string(), n.loc()))
                    .collect();
                (mident, mems)
            })
            .collect();
        let unscoped_types = N::BuiltinTypeName_::all_names()
            .into_iter()
            .map(|s| (s.to_string(), RT::BuiltinType))
            .collect();
        let unscoped_functions = N::BuiltinFunction_::all_names()
            .into_iter()
            .map(|s| (s.to_string(), RF::BuiltinFunction))
            .collect();
        Self {
            errors,
            modules,
            current_module: None,
            scoped_types,
            scoped_functions,
            unscoped_types,
            unscoped_functions,
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

    fn resolve_module_type(
        &mut self,
        loc: Loc,
        m: &ModuleIdent,
        n: &Name,
    ) -> Option<(Loc, StructName, Option<Kind>)> {
        let types = match self.scoped_types.get(m) {
            None => {
                self.error(vec![(loc, format!("Unbound module '{}'", m,))]);
                return None;
            }
            Some(members) => members,
        };
        match types.get(&n.value).cloned() {
            None => {
                self.error(vec![(
                    loc,
                    format!(
                        "Invalid module access. Unbound struct '{}' in module '{}'",
                        n, m
                    ),
                )]);
                None
            }
            Some((decl_loc, _, rloc)) => Some((decl_loc, StructName(n.clone()), rloc)),
        }
    }

    fn resolve_module_function(
        &mut self,
        loc: Loc,
        m: &ModuleIdent,
        n: &Name,
    ) -> Option<FunctionName> {
        let types = match self.scoped_functions.get(m) {
            None => {
                self.error(vec![(loc, format!("Unbound module '{}'", m,))]);
                return None;
            }
            Some(members) => members,
        };
        match types.get(&n.value).cloned() {
            None => {
                self.error(vec![(
                    loc,
                    format!(
                        "Invalid module access. Unbound function '{}' in module '{}'",
                        n, m
                    ),
                )]);
                None
            }
            Some(_) => Some(FunctionName(n.clone())),
        }
    }

    fn resolve_unscoped_type(&mut self, n: &Name) -> Option<ResolvedType> {
        match self.unscoped_types.get(&n.value) {
            None => {
                self.error(vec![(
                    n.loc,
                    format!("Unbound type '{}' in current scope", n),
                )]);
                None
            }
            Some(rn) => Some(rn.clone()),
        }
    }

    fn resolve_unscoped_function(&mut self, n: &Name) -> Option<ResolvedFunction> {
        match self.unscoped_functions.get(&n.value) {
            None => {
                self.error(vec![(
                    n.loc,
                    format!("Unbound function '{}' in current scope", n),
                )]);
                None
            }
            Some(rn) => Some(rn.clone()),
        }
    }

    fn resolve_struct_name(
        &mut self,
        verb: &str,
        sp!(loc, ma_): E::ModuleAccess,
    ) -> Option<(ModuleIdent, StructName)> {
        use ResolvedType as RT;
        use E::ModuleAccess_ as EA;
        match ma_ {
            EA::Name(n) => match self.resolve_unscoped_type(&n) {
                None => {
                    assert!(self.has_errors());
                    None
                }
                Some(RT::Struct(_, m, _)) => Some((m, StructName(n))),
                Some(rt) => {
                    self.error(vec![
                        (loc, format!("Invalid {}. Expected a struct name", verb)),
                        rt.error_msg(&n),
                    ]);
                    None
                }
            },
            EA::ModuleAccess(m, n) => match self.resolve_module_type(loc, &m, &n) {
                None => {
                    assert!(self.has_errors());
                    None
                }
                Some(_) => Some((m, StructName(n))),
            },
        }
    }

    fn bind_type(&mut self, s: String, rt: ResolvedType) {
        self.unscoped_types.insert(s, rt);
    }

    fn bind_function(&mut self, s: String, rf: ResolvedFunction) {
        self.unscoped_functions.insert(s, rf);
    }

    fn save_unscoped(
        &self,
    ) -> (
        BTreeMap<String, ResolvedType>,
        BTreeMap<String, ResolvedFunction>,
    ) {
        (self.unscoped_types.clone(), self.unscoped_functions.clone())
    }

    fn restore_unscoped(
        &mut self,
        (types, functions): (
            BTreeMap<String, ResolvedType>,
            BTreeMap<String, ResolvedFunction>,
        ),
    ) {
        self.unscoped_types = types;
        self.unscoped_functions = functions;
    }
}

//**************************************************************************************************
// Entry
//**************************************************************************************************

pub fn program(prog: E::Program, errors: Errors) -> (N::Program, Errors) {
    let mut context = Context::new(&prog, errors);
    let mut modules = modules(&mut context, prog.modules);
    let scripts = scripts(&mut context, prog.scripts);
    super::uses::verify(&mut context.errors, &mut modules);
    (N::Program { modules, scripts }, context.get_errors())
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
    context.current_module = Some(ident.clone());
    let outer_unscoped = context.save_unscoped();
    let is_source_module = mdef.is_source_module;
    let uses = mdef.uses;
    check_unused_aliases(context, mdef.unused_aliases);
    for (s, sdef) in &mdef.structs {
        let kopt = sdef.resource_opt.map(|l| sp(l, Kind_::Resource));
        let rt = ResolvedType::Struct(s.loc(), ident.clone(), kopt);
        context.bind_type(s.value().to_string(), rt)
    }
    for (f, _) in &mdef.functions {
        let rf = ResolvedFunction::Function(f.loc(), ident.clone());
        context.bind_function(f.value().to_string(), rf)
    }
    let unscoped = context.save_unscoped();
    let structs = mdef.structs.map(|name, s| {
        context.restore_unscoped(unscoped.clone());
        struct_def(context, name, s)
    });
    let functions = mdef.functions.map(|name, f| {
        context.restore_unscoped(unscoped.clone());
        function(context, name, f)
    });
    context.restore_unscoped(outer_unscoped);
    N::ModuleDefinition {
        uses,
        is_source_module,
        dependency_order: 0,
        structs,
        functions,
    }
}

fn check_unused_aliases(context: &mut Context, unused_aliases: Vec<ModuleIdent>) {
    for mident in unused_aliases {
        if !context.modules.contains(&mident) {
            context.error(vec![(
                mident.loc(),
                format!("Invalid 'use'. Unbound module: '{}'", mident),
            )]);
        }
    }
}

fn scripts(
    context: &mut Context,
    escripts: BTreeMap<String, E::Script>,
) -> BTreeMap<String, N::Script> {
    escripts
        .into_iter()
        .map(|(n, s)| (n, script(context, s)))
        .collect()
}

fn script(context: &mut Context, escript: E::Script) -> N::Script {
    let E::Script {
        loc,
        unused_aliases,
        uses: _uses,
        function_name,
        function: efunction,
        specs: _specs,
    } = escript;
    check_unused_aliases(context, unused_aliases);
    let function = function(context, function_name.clone(), efunction);
    N::Script {
        loc,
        function_name,
        function,
    }
}

//**************************************************************************************************
// Functions
//**************************************************************************************************

fn function(context: &mut Context, _name: FunctionName, f: E::Function) -> N::Function {
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
        .map(|(v, ty)| (v, type_(context, ty)))
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
    eacquires: Vec<E::ModuleAccess>,
) -> BTreeSet<StructName> {
    let mut acquires = BTreeMap::new();
    for eacquire in eacquires {
        let sn = match acquires_type(context, eacquire) {
            None => continue,
            Some(sn) => sn,
        };
        let new_loc = sn.loc();
        if let Some(old_loc) = acquires.insert(sn, new_loc) {
            context.error(vec![
                (new_loc, "Duplicate acquires item"),
                (old_loc, "Previously listed here"),
            ])
        }
    }
    acquires.into_iter().map(|(n, _)| n).collect()
}

fn acquires_type(context: &mut Context, sp!(loc, en_): E::ModuleAccess) -> Option<StructName> {
    use ResolvedType as RT;
    use E::ModuleAccess_ as EN;
    match en_ {
        EN::Name(n) => match context.resolve_unscoped_type(&n)? {
            RT::Struct(decl_loc, m, resource_opt) => {
                acquires_type_struct(context, loc, decl_loc, m, StructName(n), resource_opt)
            }
            RT::BuiltinType => {
                context.error(vec![(
                    loc,
                    "Invalid acquires item. Expected a resource name, but got a builtin type",
                )]);
                None
            }
            RT::TParam(_, _) => {
                context.error(vec![(
                    loc,
                    "Invalid acquires item. Expected a resource name, but got a type parameter",
                )]);
                None
            }
        },
        EN::ModuleAccess(m, n) => {
            let (decl_loc, _, resource_opt) = context.resolve_module_type(loc, &m, &n)?;
            acquires_type_struct(context, loc, decl_loc, m, StructName(n), resource_opt)
        }
    }
}

fn acquires_type_struct(
    context: &mut Context,
    loc: Loc,
    decl_loc: Loc,
    declared_module: ModuleIdent,
    n: StructName,
    resource_opt: Option<Kind>,
) -> Option<StructName> {
    let declared_in_current = match &context.current_module {
        Some(current_module) => current_module == &declared_module,
        None => false,
    };
    if !declared_in_current {
        let tmsg = format!(
            "The struct '{}' was not declared in the current module. Global \
             storage access is internal to the module'",
            n
        );
        context.error(vec![(loc, "Invalid acquires item".into()), (n.loc(), tmsg)]);
        return None;
    }

    if resource_opt.is_none() {
        context.error(vec![
            (loc, "Invalid acquires item. Expected a nominal resource."),
            (decl_loc, "Declared as a normal struct here"),
        ]);
        return None;
    }

    Some(n)
}

//**************************************************************************************************
// Structs
//**************************************************************************************************

fn struct_def(
    context: &mut Context,
    name: StructName,
    sdef: E::StructDefinition,
) -> N::StructDefinition {
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
            N::StructFields::Defined(em.map(|_f, (idx, t)| (idx, type_(context, t))))
        }
    }
}

fn check_no_nominal_resources(context: &mut Context, s: &StructName, field: &Field, ty: &N::Type) {
    use N::Type_ as T;
    let sp!(tloc, ty_) = ty;
    match ty_ {
        T::Apply(Some(sp!(kloc, Kind_::Resource)), _, _) => {
            let field_msg = format!(
                "Invalid resource field '{}' for struct '{}'. Structs cannot \
                 contain resource types, except through type parameters",
                field, s
            );
            let tmsg = format!(
                "Field '{}' is a resource due to the type: {}",
                field,
                core::error_format(ty, &Subst::empty()),
            );
            let kmsg = format!(
                "Type {} was declared as a resource here",
                core::error_format(ty, &Subst::empty()),
            );
            context.error(vec![
                (field.loc(), field_msg),
                (*tloc, tmsg),
                (*kloc, kmsg),
                (s.loc(), format!("'{}' declared as a `struct` here", s)),
            ])
        }
        T::Apply(None, _, tyl) => tyl
            .iter()
            .for_each(|t| check_no_nominal_resources(context, s, field, t)),
        _ => (),
    }
}

//**************************************************************************************************
// Types
//**************************************************************************************************

fn type_parameters(context: &mut Context, type_parameters: Vec<(Name, Kind)>) -> Vec<N::TParam> {
    let mut unique_tparams = UniqueMap::new();
    type_parameters
        .into_iter()
        .map(|(name, kind)| {
            let id = N::TParamID::next();
            let user_specified_name = name.clone();
            let tp = N::TParam {
                id,
                user_specified_name,
                kind,
            };
            let loc = name.loc;
            context.bind_type(
                name.value.to_string(),
                ResolvedType::TParam(loc, tp.clone()),
            );
            if let Err(old_loc) = unique_tparams.add(name.clone(), ()) {
                let msg = format!("Duplicate type parameter declared with name '{}'", name);
                context.error(vec![
                    (loc, msg),
                    (old_loc, "Previously defined here".to_string()),
                ])
            }
            tp
        })
        .collect()
}

fn types(context: &mut Context, tys: Vec<E::Type>) -> Vec<N::Type> {
    tys.into_iter().map(|t| type_(context, t)).collect()
}

fn type_(context: &mut Context, sp!(loc, ety_): E::Type) -> N::Type {
    use ResolvedType as RT;
    use E::{ModuleAccess_ as EN, Type_ as ET};
    use N::{TypeName_ as NN, Type_ as NT};
    let ty_ = match ety_ {
        ET::Unit => NT::Unit,
        ET::Multiple(tys) => {
            NT::multiple_(loc, tys.into_iter().map(|t| type_(context, t)).collect())
        }
        ET::Ref(mut_, inner) => NT::Ref(mut_, Box::new(type_(context, *inner))),
        ET::UnresolvedError => {
            assert!(context.has_errors());
            NT::UnresolvedError
        }
        ET::Apply(sp!(nloc, EN::Name(n)), tys) => match context.resolve_unscoped_type(&n) {
            None => {
                assert!(context.has_errors());
                NT::UnresolvedError
            }
            Some(RT::Struct(_, m, resource_opt)) => {
                let tn = sp(nloc, NN::ModuleType(m, StructName(n)));
                NT::Apply(resource_opt, tn, types(context, tys))
            }
            Some(RT::BuiltinType) => {
                let ty_args = types(context, tys);
                let bn_ = N::BuiltinTypeName_::resolve(&n.value).unwrap();
                NT::builtin_(sp(loc, bn_), ty_args)
            }
            Some(RT::TParam(_, tp)) => {
                if !tys.is_empty() {
                    context.error(vec![(
                        loc,
                        "Generic type parameters cannot take type arguments",
                    )]);
                    NT::UnresolvedError
                } else {
                    NT::Param(tp)
                }
            }
        },
        ET::Apply(sp!(loc, EN::ModuleAccess(m, n)), tys) => {
            match context.resolve_module_type(loc, &m, &n) {
                None => {
                    assert!(context.has_errors());
                    NT::UnresolvedError
                }
                Some((_, _, resource_opt)) => {
                    let tn = sp(loc, NN::ModuleType(m, StructName(n)));
                    NT::Apply(resource_opt, tn, types(context, tys))
                }
            }
        }
        ET::Fun(_, _) => panic!("ICE only allowed in spec context"),
    };
    sp(loc, ty_)
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

fn call_args(context: &mut Context, sp!(loc, es): Spanned<Vec<E::Exp>>) -> Spanned<Vec<N::Exp>> {
    sp(loc, exps(context, es))
}

fn exps(context: &mut Context, es: Vec<E::Exp>) -> Vec<N::Exp> {
    es.into_iter().map(|e| exp_(context, e)).collect()
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
        EE::InferredNum(u) => NE::InferredNum(u),
        EE::Value(val) => NE::Value(val),
        EE::Move(v) => NE::Move(v),
        EE::Copy(v) => NE::Copy(v),
        EE::Name(sp!(_, E::ModuleAccess_::Name(v)), None) => NE::Use(Var(v)),

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

        EE::Pack(tn, tys_opt, efields) => match context.resolve_struct_name("construction", tn) {
            None => {
                assert!(context.has_errors());
                NE::UnresolvedError
            }
            Some((m, sn)) => NE::Pack(
                m,
                sn,
                tys_opt.map(|tys| types(context, tys)),
                efields.map(|_, (idx, e)| (idx, exp_(context, e))),
            ),
        },
        EE::ExpList(es) => {
            assert!(es.len() > 1);
            NE::ExpList(exps(context, es))
        }

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

        EE::Cast(e, t) => NE::Cast(exp(context, *e), type_(context, t)),
        EE::Annotate(e, t) => NE::Annotate(exp(context, *e), type_(context, t)),

        EE::GlobalCall(n, tys_opt, rhs) => {
            let ty_args = tys_opt.map(|tys| types(context, tys));
            let nes = call_args(context, rhs);
            match resolve_builtin_function(context, eloc, &n, ty_args) {
                None => {
                    assert!(context.has_errors());
                    NE::UnresolvedError
                }
                Some(b) => NE::Builtin(sp(n.loc, b), nes),
            }
        }
        EE::Call(sp!(mloc, ma_), tys_opt, rhs) => {
            use ResolvedFunction as RF;
            use E::ModuleAccess_ as EA;
            let ty_args = tys_opt.map(|tys| types(context, tys));
            let nes = call_args(context, rhs);
            match ma_ {
                EA::Name(n) => match context.resolve_unscoped_function(&n) {
                    None => {
                        assert!(context.has_errors());
                        NE::UnresolvedError
                    }
                    Some(RF::BuiltinFunction) => {
                        match resolve_builtin_function(context, eloc, &n, ty_args) {
                            None => {
                                assert!(context.has_errors());
                                NE::UnresolvedError
                            }
                            Some(f) => NE::Builtin(sp(mloc, f), nes),
                        }
                    }
                    Some(RF::Function(_, m)) => NE::ModuleCall(m, FunctionName(n), ty_args, nes),
                },
                EA::ModuleAccess(m, n) => match context.resolve_module_function(mloc, &m, &n) {
                    None => {
                        assert!(context.has_errors());
                        NE::UnresolvedError
                    }
                    Some(_) => NE::ModuleCall(m, FunctionName(n), ty_args, nes),
                },
            }
        }
        EE::Spec(u, unbound_names) => {
            // Vars currently aren't shadowable by types/functions
            let used_locals = unbound_names.into_iter().map(Var).collect();
            NE::Spec(u, used_locals)
        }
        EE::UnresolvedError => {
            assert!(context.has_errors());
            NE::UnresolvedError
        }
        EE::Index(..) | EE::Lambda(..) |
        // matches name variants only allowed in specs (we handle the allowed ones above)
        EE::Name(..) => {
            panic!("ICE unexpected specification construct")
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

#[derive(Clone, Copy)]
enum LValueCase {
    Bind,
    Assign,
}

fn lvalue(context: &mut Context, case: LValueCase, sp!(loc, l_): E::LValue) -> Option<N::LValue> {
    use LValueCase as C;
    use E::LValue_ as EL;
    use N::LValue_ as NL;
    let nl_ = match l_ {
        EL::Var(sp!(_, E::ModuleAccess_::Name(n)), None) => {
            let v = Var(n);
            if v.starts_with_underscore() {
                NL::Ignore
            } else {
                NL::Var(v)
            }
        }
        EL::Unpack(tn, tys_opt, efields) => {
            let msg = match case {
                C::Bind => "deconstructing binding",
                C::Assign => "deconstructing assignment",
            };
            let (m, sn) = context.resolve_struct_name(msg, tn)?;
            let nfields = UniqueMap::maybe_from_opt_iter(
                efields
                    .into_iter()
                    .map(|(k, (idx, inner))| Some((k, (idx, lvalue(context, case, inner)?)))),
            )?;
            NL::Unpack(
                m,
                sn,
                tys_opt.map(|tys| types(context, tys)),
                nfields.expect("ICE fields were already unique"),
            )
        }
        EL::Var(..) => panic!("unexpected specification construct"),
    };
    Some(sp(loc, nl_))
}

fn bind_list(context: &mut Context, ls: E::LValueList) -> Option<N::LValueList> {
    lvalue_list(context, LValueCase::Bind, ls)
}

fn assign_list(context: &mut Context, ls: E::LValueList) -> Option<N::LValueList> {
    lvalue_list(context, LValueCase::Assign, ls)
}

fn lvalue_list(
    context: &mut Context,
    case: LValueCase,
    sp!(loc, b_): E::LValueList,
) -> Option<N::LValueList> {
    Some(sp(
        loc,
        b_.into_iter()
            .map(|inner| lvalue(context, case, inner))
            .collect::<Option<_>>()?,
    ))
}

fn resolve_builtin_function(
    context: &mut Context,
    loc: Loc,
    b: &Name,
    ty_args: Option<Vec<N::Type>>,
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
    ty_args: Option<Vec<N::Type>>,
) -> Option<N::Type> {
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
    ty_args: Option<Vec<N::Type>>,
) -> Option<Vec<N::Type>> {
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
            args.push(sp(loc, N::Type_::UnresolvedError));
        }

        args
    })
}

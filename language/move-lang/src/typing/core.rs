// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    errors::*,
    naming::ast::{
        self as N, BuiltinTypeName_, FunctionSignature, StructDefinition, TParam, TParamID, TVar,
        Type, TypeName, TypeName_, Type_,
    },
    parser::ast::{
        Field, FunctionName, FunctionVisibility, Kind, Kind_, ModuleIdent, ResourceLoc, StructName,
        Var,
    },
    shared::{unique_map::UniqueMap, *},
};
use move_ir_types::location::*;
use std::collections::{BTreeSet, HashMap};

//**************************************************************************************************
// Context
//**************************************************************************************************

pub enum Constraint {
    IsCopyable(Loc, String, Type),
    IsImplicitlyCopyable {
        loc: Loc,
        msg: String,
        ty: Type,
        fix: String,
    },
    KindConstraint(Loc, Type, Kind),
    NumericConstraint(Loc, &'static str, Type),
    BitsConstraint(Loc, &'static str, Type),
    OrderedConstraint(Loc, &'static str, Type),
    BaseTypeConstraint(Loc, String, Type),
    SingleTypeConstraint(Loc, String, Type),
}
pub type Constraints = Vec<Constraint>;
type TParamSubst = HashMap<TParamID, Type>;

pub struct FunctionInfo {
    pub defined_loc: Loc,
    pub visibility: FunctionVisibility,
    pub signature: FunctionSignature,
    pub acquires: BTreeSet<StructName>,
}

pub struct ModuleInfo {
    pub structs: UniqueMap<StructName, StructDefinition>,
    pub functions: UniqueMap<FunctionName, FunctionInfo>,
}

pub struct Context {
    pub modules: UniqueMap<ModuleIdent, ModuleInfo>,

    pub current_module: Option<ModuleIdent>,
    pub current_function: Option<FunctionName>,
    pub return_type: Option<Type>,
    locals: UniqueMap<Var, Type>,

    pub subst: Subst,
    pub constraints: Constraints,

    pub in_loop: bool,
    pub break_type: Option<Type>,

    errors: Errors,
}

impl Context {
    pub fn new(prog: &N::Program, errors: Errors) -> Self {
        let modules = prog.modules.ref_map(|_ident, mdef| {
            let structs = mdef.structs.clone();
            let functions = mdef.functions.ref_map(|fname, fdef| FunctionInfo {
                defined_loc: fname.loc(),
                visibility: fdef.visibility.clone(),
                signature: fdef.signature.clone(),
                acquires: fdef.acquires.clone(),
            });
            ModuleInfo { structs, functions }
        });
        Context {
            subst: Subst::empty(),
            current_module: None,
            current_function: None,
            return_type: None,
            constraints: vec![],
            errors,
            locals: UniqueMap::new(),
            in_loop: false,
            break_type: None,
            modules,
        }
    }

    pub fn reset_for_module_item(&mut self) {
        assert!(!self.in_loop, "ICE in_loop should be reset after the loop");
        self.return_type = None;
        self.locals = UniqueMap::new();
        self.subst = Subst::empty();
        self.constraints = Constraints::new();
    }

    pub fn error(&mut self, e: Vec<(Loc, impl Into<String>)>) {
        self.errors
            .push(e.into_iter().map(|(loc, msg)| (loc, msg.into())).collect())
    }

    pub fn get_errors(self) -> Errors {
        self.errors
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn error_type(&mut self, loc: Loc) -> Type {
        sp(loc, Type_::UnresolvedError)
    }

    pub fn add_implicit_copyable_constraint(
        &mut self,
        loc: Loc,
        msg: impl Into<String>,
        ty: Type,
        fix: impl Into<String>,
    ) {
        let msg = msg.into();
        let fix = fix.into();
        self.constraints
            .push(Constraint::IsImplicitlyCopyable { loc, msg, ty, fix })
    }

    pub fn add_copyable_constraint(&mut self, loc: Loc, msg: impl Into<String>, s: Type) {
        self.constraints
            .push(Constraint::IsCopyable(loc, msg.into(), s))
    }

    pub fn add_kind_constraint(&mut self, loc: Loc, t: Type, k: Kind) {
        if let sp!(_, Kind_::Unknown) = &k {
            return;
        }
        self.constraints.push(Constraint::KindConstraint(loc, t, k))
    }

    pub fn add_base_type_constraint(&mut self, loc: Loc, msg: impl Into<String>, t: Type) {
        self.constraints
            .push(Constraint::BaseTypeConstraint(loc, msg.into(), t))
    }

    pub fn add_single_type_constraint(&mut self, loc: Loc, msg: impl Into<String>, t: Type) {
        self.constraints
            .push(Constraint::SingleTypeConstraint(loc, msg.into(), t))
    }

    pub fn add_numeric_constraint(&mut self, loc: Loc, op: &'static str, t: Type) {
        self.constraints
            .push(Constraint::NumericConstraint(loc, op, t))
    }

    pub fn add_bits_constraint(&mut self, loc: Loc, op: &'static str, t: Type) {
        self.constraints
            .push(Constraint::BitsConstraint(loc, op, t))
    }

    pub fn add_ordered_constraint(&mut self, loc: Loc, op: &'static str, t: Type) {
        self.constraints
            .push(Constraint::OrderedConstraint(loc, op, t))
    }

    pub fn declare_local(&mut self, var: Var, ty_opt: Option<Type>) {
        let status = match ty_opt {
            None => make_tvar(self, var.loc()),
            Some(t) => t,
        };
        // Might overwrite (i.e. shadow) the current local's type
        self.locals.remove(&var);
        self.locals.add(var, status).unwrap();
    }

    pub fn get_local_(&mut self, var: &Var) -> Option<Type> {
        self.locals.get(var).cloned()
    }

    pub fn get_local(&mut self, loc: Loc, verb: &str, var: &Var) -> Type {
        match self.get_local_(var) {
            None => {
                self.error(vec![(
                    loc,
                    format!("Invalid {}. Unbound local '{}'", verb, var),
                )]);
                self.error_type(loc)
            }
            Some(t) => t,
        }
    }

    pub fn save_locals_scope(&self) -> UniqueMap<Var, Type> {
        self.locals.clone()
    }

    pub fn close_locals_scope(
        &mut self,
        old_locals: UniqueMap<Var, Type>,
        declared: UniqueMap<Var, ()>,
    ) {
        // remove new locals from inner scope
        for (new_local, _) in declared.iter().filter(|(v, _)| !old_locals.contains_key(v)) {
            self.locals.remove(&new_local);
        }

        // return old type
        let shadowed = old_locals
            .into_iter()
            .filter(|(k, _)| declared.contains_key(k));
        for (var, shadowed_type) in shadowed {
            self.locals.remove(&var);
            self.locals.add(var, shadowed_type).unwrap();
        }
    }

    pub fn is_current_module(&self, m: &ModuleIdent) -> bool {
        match &self.current_module {
            Some(curm) => curm == m,
            None => false,
        }
    }

    pub fn is_current_function(&self, m: &ModuleIdent, f: &FunctionName) -> bool {
        self.is_current_module(m) && matches!(&self.current_function, Some(curf) if curf == f)
    }

    fn module_info(&self, m: &ModuleIdent) -> &ModuleInfo {
        self.modules
            .get(m)
            .expect("ICE should have failed in naming")
    }

    fn struct_definition(&self, m: &ModuleIdent, n: &StructName) -> &StructDefinition {
        let minfo = self.module_info(m);
        minfo
            .structs
            .get(n)
            .expect("ICE should have failed in naming")
    }

    pub fn resource_opt(&self, m: &ModuleIdent, n: &StructName) -> ResourceLoc {
        self.struct_definition(m, n).resource_opt
    }

    pub fn struct_declared_loc(&self, m: &ModuleIdent, n: &StructName) -> Loc {
        let minfo = self.module_info(m);
        *minfo
            .structs
            .get_loc(n)
            .expect("ICE should have failed in naming")
    }

    fn struct_tparams(&self, m: &ModuleIdent, n: &StructName) -> &Vec<TParam> {
        &self.struct_definition(m, n).type_parameters
    }

    fn function_info(&mut self, m: &ModuleIdent, n: &FunctionName) -> &FunctionInfo {
        self.module_info(m)
            .functions
            .get(n)
            .expect("ICE should have failed in naming")
    }
}

//**************************************************************************************************
// Subst
//**************************************************************************************************

#[derive(Clone, Debug)]
pub struct Subst {
    tvars: HashMap<TVar, Type>,
    num_vars: HashMap<TVar, Loc>,
}

impl Subst {
    pub fn empty() -> Self {
        Self {
            tvars: HashMap::new(),
            num_vars: HashMap::new(),
        }
    }

    pub fn insert(&mut self, tvar: TVar, bt: Type) {
        self.tvars.insert(tvar, bt);
    }

    pub fn get(&self, tvar: TVar) -> Option<&Type> {
        self.tvars.get(&tvar)
    }

    pub fn new_num_var(&mut self, loc: Loc) -> TVar {
        let tvar = TVar::next();
        assert!(self.num_vars.insert(tvar, loc).is_none());
        tvar
    }

    pub fn set_num_var(&mut self, tvar: TVar, loc: Loc) {
        self.num_vars.entry(tvar).or_insert(loc);
        if let Some(sp!(_, Type_::Var(next))) = self.get(tvar) {
            let next = *next;
            self.set_num_var(next, loc)
        }
    }

    pub fn is_num_var(&self, tvar: TVar) -> bool {
        self.num_vars.contains_key(&tvar)
    }
}

impl ast_debug::AstDebug for Subst {
    fn ast_debug(&self, w: &mut ast_debug::AstWriter) {
        let Subst { tvars, num_vars } = self;

        w.write("tvars:");
        w.indent(4, |w| {
            let mut tvars = tvars.iter().collect::<Vec<_>>();
            tvars.sort_by_key(|(v, _)| *v);
            for (tvar, bt) in tvars {
                w.write(&format!("{:?} => ", tvar));
                bt.ast_debug(w);
                w.new_line();
            }
        });
        w.write("num_vars:");
        w.indent(4, |w| {
            let mut num_vars = num_vars.keys().collect::<Vec<_>>();
            num_vars.sort();
            for tvar in num_vars {
                w.writeln(&format!("{:?}", tvar))
            }
        })
    }
}

//**************************************************************************************************
// Type error display
//**************************************************************************************************

pub fn error_format(b: &Type, subst: &Subst) -> String {
    error_format_(b, subst, false)
}

fn error_format_(sp!(_, b_): &Type, subst: &Subst, nested: bool) -> String {
    use Type_::*;
    let res = match b_ {
        UnresolvedError | Anything => "_".to_string(),
        Unit => "()".to_string(),
        Var(id) => match subst.get(*id) {
            Some(t) => error_format_(t, subst, true),
            None if nested && subst.is_num_var(*id) => "{integer}".to_string(),
            None if subst.is_num_var(*id) => return "integer".to_string(),
            None => "_".to_string(),
        },
        Apply(_, sp!(_, TypeName_::Multiple(_)), tys) => {
            let inner = format_comma(tys.iter().map(|s| error_format_(s, subst, true)));
            format!("({})", inner)
        }
        Apply(_, n, tys) => {
            let tys_str = if !tys.is_empty() {
                format!(
                    "<{}>",
                    format_comma(tys.iter().map(|t| error_format_(t, subst, true)))
                )
            } else {
                "".to_string()
            };
            format!("{}{}", n, tys_str)
        }
        Param(tp) => tp.debug.value.to_string(),
        Ref(mut_, ty) => format!(
            "&{}{}",
            if *mut_ { "mut " } else { "" },
            error_format_(ty, subst, true)
        ),
    };
    if nested {
        res
    } else {
        format!("'{}'", res)
    }
}

//**************************************************************************************************
// Type utils
//**************************************************************************************************

pub fn infer_kind(context: &Context, subst: &Subst, ty: Type) -> Option<Kind> {
    use Kind_ as K;
    use Type_ as T;
    let loc = ty.loc;
    match unfold_type(subst, ty).value {
        T::Unit | T::Ref(_, _) => Some(sp(loc, Kind_::Copyable)),
        T::Var(_) => panic!("ICE unfold_type failed, which is impossible"),
        T::UnresolvedError | T::Anything => None,
        T::Param(TParam { kind, .. }) | T::Apply(Some(kind), _, _) => Some(kind),
        // if any unknown, give unkown
        // else if any resource, give resource
        // else affine
        T::Apply(None, n, tyl) => {
            // If an anything is found, we get a none. Then use the constraint for the
            // default kind
            let contraints = match &n.value {
                TypeName_::Multiple(_) | TypeName_::Builtin(_) => {
                    tyl.iter().map(|_| None).collect::<Vec<_>>()
                }
                TypeName_::ModuleType(m, n) => {
                    let sdef = context.struct_definition(m, n);
                    sdef.type_parameters
                        .iter()
                        .map(|tp| Some(tp.kind.clone()))
                        .collect::<Vec<_>>()
                }
            };
            let max = tyl
                .into_iter()
                .zip(contraints)
                .filter_map(|(t, constraint_opt)| infer_kind(context, subst, t).or(constraint_opt))
                .map(|k| match k {
                    sp!(loc, K::Copyable) => sp(loc, K::Affine),
                    k => k,
                })
                .max_by(most_general_kind);
            Some(match max {
                Some(sp!(_, K::Copyable)) => unreachable!(),
                None | Some(sp!(_, K::Affine)) => {
                    sp(type_name_declared_loc(context, &n), K::Affine)
                }
                Some(k @ sp!(_, K::Resource)) | Some(k @ sp!(_, K::Unknown)) => k,
            })
        }
    }
}

fn most_general_kind(k1: &Kind, k2: &Kind) -> std::cmp::Ordering {
    use std::cmp::Ordering as O;
    use Kind_ as K;
    match (&k1.value, &k2.value) {
        (K::Copyable, _) | (_, K::Copyable) => panic!("ICE structs cannot be copyable"),

        (K::Unknown, K::Unknown) => O::Equal,
        (K::Unknown, _) => O::Greater,
        (_, K::Unknown) => O::Less,

        (K::Resource, K::Resource) => O::Equal,
        (K::Resource, _) => O::Greater,
        (_, K::Resource) => O::Less,

        (K::Affine, K::Affine) => O::Equal,
    }
}

fn type_name_declared_loc(context: &Context, sp!(loc, n_): &TypeName) -> Loc {
    match n_ {
        TypeName_::Multiple(_) | TypeName_::Builtin(_) => *loc,
        TypeName_::ModuleType(m, n) => context.struct_declared_loc(m, n),
    }
}

pub fn make_num_tvar(context: &mut Context, loc: Loc) -> Type {
    let tvar = context.subst.new_num_var(loc);
    sp(loc, Type_::Var(tvar))
}

pub fn make_tvar(_context: &mut Context, loc: Loc) -> Type {
    sp(loc, Type_::Var(TVar::next()))
}

//**************************************************************************************************
// Structs
//**************************************************************************************************

pub fn make_struct_type(
    context: &mut Context,
    loc: Loc,
    m: &ModuleIdent,
    n: &StructName,
    ty_args_opt: Option<Vec<Type>>,
) -> (Type, Vec<Type>) {
    let tn = sp(loc, TypeName_::ModuleType(m.clone(), n.clone()));
    let sdef = context.struct_definition(m, n);
    let resource_opt = sdef.resource_opt;
    let kind_opt = resource_opt.map(|rloc| sp(rloc, Kind_::Resource));
    match ty_args_opt {
        None => {
            let constraints = sdef
                .type_parameters
                .iter()
                .map(|tp| (loc, tp.kind.clone()))
                .collect();
            let ty_args = make_tparams(context, loc, TVarCase::Base, constraints);
            (
                sp(loc, Type_::Apply(kind_opt, tn, ty_args.clone())),
                ty_args,
            )
        }
        Some(ty_args) => {
            let tapply_ = instantiate_apply(context, loc, kind_opt, tn, ty_args);
            let targs = match &tapply_ {
                Type_::Apply(_, _, targs) => targs.clone(),
                _ => panic!("ICE instantiate_apply returned non Apply"),
            };
            (sp(loc, tapply_), targs)
        }
    }
}

pub fn make_expr_list_tvars(
    context: &mut Context,
    loc: Loc,
    constraint_msg: impl Into<String>,
    locs: Vec<Loc>,
) -> Vec<Type> {
    let constraints = locs.iter().map(|l| (*l, sp(*l, Kind_::Unknown))).collect();
    let tys = make_tparams(
        context,
        loc,
        TVarCase::Single(constraint_msg.into()),
        constraints,
    );
    tys.into_iter()
        .zip(locs)
        .map(|(tvar, l)| sp(l, tvar.value))
        .collect()
}

// ty_args should come from make_struct_type
pub fn make_field_types(
    context: &mut Context,
    _loc: Loc,
    m: &ModuleIdent,
    n: &StructName,
    ty_args: Vec<Type>,
) -> N::StructFields {
    let sdef = context.struct_definition(m, n);
    let tparam_subst =
        &make_tparam_subst(&context.struct_definition(m, n).type_parameters, ty_args);
    match &sdef.fields {
        N::StructFields::Native(loc) => N::StructFields::Native(*loc),
        N::StructFields::Defined(m) => {
            N::StructFields::Defined(m.ref_map(|_, (idx, field_ty)| {
                (*idx, subst_tparams(tparam_subst, field_ty.clone()))
            }))
        }
    }
}

// ty_args should come from make_struct_type
pub fn make_field_type(
    context: &mut Context,
    loc: Loc,
    m: &ModuleIdent,
    n: &StructName,
    ty_args: Vec<Type>,
    field: &Field,
) -> Type {
    let sdef = context.struct_definition(m, n);
    let fields_map = match &sdef.fields {
        N::StructFields::Native(nloc) => {
            let nloc = *nloc;
            context.error(vec![
                (
                    loc,
                    format!("Unbound field '{}' for native struct '{}::{}'", field, m, n),
                ),
                (nloc, "Declared 'native' here".into()),
            ]);
            return context.error_type(loc);
        }
        N::StructFields::Defined(m) => m,
    };
    match fields_map.get(field).cloned() {
        None => {
            context.error(vec![(
                loc,
                format!("Unbound field '{}' in '{}::{}'", field, m, n),
            )]);
            context.error_type(loc)
        }
        Some((_, field_ty)) => {
            let tparam_subst =
                &make_tparam_subst(&context.struct_definition(m, n).type_parameters, ty_args);
            subst_tparams(tparam_subst, field_ty)
        }
    }
}

//**************************************************************************************************
// Functions
//**************************************************************************************************

pub fn make_function_type(
    context: &mut Context,
    loc: Loc,
    m: &ModuleIdent,
    f: &FunctionName,
    ty_args_opt: Option<Vec<Type>>,
) -> (Loc, Vec<Type>, Vec<(Var, Type)>, BTreeSet<StructName>, Type) {
    let in_current_module = match &context.current_module {
        Some(current) => m == current,
        None => false,
    };
    let constraints: Vec<_> = context
        .function_info(m, f)
        .signature
        .type_parameters
        .iter()
        .map(|tp| tp.kind.clone())
        .collect();

    let ty_args = match ty_args_opt {
        None => {
            let locs_constraints = constraints.into_iter().map(|k| (loc, k)).collect();
            make_tparams(context, loc, TVarCase::Base, locs_constraints)
        }
        Some(ty_args) => {
            let ty_args = check_type_argument_arity(
                context,
                loc,
                || format!("{}::{}", m, f),
                ty_args,
                &constraints,
            );
            instantiate_type_args(context, loc, None, ty_args, constraints)
        }
    };

    let finfo = context.function_info(m, f);
    let tparam_subst = &make_tparam_subst(&finfo.signature.type_parameters, ty_args.clone());
    let params = finfo
        .signature
        .parameters
        .iter()
        .map(|(n, t)| (n.clone(), subst_tparams(tparam_subst, t.clone())))
        .collect();
    let return_ty = subst_tparams(tparam_subst, finfo.signature.return_type.clone());
    let acquires = if in_current_module {
        finfo.acquires.clone()
    } else {
        BTreeSet::new()
    };
    let defined_loc = finfo.defined_loc;
    match &finfo.visibility {
        FunctionVisibility::Internal if !in_current_module => {
            let internal_msg = "This function is internal to its module. Only 'public' functions \
                                can be called outside of their module";
            context.error(vec![
                (loc, format!("Invalid call to '{}::{}'", m, f)),
                (defined_loc, internal_msg.into()),
            ])
        }
        _ => (),
    };
    (defined_loc, ty_args, params, acquires, return_ty)
}

//**************************************************************************************************
// Constraints
//**************************************************************************************************

pub fn solve_constraints(context: &mut Context) {
    use BuiltinTypeName_ as BT;
    let num_vars = context.subst.num_vars.clone();
    let mut subst = std::mem::replace(&mut context.subst, Subst::empty());
    for (num_var, loc) in num_vars {
        let tvar = sp(loc, Type_::Var(num_var));
        match unfold_type(&subst, tvar.clone()).value {
            Type_::UnresolvedError | Type_::Anything => {
                let next_subst = join(subst, &Type_::u64(loc), &tvar).unwrap().0;
                subst = next_subst;
            }
            _ => (),
        }
    }
    context.subst = subst;

    let constraints = std::mem::replace(&mut context.constraints, vec![]);
    for constraint in constraints {
        match constraint {
            Constraint::IsCopyable(loc, msg, s) => solve_copyable_constraint(context, loc, msg, s),
            Constraint::IsImplicitlyCopyable { loc, msg, ty, fix } => {
                solve_implicitly_copyable_constraint(context, loc, msg, ty, fix)
            }
            Constraint::KindConstraint(loc, b, k) => solve_kind_constraint(context, loc, b, k),
            Constraint::NumericConstraint(loc, op, t) => {
                solve_builtin_type_constraint(context, BT::numeric(), loc, op, t)
            }
            Constraint::BitsConstraint(loc, op, t) => {
                solve_builtin_type_constraint(context, BT::bits(), loc, op, t)
            }
            Constraint::OrderedConstraint(loc, op, t) => {
                solve_builtin_type_constraint(context, BT::ordered(), loc, op, t)
            }
            Constraint::BaseTypeConstraint(loc, msg, t) => {
                solve_base_type_constraint(context, loc, msg, &t)
            }
            Constraint::SingleTypeConstraint(loc, msg, t) => {
                solve_single_type_constraint(context, loc, msg, &t)
            }
        }
    }
}

fn solve_kind_constraint(context: &mut Context, loc: Loc, b: Type, k: Kind) {
    use Kind_ as K;
    let b = unfold_type(&context.subst, b);
    let bloc = b.loc;
    let b_kind = match infer_kind(&context, &context.subst, b.clone()) {
        // Anything => None
        // Unbound TVar or Anything satisfies any constraint. Will fail later in expansion
        None => return,
        Some(k) => k,
    };
    match (b_kind.value, &k.value) {
        (_, K::Copyable) => panic!("ICE tparams cannot have copyable constraints"),

        // _ <: all
        // copyable <: affine
        // affine <: affine
        // linear <: linear
        (_, K::Unknown)
        | (K::Copyable, K::Affine)
        | (K::Affine, K::Affine)
        | (K::Resource, K::Resource) => (),

        // copyable </: linear
        // affine </: linear
        // all </: linear
        (K::Copyable, K::Resource) | (K::Affine, K::Resource) | (K::Unknown, K::Resource) => {
            let ty_str = error_format(&b, &context.subst);
            let cmsg = format!(
                "The {} type {} does not satisfy the constraint '{}'",
                Kind_::VALUE_CONSTRAINT,
                ty_str,
                Kind_::RESOURCE_CONSTRAINT
            );
            context.error(vec![
                (loc, "Constraint not satisfied.".into()),
                (bloc, cmsg),
                (
                    b_kind.loc,
                    "The type's constraint information was determined here".into(),
                ),
                (
                    k.loc,
                    format!("'{}' constraint declared here", Kind_::RESOURCE_CONSTRAINT),
                ),
            ])
        }

        // all </: affine
        // linear </: affine
        (bk @ K::Unknown, K::Affine) | (bk @ K::Resource, K::Affine) => {
            let resource_msg = match bk {
                K::Copyable | K::Affine => panic!("ICE covered above"),
                K::Resource => "resource ",
                K::Unknown => "",
            };
            let ty_str = error_format(&b, &context.subst);
            let cmsg = format!(
                "The {}type {} does not satisfy the constraint '{}'",
                resource_msg,
                ty_str,
                Kind_::VALUE_CONSTRAINT
            );
            context.error(vec![
                (loc, "Constraint not satisfied.".into()),
                (bloc, cmsg),
                (
                    b_kind.loc,
                    "The type's constraint information was determined here".into(),
                ),
                (
                    k.loc,
                    format!("'{}' constraint declared here", Kind_::VALUE_CONSTRAINT),
                ),
            ])
        }
    }
}

fn solve_copyable_constraint(context: &mut Context, loc: Loc, msg: String, s: Type) {
    let s = unfold_type(&context.subst, s);
    let sloc = s.loc;
    let kind = match infer_kind(&context, &context.subst, s.clone()) {
        // Anything => None
        // Unbound TVar or Anything satisfies any constraint. Will fail later in expansion
        None => return,
        Some(k) => k,
    };
    match kind {
        sp!(_, Kind_::Copyable) | sp!(_, Kind_::Affine) => (),
        sp!(rloc, Kind_::Unknown) | sp!(rloc, Kind_::Resource) => {
            let ty_str = error_format(&s, &context.subst);
            context.error(vec![
                (loc, msg),
                (sloc, format!("The type: {}", ty_str)),
                (rloc, "Is found to be a non-copyable type here".into()),
            ])
        }
    }
}

fn solve_implicitly_copyable_constraint(
    context: &mut Context,
    loc: Loc,
    msg: String,
    ty: Type,
    fix: String,
) {
    let ty = unfold_type(&context.subst, ty);
    let tloc = ty.loc;
    let kind = match infer_kind(&context, &context.subst, ty.clone()) {
        // Anything => None
        // Unbound TVar or Anything satisfies any constraint. Will fail later in expansion
        None => return,
        Some(k) => k,
    };
    match kind {
        sp!(_, Kind_::Copyable) => (),
        sp!(kloc, Kind_::Affine) => {
            let ty_str = error_format(&ty, &context.subst);
            context.error(vec![
                (loc, format!("{} {}", msg, fix)),
                (tloc, format!("The type: {}", ty_str)),
                (
                    kloc,
                    "Is declared as a non-implicitly copyable type here".into(),
                ),
            ])
        }
        sp!(kloc, Kind_::Unknown) | sp!(kloc, Kind_::Resource) => {
            let ty_str = error_format(&ty, &context.subst);
            context.error(vec![
                (loc, msg),
                (tloc, format!("The type: {}", ty_str)),
                (kloc, "Is declared as a non-copyable type here".into()),
            ])
        }
    }
}

fn solve_builtin_type_constraint(
    context: &mut Context,
    builtin_set: BTreeSet<BuiltinTypeName_>,
    loc: Loc,
    op: &'static str,
    ty: Type,
) {
    use TypeName_::*;
    use Type_::*;
    let t = unfold_type(&context.subst, ty);
    let tloc = t.loc;
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
            "Found: {}. But {}",
            error_format(&t, &context.subst),
            set_msg
        )
    };
    match &t.value {
        Apply(k, sp!(_, Builtin(sp!(_, b))), args) if builtin_set.contains(b) => {
            if let Some(sp!(_, Kind_::Resource)) = k {
                panic!("ICE assumes this type is being consumed so shouldn't be a resource");
            }
            assert!(args.is_empty());
        }
        _ => {
            let error = vec![
                (loc, format!("Invalid argument to '{}'", op)),
                (tloc, tmsg()),
            ];
            context.error(error)
        }
    }
}

fn solve_base_type_constraint(context: &mut Context, loc: Loc, msg: String, ty: &Type) {
    use TypeName_::*;
    use Type_::*;
    let sp!(tyloc, unfolded_) = unfold_type(&context.subst, ty.clone());
    match unfolded_ {
        Var(_) => unreachable!(),
        Unit | Ref(_, _) | Apply(_, sp!(_, Multiple(_)), _) => {
            let tystr = error_format(ty, &context.subst);
            let tmsg = format!("Expected a single non-reference type, but found: {}", tystr);
            context.error(vec![(loc, msg), (tyloc, tmsg)])
        }
        UnresolvedError | Anything | Param(_) | Apply(_, _, _) => (),
    }
}

fn solve_single_type_constraint(context: &mut Context, loc: Loc, msg: String, ty: &Type) {
    use TypeName_::*;
    use Type_::*;
    let sp!(tyloc, unfolded_) = unfold_type(&context.subst, ty.clone());
    match unfolded_ {
        Var(_) => unreachable!(),
        Unit | Apply(_, sp!(_, Multiple(_)), _) => {
            let tystr = error_format(ty, &context.subst);
            let tmsg = format!(
                "Expected a single type, but found expression list type: {}",
                tystr
            );
            context.error(vec![(loc, msg), (tyloc, tmsg)])
        }
        UnresolvedError | Anything | Ref(_, _) | Param(_) | Apply(_, _, _) => (),
    }
}

//**************************************************************************************************
// Subst
//**************************************************************************************************

pub fn unfold_type(subst: &Subst, sp!(loc, t_): Type) -> Type {
    match t_ {
        Type_::Var(i) => match subst.get(i) {
            None => sp(loc, Type_::Anything),
            Some(inner) => unfold_type(subst, inner.clone()),
        },
        x => sp(loc, x),
    }
}

// Equivelent to unfold_type, but only returns the loc.
// The hope is to point to the last loc in a chain of type var's, giving the loc closest to the
// actual type in the source code
pub fn best_loc(subst: &Subst, sp!(loc, t_): &Type) -> Loc {
    match t_ {
        Type_::Var(i) => match subst.get(*i) {
            None => *loc,
            Some(inner) => best_loc(subst, inner),
        },
        _ => *loc,
    }
}

fn make_tparam_subst(tps: &[TParam], args: Vec<Type>) -> TParamSubst {
    assert!(tps.len() == args.len());
    let mut subst = TParamSubst::new();
    for (tp, arg) in tps.iter().zip(args) {
        let old_val = subst.insert(tp.id.clone(), arg);
        assert!(old_val.is_none())
    }
    subst
}

fn subst_tparams(subst: &TParamSubst, sp!(loc, t_): Type) -> Type {
    use Type_::*;
    match t_ {
        x @ Unit | x @ UnresolvedError | x @ Anything => sp(loc, x),
        Var(_) => panic!("ICE tvar in subst_tparams"),
        Ref(mut_, t) => sp(loc, Ref(mut_, Box::new(subst_tparams(subst, *t)))),
        Param(tp) => subst
            .get(&tp.id)
            .expect("ICE unmapped tparam in subst_tparams_base")
            .clone(),
        Apply(k, n, ty_args) => {
            let ftys = ty_args
                .into_iter()
                .map(|t| subst_tparams(subst, t))
                .collect();
            sp(loc, Apply(k, n, ftys))
        }
    }
}

pub fn ready_tvars(subst: &Subst, sp!(loc, t_): Type) -> Type {
    use Type_::*;
    match t_ {
        x @ UnresolvedError | x @ Unit | x @ Anything | x @ Param(_) => sp(loc, x),
        Ref(mut_, t) => sp(loc, Ref(mut_, Box::new(ready_tvars(subst, *t)))),
        Apply(k, n, tys) => {
            let tys = tys.into_iter().map(|t| ready_tvars(subst, t)).collect();
            sp(loc, Apply(k, n, tys))
        }
        Var(i) => match subst.get(i) {
            None => sp(loc, Var(i)),
            Some(t) => ready_tvars(subst, t.clone()),
        },
    }
}

//**************************************************************************************************
// Instantiate
//**************************************************************************************************

pub fn instantiate(context: &mut Context, sp!(loc, t_): Type) -> Type {
    use Type_::*;
    let it_ = match t_ {
        Unit => Unit,
        UnresolvedError => UnresolvedError,
        Anything => make_tvar(context, loc).value,
        Ref(mut_, b) => Ref(mut_, Box::new(instantiate(context, *b))),
        Apply(kopt, n, ty_args) => instantiate_apply(context, loc, kopt, n, ty_args),
        x @ Param(_) => x,
        Var(_) => panic!("ICE instantiate type variable"),
    };
    sp(loc, it_)
}

fn instantiate_apply(
    context: &mut Context,
    loc: Loc,
    kind_opt: Option<Kind>,
    n: TypeName,
    mut ty_args: Vec<Type>,
) -> Type_ {
    let tparam_constraints: Vec<Kind> = match &n {
        sp!(nloc, N::TypeName_::Builtin(b)) => b.value.tparam_constraints(*nloc),
        sp!(nloc, N::TypeName_::Multiple(len)) => {
            (0..*len).map(|_| sp(*nloc, Kind_::Unknown)).collect()
        }
        sp!(_, N::TypeName_::ModuleType(m, s)) => {
            let tps = context.struct_tparams(m, s);
            tps.iter().map(|tp| tp.kind.clone()).collect()
        }
    };
    ty_args = check_type_argument_arity(
        context,
        loc,
        || format!("{}", &n),
        ty_args,
        &tparam_constraints,
    );

    let tys = instantiate_type_args(context, loc, Some(&n.value), ty_args, tparam_constraints);
    Type_::Apply(kind_opt, n, tys)
}

// The type arguments are bound to type variables after intantiation
// i.e. vec<t1, ..., tn> ~> vec<a1, ..., an> s.t a1 => t1, ... , an => tn
// This might be needed for any variance case, and I THINK that it should be fine without it
// BUT I'm adding it as a safeguard against instantiating twice. Can always remove once this
// stabilizes
fn instantiate_type_args(
    context: &mut Context,
    loc: Loc,
    n: Option<&TypeName_>,
    mut ty_args: Vec<Type>,
    constraints: Vec<Kind>,
) -> Vec<Type> {
    assert!(ty_args.len() == constraints.len());
    let locs_constraints = constraints
        .into_iter()
        .zip(&ty_args)
        .map(|(k, t)| (t.loc, k))
        .collect();
    let tvar_case = match n {
        Some(TypeName_::Multiple(_)) => {
            TVarCase::Single("Invalid expression list type argument".to_owned())
        }
        None | Some(TypeName_::Builtin(_)) | Some(TypeName_::ModuleType(_, _)) => TVarCase::Base,
    };
    let tvars = make_tparams(context, loc, tvar_case, locs_constraints);
    ty_args = ty_args
        .into_iter()
        .map(|t| instantiate(context, t))
        .collect();

    assert!(ty_args.len() == tvars.len());
    let mut res = vec![];
    let subst = std::mem::replace(&mut context.subst, /* dummy value */ Subst::empty());
    context.subst = tvars
        .into_iter()
        .zip(ty_args)
        .fold(subst, |subst, (tvar, ty_arg)| {
            // tvar is just a type variable, so shouldn't throw ever...
            let (subst, t) = join(subst, &tvar, &ty_arg).ok().unwrap();
            res.push(t);
            subst
        });
    res
}

fn check_type_argument_arity<F: FnOnce() -> String>(
    context: &mut Context,
    loc: Loc,
    name_f: F,
    mut ty_args: Vec<Type>,
    tparam_constraints: &[Kind],
) -> Vec<Type> {
    let args_len = ty_args.len();
    let arity = tparam_constraints.len();
    if args_len != arity {
        context.error(vec![(
            loc,
            format!(
                "Invalid instantiation of '{}'. Expected {} type arguments but got {}",
                name_f(),
                arity,
                args_len
            ),
        )])
    }

    while ty_args.len() > arity {
        ty_args.pop();
    }

    while ty_args.len() < arity {
        ty_args.push(context.error_type(loc));
    }

    ty_args
}

enum TVarCase {
    Single(String),
    Base,
}

fn make_tparams(
    context: &mut Context,
    loc: Loc,
    case: TVarCase,
    tparam_constraints: Vec<(Loc, Kind)>,
) -> Vec<Type> {
    tparam_constraints
        .into_iter()
        .map(|(vloc, constraint)| {
            let tvar = make_tvar(context, vloc);
            context.add_kind_constraint(loc, tvar.clone(), constraint);
            match &case {
                TVarCase::Single(msg) => context.add_single_type_constraint(loc, msg, tvar.clone()),
                TVarCase::Base => {
                    context.add_base_type_constraint(loc, "Invalid type argument", tvar.clone())
                }
            };
            tvar
        })
        .collect()
}

//**************************************************************************************************
// Subtype and joining
//**************************************************************************************************

#[derive(Debug)]
pub enum TypingError {
    SubtypeError(Box<Type>, Box<Type>),
    Incompatible(Box<Type>, Box<Type>),
    ArityMismatch(usize, Box<Type>, usize, Box<Type>),
    RecursiveType(Loc),
}

#[derive(Clone, Copy, Debug)]
enum TypingCase {
    Join,
    Subtype,
}

pub fn subtype(subst: Subst, lhs: &Type, rhs: &Type) -> Result<(Subst, Type), TypingError> {
    join_impl(subst, TypingCase::Subtype, lhs, rhs)
}

pub fn join(subst: Subst, lhs: &Type, rhs: &Type) -> Result<(Subst, Type), TypingError> {
    join_impl(subst, TypingCase::Join, lhs, rhs)
}

fn join_impl(
    mut subst: Subst,
    case: TypingCase,
    lhs: &Type,
    rhs: &Type,
) -> Result<(Subst, Type), TypingError> {
    use TypeName_::*;
    use Type_::*;
    use TypingCase::*;
    match (lhs, rhs) {
        (sp!(_, Anything), other) | (other, sp!(_, Anything)) => Ok((subst, other.clone())),

        (sp!(_, Unit), sp!(loc, Unit)) => Ok((subst, sp(*loc, Unit))),

        (sp!(loc1, Ref(mut1, t1)), sp!(loc2, Ref(mut2, t2))) => {
            let (loc, mut_) = match (case, mut1, mut2) {
                (Join, _, _) => {
                    // if 1 is imm and 2 is mut, use loc1. Else, loc2
                    let loc = if !*mut1 && *mut2 { *loc1 } else { *loc2 };
                    (loc, *mut1 && *mut2)
                }
                // imm <: imm
                // mut <: imm
                (Subtype, false, false) | (Subtype, true, false) => (*loc2, false),
                // mut <: mut
                (Subtype, true, true) => (*loc2, true),
                // imm <\: mut
                (Subtype, false, true) => {
                    return Err(TypingError::SubtypeError(
                        Box::new(lhs.clone()),
                        Box::new(rhs.clone()),
                    ))
                }
            };
            let (subst, t) = join_impl(subst, case, t1, t2)?;
            Ok((subst, sp(loc, Ref(mut_, Box::new(t)))))
        }
        (sp!(_, Param(TParam { id: id1, .. })), sp!(_, Param(TParam { id: id2, .. })))
            if id1 == id2 =>
        {
            Ok((subst, rhs.clone()))
        }
        (sp!(_, Apply(_, sp!(_, Multiple(n1)), _)), sp!(_, Apply(_, sp!(_, Multiple(n2)), _)))
            if n1 != n2 =>
        {
            Err(TypingError::ArityMismatch(
                *n1,
                Box::new(lhs.clone()),
                *n2,
                Box::new(rhs.clone()),
            ))
        }
        (sp!(_, Apply(k1, n1, tys1)), sp!(loc, Apply(k2, n2, tys2))) if n1 == n2 => {
            assert!(
                k1 == k2,
                "ICE failed naming: {:#?}kind != {:#?}kind. {:#?} !=  {:#?}",
                n1,
                n2,
                k1,
                k2
            );
            let (subst, tys) = join_impl_types(subst, case, tys1, tys2)?;
            Ok((subst, sp(*loc, Apply(k2.clone(), n2.clone(), tys))))
        }
        (sp!(loc1, Var(id1)), sp!(loc2, Var(id2))) => {
            if *id1 == *id2 {
                Ok((subst, sp(*loc2, Var(*id2))))
            } else {
                join_tvar(subst, case, *loc1, *id1, *loc2, *id2)
            }
        }
        (sp!(loc, Var(id)), other) | (other, sp!(loc, Var(id))) if subst.get(*id).is_none() => {
            match join_bind_tvar(&mut subst, *loc, *id, other.clone()) {
                Err(()) => Err(TypingError::Incompatible(
                    Box::new(sp(*loc, Var(*id))),
                    Box::new(other.clone()),
                )),
                Ok(()) => Ok((subst, sp(*loc, Var(*id)))),
            }
        }
        (sp!(loc, Var(id)), other) => {
            let new_tvar = TVar::next();
            subst.insert(new_tvar, other.clone());
            join_tvar(subst, case, *loc, *id, other.loc, new_tvar)
        }
        (other, sp!(loc, Var(id))) => {
            let new_tvar = TVar::next();
            subst.insert(new_tvar, other.clone());
            join_tvar(subst, case, other.loc, new_tvar, *loc, *id)
        }

        (sp!(_, UnresolvedError), other) | (other, sp!(_, UnresolvedError)) => {
            Ok((subst, other.clone()))
        }
        _ => Err(TypingError::Incompatible(
            Box::new(lhs.clone()),
            Box::new(rhs.clone()),
        )),
    }
}

fn join_impl_types(
    mut subst: Subst,
    case: TypingCase,
    tys1: &[Type],
    tys2: &[Type],
) -> Result<(Subst, Vec<Type>), TypingError> {
    // if tys1.len() != tys2.len(), we will get an error when instantiating the type elsewhere
    // as all types are instantiated as a sanity check
    let mut tys = vec![];
    for (ty1, ty2) in tys1.iter().zip(tys2) {
        let (nsubst, t) = join_impl(subst, case, ty1, ty2)?;
        subst = nsubst;
        tys.push(t)
    }
    Ok((subst, tys))
}

fn join_tvar(
    mut subst: Subst,
    case: TypingCase,
    loc1: Loc,
    id1: TVar,
    loc2: Loc,
    id2: TVar,
) -> Result<(Subst, Type), TypingError> {
    use Type_::*;
    let last_id1 = forward_tvar(&subst, id1);
    let last_id2 = forward_tvar(&subst, id2);
    let ty1 = match subst.get(last_id1) {
        None => sp(loc1, Anything),
        Some(t) => t.clone(),
    };
    let ty2 = match subst.get(last_id2) {
        None => sp(loc2, Anything),
        Some(t) => t.clone(),
    };

    let new_tvar = TVar::next();
    let num_loc_1 = subst.num_vars.get(&last_id1);
    let num_loc_2 = subst.num_vars.get(&last_id2);
    match (num_loc_1, num_loc_2) {
        (_, Some(nloc)) | (Some(nloc), _) => {
            let nloc = *nloc;
            subst.set_num_var(new_tvar, nloc);
        }
        _ => (),
    }
    subst.insert(last_id1, sp(loc1, Var(new_tvar)));
    subst.insert(last_id2, sp(loc2, Var(new_tvar)));

    let (mut subst, new_ty) = join_impl(subst, case, &ty1, &ty2)?;
    match subst.get(new_tvar) {
        Some(sp!(tloc, _)) => Err(TypingError::RecursiveType(*tloc)),
        None => match join_bind_tvar(&mut subst, loc2, new_tvar, new_ty) {
            Ok(()) => Ok((subst, sp(loc2, Var(new_tvar)))),
            Err(()) => {
                let ty1 = match ty1 {
                    sp!(loc, Anything) => sp(loc, Var(id1)),
                    t => t,
                };
                let ty2 = match ty2 {
                    sp!(loc, Anything) => sp(loc, Var(id2)),
                    t => t,
                };
                Err(TypingError::Incompatible(Box::new(ty1), Box::new(ty2)))
            }
        },
    }
}

fn forward_tvar(subst: &Subst, id: TVar) -> TVar {
    match subst.get(id) {
        Some(sp!(_, Type_::Var(next))) => forward_tvar(subst, *next),
        Some(_) | None => id,
    }
}

fn join_bind_tvar(subst: &mut Subst, loc: Loc, tvar: TVar, ty: Type) -> Result<(), ()> {
    // check not necessary for soundness but improves error message structure
    if !check_num_tvar(&subst, loc, tvar, &ty) {
        return Err(());
    }
    match &ty.value {
        Type_::Anything => (),
        _ => subst.insert(tvar, ty),
    }
    Ok(())
}

fn check_num_tvar(subst: &Subst, loc: Loc, tvar: TVar, ty: &Type) -> bool {
    !subst.is_num_var(tvar) || check_num_tvar_(subst, loc, tvar, ty)
}

fn check_num_tvar_(subst: &Subst, loc: Loc, tvar: TVar, ty: &Type) -> bool {
    use Type_::*;
    match &ty.value {
        UnresolvedError | Anything => true,
        Apply(_, sp!(_, TypeName_::Builtin(sp!(_, bt))), _) => bt.is_numeric(),

        Var(v) if subst.tvars.contains_key(v) => {
            check_num_tvar_(subst, loc, tvar, subst.get(*v).unwrap())
        }
        Var(v) => subst.is_num_var(*v),
        _ => false,
    }
}

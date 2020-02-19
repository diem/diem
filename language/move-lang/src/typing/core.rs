// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::shared::unique_map::UniqueMap;
use crate::{
    errors::*,
    naming::ast::{
        self as N, BaseType, BaseType_, BuiltinTypeName_, FunctionSignature, SingleType,
        SingleType_, StructDefinition, TParam, TParamID, TVar, Type, TypeName, TypeName_, Type_,
    },
    parser::ast::{
        Field, FunctionName, FunctionVisibility, Kind, Kind_, ModuleIdent, ResourceLoc, StructName,
        Var,
    },
    shared::*,
};
use std::collections::{BTreeSet, HashMap};

//**************************************************************************************************
// Context
//**************************************************************************************************

pub enum Constraint {
    IsCopyable(Loc, String, SingleType),
    IsImplicitlyCopyable {
        loc: Loc,
        msg: String,
        ty: BaseType,
        fix: String,
    },
    KindConstraint(Loc, BaseType, Kind),
    NumericConstraint(Loc, &'static str, Type),
    BitsConstraint(Loc, &'static str, Type),
    OrderedConstraint(Loc, &'static str, Type),
    SignedConstraint(Loc, &'static str, Type),
}
pub type Constraints = Vec<Constraint>;
type TParamSubst = HashMap<TParamID, BaseType>;

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

#[derive(Clone)]
#[allow(clippy::large_enum_variant)]
pub enum LocalStatus {
    Declared(Loc),
    Typed(SingleType),
}

pub struct Context {
    pub modules: UniqueMap<ModuleIdent, ModuleInfo>,

    pub current_module: Option<ModuleIdent>,
    pub current_function: Option<FunctionName>,
    pub return_type: Option<Type>,
    pub locals: UniqueMap<Var, LocalStatus>,

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

    pub fn add_implicit_copyable_constraint(
        &mut self,
        loc: Loc,
        msg: impl Into<String>,
        ty: BaseType,
        fix: impl Into<String>,
    ) {
        let msg = msg.into();
        let fix = fix.into();
        self.constraints
            .push(Constraint::IsImplicitlyCopyable { loc, msg, ty, fix })
    }

    pub fn add_copyable_constraint(&mut self, loc: Loc, msg: impl Into<String>, s: SingleType) {
        self.constraints
            .push(Constraint::IsCopyable(loc, msg.into(), s))
    }

    pub fn add_kind_constraint(&mut self, loc: Loc, b: BaseType, k: Kind) {
        self.constraints.push(Constraint::KindConstraint(loc, b, k))
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

    pub fn add_signed_constraint(&mut self, loc: Loc, op: &'static str, t: Type) {
        self.constraints
            .push(Constraint::SignedConstraint(loc, op, t))
    }

    pub fn declare_local(&mut self, var: Var, ty_opt: Option<SingleType>) {
        let status = match ty_opt {
            None => LocalStatus::Declared(var.loc()),
            Some(t) => LocalStatus::Typed(t),
        };
        // Might overwrite (i.e. shadow) the current local's type
        self.locals.remove(&var);
        self.locals.add(var, status).unwrap();
    }

    pub fn is_current_module(&self, m: &ModuleIdent) -> bool {
        match &self.current_module {
            Some(curm) => curm == m,
            None => false,
        }
    }

    pub fn is_current_function(&self, m: &ModuleIdent, f: &FunctionName) -> bool {
        self.is_current_module(m)
            && match &self.current_function {
                Some(curf) => curf == f,
                _ => false,
            }
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

    pub fn function_acquires(&mut self, m: &ModuleIdent, n: &FunctionName) -> BTreeSet<StructName> {
        self.function_info(m, n).acquires.clone()
    }
}

//**************************************************************************************************
// Subst
//**************************************************************************************************

#[derive(Clone, Debug)]
pub struct Subst {
    tvars: HashMap<TVar, BaseType>,
    num_vars: HashMap<TVar, Loc>,
}

impl Subst {
    pub fn empty() -> Self {
        Self {
            tvars: HashMap::new(),
            num_vars: HashMap::new(),
        }
    }

    pub fn insert(&mut self, tvar: TVar, bt: BaseType) {
        self.tvars.insert(tvar, bt);
    }

    pub fn get(&self, tvar: TVar) -> Option<&BaseType> {
        self.tvars.get(&tvar)
    }

    pub fn new_num_var(&mut self, loc: Loc) -> TVar {
        let tvar = TVar::next();
        assert!(self.num_vars.insert(tvar, loc).is_none());
        tvar
    }

    pub fn set_num_var(&mut self, tvar: TVar, loc: Loc) {
        self.num_vars.entry(tvar).or_insert(loc);
        if let Some(sp!(_, BaseType_::Var(next))) = self.get(tvar) {
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

pub fn error_format_base(b: &BaseType, subst: &Subst) -> String {
    error_format_base_(b, subst, false)
}

fn error_format_base_(sp!(_, b_): &BaseType, subst: &Subst, nested: bool) -> String {
    use BaseType_::*;
    let res = match b_ {
        Apply(_, n, tys) => {
            let tys_str = if !tys.is_empty() {
                format!(
                    "<{}>",
                    format_comma(tys.iter().map(|t| error_format_base_(t, subst, true)))
                )
            } else {
                "".to_string()
            };
            format!("{}{}", n, tys_str)
        }
        Param(tp) => tp.debug.value.to_string(),
        Var(id) => match subst.get(*id) {
            Some(t) => error_format_base_(t, subst, nested),
            None if nested && subst.is_num_var(*id) => "{integer}".to_string(),
            None if subst.is_num_var(*id) => return "integer".to_string(),
            None => "_".to_string(),
        },
        Anything => "_".to_string(),
    };
    if nested {
        res
    } else {
        format!("'{}'", res)
    }
}

pub fn error_format_single(s: &SingleType, subst: &Subst) -> String {
    error_format_single_(s, subst, false)
}

fn error_format_single_(sp!(_, s_): &SingleType, subst: &Subst, nested: bool) -> String {
    use SingleType_::*;
    match s_ {
        Ref(mut_, ty) => format!(
            "'&{}{}'",
            if *mut_ { "mut " } else { "" },
            error_format_base_(ty, subst, true)
        ),
        Base(ty) => error_format_base_(ty, subst, nested),
    }
}

pub fn error_format(sp!(_, t_): &Type, subst: &Subst) -> String {
    use Type_::*;
    match t_ {
        Unit => "'()'".into(),
        Single(s) => error_format_single_(s, subst, false),
        Multiple(ss) => {
            let inner = format_comma(ss.iter().map(|s| error_format_single_(s, subst, true)));
            format!("'({})'", inner)
        }
    }
}

//**************************************************************************************************
// Type utils
//**************************************************************************************************

pub fn infer_kind(context: &Context, subst: &Subst, s: SingleType) -> Option<Kind> {
    use SingleType_ as S;
    match s.value {
        S::Ref(_, _) => Some(sp(s.loc, Kind_::Unrestricted)),
        S::Base(b) => infer_kind_base(context, subst, b),
    }
}

pub fn infer_kind_base(context: &Context, subst: &Subst, b: BaseType) -> Option<Kind> {
    use BaseType_ as B;
    match unfold_type_base(&subst, b) {
        sp!(_, B::Var(_)) => panic!("ICE unfold_type_base failed, which is impossible"),
        sp!(_, B::Anything) => None,
        sp!(_, B::Param(TParam { kind, .. })) | sp!(_, B::Apply(Some(kind), _, _)) => Some(kind),
        // if any unknown, give unkown
        // else if any resource, give resource
        // else affine
        sp!(_, B::Apply(None, n, tyl)) => {
            // If an anything is found, we get a none. Then use the constraint for the
            // default kind
            let contraints = match &n.value {
                TypeName_::Builtin(_) => tyl.iter().map(|_| None).collect::<Vec<_>>(),
                TypeName_::ModuleType(m, n) => {
                    let sdef = context.struct_definition(m, n);
                    sdef.type_parameters
                        .iter()
                        .map(|tp| Some(tp.kind.clone()))
                        .collect::<Vec<_>>()
                }
            };
            let res = tyl
                .into_iter()
                .zip(contraints)
                .filter_map(|(t, constraint_opt)| {
                    infer_kind_base(context, subst, t).or(constraint_opt)
                })
                .map(|k| match k {
                    sp!(loc, Kind_::Unrestricted) => sp(loc, Kind_::Affine),
                    k => k,
                })
                .max_by(most_general_kind)
                .unwrap_or_else(|| sp(type_name_declared_loc(context, &n), Kind_::Affine));
            Some(res)
        }
    }
}

fn most_general_kind(k1: &Kind, k2: &Kind) -> std::cmp::Ordering {
    use std::cmp::Ordering as O;
    use Kind_ as K;
    match (&k1.value, &k2.value) {
        (K::Unrestricted, _) | (_, K::Unrestricted) => panic!("ICE structs cannot be unrestricted"),

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
        TypeName_::Builtin(_) => *loc,
        TypeName_::ModuleType(m, n) => context.struct_declared_loc(m, n),
    }
}

//**************************************************************************************************
// Structs
//**************************************************************************************************

pub fn make_struct_type(
    context: &mut Context,
    loc: Loc,
    m: &ModuleIdent,
    n: &StructName,
    ty_args_opt: Option<Vec<BaseType>>,
) -> (BaseType, Vec<BaseType>) {
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
            let ty_args = make_tparams(context, loc, constraints);
            (
                sp(loc, BaseType_::Apply(kind_opt, tn, ty_args.clone())),
                ty_args,
            )
        }
        Some(ty_args) => {
            let tapply = instantiate_apply(context, loc, kind_opt, tn, ty_args);
            let targs = match &tapply.value {
                BaseType_::Apply(_, _, targs) => targs.clone(),
                _ => panic!("ICE instantiate_apply returned non Apply"),
            };
            (tapply, targs)
        }
    }
}

// ty_args should come from make_struct_type
pub fn make_field_types(
    context: &mut Context,
    _loc: Loc,
    m: &ModuleIdent,
    n: &StructName,
    ty_args: Vec<BaseType>,
) -> N::StructFields {
    let sdef = context.struct_definition(m, n);
    let tparam_subst =
        &make_tparam_subst(&context.struct_definition(m, n).type_parameters, ty_args);
    match &sdef.fields {
        N::StructFields::Native(loc) => N::StructFields::Native(*loc),
        N::StructFields::Defined(m) => N::StructFields::Defined(m.ref_map(|_, (idx, field_ty)| {
            (*idx, subst_tparams_base(tparam_subst, field_ty.clone()))
        })),
    }
}

// ty_args should come from make_struct_type
pub fn make_field_type(
    context: &mut Context,
    loc: Loc,
    m: &ModuleIdent,
    n: &StructName,
    ty_args: Vec<BaseType>,
    field: &Field,
) -> BaseType {
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
            return sp(loc, BaseType_::Anything);
        }
        N::StructFields::Defined(m) => m,
    };
    match fields_map.get(field).cloned() {
        None => {
            context.error(vec![(
                loc,
                format!("Unbound field '{}' in '{}::{}'", field, m, n),
            )]);
            sp(loc, BaseType_::Anything)
        }
        Some((_, field_ty)) => {
            let tparam_subst =
                &make_tparam_subst(&context.struct_definition(m, n).type_parameters, ty_args);
            subst_tparams_base(tparam_subst, field_ty)
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
    ty_args_opt: Option<Vec<BaseType>>,
) -> (
    Loc,
    Vec<BaseType>,
    Vec<(Var, SingleType)>,
    BTreeSet<StructName>,
    Type,
) {
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
            make_tparams(context, loc, locs_constraints)
        }
        Some(ty_args) => {
            let ty_args = check_type_argument_arity(
                context,
                loc,
                || format!("{}::{}", m, f),
                ty_args,
                &constraints,
            );
            instantiate_type_args(context, loc, ty_args, constraints)
        }
    };

    let finfo = context.function_info(m, f);
    let tparam_subst = &make_tparam_subst(&finfo.signature.type_parameters, ty_args.clone());
    let params = finfo
        .signature
        .parameters
        .iter()
        .map(|(n, t)| (n.clone(), subst_tparams_single(tparam_subst, t.clone())))
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
        let tvar = sp(loc, BaseType_::Var(num_var));
        if let BaseType_::Anything = unfold_type_base(&subst, tvar.clone()).value {
            let next_subst = join_base_type(subst, &BaseType_::u64(loc), &tvar)
                .unwrap()
                .0;
            subst = next_subst;
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
            Constraint::SignedConstraint(loc, op, t) => {
                solve_builtin_type_constraint(context, BT::signed(), loc, op, t)
            }
        }
    }
}

fn solve_kind_constraint(context: &mut Context, loc: Loc, b: BaseType, k: Kind) {
    use Kind_ as K;
    let b = unfold_type_base(&context.subst, b);
    let bloc = b.loc;
    let b_kind = match infer_kind_base(&context, &context.subst, b.clone()) {
        // Anything => None
        // Unbound TVar or Anything satisfies any constraint. Will fail later in expansion
        None => return,
        Some(k) => k,
    };
    match (b_kind.value, &k.value) {
        (_, K::Unrestricted) => panic!("ICE tparams cannot have unrestricted constraints"),

        // _ <: all
        // unrestricted <: affine
        // affine <: affine
        // linear <: linear
        (_, K::Unknown)
        | (K::Unrestricted, K::Affine)
        | (K::Affine, K::Affine)
        | (K::Resource, K::Resource) => (),

        // unrestricted </: linear
        // affine </: linear
        // all </: linear
        (K::Unrestricted, K::Resource) | (K::Affine, K::Resource) | (K::Unknown, K::Resource) => {
            let ty_str = error_format_base(&b, &context.subst);
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
                    "The type's constraint information was declared here".into(),
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
                K::Unrestricted | K::Affine => panic!("ICE covered above"),
                K::Resource => "resource ",
                K::Unknown => "",
            };
            let ty_str = error_format_base(&b, &context.subst);
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
                    "The type's constraint information was declared here".into(),
                ),
                (
                    k.loc,
                    format!("'{}' constraint declared here", Kind_::VALUE_CONSTRAINT),
                ),
            ])
        }
    }
}

fn solve_copyable_constraint(context: &mut Context, loc: Loc, msg: String, s: SingleType) {
    let s = unfold_type_single(&context.subst, s);
    let sloc = s.loc;
    let kind = match infer_kind(&context, &context.subst, s.clone()) {
        // Anything => None
        // Unbound TVar or Anything satisfies any constraint. Will fail later in expansion
        None => return,
        Some(k) => k,
    };
    match kind {
        sp!(_, Kind_::Unrestricted) | sp!(_, Kind_::Affine) => (),
        sp!(rloc, Kind_::Unknown) | sp!(rloc, Kind_::Resource) => {
            let ty_str = error_format_single(&s, &context.subst);
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
    b: BaseType,
    fix: String,
) {
    let b = unfold_type_base(&context.subst, b);
    let bloc = b.loc;
    let kind = match infer_kind_base(&context, &context.subst, b.clone()) {
        // Anything => None
        // Unbound TVar or Anything satisfies any constraint. Will fail later in expansion
        None => return,
        Some(k) => k,
    };
    match kind {
        sp!(_, Kind_::Unrestricted) => (),
        sp!(kloc, Kind_::Affine) => {
            let ty_str = error_format_base(&b, &context.subst);
            context.error(vec![
                (loc, format!("{} {}", msg, fix)),
                (bloc, format!("The type: {}", ty_str)),
                (
                    kloc,
                    "Is declared as a non-implicitly copyable type here".into(),
                ),
            ])
        }
        sp!(kloc, Kind_::Unknown) | sp!(kloc, Kind_::Resource) => {
            let ty_str = error_format_base(&b, &context.subst);
            context.error(vec![
                (loc, msg),
                (bloc, format!("The type: {}", ty_str)),
                (kloc, "Is declared as a non-copyable type here".into()),
            ])
        }
    }
}

macro_rules! TApply {
    ($k:pat, $n:pat, $args:pat) => {
        Type_::Single(sp!(
            _,
            SingleType_::Base(sp!(_, BaseType_::Apply($k, $n, $args)))
        ))
    };
}

fn solve_builtin_type_constraint(
    context: &mut Context,
    builtin_set: BTreeSet<BuiltinTypeName_>,
    loc: Loc,
    op: &'static str,
    ty: Type,
) {
    use BaseType_::*;
    use SingleType_::*;
    use TypeName_::*;
    use Type_::*;
    let tloc = ty.loc;
    let t = unfold_type(&context.subst, ty);
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
        TApply!(k, sp!(_, Builtin(sp!(_, b))), args) if builtin_set.contains(b) => {
            if let Some(sp!(_, Kind_::Resource)) = k {
                panic!("ICE assumes this type is being consumed so shouldn't be a resource");
            }
            assert!(args.is_empty());
        }
        Single(sp!(_, Base(sp!(_, Var(_))))) => panic!("ICE unfold_type failed"),
        _ => {
            let error = vec![
                (loc, format!("Invalid argument to '{}'", op)),
                (tloc, tmsg()),
            ];
            context.error(error)
        }
    }
}

//**************************************************************************************************
// Subst
//**************************************************************************************************

pub fn unfold_type(subst: &Subst, sp!(loc, t_): Type) -> Type {
    use Type_::*;
    let ft_ = match t_ {
        Single(s) => Single(unfold_type_single(subst, s)),
        x => x,
    };
    sp(loc, ft_)
}

pub fn unfold_type_single(subst: &Subst, sp!(loc, s_): SingleType) -> SingleType {
    use SingleType_::*;
    let fs_ = match s_ {
        Base(b) => Base(unfold_type_base(subst, b)),
        x => x,
    };
    sp(loc, fs_)
}

pub fn unfold_type_base(subst: &Subst, sp!(loc, b_): BaseType) -> BaseType {
    use BaseType_::*;
    match b_ {
        Var(i) => match subst.get(i) {
            None => sp(loc, Anything),
            Some(inner) => unfold_type_base(subst, inner.clone()),
        },
        x => sp(loc, x),
    }
}

fn make_tparam_subst(tps: &[TParam], args: Vec<BaseType>) -> TParamSubst {
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
    let ft_ = match t_ {
        Unit => Unit,
        Single(s) => Single(subst_tparams_single(subst, s)),
        Multiple(ss) => Multiple(
            ss.into_iter()
                .map(|t_| subst_tparams_single(subst, t_))
                .collect(),
        ),
    };
    sp(loc, ft_)
}

fn subst_tparams_single(subst: &TParamSubst, sp!(loc, s_): SingleType) -> SingleType {
    use SingleType_::*;
    let fs_ = match s_ {
        Ref(mut_, b) => Ref(mut_, subst_tparams_base(subst, b)),
        Base(b) => Base(subst_tparams_base(subst, b)),
    };
    sp(loc, fs_)
}

fn subst_tparams_base(subst: &TParamSubst, sp!(loc, b_): BaseType) -> BaseType {
    use BaseType_::*;
    match b_ {
        Var(_) => panic!("ICE tvar in subst_tparams_base"),
        Anything => sp(loc, Anything),
        Param(tp) => subst
            .get(&tp.id)
            .expect("ICE unmapped tparam in subst_tparams_base")
            .clone(),
        Apply(k, n, ty_args) => {
            let ftys = ty_args
                .into_iter()
                .map(|b| subst_tparams_base(subst, b))
                .collect();
            sp(loc, Apply(k, n, ftys))
        }
    }
}

//**************************************************************************************************
// Instantiate
//**************************************************************************************************

pub fn instantiate(context: &mut Context, sp!(loc, t_): Type) -> Type {
    use Type_::*;
    let it_ = match t_ {
        Unit => Unit,
        Single(s) => Single(instantiate_single(context, s)),
        Multiple(ss) => Multiple(
            ss.into_iter()
                .map(|t_| instantiate_single(context, t_))
                .collect(),
        ),
    };
    sp(loc, it_)
}

pub fn instantiate_single(context: &mut Context, sp!(loc, s_): SingleType) -> SingleType {
    use SingleType_::*;
    let is_ = match s_ {
        Ref(mut_, b) => Ref(mut_, instantiate_base(context, b)),
        Base(b) => Base(instantiate_base(context, b)),
    };
    sp(loc, is_)
}

pub fn instantiate_base(context: &mut Context, sp!(loc, b_): BaseType) -> BaseType {
    use BaseType_::*;
    match b_ {
        Var(_) => panic!("ICE instantiate type variable"),
        x @ Anything | x @ Param(_) => sp(loc, x),
        Apply(kopt, n, ty_args) => instantiate_apply(context, loc, kopt, n, ty_args),
    }
}

fn instantiate_apply(
    context: &mut Context,
    loc: Loc,
    kind_opt: Option<Kind>,
    n: TypeName,
    mut ty_args: Vec<BaseType>,
) -> BaseType {
    let tparam_constraints: Vec<Kind> = match &n {
        sp!(nloc, N::TypeName_::Builtin(b)) => b.value.tparam_constraints(*nloc),
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

    let tys = instantiate_type_args(context, loc, ty_args, tparam_constraints);
    sp(loc, BaseType_::Apply(kind_opt, n, tys))
}

// The type arguments are bound to type variables after intantiation
// i.e. vec<t1, ..., tn> ~> vec<a1, ..., an> s.t a1 => t1, ... , an => tn
// This might be needed for any variance case, and I THINK that it should be fine without it
// BUT I'm adding it as a safeguard against instantiating twice. Can always remove once this
// stabilizes
fn instantiate_type_args(
    context: &mut Context,
    loc: Loc,
    mut ty_args: Vec<BaseType>,
    constraints: Vec<Kind>,
) -> Vec<BaseType> {
    assert!(ty_args.len() == constraints.len());
    let locs_constraints = constraints
        .into_iter()
        .zip(&ty_args)
        .map(|(k, t)| (t.loc, k))
        .collect();
    let tvars = make_tparams(context, loc, locs_constraints);
    ty_args = ty_args
        .into_iter()
        .map(|t| instantiate_base(context, t))
        .collect();

    assert!(ty_args.len() == tvars.len());
    let mut res = vec![];
    let subst = std::mem::replace(&mut context.subst, /* dummy value */ Subst::empty());
    context.subst = tvars
        .into_iter()
        .zip(ty_args)
        .fold(subst, |subst, (tvar, ty_arg)| {
            // tvar is just a type variable, so shouldn't throw ever...
            let (subst, t) = join_base_type(subst, &tvar, &ty_arg).ok().unwrap();
            res.push(t);
            subst
        });
    res
}

fn check_type_argument_arity<F: FnOnce() -> String>(
    context: &mut Context,
    loc: Loc,
    name_f: F,
    mut ty_args: Vec<BaseType>,
    tparam_constraints: &[Kind],
) -> Vec<BaseType> {
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
        ty_args.push(BaseType_::anything(loc));
    }

    ty_args
}

pub fn make_num_tvar(context: &mut Context, loc: Loc) -> Type {
    let tvar = context.subst.new_num_var(loc);
    Type_::base(sp(loc, BaseType_::Var(tvar)))
}

fn make_tparams(
    context: &mut Context,
    loc: Loc,
    tparam_constraints: Vec<(Loc, Kind)>,
) -> Vec<BaseType> {
    tparam_constraints
        .into_iter()
        .map(|(vloc, constraint)| {
            let tvar = sp(vloc, BaseType_::Var(TVar::next()));
            context.add_kind_constraint(loc, tvar.clone(), constraint);
            tvar
        })
        .collect()
}

//**************************************************************************************************
// Subtype and joining
//**************************************************************************************************

#[derive(Debug)]
pub enum TypingError {
    SubtypeError(Box<SingleType>, Box<SingleType>),
    Incompatible(Box<Type>, Box<Type>),
    RecursiveType(Loc),
}

pub fn subtype(mut subst: Subst, lhs: &Type, rhs: &Type) -> Result<(Subst, Type), TypingError> {
    use BaseType_::Anything;
    use SingleType_::Base;
    use Type_::*;
    match (lhs, rhs) {
        (sp!(loc, Unit), sp!(_, Unit)) => Ok((subst, sp(*loc, Unit))),

        (sp!(_, Single(sp!(_, Base(sp!(_, Anything))))), other)
        | (other, sp!(_, Single(sp!(_, Base(sp!(_, Anything)))))) => Ok((subst, other.clone())),

        (sp!(loc, Single(lhs_)), sp!(_, Single(rhs_))) => {
            let (subst, t) = subtype_single(subst, lhs_, rhs_)?;
            Ok((subst, sp(*loc, Single(t))))
        }
        (sp!(loc, Multiple(ltys)), sp!(_, Multiple(rtys))) if ltys.len() == rtys.len() => {
            let mut tys = vec![];
            for (lty, rty) in ltys.iter().zip(rtys) {
                let (nsubst, t) = subtype_single(subst, lty, rty)?;
                subst = nsubst;
                tys.push(t)
            }
            Ok((subst, sp(*loc, Multiple(tys))))
        }
        _ => Err(TypingError::Incompatible(
            Box::new(lhs.clone()),
            Box::new(rhs.clone()),
        )),
    }
}

pub fn subtype_single(
    subst: Subst,
    lhs: &SingleType,
    rhs: &SingleType,
) -> Result<(Subst, SingleType), TypingError> {
    use BaseType_::Anything;
    use SingleType_::*;
    use Type_::*;
    match (lhs, rhs) {
        (sp!(_, Base(sp!(_, Anything))), other) | (other, sp!(_, Base(sp!(_, Anything)))) => {
            Ok((subst, other.clone()))
        }

        (sp!(loc, Base(t1)), sp!(_, Base(t2))) => {
            let (subst, t) = join_base_type(subst, t1, t2)?;
            Ok((subst, sp(*loc, Base(t))))
        }
        (sp!(loc, Ref(mut1, t1)), sp!(_, Ref(mut2, t2))) => {
            let mut_ = match (mut1, mut2) {
                // imm <: imm
                // mut <: imm
                (false, false) | (true, false) => false,
                // mut <: mut
                (true, true) => true,
                // imm <\: mut
                (false, true) => {
                    return Err(TypingError::SubtypeError(
                        Box::new(lhs.clone()),
                        Box::new(rhs.clone()),
                    ))
                }
            };
            let (subst, t) = join_base_type(subst, t1, t2)?;
            Ok((subst, sp(*loc, Ref(mut_, t))))
        }
        _ => {
            let t1 = sp(lhs.loc, Single(lhs.clone()));
            let t2 = sp(rhs.loc, Single(rhs.clone()));
            Err(TypingError::Incompatible(Box::new(t1), Box::new(t2)))
        }
    }
}

pub fn join(mut subst: Subst, t1: &Type, t2: &Type) -> Result<(Subst, Type), TypingError> {
    use BaseType_::Anything;
    use SingleType_::Base;
    use Type_::*;
    match (t1, t2) {
        (sp!(loc, Unit), sp!(_, Unit)) => Ok((subst, sp(*loc, Unit))),

        (sp!(_, Single(sp!(_, Base(sp!(_, Anything))))), other)
        | (other, sp!(_, Single(sp!(_, Base(sp!(_, Anything)))))) => Ok((subst, other.clone())),

        (sp!(loc, Single(t1_)), sp!(_, Single(t2_))) => {
            let (subst, t) = join_single(subst, t1_, t2_)?;
            Ok((subst, sp(*loc, Single(t))))
        }
        (sp!(loc, Multiple(tys1)), sp!(_, Multiple(tys2))) if tys1.len() == tys2.len() => {
            let mut tys = vec![];
            for (ty1, ty2) in tys1.iter().zip(tys2) {
                let (nsubst, t) = join_single(subst, ty1, ty2)?;
                subst = nsubst;
                tys.push(t)
            }
            Ok((subst, sp(*loc, Multiple(tys))))
        }
        _ => Err(TypingError::Incompatible(
            Box::new(t1.clone()),
            Box::new(t2.clone()),
        )),
    }
}

pub fn join_single(
    subst: Subst,
    t1: &SingleType,
    t2: &SingleType,
) -> Result<(Subst, SingleType), TypingError> {
    use BaseType_::Anything;
    use SingleType_::*;
    match (t1, t2) {
        (sp!(_, Base(sp!(_, Anything))), other) | (other, sp!(_, Base(sp!(_, Anything)))) => {
            Ok((subst, other.clone()))
        }

        (sp!(loc, Base(b1)), sp!(_, Base(b2))) => {
            let (subst, b) = join_base_type(subst, b1, b2)?;
            Ok((subst, sp(*loc, Base(b))))
        }
        (sp!(loc, Ref(mut1, b1)), sp!(_, Ref(mut2, b2))) => {
            let mut_ = *mut1 && *mut2;
            let (subst, b) = join_base_type(subst, b1, b2)?;
            Ok((subst, sp(*loc, Ref(mut_, b))))
        }
        _ => Err(TypingError::Incompatible(
            Box::new(Type_::single(t1.clone())),
            Box::new(Type_::single(t2.clone())),
        )),
    }
}

pub fn join_base_type(
    mut subst: Subst,
    t1: &BaseType,
    t2: &BaseType,
) -> Result<(Subst, BaseType), TypingError> {
    use BaseType_::*;
    match (t1, t2) {
        (sp!(_, Anything), other) | (other, sp!(_, Anything)) => Ok((subst, other.clone())),

        (sp!(_, Param(TParam { id: id1, .. })), sp!(_, Param(TParam { id: id2, .. })))
            if id1 == id2 =>
        {
            Ok((subst, t1.clone()))
        }

        (sp!(loc, Apply(k1, n1, tys1)), sp!(_, Apply(k2, n2, tys2))) if n1 == n2 => {
            assert!(
                k1 == k2,
                "ICE failed naming: {:#?}kind != {:#?}kind. {:#?} !=  {:#?}",
                n1,
                n2,
                k1,
                k2
            );
            let (subst, tys) = join_base_types(subst, tys1, tys2)?;
            Ok((subst, sp(*loc, Apply(k1.clone(), n1.clone(), tys))))
        }
        (sp!(loc1, Var(id1)), sp!(loc2, Var(id2))) => {
            if *id1 == *id2 {
                Ok((subst, sp(*loc1, Var(*id1))))
            } else {
                join_tvar(subst, *loc1, *id1, *loc2, *id2)
            }
        }
        (sp!(loc, Var(id)), other) | (other, sp!(loc, Var(id))) => {
            if subst.get(*id).is_none() {
                match bind_tvar(&mut subst, *loc, *id, other.clone()) {
                    Err(()) => Err(TypingError::Incompatible(
                        Box::new(Type_::base(sp(*loc, Var(*id)))),
                        Box::new(Type_::base(other.clone())),
                    )),
                    Ok(()) => Ok((subst, sp(*loc, Var(*id)))),
                }
            } else {
                let new_tvar = TVar::next();
                subst.insert(new_tvar, other.clone());
                join_tvar(subst, *loc, *id, other.loc, new_tvar)
            }
        }
        _ => Err(TypingError::Incompatible(
            Box::new(Type_::base(t1.clone())),
            Box::new(Type_::base(t2.clone())),
        )),
    }
}

fn join_base_types(
    mut subst: Subst,
    tys1: &[BaseType],
    tys2: &[BaseType],
) -> Result<(Subst, Vec<BaseType>), TypingError> {
    // if tys1.len() != tys2.len(), we will get an error when instantiating the type elsewhere
    // as all types are instantiated as a sanity check
    let mut tys = vec![];
    for (ty1, ty2) in tys1.iter().zip(tys2) {
        let (nsubst, t) = join_base_type(subst, ty1, ty2)?;
        subst = nsubst;
        tys.push(t)
    }
    Ok((subst, tys))
}

fn join_tvar(
    mut subst: Subst,
    loc1: Loc,
    id1: TVar,
    loc2: Loc,
    id2: TVar,
) -> Result<(Subst, BaseType), TypingError> {
    use BaseType_::*;
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
        (Some(nloc), _) | (_, Some(nloc)) => {
            let nloc = *nloc;
            subst.set_num_var(id1, nloc);
            subst.set_num_var(id2, nloc);
            subst.set_num_var(new_tvar, nloc);
        }
        _ => (),
    }
    subst.insert(last_id1, sp(loc1, Var(new_tvar)));
    subst.insert(last_id2, sp(loc2, Var(new_tvar)));

    let (mut subst, new_ty) = join_base_type(subst, &ty1, &ty2)?;
    match subst.get(new_tvar) {
        Some(sp!(tloc, _)) => Err(TypingError::RecursiveType(*tloc)),
        None => match bind_tvar(&mut subst, loc1, new_tvar, new_ty) {
            Ok(()) => Ok((subst, sp(loc1, Var(new_tvar)))),
            Err(()) => {
                let ty1 = match ty1 {
                    sp!(loc, Anything) => sp(loc, Var(id1)),
                    t => t,
                };
                let ty2 = match ty2 {
                    sp!(loc, Anything) => sp(loc, Var(id2)),
                    t => t,
                };
                Err(TypingError::Incompatible(
                    Box::new(Type_::base(ty1)),
                    Box::new(Type_::base(ty2)),
                ))
            }
        },
    }
}

fn forward_tvar(subst: &Subst, id: TVar) -> TVar {
    match subst.get(id) {
        Some(sp!(_, BaseType_::Var(next))) => forward_tvar(subst, *next),
        Some(_) | None => id,
    }
}

fn bind_tvar(subst: &mut Subst, loc: Loc, tvar: TVar, ty: BaseType) -> Result<(), ()> {
    // check not necessary for soundness but improves error message structure
    if !check_num_tvar(&subst, loc, tvar, &ty) {
        return Err(());
    }
    match &ty.value {
        BaseType_::Anything => (),
        _ => subst.insert(tvar, ty),
    }
    Ok(())
}

fn check_num_tvar(subst: &Subst, loc: Loc, tvar: TVar, ty: &BaseType) -> bool {
    !subst.is_num_var(tvar) || check_num_tvar_(subst, loc, tvar, ty)
}

fn check_num_tvar_(subst: &Subst, loc: Loc, tvar: TVar, ty: &BaseType) -> bool {
    use BaseType_ as B;
    match &ty.value {
        B::Anything => true,
        B::Apply(_, sp!(_, TypeName_::Builtin(sp!(_, bt))), _) => bt.is_numeric(),

        B::Var(v) if subst.tvars.contains_key(v) => {
            check_num_tvar_(subst, loc, tvar, subst.get(*v).unwrap())
        }
        B::Var(v) => subst.is_num_var(*v),
        _ => false,
    }
}

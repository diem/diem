// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::shared::remembering_unique_map::RememberingUniqueMap;
use crate::shared::unique_map::UniqueMap;
use crate::{
    errors::*,
    expansion::ast::{self as E, Fields},
    parser::ast::{
        self as P, Field, FunctionName, FunctionVisibility, Kind, ModuleIdent, ModuleIdent_,
        ModuleName, StructName, Var,
    },
    shared::*,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

//**************************************************************************************************
// Context
//**************************************************************************************************

type AliasMap = RememberingUniqueMap<Name, ModuleIdent>;

struct Context {
    errors: Errors,
    address: Option<Address>,
    aliases: AliasMap,
}
impl Context {
    fn new() -> Self {
        Self {
            errors: vec![],
            address: None,
            aliases: AliasMap::new(),
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

    fn cur_address(&self) -> Address {
        self.address.unwrap()
    }

    fn restricted_self_error(&mut self, case: &str, name: &Name) {
        let msg = format!(
            "Invalid {case} name '{name}'. '{self_ident}' is restricted and cannot be used to name \
            a {case}",
            case=case,
            name=name,
            self_ident=ModuleName::SELF_NAME,
        );
        self.error(vec![(name.loc, msg)])
    }

    /// Adds the elements of `alias_map` to the current set of aliases in `self`, shadowing any
    /// existing module aliases
    pub fn set_and_shadow_aliases(&mut self, alias_map: AliasMap) {
        for (alias, ident) in alias_map {
            if self.aliases.contains_key(&alias) {
                self.aliases.remove(&alias);
            }
            self.aliases.add(alias, ident).unwrap();
        }
    }

    /// Resets the alias map and gives the set of aliases that were used
    pub fn clear_aliases(&mut self) -> BTreeSet<Name> {
        let old = std::mem::replace(&mut self.aliases, AliasMap::new());
        old.remember()
    }

    pub fn module_alias_get(&mut self, n: &Name) -> Option<&ModuleIdent> {
        self.aliases.get(n)
    }
}

//**************************************************************************************************
// Entry
//**************************************************************************************************

pub fn program(prog: P::Program, sender: Option<Address>) -> (E::Program, Errors) {
    let mut context = Context::new();
    let mut module_map = UniqueMap::new();
    let mut main_opt = None;
    // TODO figure out how to use lib definitions
    // Mark the functions as native?
    // Ignore any main?
    for def in prog.lib_definitions {
        match def {
            P::FileDefinition::Modules(m_or_a_vec) => {
                modules_and_addresses(&mut context, sender, false, &mut module_map, m_or_a_vec)
            }
            P::FileDefinition::Main(f) => context.error(vec![(
                f.function.name.loc(),
                format!(
                    "Invalid dependency. '{}' files cannot be depedencies",
                    FunctionName::MAIN_NAME
                ),
            )]),
        }
    }
    for def in prog.source_definitions {
        match def {
            P::FileDefinition::Modules(m_or_a_vec) => {
                modules_and_addresses(&mut context, sender, true, &mut module_map, m_or_a_vec)
            }
            P::FileDefinition::Main(f) => {
                context.address = None;
                let addr = match sender {
                    Some(addr) => addr,
                    None => {
                        let loc = f.function.name.loc();
                        let msg = format!(
                            "Invalid '{}'. No sender address was given as a command line \
                             argument. Add one using --{}",
                            FunctionName::MAIN_NAME,
                            crate::command_line::SENDER
                        );
                        context.error(vec![(loc, msg)]);
                        Address::LIBRA_CORE
                    }
                };
                main(&mut context, &mut main_opt, addr, f)
            }
        }
    }
    (
        E::Program {
            modules: module_map,
            main: main_opt,
        },
        context.get_errors(),
    )
}

fn modules_and_addresses(
    context: &mut Context,
    sender: Option<Address>,
    is_source_module: bool,
    module_map: &mut UniqueMap<ModuleIdent, E::ModuleDefinition>,
    m_or_a_vec: Vec<P::ModuleOrAddress>,
) {
    let mut addr_directive = None;
    for m_or_a in m_or_a_vec {
        match m_or_a {
            P::ModuleOrAddress::Address(loc, a) => addr_directive = Some((loc, a)),
            P::ModuleOrAddress::Module(module_def) => {
                let (mident, mod_) = module(
                    context,
                    is_source_module,
                    sender,
                    addr_directive,
                    module_def,
                );
                if let Err((old_loc, _)) = module_map.add(mident.clone(), mod_) {
                    context.error(vec![
                        (
                            mident.loc(),
                            format!("Duplicate definition for module '{}'", mident),
                        ),
                        (old_loc, "Previously defined here".into()),
                    ]);
                }
            }
        }
    }
}

fn address_directive(
    context: &mut Context,
    sender: Option<Address>,
    loc: Loc,
    addr_directive: Option<(Loc, Address)>,
) {
    let addr = match (addr_directive, sender) {
        (Some((_, addr)), _) => addr,
        (None, Some(addr)) => addr,
        (None, None) => {
            let msg = format!(
                "Invalid module declaration. No sender address was given as a command line \
                 argument. Add one using --{}. Or set the address at the top of the file using \
                 'address _:'",
                crate::command_line::SENDER
            );
            context.error(vec![(loc, msg)]);
            Address::LIBRA_CORE
        }
    };
    context.address = Some(addr);
}

fn module(
    context: &mut Context,
    is_source_module: bool,
    sender: Option<Address>,
    addr_directive: Option<(Loc, Address)>,
    mdef: P::ModuleDefinition,
) -> (ModuleIdent, E::ModuleDefinition) {
    assert!(context.aliases.is_empty());
    let P::ModuleDefinition {
        uses,
        name,
        structs: pstructs,
        functions: pfunctions,
    } = mdef;
    let name_loc = name.loc();
    if name.value() == ModuleName::SELF_NAME {
        context.restricted_self_error("module", &name.0);
    }

    address_directive(context, sender, name_loc, addr_directive);

    let mident_ = ModuleIdent_ {
        address: context.cur_address(),
        name: name.clone(),
    };
    let current_module = ModuleIdent(sp(name_loc, mident_));
    let self_aliases = module_self_aliases(&current_module);
    context.set_and_shadow_aliases(self_aliases);
    let alias_map = aliases(context, uses);
    context.set_and_shadow_aliases(alias_map.clone());
    let structs = structs(context, &name, pstructs);
    let functions = functions(context, &name, pfunctions);
    let used_aliases = context.clear_aliases();
    let (uses, unused_aliases) = check_aliases(context, used_aliases, alias_map);
    let def = E::ModuleDefinition {
        uses,
        unused_aliases,
        is_source_module,
        structs,
        functions,
    };
    (current_module, def)
}

fn main(
    context: &mut Context,
    main_opt: &mut Option<(Vec<ModuleIdent>, Address, FunctionName, E::Function)>,
    addr: Address,
    main_def: P::Main,
) {
    assert!(context.aliases.is_empty());
    let P::Main {
        uses,
        function: pfunction,
    } = main_def;
    let alias_map = aliases(context, uses);
    context.set_and_shadow_aliases(alias_map.clone());
    let (fname, function) = function_def(context, pfunction);
    if fname.value() != FunctionName::MAIN_NAME {
        let msg = format!(
            "Invalid top level function. Found a function named '{}', \
             but all top level functions must be named '{}'",
            fname.value(),
            FunctionName::MAIN_NAME
        );
        context.error(vec![(fname.loc(), msg)]);
    }
    match &function.visibility {
        FunctionVisibility::Internal => (),
        FunctionVisibility::Public(loc) => context.error(vec![(
            *loc,
            "Extraneous 'public' modifier. This top-level function is assumed to be public",
        )]),
    }
    match &function.body {
        sp!(_, E::FunctionBody_::Defined(_)) => (),
        sp!(loc, E::FunctionBody_::Native) => context.error(vec![(
            *loc,
            "Invalid 'native' function. This top-level function must have a defined body",
        )]),
    }
    let used_aliases = context.clear_aliases();
    let (_uses, unused_aliases) = check_aliases(context, used_aliases, alias_map);
    match main_opt {
        None => *main_opt = Some((unused_aliases, addr, fname, function)),
        Some((_, _, old_name, _)) => context.error(vec![
            (
                fname.loc(),
                format!("Duplicate definition of '{}'", FunctionName::MAIN_NAME),
            ),
            (old_name.loc(), "Previously defined here".into()),
        ]),
    }
}

//**************************************************************************************************
// Aliases
//**************************************************************************************************

fn module_self_aliases(current_module: &ModuleIdent) -> AliasMap {
    let self_name = sp(current_module.loc(), ModuleName::SELF_NAME.into());
    let aliases = vec![(self_name, current_module.clone())];
    AliasMap::maybe_from_iter(aliases.into_iter()).unwrap()
}

fn aliases(context: &mut Context, uses: Vec<(ModuleIdent, Option<ModuleName>)>) -> AliasMap {
    let mut alias_map = AliasMap::new();
    for (mident, alias_opt) in uses {
        let alias = alias_opt.unwrap_or_else(|| mident.0.value.name.clone());
        if alias.value() == ModuleName::SELF_NAME {
            context.restricted_self_error("module alias", &alias.0);
        }
        if let Err(old_loc) = alias_map.add(alias.0.clone(), mident) {
            context.error(vec![
                (
                    alias.loc(),
                    format!("Duplicate definition for alias '{}'", alias),
                ),
                (old_loc, "Previously defined here".into()),
            ])
        }
    }
    alias_map
}

fn check_aliases(
    context: &mut Context,
    used_aliases: BTreeSet<Name>,
    declared_aliases: AliasMap,
) -> (BTreeMap<ModuleIdent, Loc>, Vec<ModuleIdent>) {
    let mut uses = BTreeMap::new();
    let mut unused = Vec::new();
    for (alias, mident) in declared_aliases {
        if used_aliases.contains(&alias) {
            uses.insert(mident, alias.loc);
        } else {
            unused.push(mident);
            context.error(vec![(
                alias.loc,
                format!("Unused 'use' of alias '{}'. Consider removing it", alias),
            )])
        }
    }
    (uses, unused)
}

//**************************************************************************************************
// Structs
//**************************************************************************************************

fn structs(
    context: &mut Context,
    mname: &ModuleName,
    pstructs: Vec<P::StructDefinition>,
) -> UniqueMap<StructName, E::StructDefinition> {
    let mut struct_map = UniqueMap::new();

    for pstruct in pstructs {
        let (sname, sdef) = struct_def(context, pstruct);
        if let Err(old_loc) = struct_map.add(sname.clone(), sdef) {
            context.error(vec![
                (
                    sname.loc(),
                    format!(
                        "Duplicate definition for struct '{}' in module '{}'",
                        sname, mname
                    ),
                ),
                (old_loc, "Previously defined here".into()),
            ]);
        }
    }
    struct_map
}

fn struct_def(
    context: &mut Context,
    pstruct: P::StructDefinition,
) -> (StructName, E::StructDefinition) {
    let P::StructDefinition {
        name,
        resource_opt,
        type_parameters: pty_params,
        fields: pfields,
    } = pstruct;
    let type_parameters = type_parameters(context, pty_params);
    let fields = struct_fields(context, &name, pfields);
    let sdef = E::StructDefinition {
        resource_opt,
        type_parameters,
        fields,
    };
    (name, sdef)
}

fn struct_fields(
    context: &mut Context,
    sname: &StructName,
    pfields: P::StructFields,
) -> E::StructFields {
    let pfields_vec = match pfields {
        P::StructFields::Native(loc) => return E::StructFields::Native(loc),
        P::StructFields::Defined(v) => v,
    };
    let mut field_map = UniqueMap::new();
    for (idx, (field, pst)) in pfields_vec.into_iter().enumerate() {
        let st = single_type(context, pst);
        if let Err(old_loc) = field_map.add(field.clone(), (idx, st)) {
            context.error(vec![
                (
                    field.loc(),
                    format!(
                        "Duplicate definition for field '{}' in struct '{}'",
                        field, sname
                    ),
                ),
                (old_loc, "Previously defined here".into()),
            ]);
        }
    }
    E::StructFields::Defined(field_map)
}

//**************************************************************************************************
// Functiosn
//**************************************************************************************************

fn functions(
    context: &mut Context,
    mname: &ModuleName,
    pfunctions: Vec<P::Function>,
) -> UniqueMap<FunctionName, E::Function> {
    let mut fn_map = UniqueMap::new();
    for pfunction in pfunctions {
        let (fname, fdef) = function_def(context, pfunction);
        if let Err(old_loc) = fn_map.add(fname.clone(), fdef) {
            context.error(vec![
                (
                    fname.loc(),
                    format!(
                        "Duplicate definition for function '{}' in module '{}'",
                        fname, mname
                    ),
                ),
                (old_loc, "Previously defined here".into()),
            ]);
        }
    }
    fn_map
}

fn function_def(context: &mut Context, pfunction: P::Function) -> (FunctionName, E::Function) {
    let P::Function {
        name,
        visibility,
        signature: psignature,
        body: pbody,
        acquires,
    } = pfunction;
    let signature = function_signature(context, psignature);
    let acquires = acquires
        .into_iter()
        .flat_map(|a| module_access(context, a))
        .collect();
    let body = function_body(context, pbody);
    let fdef = E::Function {
        visibility,
        signature,
        acquires,
        body,
    };

    (name, fdef)
}

fn function_signature(
    context: &mut Context,
    psignature: P::FunctionSignature,
) -> E::FunctionSignature {
    let P::FunctionSignature {
        type_parameters: pty_params,
        parameters: pparams,
        return_type: pret_ty,
    } = psignature;
    let type_parameters = type_parameters(context, pty_params);
    let parameters = pparams
        .into_iter()
        .map(|(v, st)| (v, single_type(context, st)))
        .collect();
    let return_type = type_(context, pret_ty);
    E::FunctionSignature {
        type_parameters,
        parameters,
        return_type,
    }
}

fn function_body(context: &mut Context, sp!(loc, pbody_): P::FunctionBody) -> E::FunctionBody {
    use E::FunctionBody_ as EF;
    use P::FunctionBody_ as PF;
    let body_ = match pbody_ {
        PF::Native => EF::Native,
        PF::Defined(seq) => EF::Defined(sequence(context, loc, seq)),
    };
    sp(loc, body_)
}

//**************************************************************************************************
// Types
//**************************************************************************************************

fn type_parameters(_context: &mut Context, pty_params: Vec<(Name, Kind)>) -> Vec<(Name, Kind)> {
    // TODO aliasing will need to happen here at some point
    pty_params
}

fn type_(context: &mut Context, sp!(loc, pt_): P::Type) -> E::Type {
    use E::Type_ as ET;
    use P::Type_ as PT;
    let t_ = match pt_ {
        PT::Unit => ET::Unit,
        PT::Single(st) => ET::Single(single_type(context, st)),
        PT::Multiple(ss) => ET::Multiple(single_types(context, ss)),
    };
    sp(loc, t_)
}

fn single_types(context: &mut Context, pss: Vec<P::SingleType>) -> Vec<E::SingleType> {
    pss.into_iter()
        .map(|pst| single_type(context, pst))
        .collect()
}

fn single_type(context: &mut Context, pst: P::SingleType) -> E::SingleType {
    let loc = pst.loc;
    match single_type_(context, pst) {
        Some(st) => st,
        None => {
            assert!(context.has_errors());
            sp(loc, E::SingleType_::UnresolvedError)
        }
    }
}

fn single_type_(context: &mut Context, sp!(loc, pst_): P::SingleType) -> Option<E::SingleType> {
    use E::SingleType_ as ES;
    use P::SingleType_ as PS;
    let st_ = match pst_ {
        PS::Apply(pn, ptyargs) => {
            let n = module_access(context, pn)?;
            let tyargs = single_types(context, ptyargs);
            ES::Apply(n, tyargs)
        }
        PS::Ref(mut_, inner) => ES::Ref(mut_, Box::new(single_type(context, *inner))),
    };
    Some(sp(loc, st_))
}

fn module_access(
    context: &mut Context,
    sp!(loc, ptn_): P::ModuleAccess,
) -> Option<E::ModuleAccess> {
    use E::ModuleAccess_ as EN;
    use P::ModuleAccess_ as PN;
    let tn_ = match ptn_ {
        PN::Name(n) => match context.module_alias_get(&n) {
            Some(_) => {
                let msg = "Modules are not types. Try accessing a struct inside the module instead";
                context.error(vec![
                    (n.loc, format!("Unexpected module alias '{}'", n)),
                    (loc, msg.into()),
                ]);
                return None;
            }
            None => EN::Name(n),
        },
        PN::ModuleAccess(mname, n) => match context.module_alias_get(&mname.0).cloned() {
            None => {
                context.error(vec![(
                    mname.loc(),
                    format!("Unbound module alias '{}'", mname),
                )]);
                return None;
            }
            Some(mident) => EN::ModuleAccess(mident, n),
        },
        PN::QualifiedModuleAccess(mident, n) => EN::ModuleAccess(mident, n),
    };
    Some(sp(loc, tn_))
}

//**************************************************************************************************
// Expressions
//**************************************************************************************************

// TODO Support uses inside functions. AliasMap will become an accumulator

fn sequence(context: &mut Context, loc: Loc, seq: P::Sequence) -> E::Sequence {
    let (pitems, pfinal_item) = seq;
    let mut items: VecDeque<E::SequenceItem> = pitems
        .into_iter()
        .map(|item| sequence_item(context, item))
        .collect();
    let final_e_opt = pfinal_item.map(|item| exp_(context, item));
    let final_e = match final_e_opt {
        None => sp(loc, E::Exp_::Unit),
        Some(e) => e,
    };
    let final_item = sp(final_e.loc, E::SequenceItem_::Seq(final_e));
    items.push_back(final_item);
    items
}

fn sequence_item(context: &mut Context, sp!(loc, pitem_): P::SequenceItem) -> E::SequenceItem {
    use E::SequenceItem_ as ES;
    use P::SequenceItem_ as PS;
    let item_ = match pitem_ {
        PS::Seq(e) => ES::Seq(exp_(context, *e)),
        PS::Declare(pb, pty_opt) => {
            let b_opt = bind_list(context, pb);
            let ty_opt = pty_opt.map(|t| type_(context, t));
            match b_opt {
                None => ES::Seq(sp(loc, E::Exp_::UnresolvedError)),
                Some(b) => ES::Declare(b, ty_opt),
            }
        }
        PS::Bind(pb, pty_opt, pe) => {
            let b_opt = bind_list(context, pb);
            let ty_opt = pty_opt.map(|t| type_(context, t));
            let e_ = exp_(context, *pe);
            let e = match ty_opt {
                None => e_,
                Some(ty) => sp(e_.loc, E::Exp_::Annotate(Box::new(e_), ty)),
            };
            match b_opt {
                None => ES::Seq(sp(loc, E::Exp_::UnresolvedError)),
                Some(b) => ES::Bind(b, e),
            }
        }
    };
    sp(loc, item_)
}

fn exps(context: &mut Context, pes: Vec<P::Exp>) -> Vec<E::Exp> {
    pes.into_iter().map(|pe| exp_(context, pe)).collect()
}

fn exp(context: &mut Context, pe: P::Exp) -> Box<E::Exp> {
    Box::new(exp_(context, pe))
}

fn exp_(context: &mut Context, sp!(loc, pe_): P::Exp) -> E::Exp {
    use E::Exp_ as EE;
    use P::Exp_ as PE;
    let e_ = match pe_ {
        PE::Unit => EE::Unit,
        PE::Value(v) => EE::Value(v),
        PE::Move(v) => EE::Move(v),
        PE::Copy(v) => EE::Copy(v),
        PE::Name(n) => match context.module_alias_get(&n) {
            Some(_) => {
                let msg = format!("Unexpected module alias '{}'. Modules are not values.'", n);
                context.error(vec![(loc, msg)]);
                EE::UnresolvedError
            }
            None => EE::Name(n),
        },
        PE::GlobalCall(n, ptys_opt, sp!(rloc, prs)) => {
            let tys_opt = ptys_opt.map(|pss| single_types(context, pss));
            let ers = sp(rloc, exps(context, prs));
            EE::GlobalCall(n, tys_opt, ers)
        }
        PE::Call(pn, ptys_opt, sp!(rloc, prs)) => {
            let en_opt = module_access(context, pn);
            let tys_opt = ptys_opt.map(|pss| single_types(context, pss));
            let ers = sp(rloc, exps(context, prs));
            match en_opt {
                Some(en) => EE::Call(en, tys_opt, ers),
                None => {
                    assert!(context.has_errors());
                    EE::UnresolvedError
                }
            }
        }
        PE::Pack(pn, ptys_opt, pfields) => {
            let en_opt = module_access(context, pn);
            let tys_opt = ptys_opt.map(|pss| single_types(context, pss));
            let efields_vec = pfields
                .into_iter()
                .map(|(f, pe)| (f, exp_(context, pe)))
                .collect();
            let efields = fields(context, loc, "construction", "argument", efields_vec);
            match en_opt {
                Some(en) => EE::Pack(en, tys_opt, efields),
                None => {
                    assert!(context.has_errors());
                    EE::UnresolvedError
                }
            }
        }
        PE::IfElse(pb, pt, pf_opt) => {
            let eb = exp(context, *pb);
            let et = exp(context, *pt);
            let ef = match pf_opt {
                None => Box::new(sp(loc, EE::Unit)),
                Some(pf) => exp(context, *pf),
            };
            EE::IfElse(eb, et, ef)
        }
        PE::While(pb, ploop) => EE::While(exp(context, *pb), exp(context, *ploop)),
        PE::Loop(ploop) => EE::Loop(exp(context, *ploop)),
        PE::Block(seq) => EE::Block(sequence(context, loc, seq)),
        PE::ExpList(pes) => {
            assert!(pes.len() > 1);
            EE::ExpList(exps(context, pes))
        }

        PE::Assign(lvalue, rhs) => {
            let l_opt = lvalues(context, *lvalue);
            let er = exp(context, *rhs);
            match l_opt {
                None => {
                    assert!(context.has_errors());
                    EE::UnresolvedError
                }
                Some(LValue::Assigns(al)) => EE::Assign(al, er),
                Some(LValue::Mutate(el)) => EE::Mutate(el, er),
                Some(LValue::FieldMutate(edotted)) => EE::FieldMutate(edotted, er),
            }
        }
        PE::Return(pe) => EE::Return(exp(context, *pe)),
        PE::Abort(pe) => EE::Abort(exp(context, *pe)),
        PE::Break => EE::Break,
        PE::Continue => EE::Continue,
        PE::Dereference(pe) => EE::Dereference(exp(context, *pe)),
        PE::UnaryExp(op, pe) => EE::UnaryExp(op, exp(context, *pe)),
        PE::BinopExp(pl, op, pr) => EE::BinopExp(exp(context, *pl), op, exp(context, *pr)),
        PE::Borrow(mut_, pr) => EE::Borrow(mut_, exp(context, *pr)),
        pdotted_ @ PE::Dot(_, _) => match exp_dotted(context, sp(loc, pdotted_)) {
            Some(edotted) => EE::ExpDotted(Box::new(edotted)),
            None => {
                assert!(context.has_errors());
                EE::UnresolvedError
            }
        },
        PE::Annotate(e, ty) => EE::Annotate(exp(context, *e), type_(context, ty)),
        PE::UnresolvedError => panic!("ICE error should have been thrown"),
    };
    sp(loc, e_)
}

fn exp_dotted(context: &mut Context, sp!(loc, pdotted_): P::Exp) -> Option<E::ExpDotted> {
    use E::ExpDotted_ as EE;
    use P::Exp_ as PE;
    let edotted_ = match pdotted_ {
        PE::Dot(plhs, field) => {
            let lhs = exp_dotted(context, *plhs)?;
            EE::Dot(Box::new(lhs), field)
        }
        pe_ => EE::Exp(exp_(context, sp(loc, pe_))),
    };
    Some(sp(loc, edotted_))
}

//**************************************************************************************************
// Fields
//**************************************************************************************************

fn fields<T>(
    context: &mut Context,
    loc: Loc,
    case: &str,
    verb: &str,
    xs: Vec<(Field, T)>,
) -> Fields<T> {
    let mut fmap = UniqueMap::new();
    for (idx, (field, x)) in xs.into_iter().enumerate() {
        if let Err(old_loc) = fmap.add(field.clone(), (idx, x)) {
            context.error(vec![
                (loc, format!("Invalid {}", case)),
                (
                    field.loc(),
                    format!("Duplicate {} given for field '{}'", verb, field),
                ),
                (old_loc, "Previously defined here".into()),
            ])
        }
    }
    fmap
}

//**************************************************************************************************
// LValues
//**************************************************************************************************

fn bind_list(context: &mut Context, sp!(loc, pbs_): P::BindList) -> Option<E::BindList> {
    let bs_: Option<Vec<E::Bind>> = pbs_.into_iter().map(|pb| bind(context, pb)).collect();
    Some(sp(loc, bs_?))
}

fn bind(context: &mut Context, sp!(loc, pb_): P::Bind) -> Option<E::Bind> {
    use E::Bind_ as EB;
    use P::Bind_ as PB;
    let b_ = match pb_ {
        PB::Var(v) => EB::Var(v),
        PB::Unpack(ptn, ptys_opt, pfields) => {
            let tn = module_access(context, ptn)?;
            let tys_opt = ptys_opt.map(|pss| single_types(context, pss));
            let vfields: Option<Vec<(Field, E::Bind)>> = pfields
                .into_iter()
                .map(|(f, pb)| Some((f, bind(context, pb)?)))
                .collect();
            let fields = fields(context, loc, "deconstruction binding", "binding", vfields?);
            EB::Unpack(tn, tys_opt, fields)
        }
    };
    Some(sp(loc, b_))
}

enum LValue {
    Assigns(E::AssignList),
    FieldMutate(Box<E::ExpDotted>),
    Mutate(Box<E::Exp>),
}

fn lvalues(context: &mut Context, sp!(loc, e_): P::Exp) -> Option<LValue> {
    use LValue as L;
    use P::Exp_ as PE;
    let al: LValue = match e_ {
        PE::Unit => L::Assigns(sp(loc, vec![])),
        PE::ExpList(pes) => {
            let al_opt: Option<Vec<E::Assign>> =
                pes.into_iter().map(|pe| assign(context, pe)).collect();
            L::Assigns(sp(loc, al_opt?))
        }
        PE::Dereference(pr) => {
            let er = exp(context, *pr);
            L::Mutate(er)
        }
        pdotted_ @ PE::Dot(_, _) => {
            let dotted = exp_dotted(context, sp(loc, pdotted_))?;
            L::FieldMutate(Box::new(dotted))
        }
        _ => L::Assigns(sp(loc, vec![assign(context, sp(loc, e_))?])),
    };
    Some(al)
}

fn assign(context: &mut Context, sp!(loc, e_): P::Exp) -> Option<E::Assign> {
    use E::Assign_ as EA;
    use P::Exp_ as PE;
    let a_ = match e_ {
        PE::Name(n) => EA::Var(Var(n)),
        PE::Pack(pn, ptys_opt, pfields) => {
            let en = module_access(context, pn)?;
            let tys_opt = ptys_opt.map(|pss| single_types(context, pss));
            let efields = assign_unpack_fields(context, loc, pfields)?;
            EA::Unpack(en, tys_opt, efields)
        }
        _ => {
            context.error(vec![(
                loc,
                "Invalid assignment lvalue. Expected: a local, a field write, or a deconstructing \
                 assignment",
            )]);
            return None;
        }
    };
    Some(sp(loc, a_))
}

fn assign_unpack_fields(
    context: &mut Context,
    loc: Loc,
    pfields: Vec<(Field, P::Exp)>,
) -> Option<Fields<E::Assign>> {
    let afields = pfields
        .into_iter()
        .map(|(f, e)| Some((f, assign(context, e)?)))
        .collect::<Option<_>>()?;
    Some(fields(
        context,
        loc,
        "deconstructing assignment",
        "assignment binding",
        afields,
    ))
}

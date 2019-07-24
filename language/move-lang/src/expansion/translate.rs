// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

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
use std::collections::VecDeque;

//**************************************************************************************************
// Context
//**************************************************************************************************

type AliasMap = UniqueMap<Name, ModuleIdent>;

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
            aliases: UniqueMap::new(),
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

    pub fn set_aliases(&mut self, alias_map: AliasMap) {
        self.aliases = alias_map;
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
            P::FileDefinition::Modules(ad, ms) => {
                modules(&mut context, sender, false, &mut module_map, ad, ms)
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
            P::FileDefinition::Modules(ad, ms) => {
                modules(&mut context, sender, true, &mut module_map, ad, ms)
            }
            P::FileDefinition::Main(f) => {
                context.address = None;
                let addr = match sender {
                    Some(addr) => addr,
                    None => {
                        let loc = f.function.name.loc();
                        let msg = format!("Invalid '{}'. No sender address was given as a command line argument. Add one using --{}", FunctionName::MAIN_NAME, crate::command_line::SENDER);
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

fn modules(
    context: &mut Context,
    sender: Option<Address>,
    is_source_module: bool,
    module_map: &mut UniqueMap<ModuleIdent, E::ModuleDefinition>,
    addr_directive: P::AddressDirective,
    module_defs: Vec<P::ModuleDefinition>,
) {
    if module_defs.is_empty() {
        return;
    }

    let loc = module_defs.get(0).unwrap().name.loc();
    address_directive(context, sender, loc, addr_directive);

    for module_def in module_defs {
        let (mident, mod_) = module(context, is_source_module, context.cur_address(), module_def);
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

fn address_directive(
    context: &mut Context,
    sender: Option<Address>,
    loc: Loc,
    addr_directive: P::AddressDirective,
) {
    use P::AddressDirective as AD;
    let addr = match addr_directive {
        AD::Specified(_, addr) => addr,
        AD::Sender => match sender {
            Some(addr) => addr,
            None => {
                context.error(vec![
                    (loc, format!("Invalid module declration. No sender address was given as a command line argument. Add one using --{}. Or set the address at the top of the file using 'address _:'", crate::command_line::SENDER)),
                ]);
                Address::LIBRA_CORE
            }
        },
    };
    context.address = Some(addr);
}

fn module(
    context: &mut Context,
    is_source_module: bool,
    address: Address,
    mdef: P::ModuleDefinition,
) -> (ModuleIdent, E::ModuleDefinition) {
    let P::ModuleDefinition {
        mut uses,
        name,
        structs: pstructs,
        functions: pfunctions,
    } = mdef;
    let name_loc = name.loc();
    let mident_ = ModuleIdent_ {
        address,
        name: name.clone(),
    };
    let current_module = ModuleIdent(sp(name_loc, mident_));
    let self_name = ModuleName(sp(name_loc, ModuleName::SELF_NAME.into()));
    uses.insert(0, (current_module.clone(), Some(self_name)));
    let alias_map = aliases(context, uses);
    context.set_aliases(alias_map);
    let structs = structs(context, &name, pstructs);
    let functions = functions(context, &name, pfunctions);
    (
        current_module,
        E::ModuleDefinition {
            is_source_module,
            structs,
            functions,
        },
    )
}

fn main(
    context: &mut Context,
    main_opt: &mut Option<(Address, FunctionName, E::Function)>,
    addr: Address,
    main_def: P::Main,
) {
    let P::Main {
        uses,
        function: pfunction,
    } = main_def;
    let alias_map = aliases(context, uses);
    context.set_aliases(alias_map);
    let (fname, function) = function_def(context, pfunction);
    if fname.value() != FunctionName::MAIN_NAME {
        context.error(vec![
                    (fname.loc(), format!("Invalid top level function. Fund a function named '{}', but all top level functions must be named '{}'", fname.value(),FunctionName::MAIN_NAME))
                ]);
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
    match main_opt {
        None => *main_opt = Some((addr, fname, function)),
        Some((_, old_name, _)) => context.error(vec![
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

fn aliases(context: &mut Context, uses: Vec<(ModuleIdent, Option<ModuleName>)>) -> AliasMap {
    let mut alias_map = AliasMap::new();
    for (mident, alias_opt) in uses {
        let alias = alias_opt.unwrap_or_else(|| mident.0.value.name.clone());
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
                    sname.loc(),
                    format!(
                        "Duplicate definition for field '{}' in sttruct '{}'",
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
    let acquires = single_types(context, acquires);
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
            let n = type_name(context, pn)?;
            let tyargs = single_types(context, ptyargs);
            ES::Apply(n, tyargs)
        }
        PS::Ref(mut_, inner) => ES::Ref(mut_, Box::new(single_type(context, *inner))),
    };
    Some(sp(loc, st_))
}

fn type_name(context: &mut Context, sp!(loc, ptn_): P::TypeName) -> Option<E::TypeName> {
    use E::TypeName_ as EN;
    use P::TypeName_ as PN;
    let tn_ = match ptn_ {
        PN::Name(n) => {
            match context.module_alias_get(&n) {
                Some(_) => {
                    context.error(vec![
                        (n.loc, format!("Unexpected module alias '{}'", n)),
                        (loc, "Modules are not types. Try accessing a struct inside the module instead".into())
                    ]                  );
                    return None;
                }
                None => EN::Name(n),
            }
        }
        PN::ModuleType(mname, sn) => match context.module_alias_get(&mname.0).cloned() {
            None => {
                context.error(vec![(
                    mname.loc(),
                    format!("Unbound module alias '{}'", mname),
                )]);
                return None;
            }
            Some(mident) => EN::ModuleType(mident, sn),
        },
        PN::QualifiedModuleType(mident, sn) => EN::ModuleType(mident, sn),
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
    let final_item = sp(loc, E::SequenceItem_::Seq(final_e));
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
        PE::Name(n) => EE::Name(n),
        PE::MName(n) => match context.module_alias_get(&n) {
            Some(n) => EE::ModuleIdent(n.clone()),
            None => EE::MName(n),
        },
        PE::MNameTypeArgs(n, _) | PE::NameTypeArgs(n, _) => {
            context.error(vec![(
                loc,
                format!("Unexpected name with type arguments '{}'", &n),
            )]);
            EE::UnresolvedError
        }
        PE::GlobalApply(plhs, parg) => match call_or_pack(context, *plhs, *parg) {
            None => {
                assert!(context.has_errors());
                EE::UnresolvedError
            }
            Some(EE::Pack(_, _, _)) => {
                context.error(vec![(
                    loc,
                    "Unexpected construction. Found a global-identifier access, and expected function call",
                )]);
                EE::UnresolvedError
            }
            Some(EE::Call(e1, tys, e2)) => EE::GlobalCall(e1, tys, e2),
            Some(_) => panic!("ICE expected call or pack from call_or_pack"),
        },
        PE::Apply(plhs, parg) => match call_or_pack(context, *plhs, *parg) {
            None => {
                assert!(context.has_errors());
                EE::UnresolvedError
            }
            Some(e_) => e_,
        },
        PE::Fields(_) => {
            context.error(vec![(
                loc,
                "Unexpected field arguments. Fields are used only to instantiate a struct",
            )]);
            EE::UnresolvedError
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
            assert!(pes.len() >= 2);
            EE::ExpList(pes.into_iter().map(|pe| exp_(context, pe)).collect())
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
                Some(LValue::Mutate(edotted)) => EE::Mutate(edotted, er),
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
        pdotted_ @ PE::MDot(_, _, _) | pdotted_ @ PE::Dot(_, _, _) => {
            match exp_dotted(context, sp(loc, pdotted_)) {
                Some(edotted) => EE::ExpDotted(Box::new(edotted)),
                None => {
                    assert!(context.has_errors());
                    EE::UnresolvedError
                }
            }
        }
        PE::Annotate(e, ty) => EE::Annotate(exp(context, *e), type_(context, ty)),
        PE::UnresolvedError => panic!("ICE error should have been thrown"),
    };
    sp(loc, e_)
}

macro_rules! mdot {
    ($loc:pat, $lhs:pat, $field:pat, $tys_opt:pat) => {
        sp!($loc, P::Exp_::MDot($lhs, $field, $tys_opt))
    };
}

fn exp_dotted(context: &mut Context, pdotted: P::Exp) -> Option<E::ExpDotted> {
    let (edotted, tys_opt) = exp_dotted_rec(context, pdotted)?;
    if let Some(_tys) = tys_opt {
        context.error(vec![(
            edotted.loc,
            "Unexpected type arguments. Expected a function applied to its arguments",
        )]);
        None
    } else {
        Some(edotted)
    }
}

fn exp_dotted_rec(
    context: &mut Context,
    sp!(loc, pdotted_): P::Exp,
) -> Option<(E::ExpDotted, Option<Vec<E::SingleType>>)> {
    use E::ExpDotted_ as EE;
    use P::{Exp_ as PE, Value_ as V};
    let (edotted_, tys_opt) = match pdotted_ {
        PE::Dot(plhs, field, ptys_opt) => {
            let lhs = exp_dotted(context, *plhs)?;
            let tys_opt = ptys_opt.map(|tys| single_types(context, tys));
            (EE::Dot(Box::new(lhs), field), tys_opt)
        }
        PE::MDot(maybe_addr, m, ptys_opt) => {
            let addr = match *maybe_addr {
                sp!(_, P::Exp_::Value(sp!(_, V::Address(addr)))) => addr,
                _ => {
                    context.error(vec![
                        (loc, "Invalid dot access. Expected a field name".into()),
                        (m.loc, format!("Found a module or type identifier: '{}'", m)),
                    ]);
                    return None;
                }
            };
            let tys_opt = ptys_opt.map(|tys| single_types(context, tys));
            let mn = ModuleName(m);
            let mi_ = ModuleIdent_ {
                address: addr,
                name: mn,
            };
            let mi = ModuleIdent(sp(loc, mi_));
            (EE::Exp(sp(loc, E::Exp_::ModuleIdent(mi))), tys_opt)
        }
        pe_ => (EE::Exp(exp_(context, sp(loc, pe_))), None),
    };
    Some((sp(loc, edotted_), tys_opt))
}

fn exp_to_type_name(
    context: &mut Context,
    sp!(loc, p_): P::Exp,
) -> Option<(E::TypeName, Option<Vec<E::SingleType>>)> {
    use E::TypeName_ as ET;
    use P::Exp_ as PE;
    match p_ {
        PE::Name(n) | PE::MName(n) => Some((sp(loc, ET::Name(n)), None)),
        PE::MNameTypeArgs(n, ptys) | PE::NameTypeArgs(n, ptys) => {
            let tys = single_types(context, ptys);
            Some((sp(loc, ET::Name(n)), Some(tys)))
        }
        pdotted_ @ PE::MDot(_, _, _) | pdotted_ @ PE::Dot(_, _, _) => {
            exp_dotted_to_type_name(context, sp(loc, pdotted_))
        }
        _ => {
            context.error(vec![
                (loc, "Invalid type name. Expected: a name, a name with type arguments, or a module identifier and struct name"),
            ]);
            None
        }
    }
}

fn exp_dotted_to_type_name(
    context: &mut Context,
    pdotted: P::Exp,
) -> Option<(E::TypeName, Option<Vec<E::SingleType>>)> {
    use E::TypeName_ as ET;
    use P::{Exp_ as PE, Value_ as V};
    let (tn, ptys_opt) = match pdotted {
        mdot!(loc, inner, nf, ptys_opt) => match *inner {
            mdot!(mloc, maybe_addr, mf, None) => {
                let addr = match *maybe_addr {
                    sp!(_, PE::Value(sp!(_, V::Address(addr)))) => addr,
                    sp!(loc, _) => {
                        context.error(vec![(
                            loc,
                            "Invalid fully qualified module type name. Expected: an address",
                        )]);
                        return None;
                    }
                };
                let mn = ModuleName(mf);
                let mi_ = ModuleIdent_ {
                    address: addr,
                    name: mn,
                };
                let mi = ModuleIdent(sp(mloc, mi_));
                let n = StructName(nf);
                (sp(loc, ET::ModuleType(mi, n)), ptys_opt)
            }
            sp!(_, PE::MName(m)) => {
                let mn = ModuleName(m);
                let sn = StructName(nf);
                let tn_ = type_name(context, sp(loc, P::TypeName_::ModuleType(mn, sn)))?.value;
                (sp(loc, tn_), ptys_opt)
            }
            sp!(loc, _) => {
                context.error(vec![(
                    loc,
                    "Invalid module type name access. Expected: a struct name",
                )]);
                return None;
            }
        },
        sp!(loc, _) => {
            context.error(vec![(
                loc,
                "Invalid type name. Expected: a module identifier and struct name",
            )]);
            return None;
        }
    };
    let tys_opt = ptys_opt.map(|tys| single_types(context, tys));
    Some((tn, tys_opt))
}

//**************************************************************************************************
// Call
//**************************************************************************************************

fn is_mname(plhs: &P::Exp) -> bool {
    use P::Exp_ as PE;
    match &plhs.value {
        PE::MName(_) | PE::MNameTypeArgs(_, _) | PE::MDot(_, _, _) => true,
        _ => false,
    }
}

fn call_or_pack(context: &mut Context, plhs: P::Exp, prhs: P::Exp) -> Option<E::Exp_> {
    Some(if is_mname(&plhs) {
        let (tn, tys_opt) = exp_to_type_name(context, plhs)?;
        let fields = exp_fields(context, prhs)?;
        E::Exp_::Pack(tn, tys_opt, fields)
    } else {
        let (lhs, tys_opt) = call(context, plhs)?;
        let rhs = exp(context, prhs);
        E::Exp_::Call(Box::new(lhs), tys_opt, rhs)
    })
}

fn call(
    context: &mut Context,
    sp!(loc, pe_): P::Exp,
) -> Option<(E::Exp, Option<Vec<E::SingleType>>)> {
    use E::Exp_ as EE;
    use P::Exp_ as PE;
    Some(match pe_ {
        PE::NameTypeArgs(n, ptys) => {
            let e = sp(loc, EE::Name(n));
            let tys = single_types(context, ptys);
            (e, Some(tys))
        }
        pdotted_ @ PE::Dot(_, _, _) => {
            let (edotted, tys_opt) = exp_dotted_rec(context, sp(loc, pdotted_))?;
            assert!(match &edotted.value {
                E::ExpDotted_::Dot(_, _) => true,
                _ => false,
            });
            (sp(loc, EE::ExpDotted(Box::new(edotted))), tys_opt)
        }
        _ => (exp_(context, sp(loc, pe_)), None),
    })
}

fn exp_fields(context: &mut Context, sp!(loc, e_): P::Exp) -> Option<Fields<E::Exp>> {
    use E::Exp_ as EE;
    use P::Exp_ as PE;
    let msg = "Invalid fields for struct construction. Expected: named arguments";
    let fields = match e_ {
        PE::Block((v, last)) => {
            if !v.is_empty() {
                context.error(vec![(loc, msg)]);
                return None;
            }
            match *last {
                None => Fields::new(),
                Some(sp!(_, PE::Name(n))) => {
                    let f = Field(n.clone());
                    let fa = sp(f.loc(), EE::Name(n));
                    let mut fields = Fields::new();
                    fields.add(f, (0, fa)).unwrap();
                    fields
                }
                Some(_) => {
                    context.error(vec![(loc, msg)]);
                    return None;
                }
            }
        }
        PE::Fields(pfields) => {
            let efields = pfields
                .into_iter()
                .map(|(f, e)| (f, exp_(context, e)))
                .collect();
            fields(context, loc, "struct construction", "argument", efields)
        }
        _ => {
            context.error(vec![(loc, msg)]);
            return None;
        }
    };
    Some(fields)
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
            let tn = type_name(context, ptn)?;
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
    Mutate(Box<E::ExpDotted>),
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
            let er = exp_(context, *pr);
            L::Mutate(Box::new(sp(er.loc, E::ExpDotted_::Exp(er))))
        }
        pdotted_ @ PE::Dot(_, _, _) => {
            let dotted = exp_dotted(context, sp(loc, pdotted_))?;
            L::Mutate(Box::new(dotted))
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
        PE::Apply(lhs, rhs) => {
            let (tn, tys_opt) = exp_to_type_name(context, *lhs)?;
            let fields = assign_unpack_fields(context, *rhs)?;
            EA::Unpack(tn, tys_opt, fields)
        }
        _ => {
            context.error(vec![
                (loc, "Invalid assignment lvalue. Expected: a local, a field write, or a deconstructing assignment"),
            ]);
            return None;
        }
    };
    Some(sp(loc, a_))
}

fn assign_unpack_fields(context: &mut Context, sp!(loc, e_): P::Exp) -> Option<Fields<E::Assign>> {
    use E::Assign_ as EA;
    use P::Exp_ as PE;
    let msg = "Invalid fields for deconstructing assignment. Expected: named arguments";
    let fields = match e_ {
        PE::Block((v, last)) => {
            if !v.is_empty() {
                context.error(vec![(loc, msg)]);
                return None;
            }
            match *last {
                None => Fields::new(),
                Some(sp!(_, PE::Name(n))) => {
                    let f = Field(n.clone());
                    let fa = sp(f.loc(), EA::Var(Var(n)));
                    let mut fields = Fields::new();
                    fields.add(f, (0, fa)).unwrap();
                    fields
                }
                Some(_) => {
                    context.error(vec![(loc, msg)]);
                    return None;
                }
            }
        }
        PE::Fields(pfields) => {
            let afields = pfields
                .into_iter()
                .map(|(f, e)| Some((f, assign(context, e)?)))
                .collect::<Option<_>>()?;
            fields(
                context,
                loc,
                "deconstructing assignment",
                "assignment binding",
                afields,
            )
        }
        _ => {
            context.error(vec![(loc, msg)]);
            return None;
        }
    };
    Some(fields)
}

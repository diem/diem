// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    errors::*,
    expansion::{
        aliases::{AliasMap, AliasSet},
        ast::{self as E, Fields, SpecId},
    },
    parser::ast::{
        self as P, Field, FunctionName, FunctionVisibility, Kind, ModuleIdent, ModuleIdent_,
        ModuleName, StructName, Var,
    },
    shared::{unique_map::UniqueMap, *},
};
use move_ir_types::location::*;
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    iter::IntoIterator,
};

//**************************************************************************************************
// Context
//**************************************************************************************************

struct ModuleMembers {
    loc: Loc,
    structs: BTreeSet<Name>,
    functions: BTreeSet<Name>,
}

impl ModuleMembers {
    fn new(loc: Loc) -> Self {
        ModuleMembers {
            loc,
            structs: BTreeSet::new(),
            functions: BTreeSet::new(),
        }
    }

    fn is_member(&self, name: &Name) -> bool {
        self.is_struct(name) || self.is_function(name)
    }

    fn is_struct(&self, name: &Name) -> bool {
        self.structs.contains(name)
    }

    fn is_function(&self, name: &Name) -> bool {
        self.functions.contains(name)
    }
}

struct Context {
    module_members: BTreeMap<ModuleIdent, ModuleMembers>,
    errors: Errors,
    address: Option<Address>,
    aliases: AliasMap,
    is_source_module: bool,
    in_spec_context: bool,
    exp_specs: BTreeMap<SpecId, E::SpecBlock>,
}
impl Context {
    fn new(module_members: BTreeMap<ModuleIdent, ModuleMembers>) -> Self {
        Self {
            module_members,
            errors: vec![],
            address: None,
            aliases: AliasMap::new(),
            is_source_module: false,
            in_spec_context: false,
            exp_specs: BTreeMap::new(),
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

    /// Adds all of the new items in the new inner scope as shadowing the outer one.
    /// Gives back the outer scope
    pub fn new_alias_scope(&mut self, inner_scope: AliasMap) -> AliasMap {
        let outer_scope = self.aliases.clone();
        self.aliases.add_and_shadow_all(inner_scope);
        outer_scope
    }

    /// Resets the alias map and gives the set of aliases that were used
    pub fn set_to_outer_scope(&mut self, outer_scope: AliasMap) {
        let inner_scope = std::mem::replace(&mut self.aliases, outer_scope);
        let AliasSet { modules, members } = self.aliases.close_scope_and_report_unused(inner_scope);
        for alias in modules {
            unused_alias(self, alias)
        }
        for alias in members {
            unused_alias(self, alias)
        }
    }

    /// Produces an error if translation is not in specification context.
    pub fn require_spec_context(&mut self, loc: Loc, item: &str) -> bool {
        if self.in_spec_context {
            true
        } else {
            self.error(vec![(loc, item)]);
            false
        }
    }

    pub fn bind_exp_spec(&mut self, spec_block: P::SpecBlock) -> (SpecId, BTreeSet<Name>) {
        let len = self.exp_specs.len();
        let id = SpecId::new(len);
        let espec_block = spec(self, spec_block);
        let mut unbound_names = BTreeSet::new();
        unbound_names_spec_block(&mut unbound_names, &espec_block);
        self.exp_specs.insert(id, espec_block);

        (id, unbound_names)
    }

    pub fn extract_exp_specs(&mut self) -> BTreeMap<SpecId, E::SpecBlock> {
        std::mem::replace(&mut self.exp_specs, BTreeMap::new())
    }
}

//**************************************************************************************************
// Entry
//**************************************************************************************************

pub fn program(prog: P::Program, sender: Option<Address>) -> (E::Program, Errors) {
    let module_members = {
        let mut members = BTreeMap::new();
        all_module_members(&mut members, sender, &prog.lib_definitions);
        all_module_members(&mut members, sender, &prog.source_definitions);
        members
    };
    let mut context = Context::new(module_members);
    let mut module_map = UniqueMap::new();
    let mut scripts = vec![];

    context.is_source_module = false;
    for def in prog.lib_definitions {
        match def {
            P::Definition::Module(m) => module(&mut context, sender, &mut module_map, m),
            P::Definition::Address(_, addr, ms) => {
                for m in ms {
                    module(&mut context, Some(addr), &mut module_map, m)
                }
            }
            P::Definition::Script(_) => (),
        }
    }

    context.is_source_module = true;
    for def in prog.source_definitions {
        match def {
            P::Definition::Module(m) => module(&mut context, sender, &mut module_map, m),
            P::Definition::Address(_, addr, ms) => {
                for m in ms {
                    module(&mut context, Some(addr), &mut module_map, m)
                }
            }
            P::Definition::Script(s) => script(&mut context, &mut scripts, s),
        }
    }

    let scripts = {
        let mut collected: BTreeMap<String, Vec<E::Script>> = BTreeMap::new();
        for s in scripts {
            collected
                .entry(s.function_name.value().to_owned())
                .or_insert_with(Vec::new)
                .push(s)
        }
        let mut keyed: BTreeMap<String, E::Script> = BTreeMap::new();
        for (n, mut ss) in collected {
            match ss.len() {
                0 => unreachable!(),
                1 => assert!(
                    keyed.insert(n, ss.pop().unwrap()).is_none(),
                    "ICE duplicate script key"
                ),
                _ => {
                    for (i, s) in ss.into_iter().enumerate() {
                        let k = format!("{}_{}", n, i);
                        assert!(keyed.insert(k, s).is_none(), "ICE duplicate script key")
                    }
                }
            }
        }
        keyed
    };
    let prog = E::Program {
        modules: module_map,
        scripts,
    };
    (prog, context.get_errors())
}

fn module(
    context: &mut Context,
    address: Option<Address>,
    module_map: &mut UniqueMap<ModuleIdent, E::ModuleDefinition>,
    module_def: P::ModuleDefinition,
) {
    assert!(context.address == None);
    set_sender_address(context, module_def.loc, address);
    let (mident, mod_) = module_(context, module_def);
    if let Err((old_loc, _)) = module_map.add(mident.clone(), mod_) {
        let mmsg = format!("Duplicate definition for module '{}'", mident);
        context.error(vec![
            (mident.loc(), mmsg),
            (old_loc, "Previously defined here".into()),
        ]);
    }
    context.address = None
}

fn set_sender_address(context: &mut Context, loc: Loc, sender: Option<Address>) {
    context.address = Some(match sender {
        Some(addr) => addr,
        None => {
            let msg = format!(
                "Invalid module declaration. No sender address was given as a command line \
                 argument. Add one using --{}. Or set the address at the top of the file using \
                 'address _:'",
                crate::command_line::SENDER
            );
            context.error(vec![(loc, msg)]);
            Address::LIBRA_CORE
        }
    })
}

fn module_(context: &mut Context, mdef: P::ModuleDefinition) -> (ModuleIdent, E::ModuleDefinition) {
    let P::ModuleDefinition { loc, name, members } = mdef;
    let name_loc = name.loc();
    if name.value() == ModuleName::SELF_NAME {
        context.restricted_self_error("module", &name.0);
    }

    let mident_ = ModuleIdent_ {
        address: context.cur_address(),
        name,
    };
    let current_module = ModuleIdent(sp(name_loc, mident_));
    let old_is_source_module = context.is_source_module;
    context.is_source_module =
        context.is_source_module && !fake_natives::is_fake_native(&current_module);

    let mut new_scope = AliasMap::new();
    module_self_aliases(&mut new_scope, &current_module);
    let members = members
        .into_iter()
        .filter_map(|member| aliases_from_member(context, &mut new_scope, &current_module, member))
        .collect::<Vec<_>>();
    let old_aliases = context.new_alias_scope(new_scope);
    assert!(
        old_aliases.is_empty(),
        "ICE there should be no aliases entering a module"
    );

    let mut functions = UniqueMap::new();
    let mut structs = UniqueMap::new();
    let mut specs = vec![];
    for member in members {
        match member {
            P::ModuleMember::Use(_) => unreachable!(),
            P::ModuleMember::Function(mut f) => {
                if !context.is_source_module {
                    f.body.value = P::FunctionBody_::Native
                }
                function(context, &mut functions, f)
            }
            P::ModuleMember::Struct(mut s) => {
                if !context.is_source_module {
                    s.fields = P::StructFields::Native(s.loc)
                }
                struct_def(context, &mut structs, s)
            }
            P::ModuleMember::Spec(s) => specs.push(spec(context, s)),
        }
    }
    context.set_to_outer_scope(old_aliases);

    let def = E::ModuleDefinition {
        loc,
        is_source_module: context.is_source_module,
        structs,
        functions,
        specs,
    };
    context.is_source_module = old_is_source_module;
    (current_module, def)
}

fn script(context: &mut Context, scripts: &mut Vec<E::Script>, pscript: P::Script) {
    scripts.push(script_(context, pscript))
}

fn script_(context: &mut Context, pscript: P::Script) -> E::Script {
    assert!(context.address == None);
    assert!(context.is_source_module);
    let P::Script {
        loc,
        uses,
        function: pfunction,
        specs: pspecs,
    } = pscript;

    let mut new_scope = AliasMap::new();
    for u in uses {
        use_(context, &mut new_scope, u);
    }
    let old_aliases = context.new_alias_scope(new_scope);
    assert!(
        old_aliases.is_empty(),
        "ICE there should be no aliases entering a script"
    );

    let (function_name, function) = function_(context, pfunction);
    if let FunctionVisibility::Public(loc) = &function.visibility {
        let msg = "Extraneous 'public' modifier. Script functions are always public";
        context.error(vec![(*loc, msg)]);
    }
    match &function.body {
        sp!(_, E::FunctionBody_::Defined(_)) => (),
        sp!(loc, E::FunctionBody_::Native) => context.error(vec![(
            *loc,
            "Invalid 'native' function. This top-level function must have a defined body",
        )]),
    }
    let specs = specs(context, pspecs);
    context.set_to_outer_scope(old_aliases);

    E::Script {
        loc,
        function_name,
        function,
        specs,
    }
}

//**************************************************************************************************
// Aliases
//**************************************************************************************************

fn all_module_members(
    members: &mut BTreeMap<ModuleIdent, ModuleMembers>,
    sender: Option<Address>,
    defs: &[P::Definition],
) {
    for def in defs {
        match def {
            P::Definition::Module(m) => module_members(members, sender.unwrap_or_default(), m),
            P::Definition::Address(_, a, ms) => {
                for m in ms {
                    module_members(members, *a, m)
                }
            }
            P::Definition::Script(_) => (),
        }
    }
}

fn module_members(
    members: &mut BTreeMap<ModuleIdent, ModuleMembers>,
    address: Address,
    m: &P::ModuleDefinition,
) {
    let mident_ = ModuleIdent_ {
        address,
        name: m.name.clone(),
    };
    let mident = ModuleIdent(sp(m.name.loc(), mident_));
    let cur_members = members
        .entry(mident)
        .or_insert_with(|| ModuleMembers::new(m.name.loc()));
    for mem in &m.members {
        match mem {
            P::ModuleMember::Function(f) => cur_members.functions.insert(f.name.0.clone()),
            P::ModuleMember::Struct(s) => cur_members.structs.insert(s.name.0.clone()),
            P::ModuleMember::Spec(_) | P::ModuleMember::Use(_) => continue,
        };
    }
}

fn module_self_aliases(acc: &mut AliasMap, current_module: &ModuleIdent) {
    let self_name = sp(current_module.loc(), ModuleName::SELF_NAME.into());
    acc.add_implicit_module_alias(self_name, current_module.clone())
        .unwrap()
}

fn aliases_from_member(
    context: &mut Context,
    acc: &mut AliasMap,
    current_module: &ModuleIdent,
    member: P::ModuleMember,
) -> Option<P::ModuleMember> {
    match member {
        P::ModuleMember::Use(u) => {
            use_(context, acc, u);
            None
        }
        P::ModuleMember::Function(f) => {
            let n = f.name.0.clone();
            if let Err(loc) =
                acc.add_implicit_member_alias(n.clone(), current_module.clone(), n.clone())
            {
                duplicate_module_member(context, loc, n)
            }
            Some(P::ModuleMember::Function(f))
        }
        P::ModuleMember::Struct(s) => {
            let n = s.name.0.clone();
            if let Err(loc) =
                acc.add_implicit_member_alias(n.clone(), current_module.clone(), n.clone())
            {
                duplicate_module_member(context, loc, n)
            }
            Some(P::ModuleMember::Struct(s))
        }
        P::ModuleMember::Spec(s) => Some(P::ModuleMember::Spec(s)),
    }
}

fn use_(context: &mut Context, acc: &mut AliasMap, u: P::Use) {
    let unbound_module = |mident: &ModuleIdent| -> Error {
        vec![(
            mident.loc(),
            format!("Invalid 'use'. Unbound module: '{}'", mident),
        )]
    };
    macro_rules! add_module_alias {
        ($ident:expr, $alias_opt:expr) => {{
            let alias: Name = $alias_opt.unwrap_or_else(|| $ident.0.value.name.0.clone());
            if &alias.value == ModuleName::SELF_NAME {
                context.restricted_self_error("module alias", &alias);
                return;
            }

            if let Err(old_loc) = acc.add_module_alias(alias.clone(), $ident) {
                duplicate_module_alias(context, old_loc, alias)
            }
        }};
    };
    match u {
        P::Use::Module(mident, alias_opt) => {
            if !context.module_members.contains_key(&mident) {
                context.error(unbound_module(&mident));
                return;
            };
            add_module_alias!(mident, alias_opt.map(|m| m.0))
        }
        P::Use::Members(mident, sub_uses) => {
            if !context.module_members.contains_key(&mident) {
                context.error(unbound_module(&mident));
                return;
            };

            for (member, alias_opt) in sub_uses {
                if member.value == ModuleName::SELF_NAME {
                    add_module_alias!(mident.clone(), alias_opt);
                    continue;
                }

                // check is member
                let members = &context.module_members[&mident];
                let mloc = members.loc;
                let is_member = members.is_member(&member);
                let is_struct = members.is_struct(&member);
                if !is_member {
                    let msg = format!(
                        "Invalid 'use'. Unbound member '{}' in module '{}'",
                        member, mident
                    );
                    context.error(vec![
                        (member.loc, msg),
                        (mloc, format!("Module '{}' declared here", mident)),
                    ]);
                    continue;
                }

                // if struct member, check valid struct name
                match alias_opt.as_ref() {
                    // No explicit alias so nothing to check
                    None => (),
                    // Not a struct, nothing to check
                    Some(_) if !is_struct => (),
                    Some(alias) => {
                        if let Err(e) = check_valid_struct_or_constant_name(
                            alias,
                            "struct alias",
                            "Struct alias",
                        ) {
                            // TODO point to struct declaration
                            context.error(e);
                            continue;
                        }
                    }
                }

                let alias = alias_opt.unwrap_or_else(|| member.clone());
                if let Err(old_loc) = acc.add_member_alias(alias.clone(), mident.clone(), member) {
                    duplicate_module_member(context, old_loc, alias)
                }
            }
        }
    }
}

fn duplicate_module_alias(context: &mut Context, old_loc: Loc, alias: Name) {
    let msg = format!(
        "Duplicate module alias '{}'. Module aliases must be unique within a given module",
        alias
    );
    context.error(vec![
        (alias.loc, msg),
        (old_loc, "Previously defined here".into()),
    ])
}

fn duplicate_module_member(context: &mut Context, old_loc: Loc, alias: Name) {
    let msg = format!(
        "Duplicate module member or alias '{}'. Top level names in a module must be unique",
        alias
    );
    context.error(vec![
        (alias.loc, msg),
        (old_loc, "Previously defined here".into()),
    ])
}

fn unused_alias(context: &mut Context, alias: Name) {
    if !context.is_source_module {
        return;
    }

    context.error(vec![(
        alias.loc,
        format!("Unused 'use' of alias '{}'. Consider removing it", alias),
    )])
}

//**************************************************************************************************
// Structs
//**************************************************************************************************

fn struct_def(
    context: &mut Context,
    structs: &mut UniqueMap<StructName, E::StructDefinition>,
    pstruct: P::StructDefinition,
) {
    let (sname, sdef) = struct_def_(context, pstruct);
    if let Err(_old_loc) = structs.add(sname, sdef) {
        assert!(context.has_errors())
    }
}

fn struct_def_(
    context: &mut Context,
    pstruct: P::StructDefinition,
) -> (StructName, E::StructDefinition) {
    let P::StructDefinition {
        loc,
        name,
        resource_opt,
        type_parameters: pty_params,
        fields: pfields,
    } = pstruct;
    let old_aliases = context.new_alias_scope(AliasMap::new());
    let type_parameters = type_parameters(context, pty_params);
    let fields = struct_fields(context, &name, pfields);
    let sdef = E::StructDefinition {
        loc,
        resource_opt,
        type_parameters,
        fields,
    };
    check_valid_struct_name(context, &name);
    context.set_to_outer_scope(old_aliases);
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
    for (idx, (field, pt)) in pfields_vec.into_iter().enumerate() {
        let t = type_(context, pt);
        if let Err(old_loc) = field_map.add(field.clone(), (idx, t)) {
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
// Functions
//**************************************************************************************************

fn function(
    context: &mut Context,
    functions: &mut UniqueMap<FunctionName, E::Function>,
    pfunction: P::Function,
) {
    let (fname, fdef) = function_(context, pfunction);
    if let Err(_old_loc) = functions.add(fname, fdef) {
        assert!(context.has_errors())
    }
}

fn function_(context: &mut Context, pfunction: P::Function) -> (FunctionName, E::Function) {
    let P::Function {
        loc,
        name,
        visibility,
        signature: psignature,
        body: pbody,
        acquires,
    } = pfunction;
    assert!(context.exp_specs.is_empty());
    let old_aliases = context.new_alias_scope(AliasMap::new());
    let signature = function_signature(context, psignature);
    let acquires = acquires
        .into_iter()
        .flat_map(|a| module_access(context, Access::Type, a))
        .collect();
    let body = function_body(context, pbody);
    let specs = context.extract_exp_specs();
    let fdef = E::Function {
        loc,
        visibility,
        signature,
        acquires,
        body,
        specs,
    };
    context.set_to_outer_scope(old_aliases);

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
        .map(|(v, t)| (v, type_(context, t)))
        .collect::<Vec<_>>();
    for (v, _) in &parameters {
        check_valid_local_name(context, v)
    }
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
// Specification Blocks
//**************************************************************************************************

fn specs(context: &mut Context, pspecs: Vec<P::SpecBlock>) -> Vec<E::SpecBlock> {
    pspecs.into_iter().map(|s| spec(context, s)).collect()
}

fn spec(context: &mut Context, sp!(loc, pspec): P::SpecBlock) -> E::SpecBlock {
    let P::SpecBlock_ {
        target,
        uses: _uses,
        members: pmembers,
    } = pspec;

    context.in_spec_context = true;
    let new_scope = AliasMap::new();
    // TODO sort out spec aliases
    // for u in uses {
    //     use_(context, &mut new_scope, u);
    // }
    let old_aliases = context.new_alias_scope(new_scope);

    let members = pmembers
        .into_iter()
        .filter_map(|m| {
            let m = spec_member(context, m);
            if m.is_none() {
                assert!(context.has_errors())
            };
            m
        })
        .collect();

    context.set_to_outer_scope(old_aliases);
    context.in_spec_context = false;

    sp(loc, E::SpecBlock_ { target, members })
}

fn spec_member(
    context: &mut Context,
    sp!(loc, pm): P::SpecBlockMember,
) -> Option<E::SpecBlockMember> {
    use E::SpecBlockMember_ as EM;
    use P::SpecBlockMember_ as PM;
    let em = match pm {
        PM::Condition { kind, exp } => {
            let exp = exp_(context, exp);
            EM::Condition { kind, exp }
        }
        PM::Function {
            name,
            signature,
            body,
        } => {
            let body = function_body(context, body);
            let signature = function_signature(context, signature);
            EM::Function {
                name,
                signature,
                body,
            }
        }
        PM::Variable {
            is_global,
            name,
            type_parameters: pty_params,
            type_: t,
        } => {
            let old_aliases = context.new_alias_scope(AliasMap::new());
            let type_parameters = type_parameters(context, pty_params);
            let t = type_(context, t);
            context.set_to_outer_scope(old_aliases);
            EM::Variable {
                is_global,
                name,
                type_parameters,
                type_: t,
            }
        }
        PM::Include { exp: pexp } => EM::Include {
            exp: exp_(context, pexp),
        },
        PM::Apply {
            exp: pexp,
            patterns,
            exclusion_patterns,
        } => EM::Apply {
            exp: exp_(context, pexp),
            patterns,
            exclusion_patterns,
        },
        PM::Pragma { properties } => EM::Pragma { properties },
    };
    Some(sp(loc, em))
}

//**************************************************************************************************
// Types
//**************************************************************************************************

fn type_parameters(context: &mut Context, pty_params: Vec<(Name, Kind)>) -> Vec<(Name, Kind)> {
    assert!(
        context.aliases.current_scope_is_empty(),
        "ICE alias scope should be cleared before handling type parameters"
    );
    for (name, _) in &pty_params {
        context.aliases.remove_member_alias(name)
    }
    pty_params
}

fn type_(context: &mut Context, sp!(loc, pt_): P::Type) -> E::Type {
    use E::Type_ as ET;
    use P::Type_ as PT;
    let t_ = match pt_ {
        PT::Unit => ET::Unit,
        PT::Multiple(ts) => ET::Multiple(types(context, ts)),
        PT::Apply(pn, ptyargs) => {
            let tyargs = types(context, ptyargs);
            match module_access(context, Access::Type, *pn) {
                None => {
                    assert!(context.has_errors());
                    ET::UnresolvedError
                }
                Some(n) => ET::Apply(n, tyargs),
            }
        }
        PT::Ref(mut_, inner) => ET::Ref(mut_, Box::new(type_(context, *inner))),
        PT::Fun(args, result) => {
            if context
                .require_spec_context(loc, "`|_|_` function type only allowed in specifications")
            {
                let args = types(context, args);
                let result = type_(context, *result);
                ET::Fun(args, Box::new(result))
            } else {
                assert!(context.has_errors());
                ET::UnresolvedError
            }
        }
    };
    sp(loc, t_)
}

fn types(context: &mut Context, pts: Vec<P::Type>) -> Vec<E::Type> {
    pts.into_iter().map(|pt| type_(context, pt)).collect()
}

fn optional_types(context: &mut Context, pts_opt: Option<Vec<P::Type>>) -> Option<Vec<E::Type>> {
    pts_opt.map(|pts| pts.into_iter().map(|pt| type_(context, pt)).collect())
}

#[derive(Clone, Copy)]
enum Access {
    Type,
    ApplyNamed,
    ApplyPositional,
    Term,
}

fn module_access(
    context: &mut Context,
    access: Access,
    sp!(loc, ptn_): P::ModuleAccess,
) -> Option<E::ModuleAccess> {
    use E::ModuleAccess_ as EN;
    use P::ModuleAccess_ as PN;
    let tn_ = match (access, ptn_) {
        (Access::ApplyPositional, PN::Name(n)) if context.in_spec_context => {
            // TODO sort out function aliases in specs
            EN::Name(n)
        }
        (Access::ApplyPositional, PN::Name(n))
        | (Access::ApplyNamed, PN::Name(n))
        | (Access::Type, PN::Name(n)) => match context.aliases.member_alias_get(&n) {
            Some((mident, mem)) => EN::ModuleAccess(mident.clone(), mem.clone()),
            None => EN::Name(n),
        },
        (Access::Term, PN::Name(n)) => {
            // TODO Constant aliases will go here
            EN::Name(n)
        }
        (_, PN::Global(n)) => {
            let msg = format!(
                "Invalid use of global name '::{}'. \
                Global names can only be used in function calls",
                n
            );
            context.error(vec![(loc, msg)]);
            return None;
        }
        (_, PN::ModuleAccess(mname, n)) => {
            match context.aliases.module_alias_get(&mname.0).cloned() {
                None => {
                    context.error(vec![(
                        mname.loc(),
                        format!("Unbound module alias '{}'", mname),
                    )]);
                    return None;
                }
                Some(mident) => EN::ModuleAccess(mident, n),
            }
        }
        (_, PN::QualifiedModuleAccess(mident, n)) => EN::ModuleAccess(mident, n),
    };
    Some(sp(loc, tn_))
}

//**************************************************************************************************
// Expressions
//**************************************************************************************************

// TODO Support uses inside functions. AliasMap will become an accumulator

fn sequence(context: &mut Context, loc: Loc, seq: P::Sequence) -> E::Sequence {
    let (pitems, maybe_last_semicolon_loc, pfinal_item) = seq;
    let mut items: VecDeque<E::SequenceItem> = pitems
        .into_iter()
        .map(|item| sequence_item(context, item))
        .collect();
    let final_e_opt = pfinal_item.map(|item| exp_(context, item));
    let final_e = match final_e_opt {
        None => {
            let last_semicolon_loc = match maybe_last_semicolon_loc {
                Some(l) => l,
                None => loc,
            };
            sp(last_semicolon_loc, E::Exp_::Unit)
        }
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
                None => {
                    assert!(context.has_errors());
                    ES::Seq(sp(loc, E::Exp_::UnresolvedError))
                }
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
                None => {
                    assert!(context.has_errors());
                    ES::Seq(sp(loc, E::Exp_::UnresolvedError))
                }
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
        PE::InferredNum(u) => EE::InferredNum(u),
        PE::Move(v) => EE::Move(v),
        PE::Copy(v) => EE::Copy(v),
        PE::Name(sp!(_, P::ModuleAccess_::ModuleAccess(..)), _)
        | PE::Name(sp!(_, P::ModuleAccess_::QualifiedModuleAccess(..)), _)
        | PE::Name(_, Some(_))
            if !context.require_spec_context(
                loc,
                "Expected name to be followed by a brace-enclosed list of field expressions or a \
              parenthesized list of arguments for a function call",
            ) =>
        {
            assert!(context.has_errors());
            EE::UnresolvedError
        }
        PE::Name(pn, ptys_opt) => {
            let en_opt = module_access(context, Access::Term, pn);
            let tys_opt = optional_types(context, ptys_opt);
            match en_opt {
                Some(en) => EE::Name(en, tys_opt),
                None => {
                    assert!(context.has_errors());
                    EE::UnresolvedError
                }
            }
        }
        PE::Call(pn, ptys_opt, sp!(rloc, prs)) => {
            let tys_opt = optional_types(context, ptys_opt);
            let ers = sp(rloc, exps(context, prs));
            match pn {
                sp!(_, P::ModuleAccess_::Global(n)) => EE::GlobalCall(n, tys_opt, ers),
                _ => {
                    let en_opt = module_access(context, Access::ApplyPositional, pn);
                    match en_opt {
                        Some(en) => EE::Call(en, tys_opt, ers),
                        None => {
                            assert!(context.has_errors());
                            EE::UnresolvedError
                        }
                    }
                }
            }
        }
        PE::Pack(pn, ptys_opt, pfields) => {
            let en_opt = module_access(context, Access::ApplyNamed, pn);
            let tys_opt = optional_types(context, ptys_opt);
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
        PE::Lambda(pbs, pe) => {
            if !context.require_spec_context(
                loc,
                "`|_| _` lambda expression only allowed in specifications",
            ) {
                assert!(context.has_errors());
                EE::UnresolvedError
            } else {
                let bs_opt = bind_list(context, pbs);
                let e = exp_(context, *pe);
                match bs_opt {
                    Some(bs) => EE::Lambda(bs, Box::new(e)),
                    None => {
                        assert!(context.has_errors());
                        EE::UnresolvedError
                    }
                }
            }
        }
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
        PE::Return(pe_opt) => {
            let ev = match pe_opt {
                None => Box::new(sp(loc, EE::Unit)),
                Some(pe) => exp(context, *pe),
            };
            EE::Return(ev)
        }
        PE::Abort(pe) => EE::Abort(exp(context, *pe)),
        PE::Break => EE::Break,
        PE::Continue => EE::Continue,
        PE::Dereference(pe) => EE::Dereference(exp(context, *pe)),
        PE::UnaryExp(op, pe) => EE::UnaryExp(op, exp(context, *pe)),
        PE::BinopExp(pl, op, pr) => {
            if op.value.is_spec_only()
                && !context.require_spec_context(
                    loc,
                    &format!(
                        "`{}` operator only allowed in specifications",
                        op.value.symbol()
                    ),
                )
            {
                assert!(context.has_errors());
                EE::UnresolvedError
            } else {
                EE::BinopExp(exp(context, *pl), op, exp(context, *pr))
            }
        }
        PE::Borrow(mut_, pr) => EE::Borrow(mut_, exp(context, *pr)),
        pdotted_ @ PE::Dot(_, _) => match exp_dotted(context, sp(loc, pdotted_)) {
            Some(edotted) => EE::ExpDotted(Box::new(edotted)),
            None => {
                assert!(context.has_errors());
                EE::UnresolvedError
            }
        },
        PE::Cast(e, ty) => EE::Cast(exp(context, *e), type_(context, ty)),
        PE::Index(e, i) => {
            if context
                .require_spec_context(loc, "`_[_]` index operator only allowed in specifications")
            {
                EE::Index(exp(context, *e), exp(context, *i))
            } else {
                EE::UnresolvedError
            }
        }
        PE::Annotate(e, ty) => EE::Annotate(exp(context, *e), type_(context, ty)),
        PE::Spec(spec_block) => {
            let (spec_id, unbound_names) = context.bind_exp_spec(spec_block);
            EE::Spec(spec_id, unbound_names)
        }
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

fn bind_list(context: &mut Context, sp!(loc, pbs_): P::BindList) -> Option<E::LValueList> {
    let bs_: Option<Vec<E::LValue>> = pbs_.into_iter().map(|pb| bind(context, pb)).collect();
    Some(sp(loc, bs_?))
}

fn bind(context: &mut Context, sp!(loc, pb_): P::Bind) -> Option<E::LValue> {
    use E::LValue_ as EL;
    use P::Bind_ as PB;
    let b_ = match pb_ {
        PB::Var(v) => {
            check_valid_local_name(context, &v);
            EL::Var(sp(loc, E::ModuleAccess_::Name(v.0)), None)
        }
        PB::Unpack(ptn, ptys_opt, pfields) => {
            let tn = module_access(context, Access::ApplyNamed, ptn)?;
            let tys_opt = optional_types(context, ptys_opt);
            let vfields: Option<Vec<(Field, E::LValue)>> = pfields
                .into_iter()
                .map(|(f, pb)| Some((f, bind(context, pb)?)))
                .collect();
            let fields = fields(context, loc, "deconstruction binding", "binding", vfields?);
            EL::Unpack(tn, tys_opt, fields)
        }
    };
    Some(sp(loc, b_))
}

enum LValue {
    Assigns(E::LValueList),
    FieldMutate(Box<E::ExpDotted>),
    Mutate(Box<E::Exp>),
}

fn lvalues(context: &mut Context, sp!(loc, e_): P::Exp) -> Option<LValue> {
    use LValue as L;
    use P::Exp_ as PE;
    let al: LValue = match e_ {
        PE::Unit => L::Assigns(sp(loc, vec![])),
        PE::ExpList(pes) => {
            let al_opt: Option<E::LValueList_> =
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

fn assign(context: &mut Context, sp!(loc, e_): P::Exp) -> Option<E::LValue> {
    use E::LValue_ as EL;
    use P::Exp_ as PE;
    let a_ = match e_ {
        PE::Name(sp!(_, P::ModuleAccess_::ModuleAccess(..)), _)
        | PE::Name(sp!(_, P::ModuleAccess_::QualifiedModuleAccess(..)), _)
        | PE::Name(_, Some(_))
            if !context.require_spec_context(
                loc,
                "only simple names allowed in assignment outside of specifications",
            ) =>
        {
            assert!(context.has_errors());
            return None;
        }
        PE::Name(pn, ptys_opt) => {
            let en = module_access(context, Access::Term, pn)?;
            let tys_opt = optional_types(context, ptys_opt);
            EL::Var(en, tys_opt)
        }
        PE::Pack(pn, ptys_opt, pfields) => {
            let en = module_access(context, Access::ApplyNamed, pn)?;
            let tys_opt = optional_types(context, ptys_opt);
            let efields = assign_unpack_fields(context, loc, pfields)?;
            EL::Unpack(en, tys_opt, efields)
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
) -> Option<Fields<E::LValue>> {
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

//**************************************************************************************************
// Unbound names
//**************************************************************************************************

fn unbound_names_spec_block(unbound: &mut BTreeSet<Name>, sp!(_, sb_): &E::SpecBlock) {
    sb_.members
        .iter()
        .for_each(|member| unbound_names_spec_block_member(unbound, member))
}

fn unbound_names_spec_block_member(unbound: &mut BTreeSet<Name>, sp!(_, m_): &E::SpecBlockMember) {
    use E::SpecBlockMember_ as M;
    match m_ {
        M::Condition { exp, .. } => unbound_names_exp(unbound, exp),
        // No unbound names
        // And will error in the move prover
        M::Function { .. }
        | M::Variable { .. }
        | M::Include { .. }
        | M::Apply { .. }
        | M::Pragma { .. } => (),
    }
}

fn unbound_names_exp(unbound: &mut BTreeSet<Name>, sp!(_, e_): &E::Exp) {
    use E::Exp_ as EE;
    match e_ {
        EE::Value(_)
        | EE::InferredNum(_)
        | EE::Break
        | EE::Continue
        | EE::UnresolvedError
        | EE::Name(sp!(_, E::ModuleAccess_::ModuleAccess(..)), _)
        | EE::Unit => (),
        EE::Copy(v) | EE::Move(v) => {
            unbound.insert(v.0.clone());
        }
        EE::Name(sp!(_, E::ModuleAccess_::Name(n)), _) => {
            unbound.insert(n.clone());
        }
        EE::GlobalCall(_, _, sp!(_, es_)) | EE::Call(_, _, sp!(_, es_)) => {
            unbound_names_exps(unbound, es_)
        }
        EE::Pack(_, _, es) => unbound_names_exps(unbound, es.iter().map(|(_, (_, e))| e)),
        EE::IfElse(econd, et, ef) => {
            unbound_names_exp(unbound, ef);
            unbound_names_exp(unbound, et);
            unbound_names_exp(unbound, econd)
        }
        EE::While(econd, eloop) => {
            unbound_names_exp(unbound, eloop);
            unbound_names_exp(unbound, econd)
        }
        EE::Loop(eloop) => unbound_names_exp(unbound, eloop),

        EE::Block(seq) => unbound_names_sequence(unbound, seq),
        EE::Lambda(ls, er) => {
            unbound_names_exp(unbound, er);
            // remove anything in `ls`
            unbound_names_binds(unbound, ls);
        }
        EE::Assign(ls, er) => {
            unbound_names_exp(unbound, er);
            // remove anything in `ls`
            unbound_names_assigns(unbound, ls);
        }
        EE::Return(e)
        | EE::Abort(e)
        | EE::Dereference(e)
        | EE::UnaryExp(_, e)
        | EE::Borrow(_, e)
        | EE::Cast(e, _)
        | EE::Annotate(e, _) => unbound_names_exp(unbound, e),
        EE::FieldMutate(ed, er) => {
            unbound_names_exp(unbound, er);
            unbound_names_dotted(unbound, ed)
        }
        EE::Mutate(el, er) | EE::BinopExp(el, _, er) => {
            unbound_names_exp(unbound, er);
            unbound_names_exp(unbound, el)
        }
        EE::ExpList(es) => unbound_names_exps(unbound, es),
        EE::ExpDotted(ed) => unbound_names_dotted(unbound, ed),
        EE::Index(el, ei) => {
            unbound_names_exp(unbound, ei);
            unbound_names_exp(unbound, el)
        }

        EE::Spec(_, unbound_names) => unbound.extend(unbound_names.iter().cloned()),
    }
}

fn unbound_names_exps<'a>(unbound: &mut BTreeSet<Name>, es: impl IntoIterator<Item = &'a E::Exp>) {
    es.into_iter().for_each(|e| unbound_names_exp(unbound, e))
}

fn unbound_names_sequence(unbound: &mut BTreeSet<Name>, seq: &E::Sequence) {
    seq.iter()
        .rev()
        .for_each(|s| unbound_names_sequence_item(unbound, s))
}

fn unbound_names_sequence_item(unbound: &mut BTreeSet<Name>, sp!(_, es_): &E::SequenceItem) {
    use E::SequenceItem_ as ES;
    match es_ {
        ES::Seq(e) => unbound_names_exp(unbound, e),
        ES::Declare(ls, _) => unbound_names_binds(unbound, ls),
        ES::Bind(ls, er) => {
            unbound_names_exp(unbound, er);
            // remove anything in `ls`
            unbound_names_binds(unbound, ls);
        }
    }
}

fn unbound_names_binds(unbound: &mut BTreeSet<Name>, sp!(_, ls_): &E::LValueList) {
    ls_.iter()
        .rev()
        .for_each(|l| unbound_names_bind(unbound, l))
}

fn unbound_names_bind(unbound: &mut BTreeSet<Name>, sp!(_, l_): &E::LValue) {
    use E::LValue_ as EL;
    match l_ {
        EL::Var(sp!(_, E::ModuleAccess_::Name(n)), _) => {
            unbound.remove(&n);
        }
        EL::Var(sp!(_, E::ModuleAccess_::ModuleAccess(..)), _) => {
            // Qualified vars are not considered in unbound set.
        }
        EL::Unpack(_, _, efields) => efields
            .iter()
            .for_each(|(_, (_, l))| unbound_names_bind(unbound, l)),
    }
}

fn unbound_names_assigns(unbound: &mut BTreeSet<Name>, sp!(_, ls_): &E::LValueList) {
    ls_.iter()
        .rev()
        .for_each(|l| unbound_names_assign(unbound, l))
}

fn unbound_names_assign(unbound: &mut BTreeSet<Name>, sp!(_, l_): &E::LValue) {
    use E::LValue_ as EL;
    match l_ {
        EL::Var(sp!(_, E::ModuleAccess_::Name(n)), _) => {
            unbound.insert(n.clone());
        }
        EL::Var(sp!(_, E::ModuleAccess_::ModuleAccess(..)), _) => {
            // Qualified vars are not considered in unbound set.
        }
        EL::Unpack(_, _, efields) => efields
            .iter()
            .for_each(|(_, (_, l))| unbound_names_assign(unbound, l)),
    }
}

fn unbound_names_dotted(unbound: &mut BTreeSet<Name>, sp!(_, edot_): &E::ExpDotted) {
    use E::ExpDotted_ as ED;
    match edot_ {
        ED::Exp(e) => unbound_names_exp(unbound, e),
        ED::Dot(d, _) => unbound_names_dotted(unbound, d),
    }
}

fn check_valid_local_name(context: &mut Context, v: &Var) {
    fn is_valid(s: &str) -> bool {
        s.starts_with('_') || s.starts_with(|c| matches!(c, 'a'..='z'))
    }
    if !is_valid(v.value()) {
        let msg = format!(
            "Invalid local name '{}'. Local names must start with 'a'..'z' (or '_')",
            v,
        );
        context.error(vec![(v.loc(), msg)])
    }
}

fn check_valid_struct_name(context: &mut Context, n: &StructName) {
    if let Err(e) = check_valid_struct_or_constant_name(&n.0, "struct", "Struct") {
        context.error(e)
    }
}

fn check_valid_struct_or_constant_name(n: &Name, lcase: &str, ucase: &str) -> Result<(), Error> {
    fn is_valid(s: &str) -> bool {
        s.starts_with(|c| matches!(c, 'A'..='Z'))
    }

    if !is_valid(&n.value) {
        let msg = format!(
            "Invalid {} name '{}'. {} names must start with 'A'..'Z'",
            lcase, n, ucase,
        );
        Err(vec![(n.loc, msg)])
    } else {
        Ok(())
    }
}

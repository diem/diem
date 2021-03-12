// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    errors::*,
    expansion::{
        aliases::{AliasMap, AliasSet},
        ast::{self as E, Fields, SpecId},
        byte_string, hex_string,
    },
    parser::ast::{
        self as P, Ability, ConstantName, Field, FunctionName, FunctionVisibility, ModuleIdent,
        ModuleName, StructName, Var,
    },
    shared::{unique_map::UniqueMap, *},
    FullyCompiledProgram,
};
use move_ir_types::location::*;
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    iter::IntoIterator,
};

//**************************************************************************************************
// Context
//**************************************************************************************************

type ModuleMembers = BTreeMap<Name, ModuleMemberKind>;
struct Context {
    module_members: UniqueMap<ModuleIdent, ModuleMembers>,
    errors: Errors,
    address: Option<Address>,
    aliases: AliasMap,
    is_source_module: bool,
    in_spec_context: bool,
    exp_specs: BTreeMap<SpecId, E::SpecBlock>,
}
impl Context {
    fn new(module_members: UniqueMap<ModuleIdent, ModuleMembers>) -> Self {
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

pub fn program(
    pre_compiled_lib: Option<&FullyCompiledProgram>,
    prog: P::Program,
) -> (E::Program, Errors) {
    let module_members = {
        let mut members = UniqueMap::new();
        all_module_members(&mut members, &prog.lib_definitions);
        all_module_members(&mut members, &prog.source_definitions);
        if let Some(pre_compiled) = pre_compiled_lib {
            assert!(pre_compiled.parser.source_definitions.is_empty());
            for def in pre_compiled.parser.lib_definitions.iter() {
                match def {
                    P::Definition::Module(_) => {
                        unimplemented!("top level modules not supported in pre compiled lib")
                    }
                    P::Definition::Address(_, a, ms) => {
                        for m in ms {
                            if members.contains_key_(&(*a, m.name.value().to_owned())) {
                                continue;
                            }
                            module_members(&mut members, *a, m)
                        }
                    }
                    P::Definition::Script(_) => (),
                }
            }
        }
        members
    };
    let mut context = Context::new(module_members);
    let mut module_map = UniqueMap::new();
    let mut scripts = vec![];

    let P::Program {
        source_definitions,
        lib_definitions,
    } = prog;

    context.is_source_module = false;
    for def in lib_definitions {
        match def {
            P::Definition::Module(m) => module(&mut context, &mut module_map, m),
            P::Definition::Address(loc, addr, ms) => {
                for mut m in ms {
                    check_module_address(&mut context, loc, addr, &mut m);
                    module(&mut context, &mut module_map, m)
                }
            }
            P::Definition::Script(_) => (),
        }
    }

    context.is_source_module = true;
    for def in source_definitions {
        match def {
            P::Definition::Module(m) => module(&mut context, &mut module_map, m),
            P::Definition::Address(loc, addr, ms) => {
                for mut m in ms {
                    check_module_address(&mut context, loc, addr, &mut m);
                    module(&mut context, &mut module_map, m)
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

    super::dependency_ordering::verify(&mut context.errors, &mut module_map);
    let prog = E::Program {
        modules: module_map,
        scripts,
    };
    (prog, context.get_errors())
}

fn check_module_address(
    context: &mut Context,
    loc: Loc,
    addr: Address,
    m: &mut P::ModuleDefinition,
) {
    match m.address {
        Some(sp!(other_loc, other_addr)) => {
            let msg = if addr == other_addr {
                "Redundant address specification"
            } else {
                "Multiple addresses specified for module"
            };
            context.error(vec![(other_loc, msg), (loc, "Previously specified here")]);
        }
        None => m.address = Some(sp(loc, addr)),
    }
}

fn module(
    context: &mut Context,
    module_map: &mut UniqueMap<ModuleIdent, E::ModuleDefinition>,
    module_def: P::ModuleDefinition,
) {
    assert!(context.address == None);
    let (mident, mod_) = module_(context, module_def);
    if let Err((mident, (old_loc, _))) = module_map.add(mident, mod_) {
        let mmsg = format!("Duplicate definition for module '{}'", mident);
        context.error(vec![
            (mident.loc(), mmsg),
            (old_loc, "Previously defined here".into()),
        ]);
    }
    context.address = None
}

fn set_sender_address(
    context: &mut Context,
    loc: Loc,
    module_name: &ModuleName,
    sender: Option<Spanned<Address>>,
) {
    context.address = Some(match sender {
        Some(sp!(_, addr)) => addr,
        None => {
            let msg = format!(
                "Invalid module declaration. The module does not have a specified address. Either \
                 declare it inside of an 'address <address> {{' block or declare it with an \
                 address 'module <address>::{}''",
                module_name
            );
            context.error(vec![(loc, msg)]);
            Address::DIEM_CORE
        }
    })
}

fn module_(context: &mut Context, mdef: P::ModuleDefinition) -> (ModuleIdent, E::ModuleDefinition) {
    let P::ModuleDefinition {
        loc,
        address,
        name,
        members,
    } = mdef;
    assert!(context.address == None);
    set_sender_address(context, loc, &name, address);
    let _ = check_restricted_self_name(context, "module", &name.0);
    if name.value().starts_with(|c| c == '_') {
        let msg = format!(
            "Invalid module name '{}'. Module names cannot start with '_'",
            name,
        );
        context.error(vec![(name.loc(), msg)]);
    }

    let name = name.0;
    let name_loc = name.loc;
    let current_module = ModuleIdent {
        locs: (name_loc, name_loc),
        value: (context.cur_address(), name.value),
    };

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

    let mut friends = UniqueMap::new();
    let mut functions = UniqueMap::new();
    let mut constants = UniqueMap::new();
    let mut structs = UniqueMap::new();
    let mut specs = vec![];
    for member in members {
        match member {
            P::ModuleMember::Use(_) => unreachable!(),
            P::ModuleMember::Friend(f) => friend(context, &mut friends, f),
            P::ModuleMember::Function(mut f) => {
                if !context.is_source_module {
                    f.body.value = P::FunctionBody_::Native
                }
                function(context, &mut functions, f)
            }
            P::ModuleMember::Constant(c) => constant(context, &mut constants, c),
            P::ModuleMember::Struct(s) => struct_def(context, &mut structs, s),
            P::ModuleMember::Spec(s) => specs.push(spec(context, s)),
        }
    }
    context.set_to_outer_scope(old_aliases);

    let def = E::ModuleDefinition {
        loc,
        is_source_module: context.is_source_module,
        dependency_order: 0,
        friends,
        structs,
        constants,
        functions,
        specs,
    };
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
        constants: pconstants,
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

    let mut constants = UniqueMap::new();
    for c in pconstants {
        // TODO remove after Self rework
        check_valid_module_member_name(context, ModuleMemberKind::Constant, c.name.0.clone());
        constant(context, &mut constants, c);
    }

    // TODO remove after Self rework
    check_valid_module_member_name(
        context,
        ModuleMemberKind::Function,
        pfunction.name.0.clone(),
    );
    let (function_name, function) = function_(context, pfunction);
    match &function.visibility {
        FunctionVisibility::Public(loc)
        | FunctionVisibility::Script(loc)
        | FunctionVisibility::Friend(loc) => {
            let msg = format!(
                "Extraneous '{}' modifier. Script functions are always '{}'",
                function.visibility,
                FunctionVisibility::SCRIPT,
            );
            context.error(vec![(*loc, msg)]);
        }
        FunctionVisibility::Internal => (),
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
        constants,
        function_name,
        function,
        specs,
    }
}

//**************************************************************************************************
// Aliases
//**************************************************************************************************

fn all_module_members<'a>(
    members: &mut UniqueMap<ModuleIdent, ModuleMembers>,
    defs: impl IntoIterator<Item = &'a P::Definition>,
) {
    for def in defs {
        match def {
            P::Definition::Module(m) => {
                module_members(members, m.address.map(|a| a.value).unwrap_or_default(), m)
            }
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
    members: &mut UniqueMap<ModuleIdent, ModuleMembers>,
    address: Address,
    m: &P::ModuleDefinition,
) {
    let mident = ModuleIdent {
        locs: (m.name.loc(), m.name.loc()),
        value: (address, m.name.value().to_string()),
    };
    let mut cur_members = members.remove(&mident).unwrap_or_else(ModuleMembers::new);
    for mem in &m.members {
        use P::{SpecBlockMember_ as SBM, SpecBlockTarget_ as SBT, SpecBlock_ as SB};
        match mem {
            P::ModuleMember::Function(f) => {
                cur_members.insert(f.name.0.clone(), ModuleMemberKind::Function);
            }
            P::ModuleMember::Constant(c) => {
                cur_members.insert(c.name.0.clone(), ModuleMemberKind::Constant);
            }
            P::ModuleMember::Struct(s) => {
                cur_members.insert(s.name.0.clone(), ModuleMemberKind::Struct);
            }
            P::ModuleMember::Spec(
                sp!(
                    _,
                    SB {
                        target,
                        members,
                        ..
                    }
                ),
            ) => match &target.value {
                SBT::Schema(n, _) => {
                    cur_members.insert(n.clone(), ModuleMemberKind::Schema);
                }
                SBT::Module => {
                    for sp!(_, smember_) in members {
                        if let SBM::Function { name, .. } = smember_ {
                            cur_members.insert(name.0.clone(), ModuleMemberKind::Function);
                        }
                    }
                }
                _ => (),
            },
            P::ModuleMember::Use(_) | P::ModuleMember::Friend(_) => (),
        };
    }
    members.add(mident, cur_members).unwrap();
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
    use P::{SpecBlockMember_ as SBM, SpecBlockTarget_ as SBT, SpecBlock_ as SB};
    macro_rules! check_name_and_add_implicit_alias {
        ($kind:expr, $name:expr) => {{
            if let Some(n) = check_valid_module_member_name(context, $kind, $name) {
                if let Err(loc) =
                    acc.add_implicit_member_alias(n.clone(), current_module.clone(), n.clone())
                {
                    duplicate_module_member(context, loc, n)
                }
            }
        }};
    }

    match member {
        P::ModuleMember::Use(u) => {
            use_(context, acc, u);
            None
        }
        f @ P::ModuleMember::Friend(_) => {
            // friend declarations do not produce implicit aliases
            Some(f)
        }
        P::ModuleMember::Function(f) => {
            let n = f.name.0.clone();
            check_name_and_add_implicit_alias!(ModuleMemberKind::Function, n);
            Some(P::ModuleMember::Function(f))
        }
        P::ModuleMember::Constant(c) => {
            let n = c.name.0.clone();
            check_name_and_add_implicit_alias!(ModuleMemberKind::Constant, n);
            Some(P::ModuleMember::Constant(c))
        }
        P::ModuleMember::Struct(s) => {
            let n = s.name.0.clone();
            check_name_and_add_implicit_alias!(ModuleMemberKind::Struct, n);
            Some(P::ModuleMember::Struct(s))
        }
        P::ModuleMember::Spec(s) => {
            let sp!(
                _,
                SB {
                    target,
                    members,
                    ..
                }
            ) = &s;
            match &target.value {
                SBT::Schema(n, _) => {
                    check_name_and_add_implicit_alias!(ModuleMemberKind::Schema, n.clone());
                }
                SBT::Module => {
                    for sp!(_, smember_) in members {
                        if let SBM::Function { name, .. } = smember_ {
                            let n = name.0.clone();
                            check_name_and_add_implicit_alias!(ModuleMemberKind::Function, n);
                        }
                    }
                }
                _ => (),
            };
            Some(P::ModuleMember::Spec(s))
        }
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
            let alias: Name =
                $alias_opt.unwrap_or_else(|| sp($ident.locs.1, $ident.value.1.clone()));
            if let Err(()) = check_restricted_self_name(context, "module alias", &alias) {
                return;
            }

            if let Err(old_loc) = acc.add_module_alias(alias.clone(), $ident) {
                duplicate_module_alias(context, old_loc, alias)
            }
        }};
    }
    match u {
        P::Use::Module(mident, alias_opt) => {
            if !context.module_members.contains_key(&mident) {
                context.error(unbound_module(&mident));
                return;
            };
            add_module_alias!(mident, alias_opt.map(|m| m.0))
        }
        P::Use::Members(mident, sub_uses) => {
            let members = match context.module_members.get(&mident) {
                Some(members) => members,
                None => {
                    context.error(unbound_module(&mident));
                    return;
                }
            };
            let mloc = context.module_members.get_loc(&mident).unwrap().0;
            let sub_uses_kinds = sub_uses
                .into_iter()
                .map(|(member, alia_opt)| {
                    let kind = members.get(&member).cloned();
                    (member, alia_opt, kind)
                })
                .collect::<Vec<_>>();

            for (member, alias_opt, member_kind_opt) in sub_uses_kinds {
                if member.value == ModuleName::SELF_NAME {
                    add_module_alias!(mident.clone(), alias_opt);
                    continue;
                }

                // check is member

                let member_kind = match member_kind_opt {
                    None => {
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
                    Some(m) => m,
                };

                let alias = alias_opt.unwrap_or_else(|| member.clone());

                let alias = match check_valid_module_member_alias(context, member_kind, alias) {
                    None => continue,
                    Some(alias) => alias,
                };
                if let Err(old_loc) = acc.add_member_alias(alias.clone(), mident.clone(), member) {
                    duplicate_module_member(context, old_loc, alias)
                }
            }
        }
    }
}

fn duplicate_module_alias(context: &mut Context, old_loc: Loc, alias: Name) {
    let msg = format!(
        "Duplicate module alias '{}'. Module aliases must be unique within a given namespace",
        alias
    );
    context.error(vec![
        (alias.loc, msg),
        (old_loc, "Previously defined here".into()),
    ])
}

fn duplicate_module_member(context: &mut Context, old_loc: Loc, alias: Name) {
    let msg = format!(
        "Duplicate module member or alias '{}'. Top level names in a namespace must be unique",
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
        abilities: abilities_vec,
        type_parameters: pty_params,
        fields: pfields,
    } = pstruct;
    let old_aliases = context.new_alias_scope(AliasMap::new());
    let type_parameters = type_parameters(context, pty_params);
    let abilities = ability_set(context, "modifier", abilities_vec);
    let fields = struct_fields(context, &name, pfields);
    let sdef = E::StructDefinition {
        loc,
        abilities,
        type_parameters,
        fields,
    };
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
        if let Err((field, old_loc)) = field_map.add(field, (idx, t)) {
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
// Friends
//**************************************************************************************************

fn friend(context: &mut Context, friends: &mut UniqueMap<ModuleIdent, Loc>, pfriend: P::Friend) {
    match friend_(context, pfriend) {
        Some((mident, loc)) => {
            if let Err((mident, (old_friend_loc, _))) = friends.add(mident, loc) {
                let msg = format!(
                    "Duplicate friend declaration '{}'. Friend declarations in a module must be \
                     unique",
                    mident
                );
                context.error(vec![
                    (loc, msg),
                    (old_friend_loc, "Previously declared here".into()),
                ]);
            }
        }
        None => assert!(context.has_errors()),
    };
}

fn friend_(context: &mut Context, sp!(loc, pfriend): P::Friend) -> Option<(ModuleIdent, Loc)> {
    assert!(context.exp_specs.is_empty());
    let mident_opt = match pfriend {
        P::Friend_::Module(mname) => match context.aliases.module_alias_get(&mname.0).cloned() {
            None => {
                context.error(vec![(
                    mname.loc(),
                    format!("Unbound module alias '{}'", mname),
                )]);
                None
            }
            Some(mident) => {
                let (_, value) = mident.drop_loc();
                Some(ModuleIdent::add_loc((mname.loc(), mname.loc()), value))
            }
        },
        P::Friend_::QualifiedModule(mident) => Some(mident),
    };
    mident_opt.map(|mident| (mident, loc))
}

//**************************************************************************************************
// Constants
//**************************************************************************************************

fn constant(
    context: &mut Context,
    constants: &mut UniqueMap<ConstantName, E::Constant>,
    pconstant: P::Constant,
) {
    let (name, constant) = constant_(context, pconstant);
    if let Err(_old_loc) = constants.add(name, constant) {
        assert!(context.has_errors())
    }
}

fn constant_(context: &mut Context, pconstant: P::Constant) -> (ConstantName, E::Constant) {
    assert!(context.exp_specs.is_empty());
    let P::Constant {
        loc,
        name,
        signature: psignature,
        value: pvalue,
    } = pconstant;
    let signature = type_(context, psignature);
    let value = exp_(context, pvalue);
    let _specs = context.extract_exp_specs();
    let constant = E::Constant {
        loc,
        signature,
        value,
    };
    (name, constant)
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
        uses,
        members: pmembers,
    } = pspec;

    context.in_spec_context = true;
    let mut new_scope = AliasMap::new();
    for u in uses {
        use_(context, &mut new_scope, u);
    }
    let old_aliases = context.new_alias_scope(new_scope);

    let members = pmembers
        .into_iter()
        .map(|m| spec_member(context, m))
        .collect();

    context.set_to_outer_scope(old_aliases);
    context.in_spec_context = false;

    sp(loc, E::SpecBlock_ { target, members })
}

fn spec_member(context: &mut Context, sp!(loc, pm): P::SpecBlockMember) -> E::SpecBlockMember {
    use E::SpecBlockMember_ as EM;
    use P::SpecBlockMember_ as PM;
    let em = match pm {
        PM::Condition {
            kind,
            properties: pproperties,
            exp,
            additional_exps,
        } => {
            let properties = pproperties
                .into_iter()
                .map(|p| pragma_property(context, p))
                .collect();
            let exp = exp_(context, exp);
            let additional_exps = additional_exps
                .into_iter()
                .map(|e| exp_(context, e))
                .collect();
            EM::Condition {
                kind,
                properties,
                exp,
                additional_exps,
            }
        }
        PM::Function {
            name,
            uninterpreted,
            signature,
            body,
        } => {
            let old_aliases = context.new_alias_scope(AliasMap::new());
            let body = function_body(context, body);
            let signature = function_signature(context, signature);
            context.set_to_outer_scope(old_aliases);
            EM::Function {
                uninterpreted,
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
        PM::Let { name, def: pdef } => {
            let def = exp_(context, pdef);
            EM::Let { name, def }
        }
        PM::Include {
            properties: pproperties,
            exp: pexp,
        } => {
            let properties = pproperties
                .into_iter()
                .map(|p| pragma_property(context, p))
                .collect();
            EM::Include {
                properties,
                exp: exp_(context, pexp),
            }
        }
        PM::Apply {
            exp: pexp,
            patterns,
            exclusion_patterns,
        } => EM::Apply {
            exp: exp_(context, pexp),
            patterns,
            exclusion_patterns,
        },
        PM::Pragma {
            properties: pproperties,
        } => {
            let properties = pproperties
                .into_iter()
                .map(|p| pragma_property(context, p))
                .collect();
            EM::Pragma { properties }
        }
    };
    sp(loc, em)
}

fn pragma_property(context: &mut Context, sp!(loc, pp_): P::PragmaProperty) -> E::PragmaProperty {
    let P::PragmaProperty_ {
        name,
        value: pv_opt,
    } = pp_;
    let value = pv_opt.and_then(|pv| pragma_value(context, pv));
    sp(loc, E::PragmaProperty_ { name, value })
}

fn pragma_value(context: &mut Context, pv: P::PragmaValue) -> Option<E::PragmaValue> {
    match pv {
        P::PragmaValue::Literal(v) => value(context, v).map(E::PragmaValue::Literal),
        P::PragmaValue::Ident(ma) => {
            module_access(context, Access::Term, ma).map(E::PragmaValue::Ident)
        }
    }
}

//**************************************************************************************************
// Types
//**************************************************************************************************

fn ability_set(context: &mut Context, case: &str, abilities_vec: Vec<Ability>) -> E::AbilitySet {
    let mut set = E::AbilitySet::empty();
    for ability in abilities_vec {
        let loc = ability.loc;
        if let Err(prev_loc) = set.add(ability) {
            context.error(vec![
                (loc, format!("Duplicate '{}' ability {}", ability, case)),
                (prev_loc, "Previously given".to_string()),
            ])
        }
    }
    set
}

fn type_parameters(
    context: &mut Context,
    pty_params: Vec<(Name, Vec<Ability>)>,
) -> Vec<(Name, E::AbilitySet)> {
    assert!(
        context.aliases.current_scope_is_empty(),
        "ICE alias scope should be cleared before handling type parameters"
    );
    pty_params
        .into_iter()
        .map(|(name, constraints_vec)| {
            context.aliases.remove_member_alias(&name);
            let constraints = ability_set(context, "constraint", constraints_vec);
            (name, constraints)
        })
        .collect()
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
        (Access::ApplyPositional, PN::Name(n))
        | (Access::ApplyNamed, PN::Name(n))
        | (Access::Type, PN::Name(n)) => match context.aliases.member_alias_get(&n) {
            Some((mident, mem)) => EN::ModuleAccess(mident.clone(), mem.clone()),
            None => EN::Name(n),
        },
        (Access::Term, PN::Name(n)) if is_valid_struct_constant_or_schema_name(&n.value) => {
            match context.aliases.member_alias_get(&n) {
                Some((mident, mem)) => EN::ModuleAccess(mident.clone(), mem.clone()),
                None => EN::Name(n),
            }
        }
        (Access::Term, PN::Name(n)) => EN::Name(n),
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
    let (uses, pitems, maybe_last_semicolon_loc, pfinal_item) = seq;

    let mut new_scope = AliasMap::new();
    for u in uses {
        use_(context, &mut new_scope, u);
    }
    let old_aliases = context.new_alias_scope(new_scope);
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
            sp(last_semicolon_loc, E::Exp_::Unit { trailing: true })
        }
        Some(e) => e,
    };
    let final_item = sp(final_e.loc, E::SequenceItem_::Seq(final_e));
    items.push_back(final_item);
    context.set_to_outer_scope(old_aliases);
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
        PE::Unit => EE::Unit { trailing: false },
        PE::Value(pv) => match value(context, pv) {
            Some(v) => EE::Value(v),
            None => {
                assert!(context.has_errors());
                EE::UnresolvedError
            }
        },
        PE::InferredNum(u) => EE::InferredNum(u),
        PE::Move(v) => EE::Move(v),
        PE::Copy(v) => EE::Copy(v),
        PE::Name(_, Some(_))
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
            let en_opt = module_access(context, Access::ApplyPositional, pn);
            match en_opt {
                Some(en) => EE::Call(en, tys_opt, ers),
                None => {
                    assert!(context.has_errors());
                    EE::UnresolvedError
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
                None => Box::new(sp(loc, EE::Unit { trailing: false })),
                Some(pf) => exp(context, *pf),
            };
            EE::IfElse(eb, et, ef)
        }
        PE::While(pb, ploop) => EE::While(exp(context, *pb), exp(context, *ploop)),
        PE::Loop(ploop) => EE::Loop(exp(context, *ploop)),
        PE::Block(seq) => EE::Block(sequence(context, loc, seq)),
        PE::Lambda(pbs, pe) => {
            if !context.require_spec_context(loc, "expression only allowed in specifications") {
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
        PE::Quant(k, prs, ptrs, pc, pe) => {
            if !context.require_spec_context(loc, "expression only allowed in specifications") {
                assert!(context.has_errors());
                EE::UnresolvedError
            } else {
                let rs_opt = bind_with_range_list(context, prs);
                let rtrs = ptrs
                    .into_iter()
                    .map(|trs| trs.into_iter().map(|tr| exp_(context, tr)).collect())
                    .collect();
                let rc = pc.map(|c| Box::new(exp_(context, *c)));
                let re = exp_(context, *pe);
                match rs_opt {
                    Some(rs) => EE::Quant(k, rs, rtrs, rc, Box::new(re)),
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
                None => Box::new(sp(loc, EE::Unit { trailing: false })),
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
        PE::Spec(_) if context.in_spec_context => {
            context.error(vec![(
                loc,
                "'spec' blocks cannot be used inside of a spec context",
            )]);
            EE::UnresolvedError
        }
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

fn value(context: &mut Context, sp!(loc, pvalue_): P::Value) -> Option<E::Value> {
    use E::Value_ as EV;
    use P::Value_ as PV;
    let value_ = match pvalue_ {
        PV::Address(addr) => EV::Address(addr),
        PV::U8(u) => EV::U8(u),
        PV::U64(u) => EV::U64(u),
        PV::U128(u) => EV::U128(u),
        PV::Bool(b) => EV::Bool(b),
        PV::HexString(s) => match hex_string::decode(loc, &s) {
            Ok(v) => EV::Bytearray(v),
            Err(e) => {
                context.errors.extend(e);
                return None;
            }
        },
        PV::ByteString(s) => match byte_string::decode(loc, &s) {
            Ok(v) => EV::Bytearray(v),
            Err(e) => {
                context.errors.extend(e);
                return None;
            }
        },
    };
    Some(sp(loc, value_))
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
        if let Err((field, old_loc)) = fmap.add(field, (idx, x)) {
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

fn bind_with_range_list(
    context: &mut Context,
    sp!(loc, prs_): P::BindWithRangeList,
) -> Option<E::LValueWithRangeList> {
    let rs_: Option<Vec<E::LValueWithRange>> = prs_
        .into_iter()
        .map(|sp!(loc, (pb, pr))| -> Option<E::LValueWithRange> {
            let r = exp_(context, pr);
            let b = bind(context, pb)?;
            Some(sp(loc, (b, r)))
        })
        .collect();
    Some(sp(loc, rs_?))
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
        PE::Name(n @ sp!(_, P::ModuleAccess_::ModuleAccess(_, _)), _)
        | PE::Name(n @ sp!(_, P::ModuleAccess_::QualifiedModuleAccess(_, _)), _)
            if !context.in_spec_context =>
        {
            let msg = format!(
                "Unexpected assignment of module access without fields outside of a spec \
                 context.\nIf you are trying to unpack a struct, try adding fields, e.g. '{} {{}}'",
                n
            );
            context.error(vec![(loc, msg)]);

            // For unused alias warnings and unbound modules
            module_access(context, Access::Term, n);

            return None;
        }
        PE::Name(n, Some(_)) if !context.in_spec_context => {
            let msg = format!(
                "Unexpected assignment of instantiated type without fields outside of a spec \
                 context.\nIf you are trying to unpack a struct, try adding fields, e.g. '{} {{}}'",
                n
            );
            context.error(vec![(loc, msg)]);

            // For unused alias warnings and unbound modules
            module_access(context, Access::Term, n);

            return None;
        }
        PE::Name(pn, ptys_opt) => {
            let en = module_access(context, Access::Term, pn)?;
            match &en.value {
                E::ModuleAccess_::ModuleAccess(m, n) if !context.in_spec_context => {
                    let msg = format!(
                        "Unexpected assignment of module access without fields outside of a spec \
                         context.\nIf you are trying to unpack a struct, try adding fields, e.g. \
                         '{}::{} {{}}'",
                        m, n,
                    );
                    context.error(vec![(loc, msg)]);
                    return None;
                }
                _ => {
                    let tys_opt = optional_types(context, ptys_opt);
                    EL::Var(en, tys_opt)
                }
            }
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
    match &m_ {
        M::Condition {
            exp,
            additional_exps,
            ..
        } => {
            unbound_names_exp(unbound, exp);
            additional_exps
                .iter()
                .for_each(|e| unbound_names_exp(unbound, e));
        }
        // No unbound names
        // And will error in the Move prover
        M::Function { .. }
        | M::Variable { .. }
        | M::Let { .. }
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
        | EE::Unit { .. } => (),
        EE::Copy(v) | EE::Move(v) => {
            unbound.insert(v.0.clone());
        }
        EE::Name(sp!(_, E::ModuleAccess_::Name(n)), _) => {
            unbound.insert(n.clone());
        }
        EE::Call(_, _, sp!(_, es_)) => unbound_names_exps(unbound, es_),
        EE::Pack(_, _, es) => unbound_names_exps(unbound, es.iter().map(|(_, _, (_, e))| e)),
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
        EE::Quant(_, rs, trs, cr_opt, er) => {
            unbound_names_exp(unbound, er);
            if let Some(cr) = cr_opt {
                unbound_names_exp(unbound, cr);
            }
            for tr in trs {
                unbound_names_exps(unbound, tr);
            }
            // remove anything in `rs`
            unbound_names_binds_with_range(unbound, rs);
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

fn unbound_names_binds_with_range(
    unbound: &mut BTreeSet<Name>,
    sp!(_, rs_): &E::LValueWithRangeList,
) {
    rs_.iter().rev().for_each(|sp!(_, (b, r))| {
        unbound_names_bind(unbound, b);
        unbound_names_exp(unbound, r)
    })
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
            .for_each(|(_, _, (_, l))| unbound_names_bind(unbound, l)),
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
            .for_each(|(_, _, (_, l))| unbound_names_assign(unbound, l)),
    }
}

fn unbound_names_dotted(unbound: &mut BTreeSet<Name>, sp!(_, edot_): &E::ExpDotted) {
    use E::ExpDotted_ as ED;
    match edot_ {
        ED::Exp(e) => unbound_names_exp(unbound, e),
        ED::Dot(d, _) => unbound_names_dotted(unbound, d),
    }
}

//**************************************************************************************************
// Valid names
//**************************************************************************************************

fn check_valid_local_name(context: &mut Context, v: &Var) {
    fn is_valid(s: &str) -> bool {
        s.starts_with('_') || s.starts_with(|c| matches!(c, 'a'..='z'))
    }
    if !is_valid(v.value()) {
        let msg = format!(
            "Invalid local variable name '{}'. Local variable names must start with 'a'..'z' (or \
             '_')",
            v,
        );
        context.error(vec![(v.loc(), msg)])
    }
}

#[derive(Copy, Clone, Debug)]
enum ModuleMemberKind {
    Constant,
    Function,
    Struct,
    Schema,
}

impl ModuleMemberKind {
    pub fn case(&self) -> &'static str {
        match self {
            ModuleMemberKind::Function => "function",
            ModuleMemberKind::Constant => "constant",
            ModuleMemberKind::Struct => "struct",
            ModuleMemberKind::Schema => "schema",
        }
    }
}

fn check_valid_module_member_name(
    context: &mut Context,
    member: ModuleMemberKind,
    name: Name,
) -> Option<Name> {
    match check_valid_module_member_name_impl(context, member, &name, member.case()) {
        Err(()) => None,
        Ok(()) => Some(name),
    }
}

fn check_valid_module_member_alias(
    context: &mut Context,
    member: ModuleMemberKind,
    alias: Name,
) -> Option<Name> {
    match check_valid_module_member_name_impl(
        context,
        member,
        &alias,
        &format!("{} alias", member.case()),
    ) {
        Err(()) => None,
        Ok(()) => Some(alias),
    }
}

fn check_valid_module_member_name_impl(
    context: &mut Context,
    member: ModuleMemberKind,
    n: &Name,
    case: &str,
) -> Result<(), ()> {
    use ModuleMemberKind as M;
    fn upper_first_letter(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }
    let lcase = case;
    match member {
        M::Function => {
            if n.value.starts_with(|c| c == '_') {
                let msg = format!(
                    "Invalid {} name '{}'. {} names cannot start with '_'",
                    lcase,
                    n,
                    upper_first_letter(case),
                );
                context.error(vec![(n.loc, msg)]);
                return Err(());
            }
        }
        M::Constant | M::Struct | M::Schema => {
            if !is_valid_struct_constant_or_schema_name(&n.value) {
                let msg = format!(
                    "Invalid {} name '{}'. {} names must start with 'A'..'Z'",
                    lcase,
                    n,
                    upper_first_letter(case),
                );
                context.error(vec![(n.loc, msg)]);
                return Err(());
            }
        }
    }

    // TODO move these names to a more central place?
    check_restricted_names(
        context,
        lcase,
        n,
        crate::naming::ast::BuiltinFunction_::all_names(),
    )?;
    check_restricted_names(
        context,
        lcase,
        n,
        crate::naming::ast::BuiltinTypeName_::all_names(),
    )?;

    // Restricting Self for now in the case where we ever have impls
    // Otherwise, we could allow it
    check_restricted_self_name(context, lcase, n)?;

    Ok(())
}

pub fn is_valid_struct_constant_or_schema_name(s: &str) -> bool {
    s.starts_with(|c| matches!(c, 'A'..='Z'))
}

fn check_restricted_self_name(context: &mut Context, case: &str, n: &Name) -> Result<(), ()> {
    if n.value == ModuleName::SELF_NAME {
        context.error(restricted_name_error(case, n.loc, ModuleName::SELF_NAME));
        Err(())
    } else {
        Ok(())
    }
}

fn check_restricted_names(
    context: &mut Context,
    case: &str,
    sp!(loc, n_): &Name,
    all_names: &BTreeSet<&str>,
) -> Result<(), ()> {
    if all_names.contains(n_.as_str()) {
        context.error(restricted_name_error(case, *loc, n_));
        Err(())
    } else {
        Ok(())
    }
}

fn restricted_name_error(case: &str, loc: Loc, restricted: &str) -> Error {
    let msg = format!(
        "Invalid {case} name '{restricted}'. '{restricted}' is restricted and cannot be used to \
         name a {case}",
        case = case,
        restricted = restricted,
    );
    vec![(loc, msg)]
}

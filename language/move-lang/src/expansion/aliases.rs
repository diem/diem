// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    parser::ast::{ModuleIdent, ModuleIdent_, ModuleName},
    shared::{remembering_unique_map::RememberingUniqueMap, *},
};
use move_ir_types::location::*;
use std::{collections::BTreeSet, iter::IntoIterator};

#[derive(Clone, Debug)]
pub struct AliasSet {
    pub modules: BTreeSet<Name>,
    pub members: BTreeSet<Name>,
}

#[derive(Clone, Debug)]
pub struct AliasMap {
    modules: RememberingUniqueMap<Name, ModuleIdent>,
    members: RememberingUniqueMap<Name, (ModuleIdent, Name)>,
    current_scope: AliasSet,
}

// unsafe function for creating ModuleIdent for auto-imported modules
// This usage is "safe" as the location should never be used
fn unsafe_no_loc_diem_module_ident(s: &str) -> ModuleIdent {
    ModuleIdent(Spanned::unsafe_no_loc(ModuleIdent_ {
        name: ModuleName(Spanned::unsafe_no_loc(s.to_string())),
        address: Address::DIEM_CORE,
    }))
}

macro_rules! prelude_modules {
    ($(const $const:ident = $v:tt; fn $fn:ident),*) => {
        $(
            const $const: &'static str = $v;
            fn $fn() -> ModuleIdent {
                unsafe_no_loc_diem_module_ident($const)
            }
        )*

        pub(super) fn all_prelude_modules() -> Vec<ModuleIdent> {
            vec![$($fn(),)*]
        }
    };
}

macro_rules! prelude_members {
    ($(mod $module_fn:ident; const $const:ident = $v:tt; fn $fn:ident),*) => {
        $(
            const $const: &'static str = $v;
            fn $fn() -> Name {
                Spanned::unsafe_no_loc($const.to_string())
            }
        )*

        pub(super) fn all_prelude_module_members() -> Vec<(ModuleIdent, Name)> {
            vec![$(($module_fn(), $fn()),)*]
        }
    };
}

// auto included Diem modules
prelude_modules!(
    const OPTION_NAME = "Option"; fn option_ident,
    const SIGNER_NAME = "Signer"; fn signer_ident,
    const VECTOR_NAME = "Vector"; fn vector_ident
);

// auto included Diem module members
prelude_members!(
    mod option_ident; const OPTION_STRUCT_NAME = OPTION_NAME; fn option_struct_name
);

impl AliasSet {
    pub fn new() -> Self {
        Self {
            modules: BTreeSet::new(),
            members: BTreeSet::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        let Self { modules, members } = self;
        modules.is_empty() && members.is_empty()
    }
}

impl AliasMap {
    pub fn new() -> Self {
        Self {
            modules: RememberingUniqueMap::new(),
            members: RememberingUniqueMap::new(),
            current_scope: AliasSet::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        let Self {
            modules,
            members,
            current_scope,
        } = self;
        let is_empty = modules.is_empty() && members.is_empty();
        // is_empty ==> current_scope.is_empty();
        assert!(!is_empty || current_scope.is_empty());
        is_empty
    }

    pub fn current_scope_is_empty(&self) -> bool {
        self.current_scope.is_empty()
    }

    fn remove_module_alias_(&mut self, alias: &Name) -> Result<(), Loc> {
        self.current_scope.modules.remove(alias);
        let loc = self.modules.get_loc(alias).cloned();
        match self.modules.remove(alias) {
            None => Ok(()),
            Some(_) => Err(loc.unwrap()),
        }
    }

    fn remove_member_alias_(&mut self, alias: &Name) -> Result<(), Loc> {
        self.current_scope.members.remove(alias);
        let loc = self.members.get_loc(alias).cloned();
        match self.members.remove(alias) {
            None => Ok(()),
            Some(_) => Err(loc.unwrap()),
        }
    }

    pub fn remove_member_alias(&mut self, alias: &Name) {
        let _ = self.remove_member_alias_(alias);
    }

    /// Adds a module alias to the map.
    /// Errors if one already bound for that alias
    pub fn add_module_alias(&mut self, alias: Name, ident: ModuleIdent) -> Result<(), Loc> {
        let result = self.remove_module_alias_(&alias);
        self.current_scope.modules.insert(alias.clone());
        self.modules.add(alias, ident).unwrap();
        result
    }

    /// Adds a member alias to the map.
    /// Errors if one already bound for that alias
    pub fn add_member_alias(
        &mut self,
        alias: Name,
        ident: ModuleIdent,
        member: Name,
    ) -> Result<(), Loc> {
        let result = self.remove_member_alias_(&alias);
        self.current_scope.members.insert(alias.clone());
        self.members.add(alias, (ident, member)).unwrap();
        result
    }

    /// Same as `add_module_alias` but it does not update the scope, and as such it will not be
    /// reported as unused
    pub fn add_implicit_module_alias(
        &mut self,
        alias: Name,
        ident: ModuleIdent,
    ) -> Result<(), Loc> {
        let result = self.remove_module_alias_(&alias);
        self.modules.add(alias, ident).unwrap();
        result
    }

    /// Same as `add_member_alias` but it does not update the scope, and as such it will not be
    /// reported as unused
    pub fn add_implicit_member_alias(
        &mut self,
        alias: Name,
        ident: ModuleIdent,
        member: Name,
    ) -> Result<(), Loc> {
        let result = self.remove_member_alias_(&alias);
        self.members.add(alias, (ident, member)).unwrap();
        result
    }

    pub fn module_alias_get(&mut self, n: &Name) -> Option<&ModuleIdent> {
        self.modules.get(n)
    }

    pub fn member_alias_get(&mut self, n: &Name) -> Option<&(ModuleIdent, Name)> {
        self.members.get(n)
    }

    // Adds prelude modules and module members if they are defined in the current program
    // If they are not defined, no alias is added,
    // see prelude_modules and prelude_members for a list of modules/members added
    pub fn add_prelude(
        &mut self,
        prelude_modules: impl Iterator<Item = ModuleIdent>,
        prelude_module_members: impl Iterator<Item = (ModuleIdent, Name)>,
    ) {
        debug_assert!(self.is_empty(), "ICE prelude should be added on empty");
        for m in prelude_modules {
            let res = self.add_implicit_module_alias(m.0.value.name.0.clone(), m);
            debug_assert!(res.is_ok(), "ICE duplicate prelude module")
        }
        for (m, member) in prelude_module_members {
            let res = self.add_implicit_member_alias(member.clone(), m, member);
            debug_assert!(res.is_ok(), "ICE duplicate prelude module member")
        }
    }

    pub fn add_and_shadow_all(&mut self, shadowing: AliasMap) {
        let Self {
            modules: new_modules,
            members: new_members,
            current_scope: new_scope,
        } = shadowing;
        for (alias, ident) in new_modules {
            let _ = self.add_implicit_module_alias(alias, ident);
        }
        for (alias, (ident, member)) in new_members {
            let _ = self.add_implicit_member_alias(alias, ident, member);
        }
        self.current_scope = new_scope
    }

    pub fn close_scope_and_report_unused(&mut self, inner: Self) -> AliasSet {
        let outer_scope = self;
        let Self {
            modules: inner_modules,
            members: inner_members,
            current_scope:
                AliasSet {
                    modules: inner_scope_modules,
                    members: inner_scope_members,
                },
        } = inner;

        let used_modules = inner_modules.remember();
        let used_members = inner_members.remember();

        // propagate uses of aliases
        used_modules
            .iter()
            // remove newly declared aliaes
            .filter(|a| !inner_scope_modules.contains(a))
            .for_each(|a| {
                // get the module alias to mark it as used
                outer_scope.module_alias_get(a);
            });
        used_members
            .iter()
            // remove newly declared aliaes
            .filter(|a| !inner_scope_members.contains(a))
            .for_each(|a| {
                // get the member alias to mark it as used
                outer_scope.member_alias_get(a);
            });

        // report unused
        let unused_modules = inner_scope_modules
            .into_iter()
            .filter(|n| !used_modules.contains(n))
            .collect();
        let unused_members = inner_scope_members
            .into_iter()
            .filter(|n| !used_members.contains(n))
            .collect();
        AliasSet {
            modules: unused_modules,
            members: unused_members,
        }
    }
}

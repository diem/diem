// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    expansion::ast::ModuleIdent,
    shared::{unique_map::UniqueMap, unique_set::UniqueSet, *},
};
use move_ir_types::location::*;

type ScopeDepth = usize;

#[derive(Clone, Debug)]
pub struct AliasSet {
    pub modules: UniqueSet<Name>,
    pub members: UniqueSet<Name>,
}

#[derive(Clone, Debug)]
pub struct AliasMapBuilder {
    modules: UniqueMap<Name, (ModuleIdent, /* is_implicit */ bool)>,
    members: UniqueMap<Name, ((ModuleIdent, Name), /* is_implicit */ bool)>,
}

#[derive(Clone, Debug)]
pub struct AliasMap {
    modules: UniqueMap<Name, (Option<ScopeDepth>, ModuleIdent)>,
    members: UniqueMap<Name, (Option<ScopeDepth>, (ModuleIdent, Name))>,
    // essentially a mapping from ScopeDepth => AliasSet, which are the unused aliases at that depth
    unused: Vec<AliasSet>,
}

pub struct OldAliasMap(Option<AliasMap>);

impl AliasSet {
    pub fn new() -> Self {
        Self {
            modules: UniqueSet::new(),
            members: UniqueSet::new(),
        }
    }

    #[allow(unused)]
    pub fn is_empty(&self) -> bool {
        let Self { modules, members } = self;
        modules.is_empty() && members.is_empty()
    }
}

impl AliasMapBuilder {
    pub fn new() -> Self {
        Self {
            modules: UniqueMap::new(),
            members: UniqueMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        let Self { modules, members } = self;
        modules.is_empty() && members.is_empty()
    }

    fn remove_module_alias_(&mut self, alias: &Name) -> Result<(), Loc> {
        let loc = self.modules.get_loc(alias).cloned();
        match self.modules.remove(alias) {
            None => Ok(()),
            Some(_) => Err(loc.unwrap()),
        }
    }

    fn remove_member_alias_(&mut self, alias: &Name) -> Result<(), Loc> {
        let loc = self.members.get_loc(alias).cloned();
        match self.members.remove(alias) {
            None => Ok(()),
            Some(_) => Err(loc.unwrap()),
        }
    }

    /// Adds a module alias to the map.
    /// Errors if one already bound for that alias
    pub fn add_module_alias(&mut self, alias: Name, ident: ModuleIdent) -> Result<(), Loc> {
        let result = self.remove_module_alias_(&alias);
        self.modules
            .add(alias, (ident, /* is_implicit */ false))
            .unwrap();
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
        self.members
            .add(alias, ((ident, member), /* is_implicit */ false))
            .unwrap();
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
        self.modules
            .add(alias, (ident, /* is_implicit */ true))
            .unwrap();
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
        self.members
            .add(alias, ((ident, member), /* is_implicit */ true))
            .unwrap();
        result
    }
}

impl AliasMap {
    pub fn new() -> Self {
        Self {
            modules: UniqueMap::new(),
            members: UniqueMap::new(),
            unused: vec![],
        }
    }

    pub fn is_empty(&self) -> bool {
        let Self {
            modules,
            members,
            unused: _,
        } = self;
        modules.is_empty() && members.is_empty()
    }

    fn current_depth(&self) -> usize {
        self.unused.len()
    }

    pub fn module_alias_get(&mut self, n: &Name) -> Option<&ModuleIdent> {
        match self.modules.get_mut(n) {
            None => None,
            Some((depth_opt, ident)) => {
                if let Some(depth) = depth_opt {
                    self.unused[*depth].modules.remove(n);
                }
                *depth_opt = None;
                Some(ident)
            }
        }
    }

    pub fn member_alias_get(&mut self, n: &Name) -> Option<&(ModuleIdent, Name)> {
        match self.members.get_mut(n) {
            None => None,
            Some((depth_opt, ident_name)) => {
                if let Some(depth) = depth_opt {
                    self.unused[*depth].members.remove(n);
                }
                *depth_opt = None;
                Some(ident_name)
            }
        }
    }

    /// Adds all of the new items in the new inner scope as shadowing the outer one.
    /// Gives back the outer scope
    pub fn add_and_shadow_all(&mut self, shadowing: AliasMapBuilder) -> OldAliasMap {
        if shadowing.is_empty() {
            return OldAliasMap(None);
        }

        let outer_scope = OldAliasMap(Some(self.clone()));
        let AliasMapBuilder {
            modules: new_modules,
            members: new_members,
        } = shadowing;

        let next_depth = self.current_depth();
        let mut current_scope = AliasSet::new();
        for (alias, (ident, is_implicit)) in new_modules {
            if !is_implicit {
                current_scope.modules.add(alias).unwrap();
            }
            self.modules.remove(&alias);
            self.modules.add(alias, (Some(next_depth), ident)).unwrap();
        }
        for (alias, (ident_member, is_implicit)) in new_members {
            if !is_implicit {
                current_scope.members.add(alias).unwrap();
            }
            self.members.remove(&alias);
            self.members
                .add(alias, (Some(next_depth), ident_member))
                .unwrap();
        }
        self.unused.push(current_scope);
        outer_scope
    }

    /// Similar to add_and_shadow but just removes aliases now shadowed by a type parameter
    pub fn shadow_for_type_parameters<'a, I: IntoIterator<Item = &'a Name>>(
        &mut self,
        tparams: I,
    ) -> OldAliasMap
    where
        I::IntoIter: ExactSizeIterator,
    {
        let tparams_iter = tparams.into_iter();
        if tparams_iter.len() == 0 {
            return OldAliasMap(None);
        }

        let outer_scope = OldAliasMap(Some(self.clone()));
        self.unused.push(AliasSet::new());
        for tp_name in tparams_iter {
            self.members.remove(tp_name);
        }
        outer_scope
    }

    /// Resets the alias map and gives the set of aliases that were unused
    pub fn set_to_outer_scope(&mut self, outer_scope: OldAliasMap) -> AliasSet {
        let outer_scope = match outer_scope.0 {
            None => return AliasSet::new(),
            Some(outer) => outer,
        };
        let mut inner_scope = std::mem::replace(self, outer_scope);
        let outer_scope = self;
        assert!(outer_scope.current_depth() + 1 == inner_scope.current_depth());
        let unused = inner_scope.unused.pop().unwrap();
        outer_scope.unused = inner_scope.unused;
        unused
    }
}

impl OldAliasMap {
    pub fn is_empty(&self) -> bool {
        match &self.0 {
            None => true,
            Some(aliases) => aliases.is_empty(),
        }
    }
}

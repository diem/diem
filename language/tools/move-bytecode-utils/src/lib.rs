// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod dependency_graph;

use crate::dependency_graph::DependencyGraph;
use move_binary_format::{access::ModuleAccess, file_format::CompiledModule};
use move_core_types::language_storage::ModuleId;

use anyhow::{anyhow, Result};
use std::collections::BTreeMap;

/// Set of Move modules indexed by module Id
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Modules<'a>(BTreeMap<ModuleId, &'a CompiledModule>);

impl<'a> Modules<'a> {
    /// Construct a set of modules from a slice `modules`.
    /// Panics if `modules` contains duplicates
    pub fn new(modules: impl IntoIterator<Item = &'a CompiledModule>) -> Self {
        let mut map = BTreeMap::new();
        for m in modules {
            assert!(
                map.insert(m.self_id(), m).is_none(),
                "Duplicate module found"
            );
        }
        Modules(map)
    }

    /// Return all modules in this set
    pub fn iter_modules(&self) -> Vec<&CompiledModule> {
        self.0.values().copied().collect()
    }

    /// Compute a dependency graph for `self`
    pub fn compute_dependency_graph(&self) -> DependencyGraph {
        DependencyGraph::new(self.0.values().copied())
    }

    /// Return the backing map of `self`
    pub fn get_map(&self) -> &BTreeMap<ModuleId, &CompiledModule> {
        &self.0
    }

    /// Return the bytecode for the module bound to `module_id`
    pub fn get_module(&self, module_id: &ModuleId) -> Result<&CompiledModule> {
        self.0
            .get(module_id)
            .copied()
            .ok_or_else(|| anyhow!("Can't find module {:?}", module_id))
    }

    /// Return the immediate dependencies for `module_id`
    pub fn get_immediate_dependencies(&self, module_id: &ModuleId) -> Result<Vec<&CompiledModule>> {
        self.get_module(module_id)?
            .immediate_dependencies()
            .into_iter()
            .map(|mid| self.get_module(&mid))
            .collect::<Result<Vec<_>>>()
    }

    fn get_transitive_dependencies_(
        &'a self,
        all_deps: &mut Vec<&'a CompiledModule>,
        module: &'a CompiledModule,
        loader: &'a Modules,
    ) -> Result<()> {
        let next_deps = module.immediate_dependencies();
        all_deps.push(module);
        for next in next_deps {
            let next_module = self.get_module(&next)?;
            self.get_transitive_dependencies_(all_deps, next_module, loader)?;
        }
        Ok(())
    }

    /// Return the transitive dependencies for `module_id`
    pub fn get_transitive_dependencies(
        &self,
        module_id: &ModuleId,
    ) -> Result<Vec<&CompiledModule>> {
        let mut all_deps = vec![];
        for dep in self.get_immediate_dependencies(module_id)? {
            self.get_transitive_dependencies_(&mut all_deps, dep, self)?;
        }
        Ok(all_deps)
    }
}

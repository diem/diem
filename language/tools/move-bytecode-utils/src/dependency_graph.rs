// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::{access::ModuleAccess, file_format::CompiledModule};
use petgraph::graphmap::DiGraphMap;

use anyhow::{bail, Result};
use std::collections::BTreeMap;

/// Directed graph capturing dependencies between modules
pub struct DependencyGraph<'a> {
    /// Set of modules guaranteed to be closed under dependencies
    modules: Vec<&'a CompiledModule>,
    graph: DiGraphMap<ModuleIndex, ()>,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord)]
struct ModuleIndex(usize);

impl<'a> DependencyGraph<'a> {
    /// Construct a dependency graph from a set of `modules`.
    /// Panics if `modules` contains duplicates or is not closed under the depedency relation
    pub fn new(module_iter: impl IntoIterator<Item = &'a CompiledModule>) -> Self {
        let mut modules = vec![];
        let mut reverse_modules = BTreeMap::new();
        for (i, m) in module_iter.into_iter().enumerate() {
            modules.push(m);
            assert!(
                reverse_modules
                    .insert(m.self_id(), ModuleIndex(i))
                    .is_none(),
                "Duplicate module found"
            );
        }
        let mut graph = DiGraphMap::new();
        for module in &modules {
            let module_idx: ModuleIndex = *reverse_modules.get(&module.self_id()).unwrap();
            let deps = module.immediate_dependencies();
            if deps.is_empty() {
                graph.add_node(module_idx);
            } else {
                for dep in deps {
                    let dep_idx = *reverse_modules.get(&dep).expect("Missing dependency");
                    graph.add_edge(dep_idx, module_idx, ());
                }
            }
        }
        DependencyGraph { modules, graph }
    }

    /// Return an iterator over the modules in `self` in topological order--modules with least deps first.
    /// Fails with an error if `self` contains circular dependencies
    pub fn compute_topological_order(&self) -> Result<impl Iterator<Item = &CompiledModule>> {
        match petgraph::algo::toposort(&self.graph, None) {
            Err(_) => bail!("Circular dependency detected"),
            Ok(ordered_idxs) => Ok(ordered_idxs.into_iter().map(move |idx| self.modules[idx.0])),
        }
    }
}

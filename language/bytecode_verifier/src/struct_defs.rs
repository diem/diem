// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module provides a checker for verifing that struct definitions in a module are not
//! recursive. Since the module dependency graph is acylic by construction, applying this checker to
//! each module in isolation guarantees that there is no structural recursion globally.
use libra_types::vm_error::{StatusCode, VMStatus};
use petgraph::{algo::toposort, Directed, Graph};
use std::collections::BTreeMap;
use vm::{
    access::ModuleAccess,
    errors::verification_error,
    file_format::{CompiledModule, StructDefinitionIndex, StructHandleIndex, TableIndex},
    internals::ModuleIndex,
    views::StructDefinitionView,
    IndexKind,
};

pub struct RecursiveStructDefChecker<'a> {
    module: &'a CompiledModule,
}

impl<'a> RecursiveStructDefChecker<'a> {
    pub fn new(module: &'a CompiledModule) -> Self {
        Self { module }
    }

    pub fn verify(self) -> Vec<VMStatus> {
        let graph_builder = StructDefGraphBuilder::new(self.module);

        let graph = graph_builder.build();

        // toposort is iterative while petgraph::algo::is_cyclic_directed is recursive. Prefer
        // the iterative solution here as this code may be dealing with untrusted data.
        match toposort(&graph, None) {
            Ok(_) => {
                // Is the result of this useful elsewhere?
                vec![]
            }
            Err(cycle) => {
                let sd_idx = graph[cycle.node_id()];
                vec![verification_error(
                    IndexKind::StructDefinition,
                    sd_idx.into_index(),
                    StatusCode::RECURSIVE_STRUCT_DEFINITION,
                )]
            }
        }
    }
}

/// Given a module, build a graph of struct definitions. This is useful when figuring out whether
/// the struct definitions in module form a cycle.
pub struct StructDefGraphBuilder<'a> {
    module: &'a CompiledModule,
    /// Used to follow field definitions' signatures' struct handles to their struct definitions.
    handle_to_def: BTreeMap<StructHandleIndex, StructDefinitionIndex>,
}

impl<'a> StructDefGraphBuilder<'a> {
    pub fn new(module: &'a CompiledModule) -> Self {
        let mut handle_to_def = BTreeMap::new();
        // the mapping from struct definitions to struct handles is already checked to be 1-1 by
        // DuplicationChecker
        for (idx, struct_def) in module.struct_defs().iter().enumerate() {
            let sh_idx = struct_def.struct_handle;
            handle_to_def.insert(sh_idx, StructDefinitionIndex::new(idx as TableIndex));
        }

        Self {
            module,
            handle_to_def,
        }
    }

    pub fn build(self) -> Graph<StructDefinitionIndex, (), Directed, u32> {
        let mut graph = Graph::new();

        let struct_def_count = self.module.struct_defs().len();

        let nodes: Vec<_> = (0..struct_def_count)
            .map(|idx| graph.add_node(StructDefinitionIndex::new(idx as TableIndex)))
            .collect();

        for idx in 0..struct_def_count {
            let sd_idx = StructDefinitionIndex::new(idx as TableIndex);
            for followed_idx in self.member_struct_defs(sd_idx) {
                graph.add_edge(nodes[idx], nodes[followed_idx.into_index()], ());
            }
        }

        graph
    }

    fn member_struct_defs(
        &'a self,
        idx: StructDefinitionIndex,
    ) -> impl Iterator<Item = StructDefinitionIndex> + 'a {
        let struct_def = self.module.struct_def_at(idx);
        let struct_def = StructDefinitionView::new(self.module, struct_def);
        // The fields iterator is an option in the case of native structs. Flatten makes an empty
        // iterator for that case
        let fields = struct_def.fields().into_iter().flatten();
        let handle_to_def = &self.handle_to_def;

        fields.filter_map(move |field| {
            let type_signature = field.type_signature();
            let sh_idx = type_signature.token().struct_index()?;
            match handle_to_def.get(&sh_idx) {
                Some(sd_idx) => Some(*sd_idx),
                None => {
                    // This field refers to a struct in another module.
                    None
                }
            }
        })
    }
}

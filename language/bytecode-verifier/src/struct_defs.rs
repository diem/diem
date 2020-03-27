// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module provides a checker for verifing that struct definitions in a module are not
//! recursive. Since the module dependency graph is acylic by construction, applying this checker to
//! each module in isolation guarantees that there is no structural recursion globally.
use libra_types::vm_error::{StatusCode, VMStatus};
use petgraph::{algo::toposort, graphmap::DiGraphMap};
use std::collections::{BTreeMap, BTreeSet};
use vm::{
    access::ModuleAccess,
    errors::{verification_error, VMResult},
    file_format::{
        CompiledModule, SignatureToken, StructDefinitionIndex, StructHandleIndex, TableIndex,
    },
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
        let graph = match StructDefGraphBuilder::new(self.module).build() {
            Err(status) => return vec![status],
            Ok(graph) => graph,
        };

        // toposort is iterative while petgraph::algo::is_cyclic_directed is recursive. Prefer
        // the iterative solution here as this code may be dealing with untrusted data.
        match toposort(&graph, None) {
            Ok(_) => {
                // Is the result of this useful elsewhere?
                vec![]
            }
            Err(cycle) => vec![verification_error(
                IndexKind::StructDefinition,
                cycle.node_id().into_index(),
                StatusCode::RECURSIVE_STRUCT_DEFINITION,
            )],
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

    pub fn build(self) -> VMResult<DiGraphMap<StructDefinitionIndex, ()>> {
        let mut neighbors = BTreeMap::new();
        for idx in 0..self.module.struct_defs().len() {
            let sd_idx = StructDefinitionIndex::new(idx as TableIndex);
            self.add_struct_defs(&mut neighbors, sd_idx)?
        }

        let edges = neighbors
            .into_iter()
            .flat_map(|(parent, children)| children.into_iter().map(move |child| (parent, child)));
        Ok(DiGraphMap::from_edges(edges))
    }

    fn add_struct_defs(
        &self,
        neighbors: &mut BTreeMap<StructDefinitionIndex, BTreeSet<StructDefinitionIndex>>,
        idx: StructDefinitionIndex,
    ) -> VMResult<()> {
        let struct_def = self.module.struct_def_at(idx);
        let struct_def = StructDefinitionView::new(self.module, struct_def);
        // The fields iterator is an option in the case of native structs. Flatten makes an empty
        // iterator for that case
        for field in struct_def.fields().into_iter().flatten() {
            self.add_signature_token(neighbors, idx, field.signature_token())?
        }
        Ok(())
    }

    fn add_signature_token(
        &self,
        neighbors: &mut BTreeMap<StructDefinitionIndex, BTreeSet<StructDefinitionIndex>>,
        cur_idx: StructDefinitionIndex,
        token: &SignatureToken,
    ) -> VMResult<()> {
        use SignatureToken as T;
        Ok(match token {
            T::Bool | T::U8 | T::U64 | T::U128 | T::Address | T::TypeParameter(_) => (),
            T::Reference(_) | T::MutableReference(_) => {
                return Err(VMStatus::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                    .with_message("Reference field when checking recursive structs".to_owned()))
            }
            T::Vector(inner) => self.add_signature_token(neighbors, cur_idx, inner)?,
            T::Struct(sh_idx, inners) => {
                if let Some(struct_def_idx) = self.handle_to_def.get(sh_idx) {
                    neighbors
                        .entry(cur_idx)
                        .or_insert_with(BTreeSet::new)
                        .insert(*struct_def_idx);
                }
                for t in inners {
                    self.add_signature_token(neighbors, cur_idx, t)?
                }
            }
        })
    }
}

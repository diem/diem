// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This implements an algorithm that detects loops during the instantiation of generics.
//!
//! It builds a graph from the given `CompiledModule` and converts the original problem into
//! finding strongly connected components in the graph with certain properties. Read the
//! documentation of the types/functions below for details of how it works.
//!
//! Note: We're doing generics only up to specialization, and are doing a conservative check of
//! generic call sites to eliminate those which could lead to an infinite number of specialized
//! instances. We do reject recursive functions that create a new type upon each call but do
//! terminate eventually.

use libra_types::vm_status::StatusCode;
use petgraph::{
    algo::tarjan_scc,
    graph::{EdgeIndex, NodeIndex},
    visit::EdgeRef,
    Graph,
};
use std::collections::{hash_map, HashMap, HashSet};
use vm::{
    access::ModuleAccess,
    errors::{Location, PartialVMError, PartialVMResult, VMResult},
    file_format::{
        Bytecode, CompiledModule, FunctionDefinition, FunctionDefinitionIndex, FunctionHandleIndex,
        SignatureIndex, SignatureToken, TypeParameterIndex,
    },
};

/// Data attached to each node.
/// Each node corresponds to a type formal of a generic function in the module.
#[derive(Eq, PartialEq, Hash, Copy, Clone)]
struct Node(FunctionDefinitionIndex, TypeParameterIndex);

/// Data attached to each edge. Indicating the type of the edge.
enum Edge<'a> {
    /// This type of edge from type formal T1 to T2 means the type bound to T1 is used to
    /// instantiate T2 unmodified, thus the name `Identity`.
    ///
    /// Example:
    /// ```
    /// //    foo<T>() { bar<T>(); return; }
    /// //
    /// //    edge: foo_T --Id--> bar_T
    /// ```
    Identity,
    /// This type of edge from type formal T1 to T2 means T2 is instantiated with a type resulted
    /// by applying one or more type constructors to T1 (potentially with other types).
    ///
    /// This is interesting to us as it creates a new (and bigger) type.
    ///
    /// Example:
    /// ```
    /// //    struct Baz<T> {}
    /// //    foo<T>() { bar<Baz<T>>(); return; }
    /// //
    /// //    edge: foo_T --TyConApp(Baz<T>)--> bar_T
    /// ```
    TyConApp(&'a SignatureToken),
}

pub struct InstantiationLoopChecker<'a> {
    module: &'a CompiledModule,

    graph: Graph<Node, Edge<'a>>,
    node_map: HashMap<Node, NodeIndex>,
    func_handle_def_map: HashMap<FunctionHandleIndex, FunctionDefinitionIndex>,
}

impl<'a> InstantiationLoopChecker<'a> {
    fn new(module: &'a CompiledModule) -> Self {
        Self {
            module,
            graph: Graph::new(),
            node_map: HashMap::new(),
            func_handle_def_map: module
                .function_defs()
                .iter()
                .enumerate()
                .map(|(def_idx, def)| (def.function, FunctionDefinitionIndex::new(def_idx as u16)))
                .collect(),
        }
    }

    pub fn verify_module(module: &'a CompiledModule) -> VMResult<()> {
        Self::verify_module_impl(module).map_err(|e| e.finish(Location::Module(module.self_id())))
    }

    fn verify_module_impl(module: &'a CompiledModule) -> PartialVMResult<()> {
        let mut checker = Self::new(module);
        checker.build_graph();
        let mut components = checker.find_non_trivial_components();

        match components.pop() {
            None => Ok(()),
            Some((nodes, edges)) => {
                let msg_edges = edges
                    .into_iter()
                    .filter_map(
                        |edge_idx| match checker.graph.edge_weight(edge_idx).unwrap() {
                            Edge::TyConApp(_) => Some(checker.format_edge(edge_idx)),
                            _ => None,
                        },
                    )
                    .collect::<Vec<_>>()
                    .join(", ");
                let msg_nodes = nodes
                    .into_iter()
                    .map(|node_idx| checker.format_node(node_idx))
                    .collect::<Vec<_>>()
                    .join(", ");
                let msg = format!(
                    "edges with constructors: [{}], nodes: [{}]",
                    msg_edges, msg_nodes
                );
                Err(PartialVMError::new(StatusCode::LOOP_IN_INSTANTIATION_GRAPH).with_message(msg))
            }
        }
    }

    /// Retrives the node corresponding to the specified type formal.
    /// If none exists in the graph yet, create one.
    fn get_or_add_node(&mut self, node: Node) -> NodeIndex {
        match self.node_map.entry(node) {
            hash_map::Entry::Occupied(entry) => *entry.get(),
            hash_map::Entry::Vacant(entry) => {
                let idx = self.graph.add_node(node);
                entry.insert(idx);
                idx
            }
        }
    }

    /// Helper function that extracts type parameters from a given type.
    /// Duplicated entries are removed.
    fn extract_type_parameters(&self, ty: &SignatureToken) -> HashSet<TypeParameterIndex> {
        use SignatureToken::*;

        let mut type_params = HashSet::new();

        fn rec(type_params: &mut HashSet<TypeParameterIndex>, ty: &SignatureToken) {
            match ty {
                Bool | Address | U8 | U64 | U128 | Signer | Struct(_) => (),
                TypeParameter(idx) => {
                    type_params.insert(*idx);
                }
                Vector(ty) => rec(type_params, ty),
                Reference(ty) | MutableReference(ty) => rec(type_params, ty),
                StructInstantiation(_, tys) => {
                    for ty in tys {
                        rec(type_params, ty);
                    }
                }
            }
        }

        rec(&mut type_params, ty);
        type_params
    }

    /// Helper function that creates an edge from one given node to the other.
    /// If a node does not exist, create one.
    fn add_edge(&mut self, node_from: Node, node_to: Node, edge: Edge<'a>) {
        let node_from_idx = self.get_or_add_node(node_from);
        let node_to_idx = self.get_or_add_node(node_to);
        self.graph.add_edge(node_from_idx, node_to_idx, edge);
    }

    /// Helper of 'fn build_graph' that inspects a function call. If type parameters of the caller
    /// appear in the type actuals to the callee, nodes and edges are added to the graph.
    fn build_graph_call(
        &mut self,
        caller_idx: FunctionDefinitionIndex,
        callee_idx: FunctionDefinitionIndex,
        type_actuals_idx: SignatureIndex,
    ) {
        let type_actuals = &self.module.signature_at(type_actuals_idx).0;

        for (formal_idx, ty) in type_actuals.iter().enumerate() {
            let formal_idx = formal_idx as TypeParameterIndex;
            match ty {
                SignatureToken::TypeParameter(actual_idx) => self.add_edge(
                    Node(caller_idx, *actual_idx),
                    Node(callee_idx, formal_idx),
                    Edge::Identity,
                ),
                _ => {
                    for type_param in self.extract_type_parameters(ty) {
                        self.add_edge(
                            Node(caller_idx, type_param),
                            Node(callee_idx, formal_idx),
                            Edge::TyConApp(&ty),
                        );
                    }
                }
            }
        }
    }

    /// Helper of `fn build_graph` that inspects a function definition for calls between two generic
    /// functions defined in the current module.
    fn build_graph_function_def(
        &mut self,
        caller_idx: FunctionDefinitionIndex,
        caller_def: &FunctionDefinition,
    ) {
        if let Some(code) = &caller_def.code {
            for instr in &code.code {
                if let Bytecode::CallGeneric(callee_inst_idx) = instr {
                    // Get the id of the definition of the function being called.
                    // Skip if the function is not defined in the current module, as we do not
                    // have mutual recursions across module boundaries.
                    let callee_si = self.module.function_instantiation_at(*callee_inst_idx);
                    if let Some(callee_idx) = self.func_handle_def_map.get(&callee_si.handle) {
                        let callee_idx = *callee_idx;
                        self.build_graph_call(caller_idx, callee_idx, callee_si.type_parameters)
                    }
                }
            }
        }
    }

    /// Builds a graph G such that
    ///   - Each type formal of a generic function is a node in G.
    ///   - There is an edge from type formal f_T to g_T if f_T is used to instantiate g_T in a
    ///     call.
    ///     - Each edge is labeled either `Identity` or `TyConApp`. See `Edge` for details.
    fn build_graph(&mut self) {
        for (def_idx, func_def) in self
            .module
            .function_defs()
            .iter()
            .filter(|def| !def.is_native())
            .enumerate()
        {
            self.build_graph_function_def(FunctionDefinitionIndex::new(def_idx as u16), func_def)
        }
    }

    /// Computes the strongly connected components of the graph built and keep the ones that
    /// contain at least one `TyConApp` edge. Such components indicate there exists a loop such
    /// that an input type can get "bigger" infinitely many times along the loop, also creating
    /// infinitely many types. This is precisely the kind of constructs we want to forbid.
    fn find_non_trivial_components(&self) -> Vec<(Vec<NodeIndex>, Vec<EdgeIndex>)> {
        tarjan_scc(&self.graph)
            .into_iter()
            .filter_map(move |nodes| {
                let node_set: HashSet<_> = nodes.iter().cloned().collect();

                let edges: Vec<_> = nodes
                    .iter()
                    .flat_map(|node_idx| {
                        self.graph.edges(*node_idx).filter_map(|edge| {
                            if node_set.contains(&edge.target()) {
                                Some(edge.id())
                            } else {
                                None
                            }
                        })
                    })
                    .collect();

                if edges.iter().any(
                    |edge_idx| match self.graph.edge_weight(*edge_idx).unwrap() {
                        Edge::Identity => false,
                        Edge::TyConApp(_) => true,
                    },
                ) {
                    Some((nodes, edges))
                } else {
                    None
                }
            })
            .collect()
    }

    fn format_node(&self, node_idx: NodeIndex) -> String {
        let Node(def_idx, param_idx) = self.graph.node_weight(node_idx).unwrap();
        format!("f{}#{}", def_idx, param_idx)
    }

    fn format_edge(&self, edge_idx: EdgeIndex) -> String {
        let (node_idx_1, node_idx_2) = self.graph.edge_endpoints(edge_idx).unwrap();
        let node_1 = self.format_node(node_idx_1);
        let node_2 = self.format_node(node_idx_2);

        match self.graph.edge_weight(edge_idx).unwrap() {
            Edge::TyConApp(ty) => format!("{} --{:?}--> {}", node_1, ty, node_2,),
            Edge::Identity => format!("{} ----> {}", node_1, node_2),
        }
    }
}

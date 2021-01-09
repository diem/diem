// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use petgraph::{
    dot::Dot,
    graph::{Graph, NodeIndex},
    Direction,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use move_core_types::{identifier::Identifier, language_storage::ModuleId};
use vm::{
    access::{ModuleAccess, ScriptAccess},
    file_format::{Bytecode, CompiledModule, CompiledScript},
};

struct VizEdge {}

impl fmt::Display for VizEdge {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

#[derive(Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord)]
enum VizNode {
    Function(ModuleId, Identifier),
    Script(String),
}

impl fmt::Display for VizNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Function(module_id, function_name) => {
                writeln!(f, "{}::{}", module_id.name(), function_name)
            }
            Self::Script(script_name) => writeln!(f, "{}", script_name),
        }
    }
}

pub struct VizGraph {
    graph: Graph<VizNode, VizEdge>,
    nodes: BTreeMap<VizNode, NodeIndex>,
}

impl VizGraph {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            nodes: BTreeMap::new(),
        }
    }

    pub fn add_script(&mut self, script_name: String) {
        let node = VizNode::Script(script_name);
        let exists = self.nodes.insert(node.clone(), self.graph.add_node(node));
        assert!(exists.is_none());
    }

    pub fn add_function(&mut self, module_id: ModuleId, function_name: Identifier) {
        let node = VizNode::Function(module_id, function_name);
        let exists = self.nodes.insert(node.clone(), self.graph.add_node(node));
        assert!(exists.is_none());
    }

    pub fn add_call_from_script(
        &mut self,
        script_name: String,
        callee_module_id: ModuleId,
        callee_function_name: Identifier,
    ) {
        let node_src = VizNode::Script(script_name);
        let index_src = self.nodes.get(&node_src).unwrap();
        let node_dst = VizNode::Function(callee_module_id, callee_function_name);
        if let Some(index_dst) = self.nodes.get(&node_dst) {
            self.graph.update_edge(*index_src, *index_dst, VizEdge {});
        }
    }

    pub fn add_call_from_function(
        &mut self,
        module_id: ModuleId,
        function_name: Identifier,
        callee_module_id: ModuleId,
        callee_function_name: Identifier,
    ) {
        // don't add self edges
        if module_id == callee_module_id {
            return;
        }

        let node_src = VizNode::Function(module_id, function_name);
        let index_src = self.nodes.get(&node_src).unwrap();
        let node_dst = VizNode::Function(callee_module_id, callee_function_name);
        if let Some(index_dst) = self.nodes.get(&node_dst) {
            self.graph.update_edge(*index_src, *index_dst, VizEdge {});
        }
    }

    pub fn prune(&mut self) {
        self.graph
            .retain_nodes(|graph, index| graph.neighbors_undirected(index).count() != 0);

        self.nodes = self
            .graph
            .node_indices()
            .map(|index| (self.graph.node_weight(index).unwrap().clone(), index))
            .collect();
    }

    pub fn to_dot(&self) -> String {
        format!(
            "{}",
            Dot::with_attr_getters(&self.graph, &[], &|_, _| "".to_string(), &|_, (_, node)| {
                match node {
                    VizNode::Script(_) => "".to_string(),
                    VizNode::Function(_, _) => "shape=box".to_string(),
                }
            })
        )
    }

    pub fn friends(
        &self,
    ) -> (
        BTreeMap<ModuleId, BTreeSet<ModuleId>>,               // m2m
        BTreeMap<ModuleId, BTreeSet<(ModuleId, Identifier)>>, // m2f
        BTreeMap<(ModuleId, Identifier), BTreeSet<ModuleId>>, // f2m
        BTreeMap<(ModuleId, Identifier), BTreeSet<(ModuleId, Identifier)>>, // f2f
        BTreeMap<(ModuleId, Identifier), BTreeSet<String>>,   // script funs
    ) {
        let mut result_m2m = BTreeMap::new();
        let mut result_m2f = BTreeMap::new();
        let mut result_f2m = BTreeMap::new();
        let mut result_f2f = BTreeMap::new();
        let mut result_script_funs = BTreeMap::new();

        for index in self.graph.node_indices() {
            let node = self.graph.node_weight(index).unwrap();
            match node {
                VizNode::Function(module_id, function_name) => {
                    let fl_m2m = result_m2m
                        .entry(module_id.clone())
                        .or_insert_with(BTreeSet::new);
                    let fl_m2f = result_m2f
                        .entry(module_id.clone())
                        .or_insert_with(BTreeSet::new);
                    let fl_f2m = result_f2m
                        .entry((module_id.clone(), function_name.clone()))
                        .or_insert_with(BTreeSet::new);
                    let fl_f2f = result_f2f
                        .entry((module_id.clone(), function_name.clone()))
                        .or_insert_with(BTreeSet::new);
                    let fl_script = result_script_funs
                        .entry((module_id.clone(), function_name.clone()))
                        .or_insert_with(BTreeSet::new);

                    for incoming in self.graph.neighbors_directed(index, Direction::Incoming) {
                        let caller = self.graph.node_weight(incoming).unwrap();
                        match caller {
                            VizNode::Function(caller_module_id, caller_function_name) => {
                                fl_m2m.insert(caller_module_id.clone());
                                fl_m2f.insert((
                                    caller_module_id.clone(),
                                    caller_function_name.clone(),
                                ));
                                fl_f2m.insert(caller_module_id.clone());
                                fl_f2f.insert((
                                    caller_module_id.clone(),
                                    caller_function_name.clone(),
                                ));
                            }
                            VizNode::Script(caller_script_name) => {
                                fl_script.insert(caller_script_name.clone());
                            }
                        }
                    }
                }
                VizNode::Script(_) => {}
            }
        }

        (
            result_m2m,
            result_m2f,
            result_f2m,
            result_f2f,
            result_script_funs,
        )
    }
}

pub(crate) trait Package {
    // interfaces to get modules and scripts
    fn get_modules(&self) -> &[CompiledModule];
    fn get_scripts(&self) -> &[(String, CompiledScript)];

    // dependency graph construction
    fn build_viz_graph(&self) -> VizGraph {
        let mut graph = VizGraph::new();

        // add nodes from script
        for (script_name, _) in self.get_scripts() {
            graph.add_script(script_name.clone());
        }

        // add nodes from modules
        for module in self.get_modules() {
            let module_id = module.self_id();
            for function_def in module.function_defs() {
                graph.add_function(
                    module_id.clone(),
                    module
                        .identifier_at(module.function_handle_at(function_def.function).name)
                        .to_owned(),
                );
            }
        }

        // add edges from script
        for (script_name, script) in self.get_scripts() {
            for bytecode in &script.code().code {
                let callee_index_opt = match bytecode {
                    Bytecode::Call(callee_index) => Some(*callee_index),
                    Bytecode::CallGeneric(callee_inst_index) => {
                        Some(script.function_instantiation_at(*callee_inst_index).handle)
                    }
                    _ => None,
                };
                if callee_index_opt.is_none() {
                    continue;
                }

                let callee_handle = script.function_handle_at(callee_index_opt.unwrap());
                let callee_module_handle = script.module_handle_at(callee_handle.module);
                let callee_module_id = ModuleId::new(
                    *script.address_identifier_at(callee_module_handle.address),
                    script.identifier_at(callee_module_handle.name).to_owned(),
                );
                let callee_function_name = script.identifier_at(callee_handle.name).to_owned();

                graph.add_call_from_script(
                    script_name.clone(),
                    callee_module_id,
                    callee_function_name,
                );
            }
        }

        // add edges from modules
        for module in self.get_modules() {
            for function_def in module.function_defs() {
                if let Some(code_unit) = &function_def.code {
                    let function_name =
                        module.identifier_at(module.function_handle_at(function_def.function).name);

                    for bytecode in &code_unit.code {
                        let callee_index_opt = match bytecode {
                            Bytecode::Call(callee_index) => Some(*callee_index),
                            Bytecode::CallGeneric(callee_inst_index) => {
                                Some(module.function_instantiation_at(*callee_inst_index).handle)
                            }
                            _ => None,
                        };
                        if callee_index_opt.is_none() {
                            continue;
                        }

                        let callee_handle = module.function_handle_at(callee_index_opt.unwrap());
                        let callee_module_handle = module.module_handle_at(callee_handle.module);
                        let callee_module_id = ModuleId::new(
                            *module.address_identifier_at(callee_module_handle.address),
                            module.identifier_at(callee_module_handle.name).to_owned(),
                        );
                        let callee_function_name =
                            module.identifier_at(callee_handle.name).to_owned();

                        graph.add_call_from_function(
                            module.self_id(),
                            function_name.to_owned(),
                            callee_module_id,
                            callee_function_name,
                        );
                    }
                }
            }
        }

        // remove redundant nodes before returning it
        graph.prune();
        graph
    }
}

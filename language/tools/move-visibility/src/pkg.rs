// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use petgraph::{
    dot::Dot,
    graph::{Graph, NodeIndex},
};
use std::{collections::BTreeMap, fmt};

use move_core_types::{identifier::Identifier, language_storage::ModuleId};
use vm::{
    access::{ModuleAccess, ScriptAccess},
    file_format::{Bytecode, CompiledModule, CompiledScript},
};

struct Edge {}

impl fmt::Display for Edge {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

// module-level granularity
#[derive(Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord)]
enum NodeModuleOrScript {
    Module(ModuleId),
    Script(String),
}

impl fmt::Display for NodeModuleOrScript {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NodeModuleOrScript::Module(module_id) => writeln!(f, "{}", module_id.name()),
            NodeModuleOrScript::Script(script_name) => writeln!(f, "{}", script_name),
        }
    }
}

pub struct GraphModuleOrScript {
    graph: Graph<NodeModuleOrScript, Edge>,
    nodes: BTreeMap<NodeModuleOrScript, NodeIndex>,
}

impl GraphModuleOrScript {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            nodes: BTreeMap::new(),
        }
    }

    pub fn add_script(&mut self, script_name: String) {
        let node = NodeModuleOrScript::Script(script_name);
        let exists = self.nodes.insert(node.clone(), self.graph.add_node(node));
        assert!(exists.is_none());
    }

    pub fn add_module(&mut self, module_id: ModuleId) {
        let node = NodeModuleOrScript::Module(module_id);
        let exists = self.nodes.insert(node.clone(), self.graph.add_node(node));
        assert!(exists.is_none());
    }

    pub fn add_call_from_script(&mut self, script_name: String, callee_module_id: ModuleId) {
        let node_src = NodeModuleOrScript::Script(script_name);
        let index_src = self.nodes.get(&node_src).unwrap();
        let node_dst = NodeModuleOrScript::Module(callee_module_id);
        if let Some(index_dst) = self.nodes.get(&node_dst) {
            self.graph.update_edge(*index_src, *index_dst, Edge {});
        }
    }

    pub fn add_call_from_module(&mut self, module_id: ModuleId, callee_module_id: ModuleId) {
        // don't add self edges
        if module_id == callee_module_id {
            return;
        }

        let node_src = NodeModuleOrScript::Module(module_id);
        let index_src = self.nodes.get(&node_src).unwrap();
        let node_dst = NodeModuleOrScript::Module(callee_module_id);
        if let Some(index_dst) = self.nodes.get(&node_dst) {
            self.graph.update_edge(*index_src, *index_dst, Edge {});
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
                    NodeModuleOrScript::Script(_) => "".to_string(),
                    NodeModuleOrScript::Module(_) => "shape=box".to_string(),
                }
            })
        )
    }
}

// function-level granularity
#[derive(Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord)]
enum NodeFunctionOrScript {
    Function(ModuleId, Identifier),
    Script(String),
}

impl fmt::Display for NodeFunctionOrScript {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NodeFunctionOrScript::Function(module_id, function_name) => {
                writeln!(f, "{}::{}", module_id.name(), function_name)
            }
            NodeFunctionOrScript::Script(script_name) => writeln!(f, "{}", script_name),
        }
    }
}

pub struct GraphFunctionOrScript {
    graph: Graph<NodeFunctionOrScript, Edge>,
    nodes: BTreeMap<NodeFunctionOrScript, NodeIndex>,
}

impl GraphFunctionOrScript {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            nodes: BTreeMap::new(),
        }
    }

    pub fn add_script(&mut self, script_name: String) {
        let node = NodeFunctionOrScript::Script(script_name);
        let exists = self.nodes.insert(node.clone(), self.graph.add_node(node));
        assert!(exists.is_none());
    }

    pub fn add_function(&mut self, module_id: ModuleId, function_name: Identifier) {
        let node = NodeFunctionOrScript::Function(module_id, function_name);
        let exists = self.nodes.insert(node.clone(), self.graph.add_node(node));
        assert!(exists.is_none());
    }

    pub fn add_call_from_script(
        &mut self,
        script_name: String,
        callee_module_id: ModuleId,
        callee_function_name: Identifier,
    ) {
        let node_src = NodeFunctionOrScript::Script(script_name);
        let index_src = self.nodes.get(&node_src).unwrap();
        let node_dst = NodeFunctionOrScript::Function(callee_module_id, callee_function_name);
        if let Some(index_dst) = self.nodes.get(&node_dst) {
            self.graph.update_edge(*index_src, *index_dst, Edge {});
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

        let node_src = NodeFunctionOrScript::Function(module_id, function_name);
        let index_src = self.nodes.get(&node_src).unwrap();
        let node_dst = NodeFunctionOrScript::Function(callee_module_id, callee_function_name);
        if let Some(index_dst) = self.nodes.get(&node_dst) {
            self.graph.update_edge(*index_src, *index_dst, Edge {});
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
                    NodeFunctionOrScript::Script(_) => "".to_string(),
                    NodeFunctionOrScript::Function(_, _) => "shape=box".to_string(),
                }
            })
        )
    }
}

pub(crate) trait Package {
    // interfaces to get modules and scripts
    fn get_modules(&self) -> &[CompiledModule];
    fn get_scripts(&self) -> &[(String, CompiledScript)];

    // dependency graph construction
    fn build_dep_graph(&self) -> (GraphModuleOrScript, GraphFunctionOrScript) {
        // maintain both module-level and function-level dependency graphs
        let mut graph_module = GraphModuleOrScript::new();
        let mut graph_function = GraphFunctionOrScript::new();

        // add nodes from script
        for (script_name, _) in self.get_scripts() {
            graph_module.add_script(script_name.clone());
            graph_function.add_script(script_name.clone());
        }

        // add nodes from modules
        for module in self.get_modules() {
            let module_id = module.self_id();
            graph_module.add_module(module_id.clone());
            for function_def in module.function_defs() {
                graph_function.add_function(
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

                graph_module.add_call_from_script(script_name.clone(), callee_module_id.clone());
                graph_function.add_call_from_script(
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

                        graph_module
                            .add_call_from_module(module.self_id(), callee_module_id.clone());
                        graph_function.add_call_from_function(
                            module.self_id(),
                            function_name.to_owned(),
                            callee_module_id,
                            callee_function_name,
                        );
                    }
                }
            }
        }

        // remove redudant nodes
        graph_module.prune();
        graph_function.prune();

        // return both dependency graphs
        (graph_module, graph_function)
    }
}

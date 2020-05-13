// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    dataflow_analysis::{
        AbstractDomain, DataflowAnalysis, JoinResult, StateMap, TransferFunctions,
    },
    function_target::{FunctionTarget, FunctionTargetData},
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder},
    livevar_analysis::LiveVarAnnotation,
    stackless_bytecode::{AssignKind, BorrowNode, Bytecode, Operation, StructDecl, TempIndex},
    stackless_control_flow_graph::{BlockId, StacklessControlFlowGraph},
};
use borrow_graph::references::RefID;
use itertools::Itertools;
use spec_lang::env::FunctionEnv;
use std::collections::{BTreeMap, BTreeSet};
use vm::file_format::CodeOffset;

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
struct Field {
    struct_decl: StructDecl,
    field_offset: usize,
}

type BorrowGraph = borrow_graph::graph::BorrowGraph<(), Field>;

#[derive(Debug, Clone)]
pub struct BorrowInfo {
    pub live_refs: BTreeSet<TempIndex>,
    pub borrowed_by: BTreeMap<BorrowNode, BTreeSet<BorrowNode>>,
    pub borrows_from: BTreeMap<BorrowNode, BTreeSet<BorrowNode>>,
}

impl BorrowInfo {
    pub fn all_refs(&self) -> BTreeSet<TempIndex> {
        let filter_fn = |node: &BorrowNode| {
            if let BorrowNode::Reference(idx) = node {
                Some(*idx)
            } else {
                None
            }
        };
        let borrowed_by_refs = self.borrowed_by.keys().filter_map(filter_fn).collect();
        self.live_refs.union(&borrowed_by_refs).cloned().collect()
    }

    pub fn is_empty(&self) -> bool {
        self.live_refs.is_empty() && self.borrowed_by.is_empty() && self.borrows_from.is_empty()
    }

    pub fn borrow_info_str(&self, func_target: &FunctionTarget<'_>) -> String {
        let live_refs_str = format!(
            "live_refs: {}",
            self.live_refs
                .iter()
                .map(|idx| {
                    let name = func_target.get_local_name(*idx);
                    format!("{}", name.display(func_target.symbol_pool()),)
                })
                .join(", ")
        );
        let borrows_str = |(node, borrows): (&BorrowNode, &BTreeSet<BorrowNode>)| {
            format!(
                "{} -> {{{}}}",
                node.display(func_target),
                borrows
                    .iter()
                    .map(|borrow| borrow.display(func_target))
                    .join(", ")
            )
        };
        let borrowed_by_str = format!(
            "borrowed_by: {}",
            self.borrowed_by.iter().map(borrows_str).join(", ")
        );
        let borrows_from_str = format!(
            "borrows_from: {}",
            self.borrows_from.iter().map(borrows_str).join(", ")
        );
        format!("{} {} {}", live_refs_str, borrowed_by_str, borrows_from_str)
    }
}

pub struct BorrowInfoAtCodeOffset {
    pub before: BorrowInfo,
    pub after: BorrowInfo,
}

/// Borrow annotation computed by the borrow analysis processor.
pub struct BorrowAnnotation(BTreeMap<CodeOffset, BorrowInfoAtCodeOffset>);

impl BorrowAnnotation {
    pub fn get_borrow_info_at(&self, code_offset: CodeOffset) -> Option<&BorrowInfoAtCodeOffset> {
        self.0.get(&code_offset)
    }
}

#[derive(Debug, Clone)]
struct PackError {
    code_offset: CodeOffset,
    indices: Vec<TempIndex>,
}

/// Borrow analysis processor.
pub struct BorrowAnalysisProcessor {}

impl BorrowAnalysisProcessor {
    pub fn new() -> Box<Self> {
        Box::new(BorrowAnalysisProcessor {})
    }
}

impl FunctionTargetProcessor for BorrowAnalysisProcessor {
    fn process(
        &self,
        _targets: &mut FunctionTargetsHolder,
        func_env: &FunctionEnv<'_>,
        mut data: FunctionTargetData,
    ) -> FunctionTargetData {
        let borrow_annotation = if func_env.is_native() {
            // Native functions have no byte code.
            BorrowAnnotation(BTreeMap::new())
        } else {
            let func_target = FunctionTarget::new(func_env, &data);
            let mut analyzer = BorrowAnalysis::new(&func_target);
            BorrowAnnotation(analyzer.analyze(&data.code))
        };
        // Annotate function target with computed borrow data.
        data.annotations.set::<BorrowAnnotation>(borrow_annotation);
        data.annotations.remove::<LiveVarAnnotation>();
        data
    }
}

#[derive(Debug, Clone, PartialEq)]
struct BorrowState {
    borrow_graph: BorrowGraph,
    dead_edges: BTreeMap<BorrowNode, BTreeSet<BorrowNode>>,
}

impl BorrowState {}

struct BorrowAnalysis<'a> {
    func_target: &'a FunctionTarget<'a>,
    livevar_annotation: &'a LiveVarAnnotation,
    ref_id_to_borrow_node: BTreeMap<RefID, BorrowNode>,
    borrow_node_to_ref_id: BTreeMap<BorrowNode, RefID>,
}

impl<'a> BorrowAnalysis<'a> {
    fn new(func_target: &'a FunctionTarget<'a>) -> Self {
        let livevar_annotation = func_target
            .get_annotations()
            .get::<LiveVarAnnotation>()
            .expect("livevar annotation");

        let mut ref_id_to_borrow_node = BTreeMap::new();
        let mut borrow_node_to_ref_id = BTreeMap::new();
        let local_count = func_target.get_local_count();
        let parameter_count = func_target.get_parameter_count();
        for idx in 0..local_count {
            let ref_id = RefID::new(idx);
            let ty = func_target.get_local_type(idx);
            let borrow_node = if ty.is_reference() {
                BorrowNode::Reference(idx)
            } else {
                BorrowNode::LocalRoot(idx)
            };
            ref_id_to_borrow_node.insert(ref_id, borrow_node.clone());
            borrow_node_to_ref_id.insert(borrow_node, ref_id);
            if ty.is_reference() && idx < parameter_count {
                let ref_param_proxy_root_idx = Self::ref_param_proxy_root_idx(func_target, idx);
                let ref_param_proxy_root = RefID::new(ref_param_proxy_root_idx);
                ref_id_to_borrow_node.insert(ref_param_proxy_root, BorrowNode::LocalRoot(idx));
                borrow_node_to_ref_id.insert(BorrowNode::LocalRoot(idx), ref_param_proxy_root);
            }
        }
        let mut next_idx = local_count + parameter_count;
        for struct_id in func_target.get_acquires_global_resources() {
            let ref_id = RefID::new(next_idx);
            let borrow_node = BorrowNode::GlobalRoot(StructDecl {
                module_id: func_target.module_env().get_id(),
                struct_id: *struct_id,
            });
            ref_id_to_borrow_node.insert(ref_id, borrow_node.clone());
            borrow_node_to_ref_id.insert(borrow_node, ref_id);
            next_idx += 1;
        }
        Self {
            func_target,
            livevar_annotation,
            ref_id_to_borrow_node,
            borrow_node_to_ref_id,
        }
    }

    fn analyze(&mut self, instrs: &[Bytecode]) -> BTreeMap<CodeOffset, BorrowInfoAtCodeOffset> {
        let cfg = StacklessControlFlowGraph::new_forward(instrs);
        let mut borrow_graph = BorrowGraph::new();
        for (ref_id, _) in self
            .ref_id_to_borrow_node
            .iter()
            .filter(|(_, node)| match node {
                BorrowNode::Reference(..) => false,
                _ => true,
            })
        {
            borrow_graph.new_ref(*ref_id, true);
        }
        for idx in 0..self.func_target.get_parameter_count() {
            if self.func_target.get_local_type(idx).is_reference() {
                let ref_id = RefID::new(idx);
                borrow_graph.new_ref(ref_id, true);
                borrow_graph.add_strong_borrow(
                    (),
                    RefID::new(Self::ref_param_proxy_root_idx(self.func_target, idx)),
                    ref_id,
                );
            }
        }
        let initial_state = BorrowState {
            borrow_graph,
            dead_edges: BTreeMap::new(),
        };
        let state_map = self.analyze_function(initial_state, instrs, &cfg);
        self.post_process(&cfg, instrs, state_map)
    }

    fn ref_param_proxy_root_idx(func_target: &FunctionTarget, ref_param_idx: usize) -> usize {
        assert!(ref_param_idx < func_target.get_parameter_count());
        func_target.get_local_count() + ref_param_idx
    }

    fn post_process(
        &mut self,
        cfg: &StacklessControlFlowGraph,
        instrs: &[Bytecode],
        state_map: StateMap<BorrowState, PackError>,
    ) -> BTreeMap<CodeOffset, BorrowInfoAtCodeOffset> {
        let mut result = BTreeMap::new();
        for (block_id, block_state) in state_map {
            let mut state = block_state.pre;
            for offset in cfg.instr_indexes(block_id) {
                let instr = &instrs[offset as usize];
                let before = self.convert_state_to_info(&state);
                state = self.execute(state, instr, offset).unwrap();
                let after = self.convert_state_to_info(&state);
                result.insert(offset, BorrowInfoAtCodeOffset { before, after });
            }
        }
        result
    }

    fn convert_state_to_info(&self, borrow_state: &BorrowState) -> BorrowInfo {
        let all_ref_ids = borrow_state.borrow_graph.all_refs();
        let live_refs = (0..self.func_target.get_local_count())
            .filter(|idx| {
                if self.func_target.get_local_type(*idx).is_reference() {
                    all_ref_ids.contains(&RefID::new(*idx))
                } else {
                    false
                }
            })
            .collect();
        let mut borrowed_by = BTreeMap::new();
        let mut borrows_from = BTreeMap::new();
        for ref_id in self.ref_id_to_borrow_node.keys() {
            if all_ref_ids.contains(ref_id) {
                let edges = borrow_state.borrow_graph.out_edges(*ref_id);
                for edge in edges {
                    let src = &self.ref_id_to_borrow_node[ref_id];
                    let dest = &self.ref_id_to_borrow_node[&edge.3];
                    Self::add_edge(&mut borrowed_by, src, dest);
                    Self::add_edge(&mut borrows_from, dest, src);
                }
            }
        }
        for (src, dests) in &borrow_state.dead_edges {
            for dest in dests {
                Self::add_edge(&mut borrowed_by, src, dest);
                Self::add_edge(&mut borrows_from, dest, src);
            }
        }
        BorrowInfo {
            live_refs,
            borrowed_by,
            borrows_from,
        }
    }

    fn add_edge(
        edges: &mut BTreeMap<BorrowNode, BTreeSet<BorrowNode>>,
        at: &BorrowNode,
        node: &BorrowNode,
    ) {
        edges.entry(at.clone()).or_insert_with(BTreeSet::new);
        edges.entry(at.clone()).and_modify(|x| {
            x.insert(node.clone());
        });
    }

    fn add_edges(
        edges: &mut BTreeMap<BorrowNode, BTreeSet<BorrowNode>>,
        other: &BTreeMap<BorrowNode, BTreeSet<BorrowNode>>,
    ) {
        for x in other.keys() {
            for y in &other[x] {
                BorrowAnalysis::add_edge(edges, x, y);
            }
        }
    }

    fn is_subset(
        other: &BTreeMap<BorrowNode, BTreeSet<BorrowNode>>,
        edges: &BTreeMap<BorrowNode, BTreeSet<BorrowNode>>,
    ) -> bool {
        other
            .iter()
            .all(|(x, y)| edges.contains_key(x) && y.is_subset(&edges[x]))
    }

    fn error_indices(&self, state: &BorrowState, dests: Vec<TempIndex>) -> Vec<TempIndex> {
        let mut indices = vec![];
        let all_refs = state.borrow_graph.all_refs();
        for idx in dests {
            if self.func_target.get_local_type(idx).is_reference()
                && (all_refs.contains(&RefID::new(idx))
                    || state.dead_edges.contains_key(&BorrowNode::Reference(idx)))
            {
                indices.push(idx);
            }
        }
        indices
    }

    fn remap_borrow_node(&self, node: &BorrowNode, id_map: &BTreeMap<RefID, RefID>) -> BorrowNode {
        match node {
            BorrowNode::Reference(idx) => {
                let ref_id = RefID::new(*idx);
                let node = if id_map.contains_key(&ref_id) {
                    &self.ref_id_to_borrow_node[&id_map[&ref_id]]
                } else {
                    node
                };
                node.clone()
            }
            _ => node.clone(),
        }
    }

    fn remap_edges(
        &self,
        edges: &mut BTreeMap<BorrowNode, BTreeSet<BorrowNode>>,
        id_map: &BTreeMap<RefID, RefID>,
    ) {
        *edges = edges
            .iter()
            .map(|(src, dests)| {
                (
                    self.remap_borrow_node(src, id_map),
                    dests
                        .iter()
                        .map(|dest| self.remap_borrow_node(dest, id_map))
                        .collect(),
                )
            })
            .collect();
    }

    fn execute(
        &mut self,
        pre: BorrowState,
        instr: &Bytecode,
        code_offset: CodeOffset,
    ) -> Result<BorrowState, PackError> {
        use Bytecode::*;
        let mut post = pre;

        // error if any unpacked ref (live or dead) is being overwritten by the instruction
        let mut indices = vec![];
        match instr {
            Assign(_, dest, _, _) => indices.push(*dest),
            Call(_, dests, _, _) => indices = dests.clone(),
            _ => {}
        };
        indices = self.error_indices(&post, indices);
        if !indices.is_empty() {
            return Err(PackError {
                code_offset,
                indices,
            });
        }

        // apply changes to post.borrow_graph based on the instruction
        match instr {
            Assign(_, dest, src, kind) => {
                if self.func_target.get_local_type(*dest).is_reference() {
                    let dest_ref_id = RefID::new(*dest);
                    let src_ref_id = RefID::new(*src);
                    match kind {
                        AssignKind::Move | AssignKind::Store => {
                            let mut id_map = BTreeMap::new();
                            id_map.insert(src_ref_id, dest_ref_id);
                            post.borrow_graph.remap_refs(&id_map);
                            self.remap_edges(&mut post.dead_edges, &id_map);
                        }
                        AssignKind::Copy => {
                            post.borrow_graph.new_ref(dest_ref_id, true);
                            post.borrow_graph
                                .add_strong_borrow((), src_ref_id, dest_ref_id);
                        }
                    }
                }
            }
            Call(_, dests, oper, srcs) => {
                use Operation::*;
                match oper {
                    BorrowLoc => {
                        let dest = dests[0];
                        let dest_ref_id = RefID::new(dest);
                        let src = srcs[0];
                        let src_ref_id = RefID::new(src);
                        post.borrow_graph.new_ref(dest_ref_id, true);
                        post.borrow_graph
                            .add_strong_borrow((), src_ref_id, dest_ref_id);
                    }
                    BorrowGlobal(mid, sid, _) => {
                        let dest = dests[0];
                        let dest_ref_id = RefID::new(dest);
                        let src_borrow_node = BorrowNode::GlobalRoot(StructDecl {
                            module_id: *mid,
                            struct_id: *sid,
                        });
                        let src_ref_id = self.borrow_node_to_ref_id[&src_borrow_node];
                        post.borrow_graph.new_ref(dest_ref_id, true);
                        post.borrow_graph
                            .add_strong_borrow((), src_ref_id, dest_ref_id);
                    }
                    BorrowField(mid, sid, _, offset) => {
                        let dest = dests[0];
                        let dest_ref_id = RefID::new(dest);
                        let src = srcs[0];
                        let src_ref_id = RefID::new(src);
                        post.borrow_graph.new_ref(dest_ref_id, true);
                        post.borrow_graph.add_strong_field_borrow(
                            (),
                            src_ref_id,
                            Field {
                                struct_decl: StructDecl {
                                    module_id: *mid,
                                    struct_id: *sid,
                                },
                                field_offset: *offset,
                            },
                            dest_ref_id,
                        );
                    }
                    Function(..) => {
                        for src in srcs
                            .iter()
                            .filter(|idx| self.func_target.get_local_type(**idx).is_reference())
                        {
                            let src_ref_id = RefID::new(*src);
                            for dest in dests
                                .iter()
                                .filter(|idx| self.func_target.get_local_type(**idx).is_reference())
                            {
                                let dest_ref_id = RefID::new(*dest);
                                post.borrow_graph.new_ref(dest_ref_id, true);
                                post.borrow_graph
                                    .add_weak_borrow((), src_ref_id, dest_ref_id);
                            }
                        }
                    }
                    _ => {
                        // Other operations do not create references
                    }
                }
            }
            _ => {
                // Other instructions do not create references
            }
        }

        // copy outgoing edges from dying refs in post.borrow_graph to post.dead_edges
        // and release dying refs
        let livevar_annotation_at = self
            .livevar_annotation
            .get_live_var_info_at(code_offset)
            .unwrap();
        for idx in livevar_annotation_at
            .before
            .difference(&livevar_annotation_at.after)
        {
            if self.func_target.get_local_type(*idx).is_reference() {
                let ref_id = RefID::new(*idx);
                if post.borrow_graph.contains_id(ref_id) {
                    let borrow_node = &self.ref_id_to_borrow_node[&ref_id];
                    let (full_borrows, field_borrows) = post.borrow_graph.borrowed_by(ref_id);
                    for dest in full_borrows.keys() {
                        Self::add_edge(
                            &mut post.dead_edges,
                            borrow_node,
                            &self.ref_id_to_borrow_node[dest],
                        );
                    }
                    for edges in field_borrows.values() {
                        for dest in edges.keys() {
                            Self::add_edge(
                                &mut post.dead_edges,
                                borrow_node,
                                &self.ref_id_to_borrow_node[dest],
                            );
                        }
                    }
                    if !post.dead_edges.contains_key(borrow_node) {
                        // add empty set if necessary to bootstrap the cleanup below
                        post.dead_edges.insert(borrow_node.clone(), BTreeSet::new());
                    }
                    post.borrow_graph.release(ref_id);
                }
            }
        }

        // clean up post.dead_edges: remove nodes until every node has at least one outgoing edge
        let mut dead_edges = std::mem::take(&mut post.dead_edges);
        loop {
            let (new_dead_edges, leaf_nodes) = Self::remove_leaves(dead_edges);
            dead_edges = new_dead_edges;
            if leaf_nodes.is_empty() {
                break;
            }
            for (_, y) in dead_edges.iter_mut() {
                for node in &leaf_nodes {
                    y.remove(node);
                }
            }
        }
        post.dead_edges = dead_edges;

        Ok(post)
    }

    fn remove_leaves(
        dead_edges: BTreeMap<BorrowNode, BTreeSet<BorrowNode>>,
    ) -> (BTreeMap<BorrowNode, BTreeSet<BorrowNode>>, Vec<BorrowNode>) {
        let mut new_dead_edges = BTreeMap::new();
        let mut leaf_nodes = vec![];
        for (x, y) in dead_edges {
            if y.is_empty() {
                leaf_nodes.push(x);
            } else {
                new_dead_edges.insert(x, y);
            }
        }
        (new_dead_edges, leaf_nodes)
    }
}

impl<'a> TransferFunctions for BorrowAnalysis<'a> {
    type State = BorrowState;
    type AnalysisError = PackError;

    fn execute_block(
        &mut self,
        block_id: BlockId,
        pre_state: Self::State,
        instrs: &[Bytecode],
        cfg: &StacklessControlFlowGraph,
    ) -> Result<Self::State, Self::AnalysisError> {
        let mut state = pre_state;
        for offset in cfg.instr_indexes(block_id) {
            let instr = &instrs[offset as usize];
            state = self.execute(state, instr, offset)?;
        }
        Ok(state)
    }
}

impl<'a> DataflowAnalysis for BorrowAnalysis<'a> {}

impl AbstractDomain for BorrowState {
    fn join(&mut self, other: &Self) -> JoinResult {
        let borrow_graph_unchanged = self.borrow_graph.leq(&other.borrow_graph);
        if !borrow_graph_unchanged {
            self.borrow_graph = self.borrow_graph.join(&other.borrow_graph);
        }
        let dead_edges_unchanged = BorrowAnalysis::is_subset(&other.dead_edges, &self.dead_edges);
        if !dead_edges_unchanged {
            BorrowAnalysis::add_edges(&mut self.dead_edges, &other.dead_edges);
        }
        if dead_edges_unchanged && borrow_graph_unchanged {
            JoinResult::Unchanged
        } else {
            JoinResult::Changed
        }
    }
}

// =================================================================================================
// Formatting

/// Format a borrow annotation.
pub fn format_borrow_annotation(
    func_target: &FunctionTarget<'_>,
    code_offset: CodeOffset,
) -> Option<String> {
    if let Some(BorrowAnnotation(map)) = func_target.get_annotations().get::<BorrowAnnotation>() {
        if let Some(map_at) = map.get(&code_offset) {
            if !map_at.before.is_empty() {
                return Some(map_at.before.borrow_info_str(func_target));
            }
        }
    }
    None
}

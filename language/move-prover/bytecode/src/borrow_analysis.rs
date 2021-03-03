// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Data flow analysis computing borrow information for preparation of memory_instrumentation.

use crate::{
    dataflow_analysis::{
        AbstractDomain, DataflowAnalysis, JoinResult, SetDomain, TransferFunctions,
    },
    function_target::{FunctionData, FunctionTarget},
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder},
    livevar_analysis::LiveVarAnnotation,
    stackless_bytecode::{AssignKind, BorrowEdge, BorrowNode, Bytecode, Operation, StrongEdge},
    stackless_control_flow_graph::StacklessControlFlowGraph,
};
use itertools::Itertools;
use move_binary_format::file_format::CodeOffset;
use move_model::{
    ast::TempIndex,
    model::{FunctionEnv, GlobalEnv, QualifiedInstId},
};
use std::{
    borrow::BorrowMut,
    collections::{BTreeMap, BTreeSet},
};

/// Borrow graph edge abstract domain.
/// `Top` corresponds to a weak edge existing.
#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub enum EdgeDomain {
    Top,
    EdgeSet(SetDomain<StrongEdge>),
}

impl std::fmt::Display for EdgeDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EdgeDomain::Top => write!(f, "*"),
            EdgeDomain::EdgeSet(edgeset) => write!(
                f,
                "{}",
                edgeset.iter().map(|edge| format!("{}", edge)).join(", ")
            ),
        }
    }
}

impl AbstractDomain for EdgeDomain {
    fn join(&mut self, other: &Self) -> JoinResult {
        match self {
            Self::Top => JoinResult::Unchanged,
            Self::EdgeSet(edges1) => match other {
                Self::Top => {
                    *self = Self::Top;
                    JoinResult::Changed
                }
                Self::EdgeSet(edges2) => edges1.join(edges2),
            },
        }
    }
}

impl std::default::Default for EdgeDomain {
    fn default() -> Self {
        Self::EdgeSet(SetDomain::default())
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Default)]
pub struct BorrowInfo {
    /// Contains the nodes which are alive. This excludes nodes which are alive because
    /// other nodes which are alive borrow from them.
    live_nodes: SetDomain<BorrowNode>,

    /// Contains the nodes which are unchecked regards their pack/unpack invariant.
    /// These are nodes derived from &mut parameters of private functions for which we do
    /// not perform pack/unpack.
    unchecked_nodes: SetDomain<BorrowNode>,

    /// Contains the nodes which have been updated via a Splice operation.
    spliced_nodes: SetDomain<BorrowNode>,

    /// Contains the nodes which have been moved via a move instruction.
    moved_nodes: SetDomain<BorrowNode>,

    /// Forward borrow information.
    borrowed_by: BTreeMap<BorrowNode, BTreeMap<BorrowNode, EdgeDomain>>,

    /// Backward borrow information. This field is not used during analysis, but computed once
    /// analysis is done.
    borrows_from: BTreeMap<BorrowNode, BTreeMap<BorrowNode, EdgeDomain>>,
}

impl BorrowInfo {
    /// Gets the children of this node.
    pub fn get_children(&self, node: &BorrowNode) -> Vec<&BorrowNode> {
        self.borrowed_by
            .get(node)
            .map(|s| s.keys().collect_vec())
            .unwrap_or_else(Vec::new)
    }

    /// Gets the parents of this node.
    pub fn get_parents(&self, node: &BorrowNode) -> Vec<&BorrowNode> {
        self.borrows_from
            .get(node)
            .map(|s| s.keys().collect_vec())
            .unwrap_or_else(Vec::new)
    }

    /// Gets incoming edges (together with sources) of this node.
    pub fn get_incoming(&self, node: &BorrowNode) -> BTreeMap<BorrowNode, EdgeDomain> {
        self.borrows_from
            .get(node)
            .cloned()
            .unwrap_or_else(BTreeMap::new)
    }

    /// Checks whether a node is in use. A node is used if it is in the live_nodes set
    /// or if it is borrowed by a node which is used.
    pub fn is_in_use(&self, node: &BorrowNode) -> bool {
        if self.live_nodes.contains(node) {
            true
        } else {
            self.get_children(node)
                .iter()
                .any(|child| self.is_in_use(child))
        }
    }

    /// Checks whether this is an unchecked node.
    pub fn is_unchecked(&self, node: &BorrowNode) -> bool {
        self.unchecked_nodes.contains(node)
    }

    /// Checks whether this is a moved node.
    pub fn is_moved(&self, node: &BorrowNode) -> bool {
        self.moved_nodes.contains(node)
    }

    /// Checks whether this is an spliced node.
    pub fn is_spliced(&self, node: &BorrowNode) -> bool {
        self.spliced_nodes.contains(node)
    }

    /// Returns nodes which are dying from this to the next state. This includes those which
    /// are directly dying plus those from which they borrow. Returns nodes in child-first order.
    pub fn dying_nodes(&self, next: &BorrowInfo) -> Vec<BorrowNode> {
        let mut visited = BTreeSet::new();
        let mut result = vec![];
        for dying in self.live_nodes.difference(&next.live_nodes) {
            // Collect ancestors, but exclude those which are still in use. Some nodes may be
            // dying regards direct usage in instructions, but they may still be ancestors of
            // living nodes (this is what `is_in_use` checks for).
            if !next.is_in_use(dying) {
                self.collect_ancestors(&mut visited, &mut result, dying, &|n| !next.is_in_use(n));
            }
        }
        result
    }

    /// Collects this node and ancestors, inserting them in child-first order into the
    /// given vector. Ancestors are only added if they fulfill the predicate.
    fn collect_ancestors<P>(
        &self,
        visited: &mut BTreeSet<BorrowNode>,
        order: &mut Vec<BorrowNode>,
        node: &BorrowNode,
        cond: &P,
    ) where
        P: Fn(&BorrowNode) -> bool,
    {
        if visited.insert(node.clone()) {
            order.push(node.clone());
            for parent in self.get_parents(node) {
                if cond(parent) {
                    self.collect_ancestors(visited, order, parent, cond);
                }
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.live_nodes.is_empty()
            && self.unchecked_nodes.is_empty()
            && self.moved_nodes.is_empty()
            && self.spliced_nodes.is_empty()
            && self.borrowed_by.is_empty()
            && self.borrows_from.is_empty()
    }

    pub fn borrow_info_str(&self, func_target: &FunctionTarget<'_>) -> String {
        let mut parts = vec![];
        let mut add = |name: &str, value: String| {
            if !value.is_empty() {
                parts.push(format!("{}: {}", name, value));
            }
        };
        add(
            "live_nodes",
            self.live_nodes
                .iter()
                .map(|node| format!("{}", node.display(func_target)))
                .join(", "),
        );
        add(
            "unchecked_nodes",
            self.unchecked_nodes
                .iter()
                .map(|node| format!("{}", node.display(func_target)))
                .join(", "),
        );
        add(
            "spliced_nodes",
            self.spliced_nodes
                .iter()
                .map(|node| format!("{}", node.display(func_target)))
                .join(", "),
        );
        add(
            "moved_nodes",
            self.moved_nodes
                .iter()
                .map(|node| format!("{}", node.display(func_target)))
                .join(", "),
        );
        let borrows_str = |(node1, borrows): (&BorrowNode, &BTreeMap<BorrowNode, EdgeDomain>)| {
            format!(
                "{} -> {{{}}}",
                node1.display(func_target),
                borrows
                    .iter()
                    .map(|(node2, edges)| format!("({}, {})", edges, node2.display(func_target)))
                    .join(", ")
            )
        };
        add(
            "borrowed_by",
            self.borrowed_by.iter().map(borrows_str).join(", "),
        );
        add(
            "borrows_from",
            self.borrows_from.iter().map(borrows_str).join(", "),
        );
        parts.iter().join("\n")
    }

    fn add_node(&mut self, node: BorrowNode) {
        self.live_nodes.insert(node);
    }

    fn remove_node(&mut self, node: &BorrowNode) {
        self.live_nodes.remove(node);
    }

    fn add_edge(&mut self, parent: BorrowNode, child: BorrowNode, weight: BorrowEdge) -> bool {
        if self.unchecked_nodes.contains(&parent) {
            // If the parent node is unchecked, so is the child node.
            self.unchecked_nodes.insert(child.clone());
        }
        match self
            .borrowed_by
            .entry(parent.clone())
            .or_default()
            .entry(child.clone())
            .or_default()
        {
            EdgeDomain::Top => false,
            EdgeDomain::EdgeSet(edges) => match weight {
                BorrowEdge::Weak => {
                    self.borrowed_by
                        .entry(parent)
                        .or_default()
                        .insert(child, EdgeDomain::Top);
                    true
                }
                BorrowEdge::Strong(edge) => edges.insert(edge),
            },
        }
    }

    fn consolidate(&mut self) {
        for (src, outgoing) in &self.borrowed_by {
            for (dst, edges) in outgoing {
                self.borrows_from
                    .entry(dst.clone())
                    .or_default()
                    .insert(src.clone(), edges.clone());
            }
        }
    }
}

/// Used to store the module and function id of Vector:borrow_mut
#[derive(Debug, Clone, Copy)]
pub struct VecBorMutInfo(Option<(move_model::model::ModuleId, move_model::model::FunId)>);

#[derive(Clone)]
pub struct BorrowInfoAtCodeOffset {
    pub before: BorrowInfo,
    pub after: BorrowInfo,
}

/// Borrow annotation computed by the borrow analysis processor.
#[derive(Clone)]
pub struct BorrowAnnotation(BTreeMap<CodeOffset, BorrowInfoAtCodeOffset>);

impl BorrowAnnotation {
    pub fn get_borrow_info_at(&self, code_offset: CodeOffset) -> Option<&BorrowInfoAtCodeOffset> {
        self.0.get(&code_offset)
    }
}

/// Borrow analysis processor.
pub struct BorrowAnalysisProcessor {
    weak_edges: bool,
}

impl BorrowAnalysisProcessor {
    pub fn new(weak_edges: bool) -> Box<Self> {
        Box::new(BorrowAnalysisProcessor { weak_edges })
    }
}

impl FunctionTargetProcessor for BorrowAnalysisProcessor {
    fn process(
        &self,
        _targets: &mut FunctionTargetsHolder,
        func_env: &FunctionEnv<'_>,
        mut data: FunctionData,
    ) -> FunctionData {
        let borrow_annotation = if func_env.is_native() {
            // Native functions have no byte code.
            BorrowAnnotation(BTreeMap::new())
        } else {
            let func_target = FunctionTarget::new(func_env, &data);
            let analyzer = BorrowAnalysis::new(&func_target, self.weak_edges);
            let result = analyzer.analyze(&data.code);
            let propagator = PropagateSplicedAnalysis::new(result);
            BorrowAnnotation(propagator.run(&data.code))
        };
        // Annotate function target with computed borrow data.
        data.annotations
            .borrow_mut()
            .set::<BorrowAnnotation>(borrow_annotation);
        data.annotations.borrow_mut().remove::<LiveVarAnnotation>();
        data
    }

    fn initialize(&self, env: &GlobalEnv, _targets: &mut FunctionTargetsHolder) {
        match env
            .get_modules()
            .into_iter()
            .find(|module| module.get_full_name_str() == "0x1::Vector")
        {
            None => env.set_extension(VecBorMutInfo(None)),
            Some(module) => {
                let mid = module.get_id();
                let fid = env
                    .get_module(mid)
                    .get_functions()
                    .into_iter()
                    .find(|fun| fun.get_full_name_str() == "Vector::borrow_mut")
                    .unwrap()
                    .get_id();
                env.set_extension(VecBorMutInfo(Some((mid, fid))))
            }
        }
    }

    fn name(&self) -> String {
        "borrow_analysis".to_string()
    }
}

struct BorrowAnalysis<'a> {
    func_target: &'a FunctionTarget<'a>,
    livevar_annotation: &'a LiveVarAnnotation,
    weak_edges: bool,
}

impl<'a> BorrowAnalysis<'a> {
    fn new(func_target: &'a FunctionTarget<'a>, weak_edges: bool) -> Self {
        let livevar_annotation = func_target
            .get_annotations()
            .get::<LiveVarAnnotation>()
            .expect("livevar annotation");

        Self {
            func_target,
            livevar_annotation,
            weak_edges,
        }
    }

    fn analyze(&self, instrs: &[Bytecode]) -> BTreeMap<CodeOffset, BorrowInfoAtCodeOffset> {
        let cfg = StacklessControlFlowGraph::new_forward(instrs);

        let mut state = BorrowInfo::default();

        // Initialize state from parameters
        for idx in 0..self.func_target.get_parameter_count() {
            let node = self.borrow_node(idx);
            if self.func_target.is_unchecked_param(idx) {
                state.unchecked_nodes.insert(node.clone());
            }
            state.add_node(self.borrow_node(idx));
        }

        let state_map = self.analyze_function(state, instrs, &cfg);
        self.state_per_instruction(state_map, instrs, &cfg, |before, after| {
            let mut before = before.clone();
            let mut after = after.clone();
            before.consolidate();
            after.consolidate();
            BorrowInfoAtCodeOffset { before, after }
        })
    }

    fn borrow_node(&self, idx: TempIndex) -> BorrowNode {
        let ty = self.func_target.get_local_type(idx);
        if ty.is_reference() {
            BorrowNode::Reference(idx)
        } else {
            BorrowNode::LocalRoot(idx)
        }
    }

    fn remap_borrow_node(&self, state: &mut BorrowInfo, from: &BorrowNode, to: &BorrowNode) {
        let remap = |node: BorrowNode| if &node == from { to.clone() } else { node };
        state.live_nodes = std::mem::take(&mut state.live_nodes)
            .into_iter()
            .map(remap)
            .collect();
        state.unchecked_nodes = std::mem::take(&mut state.unchecked_nodes)
            .into_iter()
            .map(remap)
            .collect();
        state.spliced_nodes = std::mem::take(&mut state.spliced_nodes)
            .into_iter()
            .map(remap)
            .collect();
        state.borrowed_by = std::mem::take(&mut state.borrowed_by)
            .into_iter()
            .map(|(src, dests)| {
                (
                    remap(src),
                    dests
                        .into_iter()
                        .map(|(node, edges)| (remap(node), edges))
                        .collect(),
                )
            })
            .collect();
    }
}

impl<'a> TransferFunctions for BorrowAnalysis<'a> {
    type State = BorrowInfo;
    const BACKWARD: bool = false;

    fn execute(&self, state: &mut BorrowInfo, instr: &Bytecode, code_offset: CodeOffset) {
        use Bytecode::*;
        let livevar_annotation_at = self
            .livevar_annotation
            .get_live_var_info_at(code_offset)
            .expect("livevar annotation");
        match instr {
            Assign(_, dest, src, kind) => {
                let dest_node = self.borrow_node(*dest);
                let src_node = self.borrow_node(*src);
                match kind {
                    AssignKind::Move | AssignKind::Store => {
                        self.remap_borrow_node(state, &src_node, &dest_node);
                        state.moved_nodes.insert(src_node);
                    }
                    AssignKind::Copy => {
                        state.add_node(dest_node.clone());
                        if !self.weak_edges {
                            state.add_edge(
                                src_node,
                                dest_node,
                                BorrowEdge::Strong(StrongEdge::Direct),
                            );
                        } else {
                            state.add_edge(src_node, dest_node, BorrowEdge::Weak);
                        }
                    }
                }
            }
            Call(_, dests, oper, srcs, _) => {
                use Operation::*;
                match oper {
                    // In the borrows below, we only create an edge if the
                    // borrowed value is actually alive. For a dead borrow we would
                    // otherwise never end live time, because we cannot see a node
                    // being created and dying at the very same instruction.
                    BorrowLoc if livevar_annotation_at.after.contains(&dests[0]) => {
                        let dest_node = self.borrow_node(dests[0]);
                        let src_node = self.borrow_node(srcs[0]);
                        state.add_node(dest_node.clone());
                        if !self.weak_edges {
                            state.add_edge(
                                src_node,
                                dest_node,
                                BorrowEdge::Strong(StrongEdge::Direct),
                            );
                        } else {
                            state.add_edge(src_node, dest_node, BorrowEdge::Weak);
                        }
                    }
                    BorrowGlobal(mid, sid, inst)
                        if livevar_annotation_at.after.contains(&dests[0]) =>
                    {
                        let dest_node = self.borrow_node(dests[0]);
                        let src_node = BorrowNode::GlobalRoot(QualifiedInstId {
                            module_id: *mid,
                            inst: inst.to_owned(),
                            id: *sid,
                        });
                        state.add_node(dest_node.clone());
                        if !self.weak_edges {
                            state.add_edge(
                                src_node,
                                dest_node,
                                BorrowEdge::Strong(StrongEdge::Direct),
                            );
                        } else {
                            state.add_edge(src_node, dest_node, BorrowEdge::Weak);
                        }
                    }
                    BorrowField(mid, sid, inst, field)
                        if livevar_annotation_at.after.contains(&dests[0]) =>
                    {
                        let dest_node = self.borrow_node(dests[0]);
                        let src_node = self.borrow_node(srcs[0]);
                        state.add_node(dest_node.clone());
                        if !self.weak_edges {
                            state.add_edge(
                                src_node,
                                dest_node,
                                BorrowEdge::Strong(StrongEdge::Field(
                                    mid.qualified_inst(*sid, inst.to_owned()),
                                    *field,
                                )),
                            );
                        } else {
                            state.add_edge(src_node, dest_node, BorrowEdge::Weak);
                        }
                    }
                    Splice(map) => {
                        let child_node = self.borrow_node(srcs[0]);
                        state.add_node(child_node.clone());
                        for parent in map.values() {
                            state.add_edge(
                                self.borrow_node(*parent),
                                child_node.clone(),
                                BorrowEdge::Weak,
                            );
                            state.spliced_nodes.insert(self.borrow_node(*parent));
                            state.unchecked_nodes.insert(child_node.clone());
                        }
                    }
                    Function(mid, fid, ..) => {
                        // For all &mut ref return values (that is those which are not
                        // introduced for &mut parameters), create a borrow edge from all &mut
                        // parameters. This reflects the current borrow semantics of Move.
                        // We do not known from which input parameter the returned reference
                        // borrows, so we must assume it could be any of them.
                        // TODO: this is one place where we could do better via inter-procedural
                        //   analysis.
                        let callee_env = self
                            .func_target
                            .global_env()
                            .get_module(*mid)
                            .into_function(*fid);
                        for ret_idx in (0..callee_env.get_return_count()).filter(|ret_idx| {
                            callee_env.get_return_type(*ret_idx).is_mutable_reference()
                        }) {
                            let dest = dests[ret_idx];
                            let dest_node = self.borrow_node(dest);
                            for src in srcs.iter().filter(|idx| {
                                self.func_target
                                    .get_local_type(**idx)
                                    .is_mutable_reference()
                            }) {
                                if *src != dest {
                                    let src_node = self.borrow_node(*src);
                                    if livevar_annotation_at.after.contains(&dest) {
                                        state.add_node(dest_node.clone());
                                    }
                                    // Logic to create strong edges for the function
                                    // vector::borrow_mut
                                    let vec_bor_mut_info = *self
                                        .func_target
                                        .global_env()
                                        .get_extension::<VecBorMutInfo>()
                                        .unwrap();
                                    if !self.weak_edges
                                        && vec_bor_mut_info.0.is_some()
                                        && mid == &vec_bor_mut_info.0.unwrap().0
                                        && fid == &vec_bor_mut_info.0.unwrap().1
                                    {
                                        state.add_edge(
                                            src_node.clone(),
                                            dest_node.clone(),
                                            BorrowEdge::Strong(StrongEdge::FieldUnknown),
                                        );
                                    } else {
                                        state.add_edge(
                                            src_node.clone(),
                                            dest_node.clone(),
                                            BorrowEdge::Weak,
                                        );
                                    }
                                }
                            }
                        }
                    }
                    OpaqueCallBegin(_, _, _) | OpaqueCallEnd(_, _, _) => {
                        // just skip
                    }
                    _ => {
                        // Other operations do not create references.
                    }
                }
            }
            _ => {
                // Other instructions do not create references
            }
        }

        // Update live_vars.

        for idx in livevar_annotation_at
            .before
            .difference(&livevar_annotation_at.after)
        {
            if self.func_target.get_local_type(*idx).is_reference() {
                let node = self.borrow_node(*idx);
                state.remove_node(&node);
            }
        }
    }
}

impl<'a> DataflowAnalysis for BorrowAnalysis<'a> {}

impl AbstractDomain for BorrowInfo {
    fn join(&mut self, other: &Self) -> JoinResult {
        let live_changed = self.live_nodes.join(&other.live_nodes);
        let unchecked_changed = self.unchecked_nodes.join(&other.unchecked_nodes);
        let spliced_changed = self.spliced_nodes.join(&other.spliced_nodes);
        let moved_changed = self.moved_nodes.join(&other.moved_nodes);
        let mut borrowed_changed = JoinResult::Unchanged;
        for (src, dests) in other.borrowed_by.iter() {
            for (dest, edges) in dests {
                let edges_changed = self
                    .borrowed_by
                    .entry(src.clone())
                    .or_default()
                    .entry(dest.clone())
                    .or_default()
                    .join(edges);
                borrowed_changed = borrowed_changed.combine(edges_changed)
            }
        }
        borrowed_changed
            .combine(moved_changed)
            .combine(spliced_changed)
            .combine(unchecked_changed)
            .combine(live_changed)
    }
}

fn extend_set(set: &mut BTreeSet<BorrowNode>, other: &BTreeSet<BorrowNode>) -> bool {
    let n = set.len();
    set.extend(other.iter().cloned());
    n != set.len()
}

/// Analysis for propagating the spliced node information back to the point where
/// the node is borrowed.
struct PropagateSplicedAnalysis {
    borrow: BTreeMap<CodeOffset, BorrowInfoAtCodeOffset>,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, PartialOrd)]
struct SplicedState {
    spliced: BTreeSet<BorrowNode>,
}

impl AbstractDomain for SplicedState {
    fn join(&mut self, other: &Self) -> JoinResult {
        if extend_set(&mut self.spliced, &other.spliced) {
            JoinResult::Changed
        } else {
            JoinResult::Unchanged
        }
    }
}

impl TransferFunctions for PropagateSplicedAnalysis {
    type State = SplicedState;
    const BACKWARD: bool = true;

    fn execute(&self, state: &mut Self::State, instr: &Bytecode, offset: u16) {
        use Bytecode::*;
        use Operation::*;
        if let Some(borrow) = self.borrow.get(&offset) {
            state
                .spliced
                .extend(borrow.after.spliced_nodes.iter().cloned());
        }
        match instr {
            Call(_, dests, BorrowLoc, ..)
            | Call(_, dests, BorrowGlobal(..), ..)
            | Call(_, dests, BorrowField(..), ..) => {
                state.spliced.remove(&BorrowNode::Reference(dests[0]));
            }
            _ => {}
        }
    }
}

impl DataflowAnalysis for PropagateSplicedAnalysis {}

impl PropagateSplicedAnalysis {
    fn new(borrow: BTreeMap<CodeOffset, BorrowInfoAtCodeOffset>) -> Self {
        Self { borrow }
    }

    fn run(self, instrs: &[Bytecode]) -> BTreeMap<CodeOffset, BorrowInfoAtCodeOffset> {
        let cfg = StacklessControlFlowGraph::new_backward(instrs, false);
        let state_map = self.analyze_function(SplicedState::default(), instrs, &cfg);
        let mut data = self.state_per_instruction(state_map, instrs, &cfg, |before, after| {
            (before.clone(), after.clone())
        });
        let PropagateSplicedAnalysis { mut borrow } = self;
        for (code_offset, info) in borrow.iter_mut() {
            if let Some((SplicedState { spliced: before }, SplicedState { spliced: after })) =
                data.remove(code_offset)
            {
                info.before.spliced_nodes = SetDomain::of_set(before);
                info.after.spliced_nodes = SetDomain::of_set(after);
            }
        }
        borrow
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

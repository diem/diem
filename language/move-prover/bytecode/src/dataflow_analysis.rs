// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Adapted from AbstractInterpreter for Bytecode, this module defines the data-flow analysis
//! framework for stackless bytecode.

use crate::{
    stackless_bytecode::Bytecode,
    stackless_control_flow_graph::{BlockId, StacklessControlFlowGraph},
};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fmt::Debug,
    ops::{Deref, DerefMut},
};
use vm::file_format::CodeOffset;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinResult {
    Unchanged,
    Changed,
}

impl JoinResult {
    pub fn join(self, other: JoinResult) -> JoinResult {
        use JoinResult::*;
        match (self, other) {
            (Unchanged, Unchanged) => Unchanged,
            _ => Changed,
        }
    }
}

pub trait AbstractDomain: Clone + Sized + Eq + PartialOrd + PartialEq + Debug {
    fn join(&mut self, other: &Self) -> JoinResult;
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SetDomain<Elem: Clone + Ord + Sized>(pub BTreeSet<Elem>);

impl<E: Clone + Ord + Sized> Default for SetDomain<E> {
    fn default() -> Self {
        Self(BTreeSet::new())
    }
}

impl<E: Clone + Ord + Sized> Deref for SetDomain<E> {
    type Target = BTreeSet<E>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E: Clone + Ord + Sized> DerefMut for SetDomain<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<E: Clone + Ord + Sized + Debug> AbstractDomain for SetDomain<E> {
    fn join(&mut self, other: &Self) -> JoinResult {
        if self == other {
            JoinResult::Unchanged
        } else {
            for e in other.iter() {
                self.insert(e.clone());
            }
            JoinResult::Changed
        }
    }
}

impl<E: Clone + Ord> std::iter::FromIterator<E> for SetDomain<E> {
    fn from_iter<I: IntoIterator<Item = E>>(iter: I) -> Self {
        let mut s = SetDomain::default();
        for e in iter {
            s.insert(e);
        }
        s
    }
}

impl<E: Clone + Ord> std::iter::IntoIterator for SetDomain<E> {
    type Item = E;
    type IntoIter = std::collections::btree_set::IntoIter<E>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<E: Clone + Ord + Sized> SetDomain<E> {
    pub fn singleton(e: E) -> Self {
        let mut s = SetDomain::default();
        s.insert(e);
        s
    }

    pub fn of_set(s: BTreeSet<E>) -> Self {
        Self(s)
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct MapDomain<K: Ord, V: AbstractDomain>(pub BTreeMap<K, V>);

impl<K: Ord, V: AbstractDomain> MapDomain<K, V> {
    /// join `v` with self[k] if `k` is bound, insert `v` otherwise
    pub fn insert_join(&mut self, k: K, v: V) {
        self.0
            .entry(k)
            .and_modify(|old_v| {
                old_v.join(&v);
            })
            .or_insert(v);
    }
}

impl<K: Ord, V: AbstractDomain> Default for MapDomain<K, V> {
    fn default() -> Self {
        Self(BTreeMap::new())
    }
}

impl<K: Clone + Sized + Eq + Ord + Debug, V: AbstractDomain> AbstractDomain for MapDomain<K, V> {
    fn join(&mut self, other: &Self) -> JoinResult {
        if self == other {
            JoinResult::Unchanged
        } else {
            for (k, v1) in other.iter() {
                self.entry(k.clone())
                    .and_modify(|v2| {
                        v2.join(&v1);
                    })
                    .or_insert_with(|| v1.clone());
            }
            JoinResult::Changed
        }
    }
}

impl<K: Clone + Sized + Eq + Ord, V: AbstractDomain> Deref for MapDomain<K, V> {
    type Target = BTreeMap<K, V>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K: Clone + Sized + Eq + Ord, V: AbstractDomain> DerefMut for MapDomain<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BlockState<State: Clone> {
    pub pre: State,
    pub post: State,
}

pub type StateMap<State> = BTreeMap<BlockId, BlockState<State>>;

/// Take a pre-state + instruction and mutate it to produce a post-state。
pub trait TransferFunctions {
    type State: AbstractDomain + Clone;
    const BACKWARD: bool;

    fn execute_block(
        &self,
        block_id: BlockId,
        mut state: Self::State,
        instrs: &[Bytecode],
        cfg: &StacklessControlFlowGraph,
    ) -> Self::State {
        if cfg.is_dummmy(block_id) {
            return state;
        }
        let instr_inds = cfg.instr_indexes(block_id).unwrap();
        if Self::BACKWARD {
            for offset in instr_inds.rev() {
                let instr = &instrs[offset as usize];
                self.execute(&mut state, instr, offset);
            }
        } else {
            for offset in instr_inds {
                let instr = &instrs[offset as usize];
                self.execute(&mut state, instr, offset);
            }
        }
        state
    }

    fn execute(&self, state: &mut Self::State, instr: &Bytecode, offset: CodeOffset);
}

pub trait DataflowAnalysis: TransferFunctions {
    fn analyze_function(
        &self,
        initial_state: Self::State,
        instrs: &[Bytecode],
        cfg: &StacklessControlFlowGraph,
    ) -> StateMap<Self::State> {
        let mut state_map: StateMap<Self::State> = StateMap::new();
        let mut work_list = VecDeque::new();
        work_list.push_back(cfg.entry_block());
        state_map.insert(
            cfg.entry_block(),
            BlockState {
                pre: initial_state.clone(),
                post: initial_state.clone(),
            },
        );
        while let Some(block_id) = work_list.pop_front() {
            let pre = state_map.get(&block_id).expect("basic block").pre.clone();
            let post = self.execute_block(block_id, pre, &instrs, cfg);

            // propagate postcondition of this block to successor blocks
            for next_block_id in cfg.successors(block_id) {
                match state_map.get_mut(next_block_id) {
                    Some(next_block_res) => {
                        let join_result = next_block_res.pre.join(&post);
                        match join_result {
                            JoinResult::Unchanged => {
                                // Pre is the same after join. Reanalyzing this block would produce
                                // the same post. Don't schedule it.
                                continue;
                            }
                            JoinResult::Changed => {
                                // The pre changed. Schedule the next block.
                                work_list.push_back(*next_block_id);
                            }
                        }
                    }
                    None => {
                        // Haven't visited the next block yet. Use the post of the current block as
                        // its pre and schedule it.
                        state_map.insert(
                            *next_block_id,
                            BlockState {
                                pre: post.clone(),
                                post: initial_state.clone(),
                            },
                        );
                        work_list.push_back(*next_block_id);
                    }
                }
            }
            state_map.get_mut(&block_id).expect("basic block").post = post;
        }
        state_map
    }

    /// Takes the StateMap resulting from `analyze_function` and converts it into a map
    /// from each code offset into a derived state `A`. This re-executes the analysis for
    /// each instruction within a basic block to reconstruct the intermediate results
    /// from block begin to block end. The function `f` gets passed the before/after state
    /// of the instruction at a code offset. Returns a map from code offset to `A`.
    fn state_per_instruction<A, F>(
        &self,
        state_map: StateMap<Self::State>,
        instrs: &[Bytecode],
        cfg: &StacklessControlFlowGraph,
        mut f: F,
    ) -> BTreeMap<CodeOffset, A>
    where
        F: FnMut(&Self::State, &Self::State) -> A,
    {
        let mut result = BTreeMap::new();
        for (block_id, block_state) in state_map {
            let mut state = block_state.pre;
            if !cfg.is_dummmy(block_id) {
                let instr_inds = cfg.instr_indexes(block_id).unwrap();
                if Self::BACKWARD {
                    for offset in instr_inds.rev() {
                        let after = state.clone();
                        self.execute(&mut state, &instrs[offset as usize], offset);
                        result.insert(offset, f(&state, &after));
                    }
                } else {
                    for offset in instr_inds {
                        let before = state.clone();
                        self.execute(&mut state, &instrs[offset as usize], offset);
                        result.insert(offset, f(&before, &state));
                    }
                }
            }
        }
        result
    }
}

// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::ast::*;
use crate::errors::*;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

//**************************************************************************************************
// CFG
//**************************************************************************************************

const DEAD_CODE_ERR: &str = "Unreachable code. This statement (and any following statements) will not be executed. In some cases, this will result in unused resource values.";

#[derive(Debug)]
pub struct BlockCFG<'a> {
    start: Label,
    blocks: &'a mut Blocks,
    successor_map: BTreeMap<Label, BTreeSet<Label>>,
    predecessor_map: BTreeMap<Label, BTreeSet<Label>>,
}

impl<'a> BlockCFG<'a> {
    pub fn new(start: Label, blocks: &'a mut Blocks) -> (BlockCFG<'a>, Errors) {
        let mut seen = BTreeSet::new();
        let mut work_list = VecDeque::new();
        seen.insert(start.clone());
        work_list.push_back(start.clone());

        // build successor map from reachable code
        let mut successor_map = BTreeMap::new();
        while let Some(label) = work_list.pop_front() {
            let last_cmd = blocks.get(&label).unwrap().back().unwrap();
            let successors = last_cmd.value.successors();
            for successor in &successors {
                if !seen.contains(successor) {
                    seen.insert(successor.clone());
                    work_list.push_back(successor.clone());
                }
            }
            let old = successor_map.insert(label.clone(), successors);
            assert!(old.is_none());
        }

        // build inverse map
        let mut predecessor_map = BTreeMap::new();
        for (parent, children) in &successor_map {
            for child in children {
                predecessor_map
                    .entry(child.clone())
                    .or_insert_with(BTreeSet::new)
                    .insert(parent.clone());
            }
        }

        // no dead code
        let mut errors = Errors::new();
        for label in blocks.keys() {
            if !successor_map.contains_key(label) {
                let loc = blocks.get(label).unwrap().front().unwrap().loc;
                errors.push(vec![(loc, DEAD_CODE_ERR.into())])
            }
        }

        let cfg = BlockCFG {
            start,
            blocks,
            successor_map,
            predecessor_map,
        };
        (cfg, errors)
    }

    pub fn successors(&self, label: &Label) -> &BTreeSet<Label> {
        self.successor_map.get(label).unwrap()
    }

    pub fn predecessors(&self, label: &Label) -> &BTreeSet<Label> {
        self.predecessor_map.get(label).unwrap()
    }

    pub fn block(&self, label: &Label) -> &BasicBlock {
        self.blocks.get(label).unwrap()
    }

    pub fn block_mut(&mut self, label: &Label) -> &mut BasicBlock {
        self.blocks.get_mut(label).unwrap()
    }

    pub fn blocks_iter_mut(&mut self) -> impl Iterator<Item = (&Label, &mut BasicBlock)> {
        self.blocks.iter_mut()
    }

    pub fn num_blocks(&self) -> usize {
        self.blocks.len()
    }

    pub fn start_block(&self) -> Label {
        self.start
    }
}

// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::ast::*;
use crate::errors::*;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

//**************************************************************************************************
// CFG
//**************************************************************************************************

pub trait CFG {
    fn successors(&self, label: Label) -> &BTreeSet<Label>;

    fn predecessors(&self, label: Label) -> &BTreeSet<Label>;
    fn commands<'a>(&'a self, label: Label) -> Box<dyn Iterator<Item = (usize, &'a Command)> + 'a>;
    fn num_blocks(&self) -> usize;
    fn start_block(&self) -> Label;
}

//**************************************************************************************************
// BlockCFG
//**************************************************************************************************

const DEAD_CODE_ERR: &str =
    "Unreachable code. This statement (and any following statements) will \
     not be executed. In some cases, this will result in unused resource values.";

#[derive(Debug)]
pub struct BlockCFG<'a> {
    start: Label,
    blocks: &'a mut Blocks,
    successor_map: BTreeMap<Label, BTreeSet<Label>>,
    predecessor_map: BTreeMap<Label, BTreeSet<Label>>,
}

impl<'a> BlockCFG<'a> {
    pub fn new(start: Label, blocks: &'a mut Blocks) -> (BlockCFG, Errors) {
        let mut seen = BTreeSet::new();
        let mut work_list = VecDeque::new();
        seen.insert(start);
        work_list.push_back(start);

        // build successor map from reachable code
        let mut successor_map = BTreeMap::new();
        while let Some(label) = work_list.pop_front() {
            let last_cmd = blocks.get(&label).unwrap().back().unwrap();
            let successors = last_cmd.value.successors();
            for successor in &successors {
                if !seen.contains(successor) {
                    seen.insert(*successor);
                    work_list.push_back(*successor)
                }
            }
            let old = successor_map.insert(label, successors);
            assert!(old.is_none());
        }

        // build inverse map
        let mut predecessor_map = successor_map
            .keys()
            .cloned()
            .map(|lbl| (lbl, BTreeSet::new()))
            .collect::<BTreeMap<_, _>>();
        for (parent, children) in &successor_map {
            for child in children {
                predecessor_map.get_mut(child).unwrap().insert(*parent);
            }
        }

        // no dead code
        let mut errors = Errors::new();
        let mut dead_blocks = vec![];
        for label in blocks.keys() {
            if !successor_map.contains_key(label) {
                dead_blocks.push(*label);
                let loc = blocks.get(label).unwrap().front().unwrap().loc;
                errors.push(vec![(loc, DEAD_CODE_ERR.into())])
            }
        }
        for label in dead_blocks {
            blocks.remove(&label);
        }

        let cfg = BlockCFG {
            start,
            blocks,
            successor_map,
            predecessor_map,
        };
        (cfg, errors)
    }

    pub fn blocks(&self) -> &Blocks {
        &self.blocks
    }

    pub fn blocks_mut(&mut self) -> &mut Blocks {
        &mut self.blocks
    }

    pub fn block(&self, label: Label) -> &BasicBlock {
        self.blocks.get(&label).unwrap()
    }

    pub fn block_mut(&mut self, label: Label) -> &mut BasicBlock {
        self.blocks.get_mut(&label).unwrap()
    }

    pub fn display_blocks(&self) {
        for (lbl, block) in self.blocks() {
            println!("--BLOCK {}--", lbl);
            for cmd in block {
                println!("{:#?}", cmd.value);
            }
            println!();
        }
    }
}

impl<'a> CFG for BlockCFG<'a> {
    fn successors(&self, label: Label) -> &BTreeSet<Label> {
        self.successor_map.get(&label).unwrap()
    }

    fn predecessors(&self, label: Label) -> &BTreeSet<Label> {
        self.predecessor_map.get(&label).unwrap()
    }

    fn commands<'s>(&'s self, label: Label) -> Box<dyn Iterator<Item = (usize, &'s Command)> + 's> {
        Box::new(self.block(label).iter().enumerate())
    }

    fn num_blocks(&self) -> usize {
        self.blocks.len()
    }

    fn start_block(&self) -> Label {
        self.start
    }
}

//**************************************************************************************************
// Reverse Traversal Block CFG
//**************************************************************************************************

#[derive(Debug)]
pub struct ReverseBlockCFG<'a> {
    start: Label,
    blocks: &'a mut Blocks,
    successor_map: &'a mut BTreeMap<Label, BTreeSet<Label>>,
    predecessor_map: &'a mut BTreeMap<Label, BTreeSet<Label>>,
}

impl<'a> ReverseBlockCFG<'a> {
    pub fn new(forward_cfg: &'a mut BlockCFG, infinite_loop_starts: &BTreeSet<Label>) -> Self {
        let blocks: &'a mut Blocks = &mut forward_cfg.blocks;
        let forward_successors = &mut forward_cfg.successor_map;
        let forward_predecessor = &mut forward_cfg.predecessor_map;
        let end_blocks = {
            let mut end_blocks = BTreeSet::new();
            for (lbl, successors) in forward_successors.iter() {
                let loop_start_successors = successors
                    .iter()
                    .filter(|l| infinite_loop_starts.contains(l));
                for loop_start_successor in loop_start_successors {
                    if lbl >= loop_start_successor {
                        end_blocks.insert(*lbl);
                    }
                }
            }
            for (lbl, block) in blocks.iter() {
                let last_cmd = block.back().unwrap();
                if last_cmd.value.is_exit() {
                    end_blocks.insert(*lbl);
                }
            }
            end_blocks
        };

        // setup fake terminal block that will act as the start node in reverse traversal
        let terminal = Label(blocks.keys().map(|lbl| lbl.0).max().unwrap_or(0) + 1);
        assert!(!blocks.contains_key(&terminal), "{:#?}", blocks);
        blocks.insert(terminal, BasicBlock::new());
        for terminal_predecessor in &end_blocks {
            forward_successors
                .entry(*terminal_predecessor)
                .or_insert_with(BTreeSet::new)
                .insert(terminal);
        }
        forward_predecessor.insert(terminal, end_blocks);
        // ensure map is not partial
        forward_successors.insert(terminal, BTreeSet::new());

        Self {
            start: terminal,
            blocks,
            successor_map: forward_predecessor,
            predecessor_map: forward_successors,
        }
    }

    pub fn blocks(&self) -> &Blocks {
        &self.blocks
    }

    pub fn block(&self, label: Label) -> &BasicBlock {
        self.blocks.get(&label).unwrap()
    }
}

impl<'a> Drop for ReverseBlockCFG<'a> {
    fn drop(&mut self) {
        let empty_block = self.blocks.remove(&self.start);
        assert!(empty_block.unwrap().is_empty());
        let start_predecessors = self.predecessor_map.remove(&self.start);
        assert!(
            start_predecessors.is_some(),
            "ICE missing start node from predecessors"
        );
        let start_successors = self.successor_map.remove(&self.start).unwrap();
        for start_successor in start_successors {
            self.predecessor_map
                .get_mut(&start_successor)
                .unwrap()
                .remove(&self.start);
        }
    }
}

impl<'a> CFG for ReverseBlockCFG<'a> {
    fn successors(&self, label: Label) -> &BTreeSet<Label> {
        self.successor_map.get(&label).unwrap()
    }

    fn predecessors(&self, label: Label) -> &BTreeSet<Label> {
        self.predecessor_map.get(&label).unwrap()
    }

    fn commands<'s>(&'s self, label: Label) -> Box<dyn Iterator<Item = (usize, &'s Command)> + 's> {
        Box::new(self.block(label).iter().enumerate().rev())
    }

    fn num_blocks(&self) -> usize {
        self.blocks.len()
    }

    fn start_block(&self) -> Label {
        self.start
    }
}

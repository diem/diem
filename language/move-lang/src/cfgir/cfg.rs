// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cfgir::{
        ast::{BasicBlock, BasicBlocks, BlockInfo, LoopEnd, LoopInfo},
        remove_no_ops,
    },
    errors::*,
    hlir::ast::{Command, Command_, Exp, ExpListItem, Label, UnannotatedExp_, UnitCase},
    shared::ast_debug::*,
};
use move_ir_types::location::*;
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

#[derive(Debug)]
pub struct BlockCFG<'a> {
    start: Label,
    blocks: &'a mut BasicBlocks,
    successor_map: BTreeMap<Label, BTreeSet<Label>>,
    predecessor_map: BTreeMap<Label, BTreeSet<Label>>,
}

impl<'a> BlockCFG<'a> {
    // Returns
    // - A CFG
    // - A set of infinite loop heads
    // - and any errors resulting from building the CFG
    pub fn new(
        start: Label,
        blocks: &'a mut BasicBlocks,
        block_info: &[(Label, BlockInfo)],
    ) -> (Self, BTreeSet<Label>, Errors) {
        let mut cfg = BlockCFG {
            start,
            blocks,
            successor_map: BTreeMap::new(),
            predecessor_map: BTreeMap::new(),
        };
        remove_no_ops::optimize(&mut cfg);

        // no dead code
        let dead_code = cfg.recompute();
        let mut errors = Errors::new();
        for (_lbl, block) in dead_code {
            dead_code_error(&mut errors, &block)
        }

        let infinite_loop_starts = determine_infinite_loop_starts(&cfg, block_info);

        (cfg, infinite_loop_starts, errors)
    }

    /// Recomputes successor/predecessor maps. returns removed, dead blocks
    pub fn recompute(&mut self) -> BasicBlocks {
        let blocks = &self.blocks;
        let mut seen = BTreeSet::new();
        let mut work_list = VecDeque::new();
        seen.insert(self.start);
        work_list.push_back(self.start);

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

        self.successor_map = successor_map;
        self.predecessor_map = predecessor_map;

        let mut dead_block_labels = vec![];
        for label in self.blocks.keys() {
            if !self.successor_map.contains_key(label) {
                assert!(!self.predecessor_map.contains_key(label));
                dead_block_labels.push(*label);
            }
        }

        let mut dead_blocks = BasicBlocks::new();
        for label in dead_block_labels {
            dead_blocks.insert(label, self.blocks.remove(&label).unwrap());
        }
        dead_blocks
    }

    pub fn blocks(&self) -> &BasicBlocks {
        &self.blocks
    }

    pub fn blocks_mut(&mut self) -> &mut BasicBlocks {
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

const DEAD_ERR_CMD: &str = "Unreachable code. This statement (and any following statements) will \
                            not be executed. In some cases, this will result in unused resource \
                            values.";

const DEAD_ERR_EXP: &str = "Invalid use of a divergent expression. The code following the \
                            evaluation of this expression will be dead and should be removed. In \
                            some cases, this is necessary to prevent unused resource values.";

fn dead_code_error(errors: &mut Errors, block: &BasicBlock) {
    let first_command = block.front().unwrap();
    match unreachable_loc(first_command) {
        Some(loc) => errors.add_deprecated(vec![(loc, DEAD_ERR_EXP.into())]),
        None if is_implicit_control_flow(&block) => (),
        None => errors.add_deprecated(vec![(first_command.loc, DEAD_ERR_CMD.into())]),
    }
}

fn unreachable_loc(sp!(_, cmd_): &Command) -> Option<Loc> {
    use Command_ as C;
    match cmd_ {
        C::Assign(_, e) => unreachable_loc_exp(e),
        C::Mutate(el, er) => unreachable_loc_exp(el).or_else(|| unreachable_loc_exp(er)),
        C::Return { exp: e, .. }
        | C::Abort(e)
        | C::IgnoreAndPop { exp: e, .. }
        | C::JumpIf { cond: e, .. } => unreachable_loc_exp(e),
        C::Jump { .. } => None,
        C::Break | C::Continue => panic!("ICE break/continue not translated to jumps"),
    }
}

fn unreachable_loc_exp(parent_e: &Exp) -> Option<Loc> {
    use UnannotatedExp_ as E;
    match &parent_e.exp.value {
        E::Unreachable => Some(parent_e.exp.loc),
        E::Unit { .. }
        | E::Value(_)
        | E::Constant(_)
        | E::Spec(_, _)
        | E::UnresolvedError
        | E::BorrowLocal(_, _)
        | E::Copy { .. }
        | E::Move { .. } => None,
        E::ModuleCall(mcall) => unreachable_loc_exp(&mcall.arguments),
        E::Builtin(_, e)
        | E::Freeze(e)
        | E::Dereference(e)
        | E::UnaryExp(_, e)
        | E::Borrow(_, e, _)
        | E::Cast(e, _) => unreachable_loc_exp(e),

        E::BinopExp(e1, _, e2) => unreachable_loc_exp(e1).or_else(|| unreachable_loc_exp(e2)),

        E::Pack(_, _, fields) => fields.iter().find_map(|(_, _, e)| unreachable_loc_exp(e)),

        E::ExpList(es) => es.iter().find_map(|item| unreachable_loc_item(item)),
    }
}

fn unreachable_loc_item(item: &ExpListItem) -> Option<Loc> {
    match item {
        ExpListItem::Single(e, _) | ExpListItem::Splat(_, e, _) => unreachable_loc_exp(e),
    }
}

fn is_implicit_control_flow(block: &BasicBlock) -> bool {
    use Command_ as C;
    use UnannotatedExp_ as E;
    block.len() == 1
        && match &block.front().unwrap().value {
            C::Jump { from_user, .. } => !*from_user,
            C::Return { exp: e, from_user } if !*from_user => matches!(
                &e.exp.value,
                E::Unit {
                    case: UnitCase::Implicit
                }
            ),
            _ => false,
        }
}

// Relying on the ordered block info (ordered in the linear ordering of the source code)
// Determines the infinite loop starts
// This cannot be determined in earlier passes due to dead code
fn determine_infinite_loop_starts(
    cfg: &BlockCFG,
    block_info: &[(Label, BlockInfo)],
) -> BTreeSet<Label> {
    // Filter dead code
    let block_info = block_info
        .iter()
        .filter(|(lbl, _info)| cfg.blocks().contains_key(lbl))
        .collect::<Vec<_>>();

    // Fully populate infinite loop starts to be pruned later
    // And for any block, determine the current loop
    let mut infinite_loop_starts = BTreeSet::new();

    let mut loop_stack: Vec<(Label, LoopEnd)> = vec![];
    let mut current_loop_info = Vec::with_capacity(block_info.len());
    for (lbl, info) in &block_info {
        match loop_stack.last() {
            Some((_, cur_loop_end)) if cur_loop_end.equals(*lbl) => {
                loop_stack.pop();
            }
            _ => (),
        }

        match info {
            BlockInfo::Other => (),
            BlockInfo::LoopHead(LoopInfo { is_loop_stmt, .. }) if !*is_loop_stmt => (),
            BlockInfo::LoopHead(LoopInfo { loop_end, .. }) => {
                infinite_loop_starts.insert(*lbl);
                loop_stack.push((*lbl, *loop_end))
            }
        }

        current_loop_info.push(loop_stack.last().cloned());
    }

    // Given the loop info for any block, determine which loops are infinite
    // Each 'loop' based loop starts in the set, and is removed if it's break is used, or if a
    // return or abort is used
    let mut prev_opt: Option<Label> = None;
    let zipped =
        block_info
            .into_iter()
            .zip(current_loop_info)
            .filter_map(|(block_info, cur_loop_opt)| {
                cur_loop_opt.map(|cur_loop| (block_info, cur_loop))
            });
    for ((lbl, _info), (cur_loop_start, cur_loop_end)) in zipped {
        debug_assert!(prev_opt.map(|prev| prev.0 < lbl.0).unwrap_or(true));
        maybe_unmark_infinite_loop_starts(
            &mut infinite_loop_starts,
            cur_loop_start,
            cur_loop_end,
            &cfg.blocks()[&lbl],
        );
        prev_opt = Some(*lbl);
    }

    infinite_loop_starts
}

fn maybe_unmark_infinite_loop_starts(
    infinite_loop_starts: &mut BTreeSet<Label>,
    cur_loop_start: Label,
    cur_loop_end: LoopEnd,
    block: &BasicBlock,
) {
    use Command_ as C;
    // jumps/return/abort are only found at the end of the block
    match &block.back().unwrap().value {
        C::Jump { target, .. } if cur_loop_end.equals(*target) => {
            infinite_loop_starts.remove(&cur_loop_start);
        }
        C::JumpIf {
            if_true, if_false, ..
        } if cur_loop_end.equals(*if_true) || cur_loop_end.equals(*if_false) => {
            infinite_loop_starts.remove(&cur_loop_start);
        }
        C::Return { .. } | C::Abort(_) => {
            infinite_loop_starts.remove(&cur_loop_start);
        }

        C::Jump { .. }
        | C::JumpIf { .. }
        | C::Assign(_, _)
        | C::Mutate(_, _)
        | C::IgnoreAndPop { .. } => (),
        C::Break | C::Continue => panic!("ICE break/continue not translated to jumps"),
    }
}
//**************************************************************************************************
// Reverse Traversal Block CFG
//**************************************************************************************************

#[derive(Debug)]
pub struct ReverseBlockCFG<'a> {
    start: Label,
    blocks: &'a mut BasicBlocks,
    successor_map: &'a mut BTreeMap<Label, BTreeSet<Label>>,
    predecessor_map: &'a mut BTreeMap<Label, BTreeSet<Label>>,
}

impl<'a> ReverseBlockCFG<'a> {
    pub fn new(forward_cfg: &'a mut BlockCFG, infinite_loop_starts: &BTreeSet<Label>) -> Self {
        let blocks: &'a mut BasicBlocks = &mut forward_cfg.blocks;
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

    pub fn blocks(&self) -> &BasicBlocks {
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

//**************************************************************************************************
// Debug
//**************************************************************************************************

impl AstDebug for BlockCFG<'_> {
    fn ast_debug(&self, w: &mut AstWriter) {
        let BlockCFG {
            start,
            blocks,
            successor_map,
            predecessor_map,
        } = self;
        w.writeln("--BlockCFG--");
        ast_debug_cfg(
            w,
            *start,
            blocks,
            successor_map.iter(),
            predecessor_map.iter(),
        );
    }
}

impl AstDebug for ReverseBlockCFG<'_> {
    fn ast_debug(&self, w: &mut AstWriter) {
        let ReverseBlockCFG {
            start,
            blocks,
            successor_map,
            predecessor_map,
        } = self;
        w.writeln("--ReverseBlockCFG--");
        ast_debug_cfg(
            w,
            *start,
            blocks,
            successor_map.iter(),
            predecessor_map.iter(),
        );
    }
}

fn ast_debug_cfg<'a>(
    w: &mut AstWriter,
    start: Label,
    blocks: &BasicBlocks,
    successor_map: impl Iterator<Item = (&'a Label, &'a BTreeSet<Label>)>,
    predecessor_map: impl Iterator<Item = (&'a Label, &'a BTreeSet<Label>)>,
) {
    w.write("successor_map:");
    w.indent(4, |w| {
        for (lbl, nexts) in successor_map {
            w.write(&format!("{} => [", lbl));
            w.comma(nexts, |w, next| w.write(&format!("{}", next)));
            w.writeln("]")
        }
    });

    w.write("predecessor_map:");
    w.indent(4, |w| {
        for (lbl, nexts) in predecessor_map {
            w.write(&format!("{} <= [", lbl));
            w.comma(nexts, |w, next| w.write(&format!("{}", next)));
            w.writeln("]")
        }
    });

    w.writeln(&format!("start: {}", start));
    w.writeln("blocks:");
    w.indent(4, |w| blocks.ast_debug(w));
}

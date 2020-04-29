// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cfgir::{
        self,
        ast::{self as G, BasicBlock, BasicBlocks},
        cfg::BlockCFG,
    },
    errors::Errors,
    hlir::ast::{self as H, Label},
    parser::ast::{FunctionName, ModuleIdent, StructName},
    shared::unique_map::UniqueMap,
};
use move_ir_types::location::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    mem,
};

//**************************************************************************************************
// Context
//**************************************************************************************************

struct Context {
    errors: Errors,
    start: Option<Label>,
    loop_begin: Option<Label>,
    loop_end: Option<Label>,
    next_label: Option<Label>,
    label_count: usize,
    blocks: BasicBlocks,
    block_ordering: BTreeMap<Label, usize>,
    infinite_loop_starts: BTreeSet<Label>,
}

impl Context {
    pub fn new(_prog: &H::Program, errors: Errors) -> Self {
        Context {
            errors,
            next_label: None,
            loop_begin: None,
            loop_end: None,
            start: None,
            label_count: 0,
            blocks: BasicBlocks::new(),
            block_ordering: BTreeMap::new(),
            infinite_loop_starts: BTreeSet::new(),
        }
    }

    pub fn error(&mut self, e: Vec<(Loc, impl Into<String>)>) {
        self.errors
            .push(e.into_iter().map(|(loc, msg)| (loc, msg.into())).collect())
    }

    pub fn get_errors(self) -> Errors {
        self.errors
    }

    fn new_label(&mut self) -> Label {
        let count = self.label_count;
        self.label_count += 1;
        Label(count)
    }

    fn insert_block(&mut self, lbl: Label, basic_block: BasicBlock) {
        assert!(self.block_ordering.insert(lbl, self.blocks.len()).is_none());
        assert!(self.blocks.insert(lbl, basic_block).is_none());
    }

    // Returns the blocks inserted in insertion ordering
    pub fn finish_blocks(&mut self) -> (Label, BasicBlocks, BTreeSet<Label>) {
        self.next_label = None;
        let start = mem::replace(&mut self.start, None);
        let blocks = mem::replace(&mut self.blocks, BasicBlocks::new());
        let block_ordering = mem::replace(&mut self.block_ordering, BTreeMap::new());
        let infinite_loop_starts = mem::replace(&mut self.infinite_loop_starts, BTreeSet::new());
        self.label_count = 0;
        self.loop_begin = None;
        self.loop_end = None;

        // Blocks will eventually be ordered and outputted to bytecode the label. But labels are
        // initially created depth first
        // So the labels need to be remapped based on the insertion order of the block
        // This preserves the original layout of the code as specified by the user (since code is
        // finshed+inserted into the map in original code order)
        let remapping = block_ordering
            .into_iter()
            .map(|(lbl, ordering)| (lbl, Label(ordering)))
            .collect();
        let (start, blocks) = G::remap_labels(&remapping, start.unwrap(), blocks);
        (start, blocks, infinite_loop_starts)
    }
}

//**************************************************************************************************
// Entry
//**************************************************************************************************

pub fn program(errors: Errors, prog: H::Program) -> (G::Program, Errors) {
    let mut context = Context::new(&prog, errors);
    let modules = modules(&mut context, prog.modules);
    let scripts = scripts(&mut context, prog.scripts);

    (G::Program { modules, scripts }, context.get_errors())
}

fn modules(
    context: &mut Context,
    hmodules: UniqueMap<ModuleIdent, H::ModuleDefinition>,
) -> UniqueMap<ModuleIdent, G::ModuleDefinition> {
    let modules = hmodules
        .into_iter()
        .map(|(mname, m)| module(context, mname, m));
    UniqueMap::maybe_from_iter(modules).unwrap()
}

fn module(
    context: &mut Context,
    module_ident: ModuleIdent,
    mdef: H::ModuleDefinition,
) -> (ModuleIdent, G::ModuleDefinition) {
    let is_source_module = mdef.is_source_module;
    let dependency_order = mdef.dependency_order;
    let structs = mdef.structs;
    let functions = mdef.functions.map(|name, f| function(context, name, f));
    (
        module_ident,
        G::ModuleDefinition {
            is_source_module,
            dependency_order,
            structs,
            functions,
        },
    )
}

fn scripts(
    context: &mut Context,
    hscripts: BTreeMap<String, H::Script>,
) -> BTreeMap<String, G::Script> {
    hscripts
        .into_iter()
        .map(|(n, s)| (n, script(context, s)))
        .collect()
}

fn script(context: &mut Context, hscript: H::Script) -> G::Script {
    let H::Script {
        loc,
        function_name,
        function: hfunction,
    } = hscript;
    let function = function(context, function_name.clone(), hfunction);
    G::Script {
        loc,
        function_name,
        function,
    }
}

//**************************************************************************************************
// Functions
//**************************************************************************************************

fn function(context: &mut Context, _name: FunctionName, f: H::Function) -> G::Function {
    let visibility = f.visibility;
    let signature = f.signature;
    let acquires = f.acquires;
    let body = function_body(context, &signature, &acquires, f.body);
    G::Function {
        visibility,
        signature,
        acquires,
        body,
    }
}

fn function_body(
    context: &mut Context,
    signature: &H::FunctionSignature,
    acquires: &BTreeSet<StructName>,
    sp!(loc, tb_): H::FunctionBody,
) -> G::FunctionBody {
    use G::FunctionBody_ as GB;
    use H::FunctionBody_ as HB;
    assert!(context.next_label.is_none());
    assert!(context.start.is_none());
    assert!(context.blocks.is_empty());
    assert!(context.block_ordering.is_empty());
    assert!(context.loop_begin.is_none());
    assert!(context.loop_end.is_none());
    assert!(context.infinite_loop_starts.is_empty());
    let b_ = match tb_ {
        HB::Native => GB::Native,
        HB::Defined { locals, body } => {
            function_block(context, body);
            let (start, mut blocks, infinite_loop_starts) = context.finish_blocks();

            let (mut cfg, errors) = BlockCFG::new(start, &mut blocks);
            for e in errors {
                context.error(e);
            }

            cfgir::refine_inference_and_verify(
                &mut context.errors,
                signature,
                acquires,
                &locals,
                &mut cfg,
                &infinite_loop_starts,
            );
            cfgir::optimize(signature, &locals, &mut cfg);

            GB::Defined {
                locals,
                start,
                blocks,
            }
        }
    };
    sp(loc, b_)
}

//**************************************************************************************************
// Statements
//**************************************************************************************************

fn function_block(context: &mut Context, blocks: H::Block) {
    let start = context.new_label();
    context.start = Some(start);
    block(context, start, blocks)
}

fn block(context: &mut Context, mut cur_label: Label, blocks: H::Block) {
    use H::Command_ as C;

    assert!(!blocks.is_empty());
    let loc = blocks.back().unwrap().loc;
    let mut basic_block = block_(context, &mut cur_label, blocks);

    // return if we ended with did not end with a command
    if basic_block.is_empty() {
        return;
    }

    match context.next_label {
        Some(next) if !basic_block.back().unwrap().value.is_terminal() => {
            basic_block.push_back(sp(loc, C::Jump(next)));
        }
        _ => (),
    }
    context.insert_block(cur_label, basic_block);
}

fn block_(context: &mut Context, cur_label: &mut Label, blocks: H::Block) -> BasicBlock {
    use H::{Command_ as C, Statement_ as S};

    assert!(!blocks.is_empty());
    let mut basic_block = BasicBlock::new();

    macro_rules! finish_block {
        (next_label: $next_label:expr) => {{
            let lbl = mem::replace(cur_label, $next_label);
            let bb = mem::replace(&mut basic_block, BasicBlock::new());
            context.insert_block(lbl, bb);
        }};
    }

    macro_rules! loop_block {
        (begin: $begin:expr, end: $end:expr, body: $body:expr, $block:expr) => {{
            let begin = $begin;
            let old_begin = mem::replace(&mut context.loop_begin, Some(begin));
            let old_end = mem::replace(&mut context.loop_end, Some($end));
            let old_next = mem::replace(&mut context.next_label, Some(begin));
            block(context, $body, $block);
            context.next_label = old_next;
            context.loop_end = old_end;
            context.loop_begin = old_begin;
        }};
    }

    for sp!(loc, stmt_) in blocks {
        match stmt_ {
            S::Command(mut cmd) => {
                command(context, &mut cmd);
                let is_terminal = cmd.value.is_terminal();
                basic_block.push_back(cmd);
                if is_terminal {
                    finish_block!(next_label: context.new_label());
                }
            }
            S::IfElse {
                cond,
                if_block,
                else_block,
            } => {
                let if_true = context.new_label();
                let if_false = context.new_label();
                let next_label = context.new_label();

                // If cond
                let jump_if = C::JumpIf {
                    cond: *cond,
                    if_true,
                    if_false,
                };
                basic_block.push_back(sp(loc, jump_if));
                finish_block!(next_label: next_label);

                // If branches
                let old_next = mem::replace(&mut context.next_label, Some(next_label));
                block(context, if_true, if_block);
                block(context, if_false, else_block);
                context.next_label = old_next;
            }
            S::While {
                cond: (hcond_block, cond),
                block: loop_block,
            } => {
                let loop_cond = context.new_label();
                let loop_body = context.new_label();
                let loop_end = context.new_label();

                // Jump to loop condition
                basic_block.push_back(sp(loc, C::Jump(loop_cond)));
                finish_block!(next_label: loop_cond);

                // Loop condition and case to jump into loop or end
                if !hcond_block.is_empty() {
                    assert!(basic_block.is_empty());
                    basic_block = block_(context, cur_label, hcond_block);
                }
                let jump_if = C::JumpIf {
                    cond: *cond,
                    if_true: loop_body,
                    if_false: loop_end,
                };
                basic_block.push_back(sp(loc, jump_if));
                finish_block!(next_label: loop_end);

                // Loop body
                loop_block!(begin: loop_cond, end: loop_end, body: loop_body, loop_block)
            }

            S::Loop {
                block: loop_block,
                has_break,
                has_return_abort,
            } => {
                let loop_body = context.new_label();
                let loop_end = context.new_label();
                assert!(cur_label.0 < loop_body.0);
                assert!(loop_body.0 < loop_end.0);

                if !has_return_abort && !has_break {
                    context.infinite_loop_starts.insert(loop_body);
                }

                // Jump to loop
                basic_block.push_back(sp(loc, C::Jump(loop_body)));
                finish_block!(next_label: loop_end);

                // Loop body
                loop_block!(begin: loop_body, end: loop_end, body: loop_body, loop_block)
            }
        }
    }

    basic_block
}

fn command(context: &Context, sp!(_, hc_): &mut H::Command) {
    use H::Command_ as C;
    match hc_ {
        C::Assign(_, _) | C::Mutate(_, _) | C::Abort(_) | C::Return(_) | C::IgnoreAndPop { .. } => {
        }
        C::Continue => *hc_ = C::Jump(context.loop_begin.clone().unwrap()),
        C::Break => *hc_ = C::Jump(context.loop_end.clone().unwrap()),
        C::Jump(_) | C::JumpIf { .. } => panic!("ICE unexpected jump before translation to jumps"),
    }
}

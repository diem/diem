// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cfgir::{
        self,
        ast::{self as G, BasicBlock, BasicBlocks, BlockInfo},
        cfg::BlockCFG,
    },
    diag,
    expansion::ast::{AbilitySet, ModuleIdent, Value, Value_},
    hlir::ast::{self as H, Label},
    parser::ast::{ConstantName, FunctionName, StructName, Var},
    shared::{unique_map::UniqueMap, CompilationEnv},
    FullyCompiledProgram,
};
use cfgir::ast::LoopInfo;
use move_core_types::{account_address::AccountAddress as MoveAddress, value::MoveValue};
use move_ir_types::location::*;
use move_symbol_pool::Symbol;
use std::{
    collections::{BTreeMap, BTreeSet},
    mem,
};

//**************************************************************************************************
// Context
//**************************************************************************************************

struct Context<'env> {
    env: &'env mut CompilationEnv,
    struct_declared_abilities: UniqueMap<ModuleIdent, UniqueMap<StructName, AbilitySet>>,
    start: Option<Label>,
    loop_begin: Option<Label>,
    loop_end: Option<Label>,
    next_label: Option<Label>,
    label_count: usize,
    blocks: BasicBlocks,
    block_ordering: BTreeMap<Label, usize>,
    // Used for populating block_info
    loop_bounds: BTreeMap<Label, G::LoopInfo>,
    block_info: Vec<(Label, BlockInfo)>,
}

impl<'env> Context<'env> {
    pub fn new(
        env: &'env mut CompilationEnv,
        pre_compiled_lib: Option<&FullyCompiledProgram>,
        modules: &UniqueMap<ModuleIdent, H::ModuleDefinition>,
    ) -> Self {
        let all_modules = modules
            .key_cloned_iter()
            .chain(pre_compiled_lib.iter().flat_map(|pre_compiled| {
                pre_compiled
                    .hlir
                    .modules
                    .key_cloned_iter()
                    .filter(|(mident, _m)| !modules.contains_key(mident))
            }));
        let struct_declared_abilities = UniqueMap::maybe_from_iter(
            all_modules
                .map(|(m, mdef)| (m, mdef.structs.ref_map(|_s, sdef| sdef.abilities.clone()))),
        )
        .unwrap();
        Context {
            env,
            struct_declared_abilities,
            next_label: None,
            loop_begin: None,
            loop_end: None,
            start: None,
            label_count: 0,
            blocks: BasicBlocks::new(),
            block_ordering: BTreeMap::new(),
            block_info: vec![],
            loop_bounds: BTreeMap::new(),
        }
    }

    fn new_label(&mut self) -> Label {
        let count = self.label_count;
        self.label_count += 1;
        Label(count)
    }

    fn insert_block(&mut self, lbl: Label, basic_block: BasicBlock) {
        assert!(self.block_ordering.insert(lbl, self.blocks.len()).is_none());
        assert!(self.blocks.insert(lbl, basic_block).is_none());
        let block_info = match self.loop_bounds.get(&lbl) {
            None => BlockInfo::Other,
            Some(info) => BlockInfo::LoopHead(info.clone()),
        };
        self.block_info.push((lbl, block_info));
    }

    // Returns the blocks inserted in insertion ordering
    pub fn finish_blocks(&mut self) -> (Label, BasicBlocks, Vec<(Label, BlockInfo)>) {
        self.next_label = None;
        let start = mem::replace(&mut self.start, None);
        let blocks = mem::take(&mut self.blocks);
        let block_ordering = mem::take(&mut self.block_ordering);
        let block_info = mem::take(&mut self.block_info);
        self.loop_bounds = BTreeMap::new();
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
        let block_info = block_info
            .into_iter()
            .map(|(lbl, info)| {
                let info = match info {
                    BlockInfo::Other => BlockInfo::Other,
                    BlockInfo::LoopHead(G::LoopInfo {
                        is_loop_stmt,
                        loop_end,
                    }) => {
                        let loop_end = match loop_end {
                            G::LoopEnd::Unused => G::LoopEnd::Unused,
                            G::LoopEnd::Target(end) if remapping.contains_key(&end) => {
                                G::LoopEnd::Target(remapping[&end])
                            }
                            G::LoopEnd::Target(_end) => G::LoopEnd::Unused,
                        };
                        BlockInfo::LoopHead(G::LoopInfo {
                            is_loop_stmt,
                            loop_end,
                        })
                    }
                };
                (remapping[&lbl], info)
            })
            .collect();
        (start, blocks, block_info)
    }
}

//**************************************************************************************************
// Entry
//**************************************************************************************************

pub fn program(
    compilation_env: &mut CompilationEnv,
    pre_compiled_lib: Option<&FullyCompiledProgram>,
    prog: H::Program,
) -> G::Program {
    let H::Program {
        modules: hmodules,
        scripts: hscripts,
    } = prog;

    let mut context = Context::new(compilation_env, pre_compiled_lib, &hmodules);

    let modules = modules(&mut context, hmodules);
    let scripts = scripts(&mut context, hscripts);

    G::Program { modules, scripts }
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
    let H::ModuleDefinition {
        attributes,
        is_source_module,
        dependency_order,
        friends,
        structs,
        functions: hfunctions,
        constants: hconstants,
    } = mdef;

    let constants = hconstants.map(|name, c| constant(context, name, c));
    let functions = hfunctions.map(|name, f| function(context, name, f));
    (
        module_ident,
        G::ModuleDefinition {
            attributes,
            is_source_module,
            dependency_order,
            friends,
            structs,
            constants,
            functions,
        },
    )
}

fn scripts(
    context: &mut Context,
    hscripts: BTreeMap<Symbol, H::Script>,
) -> BTreeMap<Symbol, G::Script> {
    hscripts
        .into_iter()
        .map(|(n, s)| (n, script(context, s)))
        .collect()
}

fn script(context: &mut Context, hscript: H::Script) -> G::Script {
    let H::Script {
        attributes,
        loc,
        constants: hconstants,
        function_name,
        function: hfunction,
    } = hscript;
    let constants = hconstants.map(|name, c| constant(context, name, c));
    let function = function(context, function_name, hfunction);
    G::Script {
        attributes,
        loc,
        constants,
        function_name,
        function,
    }
}

//**************************************************************************************************
// Functions
//**************************************************************************************************

fn constant(context: &mut Context, _name: ConstantName, c: H::Constant) -> G::Constant {
    let H::Constant {
        attributes,
        loc,
        signature,
        value: (locals, block),
    } = c;

    let final_value = constant_(context, loc, signature.clone(), locals, block);
    let value = final_value.and_then(|v| move_value_from_exp(context, v));

    G::Constant {
        attributes,
        loc,
        signature,
        value,
    }
}

const CANNOT_FOLD: &str =
    "Invalid expression in 'const'. This expression could not be evaluated to a value";

fn constant_(
    context: &mut Context,
    full_loc: Loc,
    signature: H::BaseType,
    locals: UniqueMap<Var, H::SingleType>,
    block: H::Block,
) -> Option<H::Exp> {
    use H::Command_ as C;
    const ICE_MSG: &str = "ICE invalid constant should have been blocked in typing";

    initial_block(context, block);
    let (start, mut blocks, block_info) = context.finish_blocks();

    let (mut cfg, infinite_loop_starts, errors) = BlockCFG::new(start, &mut blocks, &block_info);
    assert!(infinite_loop_starts.is_empty(), "{}", ICE_MSG);
    assert!(errors.is_empty(), "{}", ICE_MSG);

    let num_previous_errors = context.env.count_diags();
    let fake_signature = H::FunctionSignature {
        type_parameters: vec![],
        parameters: vec![],
        return_type: H::Type_::base(signature),
    };
    let fake_acquires = BTreeMap::new();
    let fake_infinite_loop_starts = BTreeSet::new();
    cfgir::refine_inference_and_verify(
        context.env,
        &context.struct_declared_abilities,
        &fake_signature,
        &fake_acquires,
        &locals,
        &mut cfg,
        &fake_infinite_loop_starts,
    );
    assert!(
        num_previous_errors == context.env.count_diags(),
        "{}",
        ICE_MSG
    );
    cfgir::optimize(&fake_signature, &locals, &mut cfg);

    if blocks.len() != 1 {
        context.env.add_diag(diag!(
            BytecodeGeneration::UnfoldableConstant,
            (full_loc, CANNOT_FOLD)
        ));
        return None;
    }
    let mut optimized_block = blocks.remove(&start).unwrap();
    let return_cmd = optimized_block.pop_back().unwrap();
    for sp!(cloc, cmd_) in &optimized_block {
        let e = match cmd_ {
            C::IgnoreAndPop { exp, .. } => exp,
            _ => {
                context.env.add_diag(diag!(
                    BytecodeGeneration::UnfoldableConstant,
                    (*cloc, CANNOT_FOLD)
                ));
                continue;
            }
        };
        check_constant_value(context, e)
    }

    let result = match return_cmd.value {
        C::Return { exp: e, .. } => e,
        _ => unreachable!(),
    };
    check_constant_value(context, &result);
    Some(result)
}

fn check_constant_value(context: &mut Context, e: &H::Exp) {
    use H::UnannotatedExp_ as E;
    match &e.exp.value {
        E::Value(_) => (),
        _ => context.env.add_diag(diag!(
            BytecodeGeneration::UnfoldableConstant,
            (e.exp.loc, CANNOT_FOLD)
        )),
    }
}

fn move_value_from_exp(context: &mut Context, e: H::Exp) -> Option<MoveValue> {
    use H::UnannotatedExp_ as E;
    match e.exp.value {
        E::Value(v) => move_value_from_value(context, v),
        _ => None,
    }
}

fn move_value_from_value(context: &mut Context, sp!(_, v_): Value) -> Option<MoveValue> {
    use MoveValue as MV;
    use Value_ as V;
    Some(match v_ {
        V::InferredNum(_) => panic!("ICE inferred num should have been expanded"),
        V::Address(a) => MV::Address(MoveAddress::new(
            a.into_addr_bytes(context.env.named_address_mapping())
                .into_bytes(),
        )),
        V::U8(u) => MV::U8(u),
        V::U64(u) => MV::U64(u),
        V::U128(u) => MV::U128(u),
        V::Bool(b) => MV::Bool(b),
        V::Bytearray(v) => MV::Vector(v.into_iter().map(MV::U8).collect()),
    })
}

//**************************************************************************************************
// Functions
//**************************************************************************************************

fn function(context: &mut Context, _name: FunctionName, f: H::Function) -> G::Function {
    let attributes = f.attributes;
    let visibility = f.visibility;
    let signature = f.signature;
    let acquires = f.acquires;
    let body = function_body(context, &signature, &acquires, f.body);
    G::Function {
        attributes,
        visibility,
        signature,
        acquires,
        body,
    }
}

fn function_body(
    context: &mut Context,
    signature: &H::FunctionSignature,
    acquires: &BTreeMap<StructName, Loc>,
    sp!(loc, tb_): H::FunctionBody,
) -> G::FunctionBody {
    use G::FunctionBody_ as GB;
    use H::FunctionBody_ as HB;
    assert!(context.next_label.is_none());
    assert!(context.start.is_none());
    assert!(context.blocks.is_empty());
    assert!(context.block_ordering.is_empty());
    assert!(context.block_info.is_empty());
    assert!(context.loop_bounds.is_empty());
    assert!(context.loop_begin.is_none());
    assert!(context.loop_end.is_none());
    let b_ = match tb_ {
        HB::Native => GB::Native,
        HB::Defined { locals, body } => {
            initial_block(context, body);
            let (start, mut blocks, block_info) = context.finish_blocks();

            let (mut cfg, infinite_loop_starts, diags) =
                BlockCFG::new(start, &mut blocks, &block_info);
            context.env.add_diags(diags);

            cfgir::refine_inference_and_verify(
                context.env,
                &context.struct_declared_abilities,
                signature,
                acquires,
                &locals,
                &mut cfg,
                &infinite_loop_starts,
            );
            if !context.env.has_diags() {
                cfgir::optimize(signature, &locals, &mut cfg);
            }

            let loop_heads = block_info
                .into_iter()
                .filter(|(lbl, info)| {
                    matches!(info, BlockInfo::LoopHead(_)) && blocks.contains_key(lbl)
                })
                .map(|(lbl, _info)| lbl)
                .collect();
            GB::Defined {
                locals,
                start,
                loop_heads,
                blocks,
            }
        }
    };
    sp(loc, b_)
}

//**************************************************************************************************
// Statements
//**************************************************************************************************

fn initial_block(context: &mut Context, blocks: H::Block) {
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
            basic_block.push_back(sp(
                loc,
                C::Jump {
                    target: next,
                    from_user: false,
                },
            ));
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
            let bb = mem::take(&mut basic_block);
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

                context.loop_bounds.insert(
                    loop_cond,
                    LoopInfo {
                        is_loop_stmt: false,
                        loop_end: G::LoopEnd::Target(loop_end),
                    },
                );

                // Jump to loop condition
                basic_block.push_back(sp(
                    loc,
                    C::Jump {
                        target: loop_cond,
                        from_user: false,
                    },
                ));
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
                block: loop_block, ..
            } => {
                let loop_body = context.new_label();
                let loop_end = context.new_label();
                assert!(cur_label.0 < loop_body.0);
                assert!(loop_body.0 < loop_end.0);

                context.loop_bounds.insert(
                    loop_body,
                    LoopInfo {
                        is_loop_stmt: true,
                        loop_end: G::LoopEnd::Target(loop_end),
                    },
                );

                // Jump to loop
                basic_block.push_back(sp(
                    loc,
                    C::Jump {
                        target: loop_body,
                        from_user: false,
                    },
                ));
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
        C::Assign(_, _)
        | C::Mutate(_, _)
        | C::Abort(_)
        | C::Return { .. }
        | C::IgnoreAndPop { .. } => {}
        C::Continue => {
            *hc_ = C::Jump {
                target: context.loop_begin.unwrap(),
                from_user: true,
            }
        }
        C::Break => {
            *hc_ = C::Jump {
                target: context.loop_end.unwrap(),
                from_user: true,
            }
        }
        C::Jump { .. } | C::JumpIf { .. } => {
            panic!("ICE unexpected jump before translation to jumps")
        }
    }
}

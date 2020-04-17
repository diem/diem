// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    hlir::ast::{Command, Command_, FunctionSignature, Label, SingleType, StructDefinition},
    parser::ast::{FunctionName, FunctionVisibility, ModuleIdent, StructName, Var},
    shared::{ast_debug::*, unique_map::UniqueMap, *},
};
use move_ir_types::location::*;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

// HLIR + Unstructured Control Flow + CFG

//**************************************************************************************************
// Program
//**************************************************************************************************

#[derive(Debug)]
pub struct Program {
    pub modules: UniqueMap<ModuleIdent, ModuleDefinition>,
    pub main: Option<(Address, FunctionName, Function)>,
}

//**************************************************************************************************
// Modules
//**************************************************************************************************

#[derive(Debug)]
pub struct ModuleDefinition {
    pub is_source_module: bool,
    /// `dependency_order` is the topological order/rank in the dependency graph.
    pub dependency_order: usize,
    pub structs: UniqueMap<StructName, StructDefinition>,
    pub functions: UniqueMap<FunctionName, Function>,
}

//**************************************************************************************************
// Functions
//**************************************************************************************************

#[derive(PartialEq, Debug)]
pub enum FunctionBody_ {
    Native,
    Defined {
        locals: UniqueMap<Var, SingleType>,
        start: Label,
        blocks: BasicBlocks,
    },
}
pub type FunctionBody = Spanned<FunctionBody_>;

#[derive(PartialEq, Debug)]
pub struct Function {
    pub visibility: FunctionVisibility,
    pub signature: FunctionSignature,
    pub acquires: BTreeSet<StructName>,
    pub body: FunctionBody,
}

//**************************************************************************************************
// Blocks
//**************************************************************************************************

pub type BasicBlocks = BTreeMap<Label, BasicBlock>;

pub type BasicBlock = VecDeque<Command>;

//**************************************************************************************************
// Label util
//**************************************************************************************************

pub fn remap_labels(
    remapping: &BTreeMap<Label, Label>,
    start: Label,
    blocks: BasicBlocks,
) -> (Label, BasicBlocks) {
    let blocks = blocks
        .into_iter()
        .map(|(lbl, mut block)| {
            remap_labels_block(remapping, &mut block);
            (remapping[&lbl], block)
        })
        .collect();
    (remapping[&start], blocks)
}

fn remap_labels_block(remapping: &BTreeMap<Label, Label>, block: &mut BasicBlock) {
    for cmd in block {
        remap_labels_cmd(remapping, cmd)
    }
}

fn remap_labels_cmd(remapping: &BTreeMap<Label, Label>, sp!(_, cmd_): &mut Command) {
    use Command_::*;
    match cmd_ {
        Break | Continue => panic!("ICE break/continue not translated to jumps"),
        Mutate(_, _) | Assign(_, _) | IgnoreAndPop { .. } | Abort(_) | Return(_) => (),
        Jump(lbl) => *lbl = remapping[lbl],
        JumpIf {
            if_true, if_false, ..
        } => {
            *if_true = remapping[if_true];
            *if_false = remapping[if_false];
        }
    }
}

//**************************************************************************************************
// Debug
//**************************************************************************************************

impl AstDebug for Program {
    fn ast_debug(&self, w: &mut AstWriter) {
        let Program { modules, main } = self;
        for (m, mdef) in modules {
            w.write(&format!("module {}", m));
            w.block(|w| mdef.ast_debug(w));
            w.new_line();
        }

        if let Some((addr, n, fdef)) = main {
            w.writeln(&format!("address {}:", addr));
            (n.clone(), fdef).ast_debug(w);
        }
    }
}

impl AstDebug for ModuleDefinition {
    fn ast_debug(&self, w: &mut AstWriter) {
        let ModuleDefinition {
            is_source_module,
            dependency_order,
            structs,
            functions,
        } = self;
        if *is_source_module {
            w.writeln("library module")
        } else {
            w.writeln("source module")
        }
        w.writeln(&format!("dependency order #{}", dependency_order));
        for sdef in structs {
            sdef.ast_debug(w);
            w.new_line();
        }
        for fdef in functions {
            fdef.ast_debug(w);
            w.new_line();
        }
    }
}

impl AstDebug for (FunctionName, &Function) {
    fn ast_debug(&self, w: &mut AstWriter) {
        let (
            name,
            Function {
                visibility,
                signature,
                acquires,
                body,
            },
        ) = self;
        visibility.ast_debug(w);
        if let FunctionBody_::Native = &body.value {
            w.write("native ");
        }
        w.write(&format!("{}", name));
        signature.ast_debug(w);
        if !acquires.is_empty() {
            w.write(" acquires ");
            w.comma(acquires, |w, s| w.write(&format!("{}", s)));
            w.write(" ");
        }
        match &body.value {
            FunctionBody_::Defined {
                locals,
                start,
                blocks,
            } => w.block(|w| {
                w.write("locals:");
                w.indent(4, |w| {
                    w.list(locals, ",", |w, (v, st)| {
                        w.write(&format!("{}: ", v));
                        st.ast_debug(w);
                        true
                    })
                });
                w.new_line();
                w.writeln(&format!("start={}", start.0));
                w.new_line();
                blocks.ast_debug(w);
            }),
            FunctionBody_::Native => w.writeln(";"),
        }
    }
}

impl AstDebug for BasicBlocks {
    fn ast_debug(&self, w: &mut AstWriter) {
        w.list(self, "", |w, lbl_block| {
            lbl_block.ast_debug(w);
            w.new_line();
            true
        })
    }
}

impl AstDebug for (&Label, &BasicBlock) {
    fn ast_debug(&self, w: &mut AstWriter) {
        w.write(&format!("label {}:", (self.0).0));
        w.indent(4, |w| w.semicolon(self.1, |w, cmd| cmd.ast_debug(w)))
    }
}

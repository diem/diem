// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    expansion::ast::{Attribute, Friend, ModuleIdent},
    hlir::ast::{
        BaseType, Command, Command_, FunctionSignature, Label, SingleType, StructDefinition,
    },
    parser::ast::{ConstantName, FunctionName, StructName, Var, Visibility},
    shared::{ast_debug::*, unique_map::UniqueMap},
};
use move_core_types::value::MoveValue;
use move_ir_types::location::*;
use move_symbol_pool::Symbol;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

// HLIR + Unstructured Control Flow + CFG

//**************************************************************************************************
// Program
//**************************************************************************************************

#[derive(Debug, Clone)]
pub struct Program {
    pub modules: UniqueMap<ModuleIdent, ModuleDefinition>,
    pub scripts: BTreeMap<Symbol, Script>,
}

//**************************************************************************************************
// Scripts
//**************************************************************************************************

#[derive(Debug, Clone)]
pub struct Script {
    pub attributes: Vec<Attribute>,
    pub loc: Loc,
    pub constants: UniqueMap<ConstantName, Constant>,
    pub function_name: FunctionName,
    pub function: Function,
}

//**************************************************************************************************
// Modules
//**************************************************************************************************

#[derive(Debug, Clone)]
pub struct ModuleDefinition {
    pub attributes: Vec<Attribute>,
    pub is_source_module: bool,
    /// `dependency_order` is the topological order/rank in the dependency graph.
    pub dependency_order: usize,
    pub friends: UniqueMap<ModuleIdent, Friend>,
    pub structs: UniqueMap<StructName, StructDefinition>,
    pub constants: UniqueMap<ConstantName, Constant>,
    pub functions: UniqueMap<FunctionName, Function>,
}

//**************************************************************************************************
// Constants
//**************************************************************************************************

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Constant {
    pub attributes: Vec<Attribute>,
    pub loc: Loc,
    pub signature: BaseType,
    pub value: Option<MoveValue>,
}

//**************************************************************************************************
// Functions
//**************************************************************************************************

#[derive(PartialEq, Debug, Clone)]
pub enum FunctionBody_ {
    Native,
    Defined {
        locals: UniqueMap<Var, SingleType>,
        start: Label,
        loop_heads: BTreeSet<Label>,
        blocks: BasicBlocks,
    },
}
pub type FunctionBody = Spanned<FunctionBody_>;

#[derive(PartialEq, Debug, Clone)]
pub struct Function {
    pub attributes: Vec<Attribute>,
    pub visibility: Visibility,
    pub signature: FunctionSignature,
    pub acquires: BTreeMap<StructName, Loc>,
    pub body: FunctionBody,
}

//**************************************************************************************************
// Blocks
//**************************************************************************************************

pub type BasicBlocks = BTreeMap<Label, BasicBlock>;

pub type BasicBlock = VecDeque<Command>;

#[derive(Clone, Copy, Debug)]
pub enum LoopEnd {
    // If the generated loop end block was not used
    Unused,
    // The target of breaks inside the loop
    Target(Label),
}

#[derive(Clone, Debug)]
pub struct LoopInfo {
    pub is_loop_stmt: bool,
    pub loop_end: LoopEnd,
}

#[derive(Clone, Debug)]
pub enum BlockInfo {
    LoopHead(LoopInfo),
    Other,
}

//**************************************************************************************************
// impls
//**************************************************************************************************

impl LoopEnd {
    pub fn equals(&self, lbl: Label) -> bool {
        match self {
            LoopEnd::Unused => false,
            LoopEnd::Target(t) => *t == lbl,
        }
    }
}

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
        Mutate(_, _) | Assign(_, _) | IgnoreAndPop { .. } | Abort(_) | Return { .. } => (),
        Jump { target, .. } => *target = remapping[target],
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
        let Program { modules, scripts } = self;

        for (m, mdef) in modules.key_cloned_iter() {
            w.write(&format!("module {}", m));
            w.block(|w| mdef.ast_debug(w));
            w.new_line();
        }

        for (n, s) in scripts {
            w.write(&format!("script {}", n));
            w.block(|w| s.ast_debug(w));
            w.new_line()
        }
    }
}

impl AstDebug for Script {
    fn ast_debug(&self, w: &mut AstWriter) {
        let Script {
            attributes,
            loc: _loc,
            constants,
            function_name,
            function,
        } = self;
        attributes.ast_debug(w);
        for cdef in constants.key_cloned_iter() {
            cdef.ast_debug(w);
            w.new_line();
        }
        (*function_name, function).ast_debug(w);
    }
}

impl AstDebug for ModuleDefinition {
    fn ast_debug(&self, w: &mut AstWriter) {
        let ModuleDefinition {
            attributes,
            is_source_module,
            dependency_order,
            friends,
            structs,
            constants,
            functions,
        } = self;
        attributes.ast_debug(w);
        if *is_source_module {
            w.writeln("library module")
        } else {
            w.writeln("source module")
        }
        w.writeln(&format!("dependency order #{}", dependency_order));
        for (mident, _loc) in friends.key_cloned_iter() {
            w.write(&format!("friend {};", mident));
            w.new_line();
        }
        for sdef in structs.key_cloned_iter() {
            sdef.ast_debug(w);
            w.new_line();
        }
        for cdef in constants.key_cloned_iter() {
            cdef.ast_debug(w);
            w.new_line();
        }
        for fdef in functions.key_cloned_iter() {
            fdef.ast_debug(w);
            w.new_line();
        }
    }
}

impl AstDebug for (ConstantName, &Constant) {
    fn ast_debug(&self, w: &mut AstWriter) {
        let (
            name,
            Constant {
                attributes,
                loc: _loc,
                signature,
                value,
            },
        ) = self;
        attributes.ast_debug(w);
        w.write(&format!("const {}:", name));
        signature.ast_debug(w);
        w.write(" = ");
        match value {
            None => w.write("_|_ /* unfoldable */"),
            Some(v) => v.ast_debug(w),
        }
        w.write(";");
    }
}

impl AstDebug for MoveValue {
    fn ast_debug(&self, w: &mut AstWriter) {
        use MoveValue as V;
        match self {
            V::U8(u) => w.write(&format!("{}", u)),
            V::U64(u) => w.write(&format!("{}", u)),
            V::U128(u) => w.write(&format!("{}", u)),
            V::Bool(b) => w.write(&format!("{}", b)),
            V::Address(a) => w.write(&format!("{}", a)),
            V::Vector(vs) => {
                w.write("vector[");
                w.comma(vs, |w, v| v.ast_debug(w));
                w.write("]");
            }
            V::Struct(_) => panic!("ICE struct constants not supported"),
            V::Signer(_) => panic!("ICE signer constants not supported"),
        }
    }
}

impl AstDebug for (FunctionName, &Function) {
    fn ast_debug(&self, w: &mut AstWriter) {
        let (
            name,
            Function {
                attributes,
                visibility,
                signature,
                acquires,
                body,
            },
        ) = self;
        attributes.ast_debug(w);
        visibility.ast_debug(w);
        if let FunctionBody_::Native = &body.value {
            w.write("native ");
        }
        w.write(&format!("{}", name));
        signature.ast_debug(w);
        if !acquires.is_empty() {
            w.write(" acquires ");
            w.comma(acquires.keys(), |w, s| w.write(&format!("{}", s)));
            w.write(" ");
        }
        match &body.value {
            FunctionBody_::Defined {
                locals,
                start,
                loop_heads,
                blocks,
            } => w.block(|w| {
                w.write("locals:");
                w.indent(4, |w| {
                    w.list(locals, ",", |w, (_, v, st)| {
                        w.write(&format!("{}: ", v));
                        st.ast_debug(w);
                        true
                    })
                });
                w.new_line();
                w.writeln("loop heads:");
                w.indent(4, |w| {
                    for loop_head in loop_heads {
                        w.writeln(&format!("{}", loop_head))
                    }
                });
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

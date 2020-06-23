// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    hlir::ast::{
        BaseType, Command, Command_, FunctionSignature, Label, SingleType, StructDefinition,
    },
    parser::ast::{ConstantName, FunctionName, FunctionVisibility, ModuleIdent, StructName, Var},
    shared::{ast_debug::*, unique_map::UniqueMap},
};
use move_core_types::value::MoveValue;
use move_ir_types::location::*;
use std::collections::{BTreeMap, VecDeque};

// HLIR + Unstructured Control Flow + CFG

//**************************************************************************************************
// Program
//**************************************************************************************************

#[derive(Debug)]
pub struct Program {
    pub modules: UniqueMap<ModuleIdent, ModuleDefinition>,
    pub scripts: BTreeMap<String, Script>,
}

//**************************************************************************************************
// Scripts
//**************************************************************************************************

#[derive(Debug)]
pub struct Script {
    pub loc: Loc,
    pub constants: UniqueMap<ConstantName, Constant>,
    pub function_name: FunctionName,
    pub function: Function,
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
    pub constants: UniqueMap<ConstantName, Constant>,
    pub functions: UniqueMap<FunctionName, Function>,
}

//**************************************************************************************************
// Constants
//**************************************************************************************************

#[derive(PartialEq, Debug)]
pub struct Constant {
    pub loc: Loc,
    pub signature: BaseType,
    pub value: Option<MoveValue>,
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
    pub acquires: BTreeMap<StructName, Loc>,
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
        let Program { modules, scripts } = self;
        for (m, mdef) in modules {
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
            loc: _loc,
            constants,
            function_name,
            function,
        } = self;
        for cdef in constants {
            cdef.ast_debug(w);
            w.new_line();
        }
        (function_name.clone(), function).ast_debug(w);
    }
}

impl AstDebug for ModuleDefinition {
    fn ast_debug(&self, w: &mut AstWriter) {
        let ModuleDefinition {
            is_source_module,
            dependency_order,
            structs,
            constants,
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
        for cdef in constants {
            cdef.ast_debug(w);
            w.new_line();
        }
        for fdef in functions {
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
                loc: _loc,
                signature,
                value,
            },
        ) = self;
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
            w.comma(acquires.keys(), |w, s| w.write(&format!("{}", s)));
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

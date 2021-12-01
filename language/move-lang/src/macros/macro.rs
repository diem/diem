// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    diag,
    expansion::ast::{
        Address, Exp, Exp_, LValue, LValue_, ModuleAccess, ModuleAccess_, ModuleIdent,
        ModuleIdent_, SequenceItem_, Value_,
    },
    parser::ast::{BinOp_, ModuleName},
    shared::{CompilationEnv, Name},
};
use move_ir_types::location::{sp, Loc};
use move_symbol_pool::Symbol;
use std::{
    collections::BTreeMap,
    fmt::{Debug, Formatter},
};

// =================================================================================================
// Macro Definitions

/// A generic name representation of a member of a module. Only named addresses are
/// supported.
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub struct MemberName {
    pub address: Symbol,
    pub module: Symbol,
    pub member: Symbol,
}

/// The definition of a macro.
pub struct MacroDefinition {
    pub name: MemberName,
    pub processor: Box<dyn MacroProcessor>,
}

impl Debug for MacroDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("MacroDefinition {{ name: {:?}, .. }}", self.name))
    }
}

/// A macro processor trait.
pub trait MacroProcessor {
    fn expand(&self, builder: &mut ExpansionBuilder, args: Vec<Exp>) -> Exp;
}

/// The macro registry.
#[derive(Default, Debug)]
pub struct MacroRegistry {
    macros: BTreeMap<MemberName, MacroDefinition>,
}

impl MemberName {
    pub fn create(
        address: impl Into<Symbol>,
        module: impl Into<Symbol>,
        member: impl Into<Symbol>,
    ) -> Self {
        Self {
            address: address.into(),
            module: module.into(),
            member: member.into(),
        }
    }

    /// Constructs a ModuleIdent from a member name.
    pub fn module_ident(&self, loc: Loc) -> ModuleIdent {
        let address = Address::Named(sp(loc, self.address));
        let module = ModuleName(sp(loc, self.module));
        sp(loc, ModuleIdent_ { address, module })
    }

    /// Constructs a name from a member name.
    pub fn name(&self, loc: Loc) -> Name {
        sp(loc, self.member)
    }
}

// =================================================================================================
// Extending the Compilation Environment to Handle Macros

impl CompilationEnv {
    /// Registers a macro in the macro registry.
    pub fn register_macro(
        &mut self,
        address: impl Into<Symbol>,
        module: impl Into<Symbol>,
        member: impl Into<Symbol>,
        processor: Box<dyn MacroProcessor>,
    ) {
        let name = MemberName::create(address, module, member);
        self.macro_registry
            .macros
            .insert(name, MacroDefinition { name, processor });
    }

    /// Expands a macro. Resolves the module access, looks up the macro definition,
    /// and invokes the macro expansion processor.
    pub fn expand_macro(&mut self, loc: Loc, ma: &ModuleAccess, args: Vec<Exp>) -> Exp {
        let name = match &ma.value {
            ModuleAccess_::ModuleAccess(sp!(_, ModuleIdent_ { address, module }), member) => {
                let address = match address {
                    Address::Anonymous(sp!(loc, _)) => {
                        self.add_diag(diag!(
                            NameResolution::UnboundMacro,
                            (
                                *loc,
                                "Macro cannot be invoked with numeric module address".to_string()
                            ),
                        ));
                        "<numeric>".into()
                    }
                    Address::Named(name) => name.value,
                };
                MemberName::create(address, module.0.value, member.value)
            }
            ModuleAccess_::Name(name) => {
                // TODO: should allow simple access, need to handle macros in expansion
                //   for this.
                self.add_diag(diag!(
                    NameResolution::UnboundMacro,
                    (
                        name.loc,
                        "Macro name need to be proceeded by module name (current restriction)"
                            .to_string()
                    ),
                ));
                return sp(loc, Exp_::UnresolvedError);
            }
        };
        let mut builder = ExpansionBuilder::create(self, format!("Expanding macro '{}'", ma), loc);
        if let Some(def) = builder.env.macro_registry.macros.remove(&name) {
            let result = def.processor.expand(&mut builder, args);
            self.macro_registry.macros.insert(name, def);
            result
        } else {
            builder.env.add_diag(diag!(
                NameResolution::UnboundMacro,
                (loc, format!("Unbound macro '{}'", ma)),
            ));
            builder.error()
        }
    }
}

// =================================================================================================
// Expansion Builder Helper Type

/// A helper type for building macro expansions.
#[derive(Debug)]
pub struct ExpansionBuilder<'a> {
    pub env: &'a mut CompilationEnv,
    pub error_context: String,
    pub loc: Loc,
    temp_count: usize,
}

impl<'a> ExpansionBuilder<'a> {
    /// Creates an expansion builder.
    fn create(env: &'a mut CompilationEnv, error_context: String, loc: Loc) -> Self {
        ExpansionBuilder {
            env,
            error_context,
            loc,
            temp_count: 0,
        }
    }

    /// Builds an error expressions.
    pub fn error(&self) -> Exp {
        sp(self.loc, Exp_::UnresolvedError)
    }

    /// Extracts two arguments, reporting error if the number of arguments does not match.
    pub fn extract_args2(&mut self, mut args: Vec<Exp>) -> Option<(Exp, Exp)> {
        if args.len() == 2 {
            let a2 = args.pop().unwrap();
            let a1 = args.pop().unwrap();
            Some((a1, a2))
        } else {
            self.env.add_diag(diag!(
                CompilerMacro::InvalidArgument,
                (
                    self.loc,
                    format!("{}: expected two arguments", self.error_context)
                )
            ));
            None
        }
    }

    /// Extracts lambda bound variables and body.
    pub fn extract_lambda(&mut self, arg: Exp) -> Option<(Vec<Symbol>, Exp)> {
        match arg.value {
            Exp_::Lambda(lvalues, body) => Some((
                lvalues
                    .value
                    .into_iter()
                    .filter_map(|lvalue| match lvalue.value {
                        LValue_::Var(sp!(_, ModuleAccess_::Name(name)), None) => Some(name.value),
                        _ => {
                            self.env.add_diag(diag!(
                                CompilerMacro::InvalidArgument,
                                (
                                    lvalue.loc,
                                    format!(
                                        "{}: expected simple name as lambda argument",
                                        self.error_context
                                    )
                                )
                            ));
                            None
                        }
                    })
                    .collect(),
                *body,
            )),
            _ => {
                self.env.add_diag(diag!(
                    CompilerMacro::InvalidArgument,
                    (
                        arg.loc,
                        format!("{}: argument must be a lambda", self.error_context)
                    )
                ));
                None
            }
        }
    }

    /// Extracts single lambda bound variable and body.
    pub fn extract_lambda1(&mut self, arg: Exp) -> Option<(Symbol, Exp)> {
        let loc = arg.loc;
        self.extract_lambda(arg).and_then(|(mut vars, body)| {
            if vars.len() == 1 {
                Some((vars.pop().unwrap(), body))
            } else {
                self.env.add_diag(diag!(
                    CompilerMacro::InvalidArgument,
                    (
                        loc,
                        format!(
                            "{}: lambda must have exactly one argument",
                            self.error_context
                        )
                    )
                ));
                None
            }
        })
    }

    /// Creates a new distinct local.
    pub fn new_local(&mut self) -> Symbol {
        self.temp_count += 1;
        format!("__m_{}", self.temp_count).into()
    }

    /// Creates a value expressions.
    pub fn value(&self, val: Value_) -> Exp {
        sp(self.loc, Exp_::Value(sp(self.loc, val)))
    }

    /// Creates module access for local.
    pub fn local_access(&self, name: Symbol) -> ModuleAccess {
        sp(self.loc, ModuleAccess_::Name(sp(self.loc, name)))
    }

    /// Creates an lvalue for local.
    pub fn local_lvalue(&self, name: Symbol) -> LValue {
        sp(self.loc, LValue_::Var(self.local_access(name), None))
    }

    /// Creates a reference to a local.
    pub fn local(&self, name: Symbol) -> Exp {
        sp(self.loc, Exp_::Name(self.local_access(name), None))
    }

    /// Calls a function.
    pub fn call(&self, fun: MemberName, args: Vec<Exp>) -> Exp {
        sp(
            self.loc,
            Exp_::Call(
                sp(
                    self.loc,
                    ModuleAccess_::ModuleAccess(fun.module_ident(self.loc), fun.name(self.loc)),
                ),
                false,
                None,
                sp(self.loc, args),
            ),
        )
    }

    /// Calls a binary operator.
    pub fn binary(&self, op: BinOp_, lhs: Exp, rhs: Exp) -> Exp {
        sp(
            self.loc,
            Exp_::BinopExp(Box::new(lhs), sp(self.loc, op), Box::new(rhs)),
        )
    }

    /// Increments a local.
    pub fn increment(&self, local: Symbol) -> Exp {
        sp(
            self.loc,
            Exp_::Assign(
                sp(self.loc, vec![self.local_lvalue(local)]),
                Box::new(self.binary(BinOp_::Add, self.local(local), self.value(Value_::U64(1)))),
            ),
        )
    }

    /// Creates a block from bindings and statements.
    pub fn block(&self, bindings: Vec<(Symbol, Exp)>, stms: Vec<Exp>) -> Exp {
        let items = bindings
            .into_iter()
            .map(|(v, e)| {
                sp(
                    self.loc,
                    SequenceItem_::Bind(sp(self.loc, vec![self.local_lvalue(v)]), e),
                )
            })
            .chain(
                stms.into_iter()
                    .map(|s| sp(self.loc, SequenceItem_::Seq(s))),
            )
            .collect();
        sp(self.loc, Exp_::Block(items))
    }

    /// Creates a while loop.
    pub fn while_(&self, cond: Exp, body: Exp) -> Exp {
        sp(self.loc, Exp_::While(Box::new(cond), Box::new(body)))
    }
}

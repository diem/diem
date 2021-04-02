// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use num::ToPrimitive;
use std::collections::{BTreeMap, BTreeSet};

use bytecode_source_map::source_map::{FunctionSourceMap, SourceMap};
use move_ir_types::location::{sp, Loc as MoveLoc};
use move_lang::{
    compiled_unit::FunctionInfo,
    expansion::ast::{SpecId, Value_ as MoveValue},
    naming::ast::{
        Exp as MoveExp, Exp_ as MoveExp_, Sequence, SequenceItem as MoveStmt,
        SequenceItem_ as MoveStmt_,
    },
    parser::ast::{BinOp_, FunctionName, UnaryOp_, Var},
    shared::{unique_map::UniqueMap, Address},
};
use vm::file_format::FunctionDefinitionIndex;

use crate::{
    ast::{ConditionKind, Exp as SpecExp, Operation, TempIndex, Value as SpecValue},
    exp_generator::ExpGenerator,
    model::{FunctionEnv, GlobalEnv, Loc as SpecLoc},
    spec_translator::TranslatedSpec,
    ty::Type as SpecType,
};

/// A context structure capturing the additional information available in the CompiledUnit
/// that may have lost during the construction of the GlobalEnv.
pub struct CompilationContext {
    source_map: SourceMap<MoveLoc>,
    function_infos: UniqueMap<FunctionName, FunctionInfo>,
}

impl CompilationContext {
    pub fn new(
        source_map: SourceMap<MoveLoc>,
        function_infos: UniqueMap<FunctionName, FunctionInfo>,
    ) -> Self {
        Self {
            source_map,
            function_infos,
        }
    }

    pub fn function_locs(&self, idx: FunctionDefinitionIndex) -> &FunctionSourceMap<MoveLoc> {
        self.source_map.get_function_source_map(idx).unwrap()
    }
}

/// A context structure capturing the information needed / modified during spec translation
pub struct SpecTranslationContext<'env> {
    fun_env: &'env FunctionEnv<'env>,
    current_loc: SpecLoc,
    pub local_types: Vec<SpecType>,
}

impl<'env> SpecTranslationContext<'env> {
    pub fn new(fun_env: &'env FunctionEnv) -> Self {
        let local_types = (0..fun_env.get_local_count())
            .map(|i| fun_env.get_local_type(i))
            .collect();
        Self {
            fun_env,
            current_loc: fun_env.get_loc(),
            local_types,
        }
    }
}

impl<'env> ExpGenerator<'env> for SpecTranslationContext<'env> {
    fn function_env(&self) -> &FunctionEnv<'env> {
        self.fun_env
    }

    fn get_current_loc(&self) -> SpecLoc {
        self.current_loc.clone()
    }

    fn set_loc(&mut self, loc: SpecLoc) {
        self.current_loc = loc;
    }

    fn add_local(&mut self, ty: SpecType) -> TempIndex {
        let idx = self.local_types.len();
        self.local_types.push(ty);
        idx
    }

    fn get_local_type(&self, temp: TempIndex) -> SpecType {
        self.local_types.get(temp).expect("local variable").clone()
    }
}

/// An instrumented block is a self-contained scope for local variables and statements
struct InstrumentedBlock {
    /// The named local variables (local temporaries) accumulated during the instrumentation
    locals: BTreeMap<TempIndex, Var>,
    /// The instrumented function body (i.e., statements accumulated during the instrumentation)
    statements: Sequence,
}

/// Holds the information needed / modified during function instrumentation
pub struct InstrumentedFunction<'env> {
    /// The function being instrumented
    fenv: &'env FunctionEnv<'env>,
    /// Additional information passed from the compiler
    ctxt: &'env CompilationContext,
    /// The translated spec (with old(.) reduction, etc) of this function
    spec: &'env TranslatedSpec,
    /// The block stack
    stack: Vec<InstrumentedBlock>,
}

impl<'env> InstrumentedFunction<'env> {
    pub fn new(
        fenv: &'env FunctionEnv<'env>,
        ctxt: &'env CompilationContext,
        spec: &'env TranslatedSpec,
        local_types: &'env [SpecType],
    ) -> Option<Self> {
        let env = fenv.module_env.env;

        // collect local variables
        let func_locs = ctxt.function_locs(fenv.get_def_idx());
        let locals: BTreeMap<_, _> = func_locs
            .parameters
            .iter()
            .chain(func_locs.locals.iter())
            .cloned()
            .enumerate()
            .map(|(i, (var_name, var_loc))| {
                assert_eq!(
                    env.symbol_pool().string(fenv.get_local_name(i)).as_ref(),
                    &var_name
                );
                (i as TempIndex, Var(sp(var_loc, var_name)))
            })
            .collect();
        assert_eq!(locals.len(), fenv.get_local_count());

        // TODO(mengxu) variables created from old(.) expressions are not supported yet
        // supporting old(.) should be easy for primitive types or structs with copy ability,
        // but other cases will require an auto-generated clone function for the struct type.
        if !spec.saved_memory.is_empty()
            || !spec.saved_spec_vars.is_empty()
            || !spec.saved_params.is_empty()
        {
            env.error(
                &fenv.get_spec_loc(),
                "Support for variables reduced from old(.) expression is not implemented yet",
            );
            return None;
        }
        assert_eq!(local_types.len(), locals.len());

        // create the root block in the stack
        let root = InstrumentedBlock {
            locals,
            statements: Sequence::new(),
        };

        // initialize the instrumented function
        Some(Self {
            fenv,
            ctxt,
            spec,
            stack: vec![root],
        })
    }

    //
    // Instrumentation points
    //

    pub fn instrument_inline_spec(
        &mut self,
        loc: &MoveLoc,
        spec_id: &SpecId,
        used_vars: &BTreeSet<Var>,
    ) {
        let env = self.global_env();
        let info = self.function_info();
        match info.spec_info.get(spec_id) {
            None => env.error(
                &env.to_loc(loc),
                "Unable to find the CodeOffset in FunctionInfo for this spec block",
            ),
            Some(spec_info) => match self.fenv.get_spec().on_impl.get(&spec_info.offset) {
                None => env.error(
                    &env.to_loc(loc),
                    "Unable to find the Spec in FunctionEnv for this spec block",
                ),
                Some(inline_spec) => {
                    // TODO (mengxu): remove these checking once we know that
                    // `spec_info.used_locals` and `vars` are in fact the same thing
                    assert_eq!(spec_info.used_locals.len(), used_vars.len());
                    for v in used_vars {
                        assert!(spec_info.used_locals.contains_key(v));
                    }

                    // an inline spec should have no `on_impl` specs
                    assert!(inline_spec.on_impl.is_empty());

                    // iterate and convert the conditions
                    for cond in &inline_spec.conditions {
                        // only `assert` and `assume` are allowed for in-code spec blocks
                        assert!(matches!(
                            &cond.kind,
                            ConditionKind::Assert | ConditionKind::Assume
                        ));
                        // `assert` and `assume` expressions do not have other info attached
                        assert!(cond.properties.is_empty());
                        assert!(cond.additional_exps.is_empty());

                        // now the actual instrumentation
                        self.add_spec_assert(&cond.loc, &cond.exp);
                    }
                }
            },
        }
    }

    pub fn instrument_pre_conditions(&mut self) {
        for (cond_loc, cond_exp) in &self.spec.pre {
            self.add_spec_assert(cond_loc, cond_exp);
        }
    }

    pub fn instrument_post_conditions(&mut self) {
        for (cond_loc, cond_exp) in &self.spec.post {
            self.add_spec_assert(cond_loc, cond_exp);
        }
    }

    //
    // Move program patterns
    //

    fn add_spec_assert(&mut self, loc: &SpecLoc, exp: &SpecExp) {
        let env = self.global_env();
        let move_loc = env.to_move_loc(&loc);

        // convert the expression
        let cond_exp = self.convert_spec_expression(loc, exp);

        // an "assert <expr>" will be translated into
        // if (expr) { //no-op } else { abort 0 }
        let checker = sp(
            move_loc,
            MoveExp_::IfElse(
                Box::new(cond_exp),
                Box::new(Self::mk_unit(move_loc)),
                Box::new(Self::mk_abort(move_loc)),
            ),
        );
        self.block()
            .statements
            .push_back(sp(move_loc, MoveStmt_::Seq(checker)));
    }

    //
    // Spec expression conversion
    //

    fn convert_spec_expression(&mut self, loc: &SpecLoc, exp: &SpecExp) -> MoveExp {
        match exp {
            SpecExp::Value(_, val) => self.convert_spec_value(loc, val),
            SpecExp::Temporary(_, idx) => self.convert_spec_var_temporary(loc, *idx),
            SpecExp::Call(_, Operation::MaxU8, args) => {
                self.convert_spec_op_const_val(loc, MoveValue::U8(u8::MAX), args)
            }
            SpecExp::Call(_, Operation::MaxU64, args) => {
                self.convert_spec_op_const_val(loc, MoveValue::U64(u64::MAX), args)
            }
            SpecExp::Call(_, Operation::MaxU128, args) => {
                self.convert_spec_op_const_val(loc, MoveValue::U128(u128::MAX), args)
            }
            SpecExp::Call(_, Operation::Not, args) => {
                self.convert_spec_op_unary(loc, UnaryOp_::Not, args)
            }
            SpecExp::Call(_, Operation::Add, args) => {
                self.convert_spec_op_binary(loc, BinOp_::Add, args)
            }
            SpecExp::Call(_, Operation::Sub, args) => {
                self.convert_spec_op_binary(loc, BinOp_::Sub, args)
            }
            SpecExp::Call(_, Operation::Mul, args) => {
                self.convert_spec_op_binary(loc, BinOp_::Mul, args)
            }
            SpecExp::Call(_, Operation::Mod, args) => {
                self.convert_spec_op_binary(loc, BinOp_::Mod, args)
            }
            SpecExp::Call(_, Operation::Div, args) => {
                self.convert_spec_op_binary(loc, BinOp_::Div, args)
            }
            SpecExp::Call(_, Operation::BitOr, args) => {
                self.convert_spec_op_binary(loc, BinOp_::BitOr, args)
            }
            SpecExp::Call(_, Operation::BitAnd, args) => {
                self.convert_spec_op_binary(loc, BinOp_::BitAnd, args)
            }
            SpecExp::Call(_, Operation::Xor, args) => {
                self.convert_spec_op_binary(loc, BinOp_::Xor, args)
            }
            SpecExp::Call(_, Operation::Shl, args) => {
                self.convert_spec_op_binary(loc, BinOp_::Shl, args)
            }
            SpecExp::Call(_, Operation::Shr, args) => {
                self.convert_spec_op_binary(loc, BinOp_::Shr, args)
            }
            SpecExp::Call(_, Operation::And, args) => {
                self.convert_spec_op_binary(loc, BinOp_::And, args)
            }
            SpecExp::Call(_, Operation::Or, args) => {
                self.convert_spec_op_binary(loc, BinOp_::Or, args)
            }
            SpecExp::Call(_, Operation::Eq, args) => {
                self.convert_spec_op_binary(loc, BinOp_::Eq, args)
            }
            SpecExp::Call(_, Operation::Neq, args) => {
                self.convert_spec_op_binary(loc, BinOp_::Neq, args)
            }
            SpecExp::Call(_, Operation::Lt, args) => {
                self.convert_spec_op_binary(loc, BinOp_::Lt, args)
            }
            SpecExp::Call(_, Operation::Gt, args) => {
                self.convert_spec_op_binary(loc, BinOp_::Gt, args)
            }
            SpecExp::Call(_, Operation::Le, args) => {
                self.convert_spec_op_binary(loc, BinOp_::Le, args)
            }
            SpecExp::Call(_, Operation::Ge, args) => {
                self.convert_spec_op_binary(loc, BinOp_::Ge, args)
            }
            // TODO (mengxu) implement rest of operations for call
            SpecExp::Call(_, _, _) => unimplemented!("call"),
            // TODO (mengxu) handle these unimplemented cases
            SpecExp::LocalVar(..) => unimplemented!("local variables (in quantifiers and lambda)"),
            SpecExp::SpecVar(..) => unimplemented!("ghost variables"),
            SpecExp::Lambda(..) => unimplemented!("lambda definition"),
            SpecExp::Invoke(..) => unimplemented!("lambda invocation"),
            SpecExp::Quant(..) => unimplemented!("quantifiers"),
            SpecExp::Block(..) => unimplemented!("block"),
            SpecExp::IfElse(..) => unimplemented!("if-else"),
            // a valid spec-lang ast should not have invalid spec expressions
            SpecExp::Invalid(..) => unreachable!(),
        }
    }

    fn convert_spec_value(&mut self, loc: &SpecLoc, value: &SpecValue) -> MoveExp {
        let env = self.global_env();
        let move_loc = env.to_move_loc(loc);
        let exp = match value {
            SpecValue::Address(v) => {
                let addr = Address::parse_str(&v.to_str_radix(16)).unwrap();
                MoveExp_::Value(sp(move_loc, MoveValue::Address(addr)))
            }
            SpecValue::Number(v) => MoveExp_::InferredNum(v.to_u128().unwrap()),
            SpecValue::Bool(v) => MoveExp_::Value(sp(move_loc, MoveValue::Bool(*v))),
            SpecValue::ByteArray(v) => {
                MoveExp_::Value(sp(move_loc, MoveValue::Bytearray(v.to_owned())))
            }
        };
        sp(move_loc, exp)
    }

    fn convert_spec_var_temporary(&mut self, loc: &SpecLoc, idx: TempIndex) -> MoveExp {
        let env = self.global_env();
        let move_loc = env.to_move_loc(loc);
        match self.block().locals.get(&idx) {
            None => {
                env.error(loc, "Unable to find TempIndex for this Temporary variable");
                Self::mk_unresolved(move_loc)
            }
            Some(var) => {
                // TODO (mengxu) this is too coarse-grained, refined it
                let use_var = sp(move_loc, MoveExp_::Use(var.clone()));
                match self.fenv.get_local_type(idx as usize) {
                    SpecType::Primitive(..)
                    | SpecType::Tuple(..)
                    | SpecType::Vector(..)
                    | SpecType::Struct(..)
                    | SpecType::TypeParameter(..) => use_var,
                    SpecType::Reference(..) => {
                        sp(move_loc, MoveExp_::Dereference(Box::new(use_var)))
                    }
                    _ => unimplemented!(),
                }
            }
        }
    }

    fn convert_spec_op_const_val(
        &mut self,
        loc: &SpecLoc,
        val: MoveValue,
        args: &[SpecExp],
    ) -> MoveExp {
        let env = self.global_env();
        let move_loc = env.to_move_loc(loc);
        if !args.is_empty() {
            env.error(
                loc,
                &format!("Expect no operands for ConstantOp, got: {}", args.len()),
            );
            return Self::mk_unresolved(move_loc);
        }
        sp(move_loc, MoveExp_::Value(sp(move_loc, val)))
    }

    fn convert_spec_op_unary(&mut self, loc: &SpecLoc, op: UnaryOp_, args: &[SpecExp]) -> MoveExp {
        let env = self.global_env();
        let move_loc = env.to_move_loc(loc);
        if args.len() != 1 {
            env.error(
                loc,
                &format!("Expect 1 operand for UnaryOp, got: {}", args.len()),
            );
            return Self::mk_unresolved(move_loc);
        }
        let mut converted_args: Vec<_> = args
            .iter()
            .map(|arg| self.convert_spec_expression(loc, arg))
            .collect();
        let operand = converted_args.pop().unwrap();
        sp(
            move_loc,
            MoveExp_::UnaryExp(sp(move_loc, op), Box::new(operand)),
        )
    }

    fn convert_spec_op_binary(&mut self, loc: &SpecLoc, op: BinOp_, args: &[SpecExp]) -> MoveExp {
        let env = self.global_env();
        let move_loc = env.to_move_loc(loc);
        if args.len() != 2 {
            env.error(
                loc,
                &format!("Expect 2 operands for BinOp, got: {}", args.len()),
            );
            return Self::mk_unresolved(move_loc);
        }
        let mut converted_args: Vec<_> = args
            .iter()
            .map(|arg| self.convert_spec_expression(loc, arg))
            .collect();
        let rhs = converted_args.pop().unwrap();
        let lhs = converted_args.pop().unwrap();
        sp(
            move_loc,
            MoveExp_::BinopExp(Box::new(lhs), sp(move_loc, op), Box::new(rhs)),
        )
    }

    //
    // Move statement builders
    //

    fn mk_unit(loc: MoveLoc) -> MoveExp {
        sp(loc, MoveExp_::Unit { trailing: false })
    }

    fn mk_abort(loc: MoveLoc) -> MoveExp {
        let abort_code_val = sp(loc, MoveValue::U64(0));
        let abort_code_exp = sp(loc, MoveExp_::Value(abort_code_val));
        sp(loc, MoveExp_::Abort(Box::new(abort_code_exp)))
    }

    fn mk_unresolved(loc: MoveLoc) -> MoveExp {
        sp(loc, MoveExp_::UnresolvedError)
    }

    //
    // Instrumentation statement and block management
    //

    fn block(&mut self) -> &mut InstrumentedBlock {
        self.stack.last_mut().unwrap()
    }

    pub fn push_scope(&mut self) {
        let current = self.block();
        let block = InstrumentedBlock {
            locals: current.locals.clone(),
            statements: Sequence::new(),
        };
        self.stack.push(block);
    }

    pub fn pop_scope(&mut self) -> Sequence {
        let block = self.stack.pop().unwrap();
        block.statements
    }

    pub fn end_of_life(&self) -> bool {
        self.stack.is_empty()
    }

    pub fn add_statement(&mut self, stmt: MoveStmt) {
        self.block().statements.push_back(stmt);
    }

    //
    // Other utilities
    //

    pub fn global_env(&self) -> &'env GlobalEnv {
        self.fenv.module_env.env
    }

    fn function_info(&self) -> &FunctionInfo {
        let env = self.global_env();
        self.ctxt
            .function_infos
            .get_(env.symbol_pool().string(self.fenv.get_name()).as_ref())
            .unwrap()
    }
}

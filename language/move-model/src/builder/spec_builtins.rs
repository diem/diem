// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Defines builtin functions for specifications, adding them to the build

use crate::{
    ast::{Operation, Value},
    builder::model_builder::{ConstEntry, ModelBuilder, SpecFunEntry},
    ty::{PrimitiveType, Type},
};
use move_lang::parser::ast::{self as PA};
use num::BigInt;

/// Declares builtins in the build. This adds functions and operators
/// to the build which will be treated the same as user defined specification functions.
pub(crate) fn declare_spec_builtins(trans: &mut ModelBuilder<'_>) {
    let loc = trans.env.internal_loc();
    let bool_t = &Type::new_prim(PrimitiveType::Bool);
    let num_t = &Type::new_prim(PrimitiveType::Num);
    let range_t = &Type::new_prim(PrimitiveType::Range);
    let address_t = &Type::new_prim(PrimitiveType::Address);

    let param_t = &Type::TypeParameter(0);
    let mk_num_const = |value: BigInt| ConstEntry {
        loc: loc.clone(),
        ty: num_t.clone(),
        value: Value::Number(value),
    };

    {
        // Constants
        trans.define_const(
            trans.builtin_qualified_symbol("MAX_U8"),
            mk_num_const(BigInt::from(u8::MAX)),
        );
        trans.define_const(
            trans.builtin_qualified_symbol("MAX_U64"),
            mk_num_const(BigInt::from(u64::MAX)),
        );
        trans.define_const(
            trans.builtin_qualified_symbol("MAX_U128"),
            mk_num_const(BigInt::from(u128::MAX)),
        );
        trans.define_const(
            trans.builtin_qualified_symbol("EXECUTION_FAILURE"),
            mk_num_const(BigInt::from(-1)),
        );

        // Binary operators.
        let mut declare_bin =
            |op: PA::BinOp_, oper: Operation, param_type: &Type, result_type: &Type| {
                trans.define_spec_fun(
                    trans.bin_op_symbol(&op),
                    SpecFunEntry {
                        loc: loc.clone(),
                        oper,
                        type_params: vec![],
                        arg_types: vec![param_type.clone(), param_type.clone()],
                        result_type: result_type.clone(),
                    },
                );
            };
        use PA::BinOp_::*;
        declare_bin(Add, Operation::Add, num_t, num_t);
        declare_bin(Sub, Operation::Sub, num_t, num_t);
        declare_bin(Mul, Operation::Mul, num_t, num_t);
        declare_bin(Mod, Operation::Mod, num_t, num_t);
        declare_bin(Div, Operation::Div, num_t, num_t);
        declare_bin(BitOr, Operation::BitOr, num_t, num_t);
        declare_bin(BitAnd, Operation::BitAnd, num_t, num_t);
        declare_bin(Xor, Operation::Xor, num_t, num_t);
        declare_bin(Shl, Operation::Shl, num_t, num_t);
        declare_bin(Shr, Operation::Shr, num_t, num_t);

        declare_bin(Range, Operation::Range, num_t, range_t);

        declare_bin(Implies, Operation::Implies, bool_t, bool_t);
        declare_bin(Iff, Operation::Iff, bool_t, bool_t);
        declare_bin(And, Operation::And, bool_t, bool_t);
        declare_bin(Or, Operation::Or, bool_t, bool_t);

        declare_bin(Lt, Operation::Lt, num_t, bool_t);
        declare_bin(Le, Operation::Le, num_t, bool_t);
        declare_bin(Gt, Operation::Gt, num_t, bool_t);
        declare_bin(Ge, Operation::Ge, num_t, bool_t);

        // Eq and Neq have special treatment because they are generic.
        trans.define_spec_fun(
            trans.bin_op_symbol(&PA::BinOp_::Eq),
            SpecFunEntry {
                loc: loc.clone(),
                oper: Operation::Eq,
                type_params: vec![param_t.clone()],
                arg_types: vec![param_t.clone(), param_t.clone()],
                result_type: bool_t.clone(),
            },
        );
        trans.define_spec_fun(
            trans.bin_op_symbol(&PA::BinOp_::Neq),
            SpecFunEntry {
                loc: loc.clone(),
                oper: Operation::Neq,
                type_params: vec![param_t.clone()],
                arg_types: vec![param_t.clone(), param_t.clone()],
                result_type: bool_t.clone(),
            },
        );
    }

    {
        // Unary operators.
        trans.define_spec_fun(
            trans.unary_op_symbol(&PA::UnaryOp_::Not),
            SpecFunEntry {
                loc: loc.clone(),
                oper: Operation::Not,
                type_params: vec![],
                arg_types: vec![bool_t.clone()],
                result_type: bool_t.clone(),
            },
        );
    }

    {
        // Builtin functions.
        let vector_t = &Type::Vector(Box::new(param_t.clone()));
        let domain_t = &Type::TypeDomain(Box::new(param_t.clone()));

        // Constants (max_u8(), etc.)
        trans.define_spec_fun(
            trans.builtin_qualified_symbol("max_u8"),
            SpecFunEntry {
                loc: loc.clone(),
                oper: Operation::MaxU8,
                type_params: vec![],
                arg_types: vec![],
                result_type: num_t.clone(),
            },
        );

        trans.define_spec_fun(
            trans.builtin_qualified_symbol("max_u64"),
            SpecFunEntry {
                loc: loc.clone(),
                oper: Operation::MaxU64,
                type_params: vec![],
                arg_types: vec![],
                result_type: num_t.clone(),
            },
        );

        trans.define_spec_fun(
            trans.builtin_qualified_symbol("max_u128"),
            SpecFunEntry {
                loc: loc.clone(),
                oper: Operation::MaxU128,
                type_params: vec![],
                arg_types: vec![],
                result_type: num_t.clone(),
            },
        );

        // Vectors
        trans.define_spec_fun(
            trans.builtin_qualified_symbol("len"),
            SpecFunEntry {
                loc: loc.clone(),
                oper: Operation::Len,
                type_params: vec![param_t.clone()],
                arg_types: vec![vector_t.clone()],
                result_type: num_t.clone(),
            },
        );
        trans.define_spec_fun(
            trans.builtin_qualified_symbol("update"),
            SpecFunEntry {
                loc: loc.clone(),
                oper: Operation::UpdateVec,
                type_params: vec![param_t.clone()],
                arg_types: vec![vector_t.clone(), num_t.clone(), param_t.clone()],
                result_type: vector_t.clone(),
            },
        );
        trans.define_spec_fun(
            trans.builtin_qualified_symbol("vec"),
            SpecFunEntry {
                loc: loc.clone(),
                oper: Operation::EmptyVec,
                type_params: vec![param_t.clone()],
                arg_types: vec![],
                result_type: vector_t.clone(),
            },
        );
        trans.define_spec_fun(
            trans.builtin_qualified_symbol("vec"),
            SpecFunEntry {
                loc: loc.clone(),
                oper: Operation::SingleVec,
                type_params: vec![param_t.clone()],
                arg_types: vec![param_t.clone()],
                result_type: vector_t.clone(),
            },
        );
        trans.define_spec_fun(
            trans.builtin_qualified_symbol("concat"),
            SpecFunEntry {
                loc: loc.clone(),
                oper: Operation::ConcatVec,
                type_params: vec![param_t.clone()],
                arg_types: vec![vector_t.clone(), vector_t.clone()],
                result_type: vector_t.clone(),
            },
        );
        trans.define_spec_fun(
            trans.builtin_qualified_symbol("contains"),
            SpecFunEntry {
                loc: loc.clone(),
                oper: Operation::ContainsVec,
                type_params: vec![param_t.clone()],
                arg_types: vec![vector_t.clone(), param_t.clone()],
                result_type: bool_t.clone(),
            },
        );
        trans.define_spec_fun(
            trans.builtin_qualified_symbol("index_of"),
            SpecFunEntry {
                loc: loc.clone(),
                oper: Operation::IndexOfVec,
                type_params: vec![param_t.clone()],
                arg_types: vec![vector_t.clone(), param_t.clone()],
                result_type: num_t.clone(),
            },
        );
        trans.define_spec_fun(
            trans.builtin_qualified_symbol("in_range"),
            SpecFunEntry {
                loc: loc.clone(),
                oper: Operation::InRangeVec,
                type_params: vec![param_t.clone()],
                arg_types: vec![vector_t.clone(), num_t.clone()],
                result_type: bool_t.clone(),
            },
        );
        trans.define_spec_fun(
            trans.builtin_qualified_symbol("in_range"),
            SpecFunEntry {
                loc: loc.clone(),
                oper: Operation::InRangeRange,
                type_params: vec![],
                arg_types: vec![range_t.clone(), num_t.clone()],
                result_type: bool_t.clone(),
            },
        );
        trans.define_spec_fun(
            trans.builtin_qualified_symbol("range"),
            SpecFunEntry {
                loc: loc.clone(),
                oper: Operation::RangeVec,
                type_params: vec![param_t.clone()],
                arg_types: vec![vector_t.clone()],
                result_type: range_t.clone(),
            },
        );

        // Resources.
        trans.define_spec_fun(
            trans.builtin_qualified_symbol("global"),
            SpecFunEntry {
                loc: loc.clone(),
                oper: Operation::Global(None),
                type_params: vec![param_t.clone()],
                arg_types: vec![address_t.clone()],
                result_type: param_t.clone(),
            },
        );
        // TODO(emmazzz): declaring these as builtins will allow users to
        // use borrow_global and borrow_global_mut in specs. Later we should
        // map them to `global` instead.
        trans.define_spec_fun(
            trans.builtin_qualified_symbol("borrow_global"),
            SpecFunEntry {
                loc: loc.clone(),
                oper: Operation::Global(None),
                type_params: vec![param_t.clone()],
                arg_types: vec![address_t.clone()],
                result_type: param_t.clone(),
            },
        );
        trans.define_spec_fun(
            trans.builtin_qualified_symbol("borrow_global_mut"),
            SpecFunEntry {
                loc: loc.clone(),
                oper: Operation::Global(None),
                type_params: vec![param_t.clone()],
                arg_types: vec![address_t.clone()],
                result_type: param_t.clone(),
            },
        );
        trans.define_spec_fun(
            trans.builtin_qualified_symbol("exists"),
            SpecFunEntry {
                loc: loc.clone(),
                oper: Operation::Exists(None),
                type_params: vec![param_t.clone()],
                arg_types: vec![address_t.clone()],
                result_type: bool_t.clone(),
            },
        );

        trans.define_spec_fun(
            trans.builtin_qualified_symbol("$spec_domain"),
            SpecFunEntry {
                loc: loc.clone(),
                oper: Operation::TypeDomain,
                type_params: vec![param_t.clone()],
                arg_types: vec![],
                result_type: domain_t.clone(),
            },
        );

        // Old
        trans.define_spec_fun(
            trans.builtin_qualified_symbol("old"),
            SpecFunEntry {
                loc: loc.clone(),
                oper: Operation::Old,
                type_params: vec![param_t.clone()],
                arg_types: vec![param_t.clone()],
                result_type: param_t.clone(),
            },
        );

        // Tracing
        trans.define_spec_fun(
            trans.builtin_qualified_symbol("TRACE"),
            SpecFunEntry {
                loc,
                oper: Operation::Trace,
                type_params: vec![param_t.clone()],
                arg_types: vec![param_t.clone()],
                result_type: param_t.clone(),
            },
        );
    }
}

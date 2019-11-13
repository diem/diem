// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    loaded_data::{struct_def::StructDef, types::Type},
    native_functions::dispatch::{native_gas, NativeResult},
    native_structs::def::{NativeStructTag, NativeStructType},
    pop_arg,
    value::{MutVal, ReferenceValue, Value},
};
use libra_types::{
    language_storage::TypeTag,
    vm_error::{sub_status::NFE_VECTOR_ERROR_BASE, StatusCode, VMStatus},
};
use serde::Serialize;
use std::{collections::VecDeque, ops::Add};
use vm::errors::VMResult;
use vm::gas_schedule::{
    AbstractMemorySize, CostTable, GasAlgebra, GasCarrier, NativeCostIndex, STRUCT_SIZE,
};

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct NativeVector(pub(crate) Vec<MutVal>);

pub const INDEX_OUT_OF_BOUNDS: u64 = NFE_VECTOR_ERROR_BASE + 1;
pub const POP_EMPTY_VEC: u64 = NFE_VECTOR_ERROR_BASE + 2;
pub const DESTROY_NON_EMPTY_VEC: u64 = NFE_VECTOR_ERROR_BASE + 3;

macro_rules! pop_ref {
    ($args: expr) => {
        match $args.pop_front() {
            Some(v) => v.value_as::<ReferenceValue>()?,
            None => {
                return Err(VMStatus::new(StatusCode::UNREACHABLE)
                    .with_message("native vector reference must exist".to_string()));
            }
        };
    };
}

macro_rules! ensure_len {
    ($v: expr, $expected_len: expr, $type: expr, $fn: expr) => {{
        let actual_len = ($v).len();
        let expected_len = $expected_len;
        if actual_len != expected_len {
            let msg = format!(
                "wrong number of {} for {} expected {} found {}",
                ($type),
                ($fn),
                expected_len,
                actual_len,
            );
            return Err(VMStatus::new(StatusCode::UNREACHABLE).with_message(msg));
        }
    }};
}

impl NativeVector {
    pub fn native_empty(
        type_args: Vec<TypeTag>,
        args: VecDeque<Value>,
        cost_table: &CostTable,
    ) -> VMResult<NativeResult> {
        ensure_len!(type_args, 1, "type arguments", "empty");
        ensure_len!(args, 0, "arguments", "empty");

        let cost = native_gas(cost_table, NativeCostIndex::EMPTY, 1);
        let vec = NativeVector(vec![]);

        Ok(NativeResult::ok(cost, vec![Value::native_vector(vec)]))
    }

    pub fn native_length(
        type_args: Vec<TypeTag>,
        mut args: VecDeque<Value>,
        cost_table: &CostTable,
    ) -> VMResult<NativeResult> {
        ensure_len!(type_args, 1, "type arguments", "length");
        ensure_len!(args, 1, "arguments", "length");

        let r = pop_ref!(args);
        let v = r.borrow_as::<NativeVector>()?;

        let v_len = v.0.len();
        let cost = native_gas(cost_table, NativeCostIndex::LENGTH, 1);

        Ok(NativeResult::ok(cost, vec![Value::u64(v_len as u64)]))
    }

    pub fn native_push_back(
        type_args: Vec<TypeTag>,
        mut args: VecDeque<Value>,
        cost_table: &CostTable,
    ) -> VMResult<NativeResult> {
        ensure_len!(type_args, 1, "type arguments", "push back");
        ensure_len!(args, 2, "arguments", "push back");

        let r = pop_ref!(args);
        let elem = match args.pop_front() {
            Some(v) => MutVal::new(v),
            None => {
                return Err(VMStatus::new(StatusCode::UNREACHABLE).with_message(
                    "wrong number of arguments for push_back, expected value".to_string(),
                ));
            }
        };

        let cost = cost_table
            .native_cost(NativeCostIndex::PUSH_BACK)
            .total()
            .mul(elem.size());

        let mut v = r.borrow_mut_as::<NativeVector>()?;
        v.0.push(elem);

        Ok(NativeResult::ok(cost, vec![]))
    }

    pub fn native_borrow(
        type_args: Vec<TypeTag>,
        mut args: VecDeque<Value>,
        cost_table: &CostTable,
    ) -> VMResult<NativeResult> {
        ensure_len!(type_args, 1, "type arguments", "borrow");
        ensure_len!(args, 2, "arguments", "borrow");

        let r = pop_ref!(args);
        let idx = pop_arg!(args, u64);
        let cost = native_gas(cost_table, NativeCostIndex::BORROW, 1);
        let v = r.borrow_as::<NativeVector>()?;

        match v.0.get(idx as usize) {
            Some(val) => Ok(NativeResult::ok(cost, vec![r.extend_ref(val.clone())])),
            None => {
                let err = VMStatus::new(StatusCode::NATIVE_FUNCTION_ERROR)
                    .with_sub_status(INDEX_OUT_OF_BOUNDS);
                Ok(NativeResult::err(cost, err))
            }
        }
    }

    pub fn native_pop(
        type_args: Vec<TypeTag>,
        mut args: VecDeque<Value>,
        cost_table: &CostTable,
    ) -> VMResult<NativeResult> {
        ensure_len!(type_args, 1, "type arguments", "pop");
        ensure_len!(args, 1, "arguments", "pop");

        let r = pop_ref!(args);
        let mut v = r.borrow_mut_as::<NativeVector>()?;
        let cost = native_gas(cost_table, NativeCostIndex::POP_BACK, 1);

        match v.0.pop() {
            Some(val) => Ok(NativeResult::ok(cost, vec![val.into_value()?])),
            None => Ok(NativeResult::err(
                cost,
                VMStatus::new(StatusCode::NATIVE_FUNCTION_ERROR).with_sub_status(POP_EMPTY_VEC),
            )),
        }
    }

    pub fn native_destroy_empty(
        type_args: Vec<TypeTag>,
        mut args: VecDeque<Value>,
        cost_table: &CostTable,
    ) -> VMResult<NativeResult> {
        ensure_len!(type_args, 1, "type arguments", "destroy empty");
        ensure_len!(args, 1, "arguments", "destroy empty");

        let cost = native_gas(cost_table, NativeCostIndex::DESTROY_EMPTY, 1);
        let v = args.pop_front().unwrap().value_as::<NativeVector>()?;

        if v.0.is_empty() {
            Ok(NativeResult::ok(cost, vec![]))
        } else {
            Ok(NativeResult::err(
                cost,
                VMStatus::new(StatusCode::NATIVE_FUNCTION_ERROR)
                    .with_sub_status(DESTROY_NON_EMPTY_VEC),
            ))
        }
    }

    pub fn native_swap(
        type_args: Vec<TypeTag>,
        mut args: VecDeque<Value>,
        cost_table: &CostTable,
    ) -> VMResult<NativeResult> {
        ensure_len!(type_args, 1, "type arguments", "swap");
        ensure_len!(args, 3, "arguments", "swap");

        let r = pop_ref!(args);
        let idx1 = pop_arg!(args, u64) as usize;
        let idx2 = pop_arg!(args, u64) as usize;

        let cost = native_gas(cost_table, NativeCostIndex::SWAP, 1);

        // We need to check the indices before performing the swap in order to make sure the
        // indices are within bounds.
        let mut v = r.borrow_mut_as::<NativeVector>()?;
        let len = v.0.len();

        if idx1 >= len || idx2 >= len {
            return Ok(NativeResult::err(
                cost,
                VMStatus::new(StatusCode::NATIVE_FUNCTION_ERROR)
                    .with_sub_status(INDEX_OUT_OF_BOUNDS),
            ));
        }

        v.0.swap(idx1, idx2);
        Ok(NativeResult::ok(cost, vec![]))
    }

    pub fn size(&self) -> AbstractMemorySize<GasCarrier> {
        self.0
            .iter()
            .fold(*STRUCT_SIZE, |acc, vl| acc.map2(vl.size(), Add::add))
    }

    #[allow(non_snake_case)]
    #[doc(hidden)]
    pub(crate) fn to_struct_def_FOR_TESTING(&self) -> StructDef {
        let elem_ty = self
            .0
            .get(0)
            .map(|v| v.to_type_FOR_TESTING())
            .unwrap_or(Type::Bool);
        StructDef::Native(NativeStructType::new(
            NativeStructTag::Vector,
            vec![elem_ty],
        ))
    }
}

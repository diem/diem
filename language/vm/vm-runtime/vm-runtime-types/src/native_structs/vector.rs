// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    loaded_data::{struct_def::StructDef, types::Type},
    native_functions::dispatch::{native_gas, NativeResult},
    native_structs::def::{NativeStructTag, NativeStructType},
    pop_arg_front,
    value::{DirectRef, MutVal, Reference, Referenceable, VMRef, VMRefMut, Value, ValueImpl},
};
use libra_types::{
    language_storage::TypeTag,
    vm_error::{sub_status::NFE_VECTOR_ERROR_BASE, StatusCode, VMStatus},
};
use serde::Serialize;
use std::{
    cell::{Ref, RefMut},
    collections::VecDeque,
    mem::replace,
    ops::Add,
};
use vm::errors::VMResult;
use vm::gas_schedule::{
    words_in, AbstractMemorySize, CostTable, GasAlgebra, GasCarrier, NativeCostIndex,
    REFERENCE_SIZE, STRUCT_SIZE,
};

/// A Move native vector with memory layout specialized for certain primitive types.
#[derive(Debug, Clone)]
pub(crate) enum NativeVector {
    U64(Vec<u64>),
    Bool(Vec<bool>),
    General(Vec<ValueImpl>),
}

impl Serialize for NativeVector {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;

        macro_rules! serialize_elements {
            ($v: expr) => {{
                let v = &$v;
                let mut t = serializer.serialize_seq(Some(v.len()))?;
                for x in $v {
                    t.serialize_element(x)?;
                }
                t.end()
            }};
        }

        match self {
            Self::U64(v) => serialize_elements!(v),
            Self::Bool(v) => serialize_elements!(v),
            Self::General(v) => serialize_elements!(v),
        }
    }
}

pub const INDEX_OUT_OF_BOUNDS: u64 = NFE_VECTOR_ERROR_BASE + 1;
pub const POP_EMPTY_VEC: u64 = NFE_VECTOR_ERROR_BASE + 2;
pub const DESTROY_NON_EMPTY_VEC: u64 = NFE_VECTOR_ERROR_BASE + 3;

/// An indirect reference to an element stored in a `NativeVector`.
///
/// Consists of a `DirecRef` to the `NativeVector` and the index of the element referenced.
#[derive(Debug, Clone)]
pub struct VectorElemRef {
    v_ref: DirectRef,
    idx: usize,
}

impl VectorElemRef {
    pub(crate) fn size(&self) -> AbstractMemorySize<GasCarrier> {
        words_in(*REFERENCE_SIZE)
    }

    pub(crate) fn pretty_string(&self) -> String {
        format!("({}, idx: {})", self.v_ref.pretty_string(), self.idx)
    }
}

impl Referenceable for VectorElemRef {
    fn borrow(&self) -> VMResult<VMRef> {
        let r = self.v_ref.borrow_as::<NativeVector>()?;

        macro_rules! borrow_native {
            ($v: expr, $ty: ty, $tc: ident) => {{
                match $v.get(self.idx) {
                    Some(x) => {
                        let raw = x as *const $ty;
                        Ok(VMRef::$tc(Ref::map(r, |_| unsafe { &*raw })))
                    }
                    None => Err(VMStatus::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                        .with_message("vector elem ref index out of bounds".to_string())),
                }
            }};
        }

        match &*r {
            NativeVector::U64(v) => borrow_native!(v, u64, U64),
            NativeVector::Bool(v) => borrow_native!(v, bool, Bool),
            NativeVector::General(v) => match v.get(self.idx) {
                Some(x) => {
                    let raw = x as *const ValueImpl;
                    VMRef::from_value_impl_ref(Ref::map(r, |_| unsafe { &*raw }))
                }
                None => Err(VMStatus::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                    .with_message("vector elem ref index out of bounds".to_string())),
            },
        }
    }

    fn borrow_mut(&self) -> VMResult<VMRefMut> {
        let mut r = self.v_ref.borrow_mut_as::<NativeVector>()?;

        macro_rules! borrow_native_mut {
            ($v: expr, $ty: ty, $tc: ident) => {{
                match $v.get_mut(self.idx) {
                    Some(b) => {
                        let raw = b as *mut $ty;
                        Ok(VMRefMut::$tc(RefMut::map(r, |_| unsafe { &mut *raw })))
                    }
                    None => Err(VMStatus::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                        .with_message("vector elem ref index out of bounds".to_string())),
                }
            }};
        }

        match &mut *r {
            NativeVector::U64(v) => borrow_native_mut!(v, u64, U64),
            NativeVector::Bool(v) => borrow_native_mut!(v, bool, Bool),
            NativeVector::General(v) => match v.get_mut(self.idx) {
                Some(x) => {
                    let raw = x as *mut ValueImpl;
                    VMRefMut::from_value_impl_ref_mut(RefMut::map(r, |_| unsafe { &mut *raw }))
                }
                None => Err(VMStatus::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                    .with_message("vector elem ref index out of bounds".to_string())),
            },
        }
    }
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

macro_rules! err_vector_elem_ty_mismatch {
    () => {{
        return Err(VMStatus::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
            .with_message("vector elem type mismatch".to_string()));
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
        let vec = match &type_args[0] {
            TypeTag::U64 => Self::U64(vec![]),
            TypeTag::Bool => Self::Bool(vec![]),
            _ => Self::General(vec![]),
        };

        Ok(NativeResult::ok(cost, vec![Value::native_vector(vec)]))
    }

    pub fn native_length(
        type_args: Vec<TypeTag>,
        mut args: VecDeque<Value>,
        cost_table: &CostTable,
    ) -> VMResult<NativeResult> {
        ensure_len!(type_args, 1, "type arguments", "length");
        ensure_len!(args, 1, "arguments", "length");

        let r = pop_arg_front!(args, Reference);
        let v = r.borrow_as::<NativeVector>()?;

        let cost = native_gas(cost_table, NativeCostIndex::LENGTH, 1);
        let v_len = match (&type_args[0], &*v) {
            (TypeTag::U64, Self::U64(v)) => v.len(),
            (TypeTag::Bool, Self::Bool(v)) => v.len(),
            (TypeTag::Struct(_), Self::General(v))
            | (TypeTag::Address, Self::General(v))
            | (TypeTag::ByteArray, Self::General(v)) => v.len(),
            _ => err_vector_elem_ty_mismatch!(),
        };

        Ok(NativeResult::ok(cost, vec![Value::u64(v_len as u64)]))
    }

    pub fn native_push_back(
        type_args: Vec<TypeTag>,
        mut args: VecDeque<Value>,
        cost_table: &CostTable,
    ) -> VMResult<NativeResult> {
        ensure_len!(type_args, 1, "type arguments", "push back");
        ensure_len!(args, 2, "arguments", "push back");

        let r = pop_arg_front!(args, Reference);
        let elem = match args.pop_front() {
            Some(v) => v,
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
        match (&type_args[0], &mut *v) {
            (TypeTag::U64, Self::U64(v)) => v.push(elem.value_as::<u64>()?),
            (TypeTag::Bool, Self::Bool(v)) => v.push(elem.value_as::<bool>()?),
            (TypeTag::Struct(_), Self::General(v))
            | (TypeTag::Address, Self::General(v))
            | (TypeTag::ByteArray, Self::General(v)) => v.push(elem.0),
            _ => err_vector_elem_ty_mismatch!(),
        }

        Ok(NativeResult::ok(cost, vec![]))
    }

    pub fn native_borrow(
        type_args: Vec<TypeTag>,
        mut args: VecDeque<Value>,
        cost_table: &CostTable,
    ) -> VMResult<NativeResult> {
        ensure_len!(type_args, 1, "type arguments", "borrow");
        ensure_len!(args, 2, "arguments", "borrow");

        let r = pop_arg_front!(args, DirectRef);
        let idx = pop_arg_front!(args, u64) as usize;
        let cost = native_gas(cost_table, NativeCostIndex::BORROW, 1);

        macro_rules! borrow_native {
            ($v: expr) => {{
                if idx >= $v.len() {
                    return Ok(NativeResult::err(
                        cost,
                        VMStatus::new(StatusCode::NATIVE_FUNCTION_ERROR)
                            .with_sub_status(INDEX_OUT_OF_BOUNDS),
                    ));
                }
                Value::vector_elem_ref(VectorElemRef {
                    v_ref: r.clone(),
                    idx,
                })
            }};
        }

        let res = {
            let mut v = r.borrow_mut_as::<NativeVector>()?;

            match (&type_args[0], &mut *v) {
                (TypeTag::U64, Self::U64(v)) => borrow_native!(v),
                (TypeTag::Bool, Self::Bool(v)) => borrow_native!(v),
                (TypeTag::Struct(_), Self::General(v))
                | (TypeTag::Address, Self::General(v))
                | (TypeTag::ByteArray, Self::General(v)) => match v.get_mut(idx) {
                    // When borrowing an element of a vector with general layout, we need
                    // to promote it to a `MutVal` and return a `DirectRef` to the element.
                    // This is to allow the reference to be further extended. (Consider a
                    // vector in another vector.)
                    Some(ValueImpl::PromotedReference(mv)) => {
                        Value::direct_ref(r.extend(mv.clone()))
                    }
                    Some(local_ref) => {
                        // TODO: get rid of this clone?
                        let mv = MutVal::new(Value::new(local_ref.clone()));
                        let new_local_ref = ValueImpl::PromotedReference(mv.clone());
                        replace(local_ref, new_local_ref);
                        Value::direct_ref(r.extend(mv))
                    }
                    None => {
                        return Ok(NativeResult::err(
                            cost,
                            VMStatus::new(StatusCode::NATIVE_FUNCTION_ERROR)
                                .with_sub_status(INDEX_OUT_OF_BOUNDS),
                        ));
                    }
                },
                _ => err_vector_elem_ty_mismatch!(),
            }
        };

        Ok(NativeResult::ok(cost, vec![res]))
    }

    pub fn native_pop(
        type_args: Vec<TypeTag>,
        mut args: VecDeque<Value>,
        cost_table: &CostTable,
    ) -> VMResult<NativeResult> {
        ensure_len!(type_args, 1, "type arguments", "pop");
        ensure_len!(args, 1, "arguments", "pop");

        let r = pop_arg_front!(args, Reference);
        let mut v = r.borrow_mut_as::<NativeVector>()?;
        let cost = native_gas(cost_table, NativeCostIndex::POP_BACK, 1);

        match (&type_args[0], &mut *v) {
            (TypeTag::U64, Self::U64(v)) => match v.pop() {
                Some(x) => Ok(NativeResult::ok(cost, vec![Value::u64(x)])),
                None => Ok(NativeResult::err(
                    cost,
                    VMStatus::new(StatusCode::NATIVE_FUNCTION_ERROR).with_sub_status(POP_EMPTY_VEC),
                )),
            },
            (TypeTag::Bool, Self::Bool(v)) => match v.pop() {
                Some(b) => Ok(NativeResult::ok(cost, vec![Value::bool(b)])),
                None => Ok(NativeResult::err(
                    cost,
                    VMStatus::new(StatusCode::NATIVE_FUNCTION_ERROR).with_sub_status(POP_EMPTY_VEC),
                )),
            },
            (TypeTag::Struct(_), Self::General(v))
            | (TypeTag::Address, Self::General(v))
            | (TypeTag::ByteArray, Self::General(v)) => match v.pop() {
                Some(v) => Ok(NativeResult::ok(cost, vec![Value(v)])),
                None => Ok(NativeResult::err(
                    cost,
                    VMStatus::new(StatusCode::NATIVE_FUNCTION_ERROR).with_sub_status(POP_EMPTY_VEC),
                )),
            },
            _ => err_vector_elem_ty_mismatch!(),
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

        let is_empty = match (&type_args[0], v) {
            (TypeTag::U64, Self::U64(v)) => v.is_empty(),
            (TypeTag::Bool, Self::Bool(v)) => v.is_empty(),
            (TypeTag::Struct(_), Self::General(v))
            | (TypeTag::Address, Self::General(v))
            | (TypeTag::ByteArray, Self::General(v)) => v.is_empty(),
            _ => err_vector_elem_ty_mismatch!(),
        };

        if is_empty {
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

        let r = pop_arg_front!(args, Reference);
        let idx1 = pop_arg_front!(args, u64) as usize;
        let idx2 = pop_arg_front!(args, u64) as usize;

        let cost = native_gas(cost_table, NativeCostIndex::SWAP, 1);

        let mut v = r.borrow_mut_as::<NativeVector>()?;

        macro_rules! swap {
            ($v: ident) => {{
                if idx1 >= $v.len() || idx2 >= $v.len() {
                    return Ok(NativeResult::err(
                        cost,
                        VMStatus::new(StatusCode::NATIVE_FUNCTION_ERROR)
                            .with_sub_status(INDEX_OUT_OF_BOUNDS),
                    ));
                }
                $v.swap(idx1, idx2);
            }};
        }

        match (&type_args[0], &mut *v) {
            (TypeTag::U64, Self::U64(v)) => swap!(v),
            (TypeTag::Bool, Self::Bool(v)) => swap!(v),
            (TypeTag::Struct(_), Self::General(v))
            | (TypeTag::Address, Self::General(v))
            | (TypeTag::ByteArray, Self::General(v)) => swap!(v),
            _ => err_vector_elem_ty_mismatch!(),
        }

        Ok(NativeResult::ok(cost, vec![]))
    }

    pub fn size(&self) -> AbstractMemorySize<GasCarrier> {
        match self {
            Self::U64(v) => {
                vm::gas_schedule::CONST_SIZE.mul(AbstractMemorySize::new(v.len() as GasCarrier))
            }
            Self::Bool(v) => {
                vm::gas_schedule::CONST_SIZE.mul(AbstractMemorySize::new(v.len() as GasCarrier))
            }
            Self::General(v) => v
                .iter()
                .fold(*STRUCT_SIZE, |acc, vl| acc.map2(vl.size(), Add::add)),
        }
    }

    #[allow(non_snake_case)]
    #[doc(hidden)]
    pub(crate) fn to_struct_def_FOR_TESTING(&self) -> StructDef {
        let elem_ty = match self {
            Self::U64(_) => Type::U64,
            Self::Bool(_) => Type::Bool,
            Self::General(v) => v
                .get(0)
                .map(|v| v.to_type_FOR_TESTING())
                .unwrap_or(Type::Bool),
        };
        StructDef::Native(NativeStructType::new(
            NativeStructTag::Vector,
            vec![elem_ty],
        ))
    }
}

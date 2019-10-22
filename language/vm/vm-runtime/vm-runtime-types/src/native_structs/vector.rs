use crate::{
    native_functions::dispatch::NativeResult,
    native_structs::NativeStructValue,
    pop_arg,
    value::{MutVal, ReferenceValue, Value},
};
use libra_types::vm_error::{sub_status::NFE_VECTOR_ERROR_BASE, StatusCode, StatusType, VMStatus};
use serde::Serialize;
use std::{collections::VecDeque, ops::Add};
use vm::errors::VMResult;
use vm::gas_schedule::{AbstractMemorySize, GasAlgebra, GasCarrier, STRUCT_SIZE};

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct NativeVector(pub(crate) Vec<MutVal>);

const BORROW_COST: u64 = 30; // TODO: determine experimentally
const EMPTY_COST: u64 = 30; // TODO: determine experimentally
const LENGTH_COST: u64 = 30; // TODO: determine experimentally
const PUSH_BACK_COST: u64 = 30; // TODO: determine experimentally
const POP_COST: u64 = 30; // TODO: determine experimentally
const DESTROY_EMPTY_VEC_COST: u64 = 30; // TODO: determine experimentally
const SWAP_COST: u64 = 30; // TODO: determine experimentally

pub const INDEX_OUT_OF_BOUNDS: u64 = NFE_VECTOR_ERROR_BASE + 1;
pub const POP_EMPTY_VEC: u64 = NFE_VECTOR_ERROR_BASE + 2;
pub const DESTROY_NON_EMPTY_VEC: u64 = NFE_VECTOR_ERROR_BASE + 3;

#[allow(dead_code)]
fn get_mut_vector(v: &mut NativeStructValue) -> VMResult<&mut NativeVector> {
    match v {
        NativeStructValue::Vector(v) => Ok(v),
    }
}

fn get_vector(v: &NativeStructValue) -> VMResult<&NativeVector> {
    match v {
        NativeStructValue::Vector(v) => Ok(v),
    }
}

macro_rules! get_vector_ref {
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

impl NativeVector {
    pub fn native_empty(_args: VecDeque<Value>) -> VMResult<NativeResult> {
        Ok(NativeResult::ok(
            EMPTY_COST,
            vec![Value::native_struct(NativeStructValue::Vector(
                NativeVector(vec![]),
            ))],
        ))
    }

    pub fn native_length(mut args: VecDeque<Value>) -> VMResult<NativeResult> {
        let reference: ReferenceValue = get_vector_ref!(args);
        reference.read_native_struct(|struct_ref| {
            get_vector(struct_ref).and_then(|native_vec| {
                Ok(NativeResult::ok(
                    LENGTH_COST,
                    vec![Value::u64(native_vec.0.len() as u64)],
                ))
            })
        })
    }

    pub fn native_push_back(mut args: VecDeque<Value>) -> VMResult<NativeResult> {
        if args.len() != 2 {
            let msg = format!(
                "wrong number of arguments for push_back expected 2 found {}",
                args.len()
            );
            return Err(VMStatus::new(StatusCode::UNREACHABLE).with_message(msg));
        }
        let reference: ReferenceValue = get_vector_ref!(args);
        let elem = match args.pop_front() {
            Some(v) => MutVal::new(v),
            None => {
                return Err(VMStatus::new(StatusCode::UNREACHABLE).with_message(
                    "wrong number of arguments for push_back, expected value".to_string(),
                ));
            }
        };
        reference.mutate_native_struct(|struct_ref| {
            get_mut_vector(struct_ref).and_then(|native_vec| {
                native_vec.0.push(elem);
                Ok(NativeResult::ok(PUSH_BACK_COST, vec![]))
            })
        })
    }

    pub fn native_borrow(mut args: VecDeque<Value>) -> VMResult<NativeResult> {
        if args.len() != 2 {
            let msg = format!(
                "wrong number of arguments for borrow expected 2 found {}",
                args.len()
            );
            return Err(VMStatus::new(StatusCode::UNREACHABLE).with_message(msg));
        }
        let reference: ReferenceValue = get_vector_ref!(args);
        let idx = pop_arg!(args, u64);
        match reference.get_native_struct_reference(|struct_ref| {
            get_vector(struct_ref).and_then(|native_vec| match native_vec.0.get(idx as usize) {
                Some(val) => Ok(val.clone()),
                None => Err(VMStatus::new(StatusCode::NATIVE_FUNCTION_ERROR)
                    .with_sub_status(INDEX_OUT_OF_BOUNDS)),
            })
        }) {
            Ok(val) => Ok(NativeResult::ok(BORROW_COST, vec![val])),
            Err(err) => {
                if err.is(StatusType::InvariantViolation) {
                    Ok(NativeResult::err(BORROW_COST, err))
                } else {
                    Err(err)
                }
            }
        }
    }

    pub fn native_pop(mut args: VecDeque<Value>) -> VMResult<NativeResult> {
        if args.len() != 1 {
            let msg = format!(
                "wrong number of arguments for pop expected 1 found {}",
                args.len()
            );
            return Err(VMStatus::new(StatusCode::UNREACHABLE).with_message(msg));
        }

        let reference: ReferenceValue = get_vector_ref!(args);
        reference.mutate_native_struct(|struct_ref| {
            get_mut_vector(struct_ref).and_then(|native_vec| match native_vec.0.pop() {
                Some(val) => Ok(NativeResult::ok(POP_COST, vec![val.into_value()?])),
                None => Ok(NativeResult::err(
                    POP_COST,
                    VMStatus::new(StatusCode::NATIVE_FUNCTION_ERROR).with_sub_status(POP_EMPTY_VEC),
                )),
            })
        })
    }

    pub fn native_destroy_empty(mut args: VecDeque<Value>) -> VMResult<NativeResult> {
        if let Some(v) = args.pop_front() {
            if let Ok(NativeStructValue::Vector(NativeVector(v))) =
                v.value_as::<NativeStructValue>()
            {
                return if v.is_empty() {
                    Ok(NativeResult::ok(DESTROY_EMPTY_VEC_COST, vec![]))
                } else {
                    Ok(NativeResult::err(
                        DESTROY_EMPTY_VEC_COST,
                        VMStatus::new(StatusCode::NATIVE_FUNCTION_ERROR)
                            .with_sub_status(DESTROY_NON_EMPTY_VEC),
                    ))
                };
            }
        }
        Err(VMStatus::new(StatusCode::UNREACHABLE)
            .with_message("bad arguments to destroy_empty".to_string()))
    }

    pub fn native_swap(mut args: VecDeque<Value>) -> VMResult<NativeResult> {
        if args.len() != 3 {
            let msg = format!(
                "wrong number of arguments for swap expected 3 found {}",
                args.len()
            );
            return Err(VMStatus::new(StatusCode::UNREACHABLE).with_message(msg));
        }

        let reference: ReferenceValue = get_vector_ref!(args);
        let index1 = pop_arg!(args, u64);
        let index2 = pop_arg!(args, u64);

        // We need to check the indices before performing the swap in order to make sure the
        // indices are within bounds.
        let len = reference.read_native_struct(|struct_ref| {
            get_vector(struct_ref).and_then(|native_vec| Ok(native_vec.0.len() as u64))
        })?;
        if index1 >= len || index2 >= len {
            return Ok(NativeResult::err(
                SWAP_COST,
                VMStatus::new(StatusCode::NATIVE_FUNCTION_ERROR)
                    .with_sub_status(INDEX_OUT_OF_BOUNDS),
            ));
        }

        reference.mutate_native_struct(|struct_ref| {
            get_mut_vector(struct_ref).and_then(|native_vec| {
                native_vec.0.swap(index1 as usize, index2 as usize);
                Ok(NativeResult::ok(SWAP_COST, vec![]))
            })
        })
    }

    pub(crate) fn get(&self, idx: u64) -> Option<MutVal> {
        self.0.get(idx as usize).map(MutVal::clone)
    }

    pub fn size(&self) -> AbstractMemorySize<GasCarrier> {
        self.0
            .iter()
            .fold(*STRUCT_SIZE, |acc, vl| acc.map2(vl.size(), Add::add))
    }
}

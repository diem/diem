use crate::{
    native_functions::dispatch::NativeReturnStatus,
    native_structs::NativeStructValue,
    pop_arg,
    value::{MutVal, ReferenceValue, Value},
};
use libra_types::vm_error::{sub_status::NFE_VECTOR_ERROR_BASE, StatusCode, VMStatus};
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
            Some(v) => match v.value_as::<ReferenceValue>() {
                Ok(reference) => reference,
                Err(err) => return NativeReturnStatus::InvariantError(err),
            },
            None => {
                return NativeReturnStatus::InvariantError(
                    VMStatus::new(StatusCode::UNREACHABLE)
                        .with_message("native vector reference must exist".to_string()),
                )
            }
        };
    };
}

impl NativeVector {
    pub fn native_empty(_args: VecDeque<Value>) -> NativeReturnStatus {
        NativeReturnStatus::Success {
            return_values: vec![Value::native_struct(NativeStructValue::Vector(
                NativeVector(vec![]),
            ))],
            cost: EMPTY_COST,
        }
    }

    pub fn native_length(mut args: VecDeque<Value>) -> NativeReturnStatus {
        let reference = get_vector_ref!(args);
        match reference.read_native_struct(|struct_ref| {
            get_vector(struct_ref).and_then(|native_vec| Ok(native_vec.0.len()))
        }) {
            Ok(len) => NativeReturnStatus::Success {
                cost: LENGTH_COST,
                return_values: vec![Value::u64(len as u64)],
            },
            Err(err) => NativeReturnStatus::InvariantError(err),
        }
    }

    pub fn native_push_back(mut args: VecDeque<Value>) -> NativeReturnStatus {
        if args.len() != 2 {
            let msg = format!(
                "wrong number of arguments for push_back expected 2 found {}",
                args.len()
            );
            return NativeReturnStatus::InvariantError(
                VMStatus::new(StatusCode::UNREACHABLE).with_message(msg),
            );
        }
        let reference = get_vector_ref!(args);
        let elem = match args.pop_front() {
            Some(v) => MutVal::new(v),
            None => {
                return NativeReturnStatus::InvariantError(
                    VMStatus::new(StatusCode::UNREACHABLE).with_message(
                        "wrong number of arguments for push_back, expected value".to_string(),
                    ),
                );
            }
        };
        match reference.mutate_native_struct(|struct_ref| {
            get_mut_vector(struct_ref).and_then(|native_vec| Ok(native_vec.0.push(elem)))
        }) {
            Ok(_) => NativeReturnStatus::Success {
                cost: PUSH_BACK_COST,
                return_values: vec![],
            },
            Err(err) => NativeReturnStatus::InvariantError(err),
        }
    }

    pub fn native_borrow(mut args: VecDeque<Value>) -> NativeReturnStatus {
        if args.len() != 2 {
            let msg = format!(
                "wrong number of arguments for borrow expected 2 found {}",
                args.len()
            );
            return NativeReturnStatus::InvariantError(
                VMStatus::new(StatusCode::UNREACHABLE).with_message(msg),
            );
        }
        let reference = get_vector_ref!(args);
        let idx = pop_arg!(args, u64);
        match reference.get_native_struct_reference(|struct_ref| {
            get_vector(struct_ref).and_then(|native_vec| match native_vec.0.get(idx as usize) {
                Some(val) => Ok(val.clone()),
                None => Err(VMStatus::new(StatusCode::NATIVE_FUNCTION_ERROR)
                    .with_sub_status(INDEX_OUT_OF_BOUNDS)),
            })
        }) {
            Ok(v) => NativeReturnStatus::Success {
                cost: BORROW_COST,
                return_values: vec![v],
            },
            Err(err) => match err.major_status {
                StatusCode::NATIVE_FUNCTION_ERROR => NativeReturnStatus::Aborted {
                    cost: BORROW_COST,
                    error_code: err,
                },
                _ => NativeReturnStatus::InvariantError(err),
            },
        }
    }

    pub fn native_pop(mut args: VecDeque<Value>) -> NativeReturnStatus {
        if args.len() != 1 {
            let msg = format!(
                "wrong number of arguments for pop expected 1 found {}",
                args.len()
            );
            return NativeReturnStatus::InvariantError(
                VMStatus::new(StatusCode::UNREACHABLE).with_message(msg),
            );
        }

        let reference = get_vector_ref!(args);
        match reference.mutate_native_struct(|struct_ref| {
            get_mut_vector(struct_ref).and_then(|native_vec| match native_vec.0.pop() {
                Some(val) => val.into_value(),
                None => {
                    Err(VMStatus::new(StatusCode::NATIVE_FUNCTION_ERROR)
                        .with_sub_status(POP_EMPTY_VEC))
                }
            })
        }) {
            Ok(v) => NativeReturnStatus::Success {
                cost: POP_COST,
                return_values: vec![v],
            },
            Err(err) => match err.major_status {
                StatusCode::NATIVE_FUNCTION_ERROR => NativeReturnStatus::Aborted {
                    cost: POP_COST,
                    error_code: err,
                },
                _ => NativeReturnStatus::InvariantError(err),
            },
        }
    }

    pub fn native_destroy_empty(mut args: VecDeque<Value>) -> NativeReturnStatus {
        if let Some(v) = args.pop_front() {
            if let Ok(NativeStructValue::Vector(NativeVector(v))) =
                v.value_as::<NativeStructValue>()
            {
                return if v.is_empty() {
                    NativeReturnStatus::Success {
                        cost: DESTROY_EMPTY_VEC_COST,
                        return_values: vec![],
                    }
                } else {
                    NativeReturnStatus::Aborted {
                        cost: DESTROY_EMPTY_VEC_COST,
                        error_code: VMStatus::new(StatusCode::NATIVE_FUNCTION_ERROR)
                            .with_sub_status(DESTROY_NON_EMPTY_VEC),
                    }
                };
            }
        }
        NativeReturnStatus::InvariantError(
            VMStatus::new(StatusCode::UNREACHABLE)
                .with_message("wrong number of arguments for empty".to_string()),
        )
    }

    pub fn native_swap(mut args: VecDeque<Value>) -> NativeReturnStatus {
        if args.len() != 3 {
            let msg = format!(
                "wrong number of arguments for swap expected 3 found {}",
                args.len()
            );
            return NativeReturnStatus::InvariantError(
                VMStatus::new(StatusCode::UNREACHABLE).with_message(msg),
            );
        }

        let reference = get_vector_ref!(args);
        let index1 = pop_arg!(args, u64);
        let index2 = pop_arg!(args, u64);

        // We need to check the indices before performing the swap in order to make sure the
        // indices are within bounds.
        match reference.read_native_struct(|struct_ref| {
            get_vector(struct_ref).and_then(|native_vec| Ok(native_vec.0.len() as u64))
        }) {
            Ok(len) => {
                if index1 >= len || index2 >= len {
                    return NativeReturnStatus::Aborted {
                        cost: SWAP_COST,
                        error_code: VMStatus::new(StatusCode::NATIVE_FUNCTION_ERROR)
                            .with_sub_status(INDEX_OUT_OF_BOUNDS),
                    };
                }
            }
            Err(err) => return NativeReturnStatus::InvariantError(err),
        }

        match reference.mutate_native_struct(|struct_ref| {
            get_mut_vector(struct_ref)
                .and_then(|native_vec| Ok(native_vec.0.swap(index1 as usize, index2 as usize)))
        }) {
            Ok(_) => NativeReturnStatus::Success {
                cost: SWAP_COST,
                return_values: vec![],
            },
            Err(err) => NativeReturnStatus::InvariantError(err),
        }
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

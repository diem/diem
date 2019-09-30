use crate::{
    native_functions::dispatch::NativeReturnStatus,
    native_structs::NativeStructValue,
    pop_arg,
    value::{MutVal, ReferenceValue, Value},
};
use std::{collections::VecDeque, ops::Add};
use types::vm_error::sub_status::NFE_VECTOR_ERROR_BASE;
use vm::gas_schedule::{AbstractMemorySize, GasAlgebra, GasCarrier, STRUCT_SIZE};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NativeVector(pub(crate) Vec<MutVal>);

const BORROW_COST: u64 = 30; // TODO: determine experimentally
const EMPTY_COST: u64 = 30; // TODO: determine experimentally
const LENGTH_COST: u64 = 30; // TODO: determine experimentally
const PUSH_BACK_COST: u64 = 30; // TODO: determine experimentally
const POP_COST: u64 = 30; // TODO: determine experimentally
const DESTROY_EMPTY_VEC_COST: u64 = 30; // TODO: determine experimentally

pub const INDEX_OUT_OF_BOUND: u64 = NFE_VECTOR_ERROR_BASE + 1;
pub const POP_EMPTY_VEC: u64 = NFE_VECTOR_ERROR_BASE + 2;
pub const DESTROY_NON_EMPTY_VEC: u64 = NFE_VECTOR_ERROR_BASE + 3;

#[allow(dead_code)]
fn get_mut_vector(v: &mut NativeStructValue) -> Option<&mut NativeVector> {
    match v {
        NativeStructValue::Vector(v) => Some(v),
    }
}

fn get_vector(v: &NativeStructValue) -> Option<&NativeVector> {
    match v {
        NativeStructValue::Vector(v) => Some(v),
    }
}

macro_rules! get_vector_ref {
    ($args: expr) => {
        match $args
            .pop_front()
            .and_then(|v| v.value_as::<ReferenceValue>())
        {
            Some(v) => v,
            None => return NativeReturnStatus::InvalidArguments,
        }
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
        reference
            .read_native_struct(|native_val| Some(get_vector(native_val)?.0.len()))
            .map(|len| NativeReturnStatus::Success {
                cost: LENGTH_COST,
                return_values: vec![Value::u64(len as u64)],
            })
            .unwrap_or(NativeReturnStatus::InvalidArguments)
    }

    pub fn native_push_back(mut args: VecDeque<Value>) -> NativeReturnStatus {
        if args.len() != 2 {
            return NativeReturnStatus::InvalidArguments;
        }
        let reference = get_vector_ref!(args);
        let elem = match args.pop_front() {
            Some(v) => MutVal::new(v),
            None => return NativeReturnStatus::InvalidArguments,
        };
        reference
            .mutate_native_struct(|native_val| Some(get_mut_vector(native_val)?.0.push(elem)))
            .map(|_| NativeReturnStatus::Success {
                cost: PUSH_BACK_COST,
                return_values: vec![],
            })
            .unwrap_or(NativeReturnStatus::InvalidArguments)
    }

    pub fn native_borrow(mut args: VecDeque<Value>) -> NativeReturnStatus {
        if args.len() != 2 {
            return NativeReturnStatus::InvalidArguments;
        }
        let reference = get_vector_ref!(args);
        let idx = pop_arg!(args, u64);
        match reference.get_native_struct_reference(|native_val| {
            get_vector(native_val)?
                .0
                .get(idx as usize)
                .map(MutVal::clone)
        }) {
            Some(v) => NativeReturnStatus::Success {
                cost: BORROW_COST,
                return_values: vec![v],
            },
            None => NativeReturnStatus::Aborted {
                cost: BORROW_COST,
                error_code: INDEX_OUT_OF_BOUND,
            },
        }
    }

    pub fn native_pop(mut args: VecDeque<Value>) -> NativeReturnStatus {
        if args.len() != 1 {
            return NativeReturnStatus::InvalidArguments;
        }

        let reference = get_vector_ref!(args);
        match reference.mutate_native_struct(|native_val| {
            get_mut_vector(native_val)?.0.pop().map(MutVal::into_value)
        }) {
            // Vector is already empty.
            None => NativeReturnStatus::Aborted {
                cost: POP_COST,
                error_code: POP_EMPTY_VEC,
            },
            Some(Ok(v)) => NativeReturnStatus::Success {
                cost: POP_COST,
                return_values: vec![v],
            },
            // The popped element has dangling references.
            Some(_) => NativeReturnStatus::InvalidArguments,
        }
    }

    pub fn native_destroy_empty(mut args: VecDeque<Value>) -> NativeReturnStatus {
        if let Some(v) = args.pop_front() {
            if let Some(NativeStructValue::Vector(NativeVector(v))) =
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
                        error_code: DESTROY_NON_EMPTY_VEC,
                    }
                };
            }
        }
        NativeReturnStatus::InvalidArguments
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

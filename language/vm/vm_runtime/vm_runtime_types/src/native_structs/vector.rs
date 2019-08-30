use crate::{
    native_functions::dispatch::NativeReturnStatus,
    native_structs::NativeStructValue,
    value::{MutVal, ReferenceValue, Struct, Value},
};
use std::{collections::VecDeque, ops::Add};
use vm::gas_schedule::{AbstractMemorySize, GasAlgebra, GasCarrier, STRUCT_SIZE};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NativeVector(pub(crate) Vec<MutVal>);

const BORROW_COST: u64 = 30; // TODO: determine experimentally
const EMPTY_COST: u64 = 30; // TODO: determine experimentally
const LENGTH_COST: u64 = 30; // TODO: determine experimentally
const PUSH_BACK_COST: u64 = 30; // TODO: determine experimentally

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

fn native_vector_dispatcher(
    mut args: VecDeque<Value>,
    op: fn(ReferenceValue) -> NativeReturnStatus,
) -> NativeReturnStatus {
    args.pop_front()
        .and_then(|v| v.value_as::<ReferenceValue>())
        .map(op)
        .unwrap_or(NativeReturnStatus::InvalidArguments)
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
    pub fn native_length(args: VecDeque<Value>) -> NativeReturnStatus {
        native_vector_dispatcher(args, |reference| {
            reference
                .read_native_struct(|native_val| Some(get_vector(native_val)?.0.len()))
                .map(|len| NativeReturnStatus::Success {
                    cost: LENGTH_COST,
                    return_values: vec![Value::u64(len as u64)],
                })
                .unwrap_or(NativeReturnStatus::InvalidArguments)
        })
    }

    #[allow(unreachable_code)]
    pub fn native_borrow(_arguments: VecDeque<Value>) -> NativeReturnStatus {
        unimplemented!("borrowing an element from a vector");
        let cost = BORROW_COST;
        // TODO: bounds check + implement retrieving reference to element of vector here
        let vector_element = Value::struct_(Struct::new(vec![]));
        let return_values = vec![vector_element];
        NativeReturnStatus::Success {
            cost,
            return_values,
        }
    }

    #[allow(unreachable_code)]
    pub fn native_push_back(_arguments: VecDeque<Value>) -> NativeReturnStatus {
        unimplemented!("Adding an element to a vector");
        let cost = PUSH_BACK_COST;
        let return_values = vec![];
        NativeReturnStatus::Success {
            cost,
            return_values,
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

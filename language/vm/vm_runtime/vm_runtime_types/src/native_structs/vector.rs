use crate::{
    native_functions::dispatch::NativeReturnStatus,
    native_structs::NativeStructValue,
    pop_arg,
    value::{Local, MutVal, Value, GlobalRef},
};
use std::{collections::VecDeque, ops::Add, rc::Rc, cell::RefCell};
use vm::gas_schedule::{AbstractMemorySize, GasAlgebra, GasCarrier, STRUCT_SIZE};
use crate::value::RootAccessPath;

#[derive(Debug, Clone)]
pub struct NativeVector(pub Vec<MutVal>);

const LENGTH_COST: u64 = 30; // TODO: determine experimentally

fn native_vector_dispatcher(
    mut args: VecDeque<Local>,
    body: fn(&mut NativeVector, VecDeque<Local>, Option<Rc<RefCell<RootAccessPath>>>) -> NativeReturnStatus,
) -> NativeReturnStatus {
    let (vec, root_opt) = match args.pop_front() {
        Some(v) => match v {
            Local::Ref(v) => (v, None),
            Local::GlobalRef(v) => (v.get_reference(), Some(v.get_root())),
            _ => return NativeReturnStatus::InvalidArguments,
        },
        None => return NativeReturnStatus::InvalidArguments,
    };
    let ret = match &mut *vec.get_mut() {
        Value::Native(NativeStructValue::Vector(v)) => body(v, args, root_opt),
        _ => NativeReturnStatus::InvalidArguments,
    };
    ret
}

fn mark_dirty(root_opt: Option<Rc<RefCell<RootAccessPath>>>) {
    match root_opt {
        Some(root) => root.borrow_mut().mark_dirty(),
        None => (),
    }
}

impl NativeVector {
    pub fn native_new(_args: VecDeque<Local>) -> NativeReturnStatus {
        NativeReturnStatus::Success {
            return_values: vec![Local::Value(MutVal::new(Value::Native(
                NativeStructValue::Vector(NativeVector(vec![])),
            )))],
            cost: LENGTH_COST,
        }
    }
    pub fn native_length(args: VecDeque<Local>) -> NativeReturnStatus {
        native_vector_dispatcher(args, |v, args, _| {
            if args.len() != 0 {
                return NativeReturnStatus::InvalidArguments;
            }
            let cost = LENGTH_COST;
            let return_values = vec![Local::u64(v.0.len() as u64)];
            NativeReturnStatus::Success {
                cost,
                return_values,
            }
        })
    }

    pub fn native_push(args: VecDeque<Local>) -> NativeReturnStatus {
        native_vector_dispatcher(args, |v, mut args, root_opt| {
            if args.len() != 1 {
                return NativeReturnStatus::InvalidArguments;
            }
            mark_dirty(root_opt);
            let elem = match args.pop_front().and_then(|v| v.value()) {
                Some(v) => v,
                None => return NativeReturnStatus::InvalidArguments,
            };
            v.0.push(elem);
            let cost = LENGTH_COST;
            let return_values = vec![];
            NativeReturnStatus::Success {
                cost,
                return_values,
            }
        })
    }

    pub fn native_get(args: VecDeque<Local>) -> NativeReturnStatus {
        native_vector_dispatcher(args, |v, mut args, root_opt| {
            if args.len() != 1 {
                return NativeReturnStatus::InvalidArguments;
            }
            let idx = pop_arg!(args, u64);
            match v.0.get(idx as usize) {
                Some(v) => NativeReturnStatus::Success {
                    cost: LENGTH_COST,
                    return_values: vec![match root_opt {
                        Some(root) => Local::GlobalRef(GlobalRef::new(root, v.shallow_clone())),
                        None =>Local::Ref(v.shallow_clone())
                    }],
                },
                None => NativeReturnStatus::Aborted {
                    cost: LENGTH_COST,
                    error_code: 0,
                },
            }
        })
    }

    pub fn native_pop(args: VecDeque<Local>) -> NativeReturnStatus {
        native_vector_dispatcher(args, |v, args, root_opt| {
            if args.len() != 0 {
                return NativeReturnStatus::InvalidArguments;
            }
            mark_dirty(root_opt);
            match v.0.pop() {
                Some(v) => NativeReturnStatus::Success {
                    cost: LENGTH_COST,
                    return_values: vec![Local::Value(v)],
                },
                None => NativeReturnStatus::Aborted {
                    cost: LENGTH_COST,
                    error_code: 0,
                },
            }
        })
    }

    pub fn get(&self, idx: u64) -> Option<MutVal> {
        self.0.get(idx as usize).map(MutVal::shallow_clone)
    }
    pub fn size(&self) -> AbstractMemorySize<GasCarrier> {
        self.0
            .iter()
            .fold(*STRUCT_SIZE, |acc, vl| acc.map2(vl.size(), Add::add))
    }
}

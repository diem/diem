use move_binary_format::errors::PartialVMResult;
use move_core_types::gas_schedule::GasAlgebra;
use move_vm_types::{
    gas_schedule::NativeCostIndex,
    loaded_data::runtime_types::Type,
    natives::function::{native_gas, NativeContext, NativeFunction, NativeResult},
    pop_arg,
    values::{Value, Vector, VectorRef},
};
use smallvec::smallvec;
use std::collections::VecDeque;

#[derive(Copy, Clone)]
pub struct NativeVectorEmpty;
impl NativeFunction for NativeVectorEmpty {
    fn run<C>(
        &self,
        context: &mut C,
        ty_args: Vec<Type>,
        args: VecDeque<Value>,
    ) -> PartialVMResult<NativeResult>
    where
        C: NativeContext,
    {
        debug_assert!(ty_args.len() == 1);
        debug_assert!(args.is_empty());

        let cost = native_gas(context.cost_table(), NativeCostIndex::EMPTY, 1);
        Vector::empty(cost, &ty_args[0], context)
    }
}

#[derive(Copy, Clone)]
pub struct NativeVectorLength;
impl NativeFunction for NativeVectorLength {
    fn run<C>(
        &self,
        context: &mut C,
        ty_args: Vec<Type>,
        mut args: VecDeque<Value>,
    ) -> PartialVMResult<NativeResult>
    where
        C: NativeContext,
    {
        debug_assert!(ty_args.len() == 1);
        debug_assert!(args.len() == 1);

        let r = pop_arg!(args, VectorRef);

        let cost = native_gas(context.cost_table(), NativeCostIndex::LENGTH, 1);

        let len = r.len(&ty_args[0], context)?;
        Ok(NativeResult::ok(cost, smallvec![len]))
    }
}

#[derive(Copy, Clone)]
pub struct NativeVectorPushBack;
impl NativeFunction for NativeVectorPushBack {
    fn run<C>(
        &self,
        context: &mut C,
        ty_args: Vec<Type>,
        mut args: VecDeque<Value>,
    ) -> PartialVMResult<NativeResult>
    where
        C: NativeContext,
    {
        debug_assert!(ty_args.len() == 1);
        debug_assert!(args.len() == 2);

        let e = args.pop_back().unwrap();
        let r = pop_arg!(args, VectorRef);

        let cost = native_gas(
            context.cost_table(),
            NativeCostIndex::PUSH_BACK,
            e.size().get() as usize,
        );

        r.push_back(e, &ty_args[0], context)?;
        Ok(NativeResult::ok(cost, smallvec![]))
    }
}

#[derive(Copy, Clone)]
pub struct NativeVectorBorrow;
impl NativeFunction for NativeVectorBorrow {
    fn run<C>(
        &self,
        context: &mut C,
        ty_args: Vec<Type>,
        mut args: VecDeque<Value>,
    ) -> PartialVMResult<NativeResult>
    where
        C: NativeContext,
    {
        debug_assert!(ty_args.len() == 1);
        debug_assert!(args.len() == 2);

        let idx = pop_arg!(args, u64) as usize;
        let r = pop_arg!(args, VectorRef);

        let cost = native_gas(context.cost_table(), NativeCostIndex::BORROW, 1);

        r.borrow_elem(idx, cost, &ty_args[0], context)
    }
}

#[derive(Copy, Clone)]
pub struct NativeVectorPopBack;
impl NativeFunction for NativeVectorPopBack {
    fn run<C>(
        &self,
        context: &mut C,
        ty_args: Vec<Type>,
        mut args: VecDeque<Value>,
    ) -> PartialVMResult<NativeResult>
    where
        C: NativeContext,
    {
        debug_assert!(ty_args.len() == 1);
        debug_assert!(args.len() == 1);

        let r = pop_arg!(args, VectorRef);

        let cost = native_gas(context.cost_table(), NativeCostIndex::POP_BACK, 1);

        r.pop(cost, &ty_args[0], context)
    }
}

#[derive(Copy, Clone)]
pub struct NativeVectorDestroyEmpty;
impl NativeFunction for NativeVectorDestroyEmpty {
    fn run<C>(
        &self,
        context: &mut C,
        ty_args: Vec<Type>,
        mut args: VecDeque<Value>,
    ) -> PartialVMResult<NativeResult>
    where
        C: NativeContext,
    {
        debug_assert!(ty_args.len() == 1);
        debug_assert!(args.len() == 1);

        let v = pop_arg!(args, Vector);

        let cost = native_gas(context.cost_table(), NativeCostIndex::DESTROY_EMPTY, 1);

        v.destroy_empty(cost, &ty_args[0], context)
    }
}

#[derive(Copy, Clone)]
pub struct NativeVectorSwap;
impl NativeFunction for NativeVectorSwap {
    fn run<C>(
        &self,
        context: &mut C,
        ty_args: Vec<Type>,
        mut args: VecDeque<Value>,
    ) -> PartialVMResult<NativeResult>
    where
        C: NativeContext,
    {
        debug_assert!(ty_args.len() == 1);
        debug_assert!(args.len() == 3);

        let idx2 = pop_arg!(args, u64) as usize;
        let idx1 = pop_arg!(args, u64) as usize;
        let r = pop_arg!(args, VectorRef);

        let cost = native_gas(context.cost_table(), NativeCostIndex::SWAP, 1);

        r.swap(idx1, idx2, cost, &ty_args[0], context)
    }
}

use move_binary_format::errors::PartialVMResult;
use move_vm_types::{
    gas_schedule::NativeCostIndex,
    loaded_data::runtime_types::Type,
    natives::function::{native_gas, NativeContext, NativeFunction, NativeResult},
    pop_arg,
    values::{SignerRef, Value},
};
use smallvec::smallvec;
use std::collections::VecDeque;

#[derive(Copy, Clone)]
pub struct NativeSignerBorrowAddress;
impl NativeFunction for NativeSignerBorrowAddress {
    fn run<C>(
        &self,
        context: &mut C,
        ty_args: Vec<Type>,
        mut args: VecDeque<Value>,
    ) -> PartialVMResult<NativeResult>
    where
        C: NativeContext,
    {
        debug_assert!(ty_args.is_empty());
        debug_assert!(args.len() == 1);

        let signer_reference = pop_arg!(args, SignerRef);
        let cost = native_gas(context.cost_table(), NativeCostIndex::SIGNER_BORROW, 1);

        Ok(NativeResult::ok(
            cost,
            smallvec![signer_reference.borrow_signer()?],
        ))
    }
}

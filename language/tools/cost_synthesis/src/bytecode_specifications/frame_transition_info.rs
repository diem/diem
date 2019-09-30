// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Frame transition rules for the execution stack.
use libra_vm::file_format::{Bytecode, FunctionDefinitionIndex};
use libra_vm_runtime::{
    code_cache::module_cache::ModuleCache,
    execution_stack::ExecutionStack,
    loaded_data::{
        function::{FunctionRef, FunctionReference},
        loaded_module::LoadedModule,
    },
};

fn should_push_frame(instr: &Bytecode) -> bool {
    *instr == Bytecode::Ret
}

/// Certain instructions require specific frame configurations. In particular, Ret requires that
/// there be at least one frame on the stack, and for others (e.g. Call) we expect a specific frame
/// (that we have chosen previously) to be at the top of the frame stack. This function makes sure
/// that the execution stack has the number and/or requested frames at the top.
pub(crate) fn frame_transitions<'alloc, 'txn, P>(
    stk: &mut ExecutionStack<'alloc, 'txn, P>,
    instr: &Bytecode,
    module_info: (&'txn LoadedModule, Option<FunctionDefinitionIndex>),
) where
    'alloc: 'txn,
    P: ModuleCache<'alloc>,
{
    while stk.call_stack_height() > 1 {
        stk.pop_call().unwrap();
    }

    let module = module_info.0;
    if should_push_frame(instr) {
        let empty_frame = FunctionRef::new(module, FunctionDefinitionIndex::new(0));
        // We push a frame here since it won't pop anything off of the value stack.
        stk.push_frame(empty_frame).unwrap();
    }

    if let Some(function_idx) = module_info.1 {
        let frame = FunctionRef::new(module, function_idx);
        // NB: push_call will pop |function_args| number of values off of the value stack.
        stk.push_call(frame).unwrap();
    }
}

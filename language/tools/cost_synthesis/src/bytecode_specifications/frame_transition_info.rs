// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Frame transition rules for the execution stack.
use vm::file_format::{Bytecode, FunctionDefinitionIndex};
use vm_runtime::{
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
    let module = module_info.0;
    if should_push_frame(instr) {
        let empty_frame = FunctionRef::new(module, FunctionDefinitionIndex::new(0));
        stk.push_frame(empty_frame)
    }

    if let Some(function_idx) = module_info.1 {
        let frame = FunctionRef::new(module, function_idx);
        stk.push_frame(frame);
    }
}

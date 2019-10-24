// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Frame transition rules for the execution stack.
use vm::file_format::{Bytecode, FunctionDefinitionIndex};
use vm_runtime::{
    code_cache::module_cache::ModuleCache,
    interpreter::InterpreterForCostSynthesis,
    loaded_data::{
        function::{FunctionRef, FunctionReference},
        loaded_module::LoadedModule,
    },
};

fn should_push_frame(instr: &Bytecode) -> bool {
    *instr == Bytecode::Ret
}

/// Certain instructions require specific frame configurations. In particular, Ret requires that
/// there be at least one frame on the stack. This function makes sure
/// that the execution stack has the number and/or requested frames at the top.
pub(crate) fn frame_transitions<'alloc, 'txn, P>(
    interpreter: &mut InterpreterForCostSynthesis<'alloc, 'txn, P>,
    instr: &Bytecode,
    module_info: (&'txn LoadedModule, Option<FunctionDefinitionIndex>),
) where
    'alloc: 'txn,
    P: ModuleCache<'alloc>,
{
    while interpreter.call_stack_height() > 1 {
        interpreter.pop_call();
    }

    let module = module_info.0;
    if should_push_frame(instr) {
        let empty_frame = FunctionRef::new(module, FunctionDefinitionIndex::new(0));
        // We push a frame here since it won't pop anything off of the value stack.
        interpreter.push_frame(empty_frame, vec![]);
    }

    if let Some(function_idx) = module_info.1 {
        let func = FunctionRef::new(module, function_idx);
        // NB: push_call will pop |function_args| number of values off of the value stack.
        interpreter.push_frame(func, vec![]);
    }
}

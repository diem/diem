// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

extern crate test_generation;
use move_core_types::identifier::Identifier;
use std::collections::HashMap;
use test_generation::abstract_state::{AbstractState, AbstractValue, CallGraph};
use vm::file_format::{
    empty_module, Bytecode, CompiledModuleMut, FunctionHandle, FunctionHandleIndex,
    FunctionSignature, FunctionSignatureIndex, IdentifierIndex, LocalsSignature,
    LocalsSignatureIndex, ModuleHandleIndex, SignatureToken,
};

mod common;

fn generate_module_with_function() -> CompiledModuleMut {
    let mut module: CompiledModuleMut = empty_module();
    let offset = module.identifiers.len();
    let function_sig_offset = module.function_signatures.len();
    module.identifiers.push(Identifier::new("func0").unwrap());

    let sigs = vec![(
        vec![],
        FunctionSignature {
            arg_types: vec![SignatureToken::U64, SignatureToken::Bool],
            return_types: vec![SignatureToken::Address],
            type_formals: vec![],
        },
    )];

    module.function_handles = sigs
        .iter()
        .enumerate()
        .map(|(i, _)| FunctionHandle {
            name: IdentifierIndex::new((i + offset) as u16),
            signature: FunctionSignatureIndex::new((i + function_sig_offset) as u16),
            module: ModuleHandleIndex::new(0),
        })
        .collect();
    let (local_sigs, mut function_sigs): (Vec<_>, Vec<_>) = sigs.into_iter().unzip();
    module.function_signatures.append(&mut function_sigs);
    module
        .locals_signatures
        .append(&mut local_sigs.into_iter().map(LocalsSignature).collect());
    module
}

#[test]
fn bytecode_call() {
    let module = generate_module_with_function();
    let mut state1 =
        AbstractState::from_locals(module, HashMap::new(), vec![], vec![], CallGraph::new(0));
    state1.stack_push(AbstractValue::new_primitive(SignatureToken::U64));
    state1.stack_push(AbstractValue::new_primitive(SignatureToken::Bool));
    let (state2, _) = common::run_instruction(
        Bytecode::Call(FunctionHandleIndex::new(0), LocalsSignatureIndex::new(0)),
        state1,
    );
    assert_eq!(
        state2.stack_peek(0),
        Some(AbstractValue::new_primitive(SignatureToken::Address)),
        "stack type postcondition not satisfied",
    );
}

#[test]
#[should_panic]
fn bytecode_call_function_signature_not_satisfied() {
    let module = generate_module_with_function();
    let state1 =
        AbstractState::from_locals(module, HashMap::new(), vec![], vec![], CallGraph::new(0));
    common::run_instruction(
        Bytecode::Call(FunctionHandleIndex::new(0), LocalsSignatureIndex::new(0)),
        state1,
    );
}

#[test]
#[should_panic]
fn bytecode_call_return_not_pushed() {
    let module = generate_module_with_function();
    let mut state1 =
        AbstractState::from_locals(module, HashMap::new(), vec![], vec![], CallGraph::new(0));
    state1.stack_push(AbstractValue::new_primitive(SignatureToken::U64));
    state1.stack_push(AbstractValue::new_primitive(SignatureToken::Bool));
    let (state2, _) = common::run_instruction(
        Bytecode::Call(FunctionHandleIndex::new(0), LocalsSignatureIndex::new(0)),
        state1,
    );
    assert_eq!(state2.stack_len(), 0,);
}

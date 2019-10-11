// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::{
    code_cache::module_cache::{ModuleCache, VMModuleCache},
    data_cache::RemoteCache,
    loaded_data::{
        function::{FunctionRef, FunctionReference},
        loaded_module::LoadedModule,
    },
    txn_executor::TransactionExecutor,
};
use bytecode_verifier::{VerifiedModule, VerifiedScript};
use crypto::ed25519::compat;
use libra_types::{
    access_path::AccessPath, account_address::AccountAddress, byte_array::ByteArray,
    vm_error::StatusCode,
};
use std::collections::HashMap;
use vm::{
    access::ModuleAccess,
    errors::VMResult,
    file_format::{
        AddressPoolIndex, Bytecode, CodeUnit, CompiledModuleMut, CompiledScript, CompiledScriptMut,
        FunctionDefinition, FunctionHandle, FunctionHandleIndex, FunctionSignature,
        FunctionSignatureIndex, IdentifierIndex, LocalsSignature, LocalsSignatureIndex,
        ModuleHandle, ModuleHandleIndex, SignatureToken, UserStringIndex, NO_TYPE_ACTUALS,
    },
    gas_schedule::{AbstractMemorySize, GasAlgebra, GasPrice, GasUnits},
    transaction_metadata::TransactionMetadata,
};
use vm_cache_map::Arena;
use vm_runtime_types::value::{Locals, Value};

// Trait for the data cache to build a TransactionProcessor
struct FakeDataCache {
    #[allow(dead_code)]
    data: HashMap<AccessPath, Vec<u8>>,
}

impl FakeDataCache {
    fn new() -> Self {
        FakeDataCache {
            data: HashMap::new(),
        }
    }
}

impl RemoteCache for FakeDataCache {
    fn get(&self, _access_path: &AccessPath) -> VMResult<Option<Vec<u8>>> {
        Ok(None)
    }
}

fn fake_script() -> VerifiedScript {
    let compiled_script = CompiledScriptMut {
        main: FunctionDefinition {
            function: FunctionHandleIndex::new(0),
            flags: CodeUnit::PUBLIC,
            acquires_global_resources: vec![],
            code: CodeUnit {
                max_stack_size: 10,
                locals: LocalsSignatureIndex(0),
                code: vec![Bytecode::Ret],
            },
        },
        module_handles: vec![ModuleHandle {
            address: AddressPoolIndex::new(0),
            name: IdentifierIndex::new(0),
        }],
        struct_handles: vec![],
        function_handles: vec![FunctionHandle {
            name: IdentifierIndex::new(0),
            signature: FunctionSignatureIndex::new(0),
            module: ModuleHandleIndex::new(0),
        }],
        type_signatures: vec![],
        function_signatures: vec![FunctionSignature {
            arg_types: vec![],
            return_types: vec![],
            type_formals: vec![],
        }],
        locals_signatures: vec![LocalsSignature(vec![])],
        identifiers: idents(vec!["hello"]),
        user_strings: vec!["hello world".into()],
        byte_array_pool: vec![ByteArray::new(vec![0u8; 32])],
        address_pool: vec![AccountAddress::default()],
    }
    .freeze()
    .expect("test script should satisfy bounds checker");
    VerifiedScript::new(compiled_script).expect("test script should satisfy bytecode verifier")
}

fn test_simple_instruction_impl<'alloc, 'txn>(
    vm: &mut TransactionExecutor<'alloc, 'txn, VMModuleCache<'alloc>>,
    instr: Bytecode,
    value_stack_before: Vec<Value>,
    value_stack_after: Vec<Value>,
    local_before: Locals,
    local_after: Locals,
    expected_offset: u16,
) -> VMResult<()> {
    let code = vec![instr];
    vm.execution_stack
        .top_frame_mut()?
        .set_with_states(0, local_before);
    vm.execution_stack.set_stack(value_stack_before);
    let offset = vm.execute_block(code.as_slice(), 0)?;
    let stack_before_and_after = vm
        .execution_stack
        .get_value_stack()
        .iter()
        .zip(value_stack_after);
    for (v_before, v_after) in stack_before_and_after {
        assert!(v_before.clone().equals(&v_after).unwrap())
    }
    let top_frame = vm.execution_stack.top_frame()?;
    let locals = top_frame.get_locals();
    assert!(locals.equals(&local_after));
    assert_eq!(offset, expected_offset);
    Ok(())
}

fn test_simple_instruction<'alloc, 'txn>(
    vm: &mut TransactionExecutor<'alloc, 'txn, VMModuleCache<'alloc>>,
    instr: Bytecode,
    value_stack_before: Vec<Value>,
    value_stack_after: Vec<Value>,
    local_before: Locals,
    local_after: Locals,
    expected_offset: u16,
) {
    test_simple_instruction_impl(
        vm,
        instr,
        value_stack_before,
        value_stack_after,
        local_before,
        local_after,
        expected_offset,
    )
    .unwrap();
}

fn test_binop_instruction_impl<'alloc, 'txn>(
    vm: &mut TransactionExecutor<'alloc, 'txn, VMModuleCache<'alloc>>,
    instr: Bytecode,
    stack: Vec<Value>,
    expected_value: Value,
) -> VMResult<()> {
    test_simple_instruction_impl(
        vm,
        instr,
        stack,
        vec![expected_value],
        Locals::new(0),
        Locals::new(0),
        1,
    )
}

fn test_binop_instruction<'alloc, 'txn>(
    vm: &mut TransactionExecutor<'alloc, 'txn, VMModuleCache<'alloc>>,
    instr: Bytecode,
    stack: Vec<Value>,
    expected_value: Value,
) {
    test_binop_instruction_impl(vm, instr, stack, expected_value).unwrap()
}

fn test_binop_instruction_overflow<'alloc, 'txn>(
    vm: &mut TransactionExecutor<'alloc, 'txn, VMModuleCache<'alloc>>,
    instr: Bytecode,
    stack: Vec<Value>,
) {
    assert_eq!(
        test_binop_instruction_impl(vm, instr, stack, Value::u64(0))
            .unwrap_err()
            .major_status,
        StatusCode::ARITHMETIC_ERROR
    );
}

#[test]
fn test_simple_instruction_transition() {
    let allocator = Arena::new();
    let module_cache = VMModuleCache::new(&allocator);
    let main_module = fake_script().into_module();
    let loaded_main = LoadedModule::new(main_module);
    let entry_func = FunctionRef::new(&loaded_main, CompiledScript::MAIN_INDEX);
    let data_cache = FakeDataCache::new();
    let mut vm =
        TransactionExecutor::new(module_cache, &data_cache, TransactionMetadata::default());
    vm.execution_stack
        .push_frame(entry_func)
        .expect("push to empty execution stack should succeed");

    test_simple_instruction(
        &mut vm,
        Bytecode::Pop,
        vec![Value::u64(0)],
        vec![],
        Locals::new(0),
        Locals::new(0),
        1,
    );

    test_simple_instruction(
        &mut vm,
        Bytecode::BrTrue(100),
        vec![Value::bool(true)],
        vec![],
        Locals::new(0),
        Locals::new(0),
        100,
    );

    test_simple_instruction(
        &mut vm,
        Bytecode::BrTrue(100),
        vec![Value::bool(false)],
        vec![],
        Locals::new(0),
        Locals::new(0),
        1,
    );

    test_simple_instruction(
        &mut vm,
        Bytecode::BrFalse(100),
        vec![Value::bool(true)],
        vec![],
        Locals::new(0),
        Locals::new(0),
        1,
    );

    test_simple_instruction(
        &mut vm,
        Bytecode::BrFalse(100),
        vec![Value::bool(false)],
        vec![],
        Locals::new(0),
        Locals::new(0),
        100,
    );

    test_simple_instruction(
        &mut vm,
        Bytecode::Branch(100),
        vec![],
        vec![],
        Locals::new(0),
        Locals::new(0),
        100,
    );

    test_simple_instruction(
        &mut vm,
        Bytecode::LdConst(100),
        vec![],
        vec![Value::u64(100)],
        Locals::new(0),
        Locals::new(0),
        1,
    );

    let addr = AccountAddress::default();
    test_simple_instruction(
        &mut vm,
        Bytecode::LdAddr(AddressPoolIndex::new(0)),
        vec![],
        vec![Value::address(addr)],
        Locals::new(0),
        Locals::new(0),
        1,
    );

    test_simple_instruction(
        &mut vm,
        Bytecode::LdStr(UserStringIndex::new(0)),
        vec![],
        vec![Value::string("hello world".into())],
        Locals::new(0),
        Locals::new(0),
        1,
    );

    test_simple_instruction(
        &mut vm,
        Bytecode::LdTrue,
        vec![],
        vec![Value::bool(true)],
        Locals::new(0),
        Locals::new(0),
        1,
    );

    test_simple_instruction(
        &mut vm,
        Bytecode::LdFalse,
        vec![],
        vec![Value::bool(false)],
        Locals::new(0),
        Locals::new(0),
        1,
    );

    let mut locals_before = Locals::new(2);
    locals_before
        .store_loc(1, Value::u64(10))
        .expect("local must exist");
    test_simple_instruction(
        &mut vm,
        Bytecode::CopyLoc(1),
        vec![],
        vec![Value::u64(10)],
        locals_before.clone(),
        locals_before,
        1,
    );

    let mut locals_before = Locals::new(2);
    locals_before
        .store_loc(1, Value::u64(10))
        .expect("local must exist");
    let locals_after = Locals::new(2);
    test_simple_instruction(
        &mut vm,
        Bytecode::MoveLoc(1),
        vec![],
        vec![Value::u64(10)],
        locals_before,
        locals_after,
        1,
    );

    let locals_before = Locals::new(1);
    let mut locals_after = Locals::new(1);
    locals_after
        .store_loc(0, Value::bool(true))
        .expect("local must exist");
    test_simple_instruction(
        &mut vm,
        Bytecode::StLoc(0),
        vec![Value::bool(true)],
        vec![],
        locals_before,
        locals_after,
        1,
    );

    let locals_before = Locals::new(2);
    let mut locals_after = Locals::new(2);
    locals_after
        .store_loc(1, Value::u64(10))
        .expect("local must exist");
    test_simple_instruction(
        &mut vm,
        Bytecode::StLoc(1),
        vec![Value::u64(10)],
        vec![],
        locals_before,
        locals_after,
        1,
    );

    let err = test_simple_instruction_impl(
        &mut vm,
        Bytecode::Abort,
        vec![Value::u64(777)],
        vec![],
        Locals::new(0),
        Locals::new(0),
        1,
    )
    .unwrap_err();

    assert_eq!(err.major_status, StatusCode::ABORTED);
    assert_eq!(err.sub_status, Some(777));
}

#[test]
fn test_arith_instructions() {
    let allocator = Arena::new();
    let module_cache = VMModuleCache::new(&allocator);
    let main_module = fake_script().into_module();
    let loaded_main = LoadedModule::new(main_module);
    let entry_func = FunctionRef::new(&loaded_main, CompiledScript::MAIN_INDEX);
    let data_cache = FakeDataCache::new();

    let mut vm =
        TransactionExecutor::new(module_cache, &data_cache, TransactionMetadata::default());

    vm.execution_stack
        .push_frame(entry_func)
        .expect("push to empty execution stack should succeed");

    test_binop_instruction(
        &mut vm,
        Bytecode::Add,
        vec![Value::u64(1), Value::u64(2)],
        Value::u64(3),
    );
    test_binop_instruction_overflow(
        &mut vm,
        Bytecode::Add,
        vec![Value::u64(u64::max_value()), Value::u64(1)],
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::Sub,
        vec![Value::u64(10), Value::u64(2)],
        Value::u64(8),
    );
    test_binop_instruction_overflow(&mut vm, Bytecode::Sub, vec![Value::u64(0), Value::u64(1)]);

    test_binop_instruction(
        &mut vm,
        Bytecode::Mul,
        vec![Value::u64(2), Value::u64(3)],
        Value::u64(6),
    );
    test_binop_instruction_overflow(
        &mut vm,
        Bytecode::Mul,
        vec![Value::u64(u64::max_value() / 2), Value::u64(3)],
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::Mod,
        vec![Value::u64(10), Value::u64(4)],
        Value::u64(2),
    );
    test_binop_instruction_overflow(&mut vm, Bytecode::Mod, vec![Value::u64(1), Value::u64(0)]);

    test_binop_instruction(
        &mut vm,
        Bytecode::Div,
        vec![Value::u64(6), Value::u64(2)],
        Value::u64(3),
    );
    test_binop_instruction_overflow(&mut vm, Bytecode::Div, vec![Value::u64(1), Value::u64(0)]);

    test_binop_instruction(
        &mut vm,
        Bytecode::BitOr,
        vec![Value::u64(5), Value::u64(6)],
        Value::u64(7),
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::BitAnd,
        vec![Value::u64(5), Value::u64(6)],
        Value::u64(4),
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::Xor,
        vec![Value::u64(5), Value::u64(6)],
        Value::u64(3),
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::Or,
        vec![Value::bool(false), Value::bool(true)],
        Value::bool(true),
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::Or,
        vec![Value::bool(false), Value::bool(false)],
        Value::bool(false),
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::And,
        vec![Value::bool(false), Value::bool(true)],
        Value::bool(false),
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::And,
        vec![Value::bool(true), Value::bool(true)],
        Value::bool(true),
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::Eq,
        vec![Value::bool(false), Value::bool(true)],
        Value::bool(false),
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::Eq,
        vec![Value::u64(5), Value::u64(6)],
        Value::bool(false),
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::Neq,
        vec![Value::bool(false), Value::bool(true)],
        Value::bool(true),
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::Neq,
        vec![Value::u64(5), Value::u64(6)],
        Value::bool(true),
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::Lt,
        vec![Value::u64(5), Value::u64(6)],
        Value::bool(true),
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::Lt,
        vec![Value::u64(5), Value::u64(5)],
        Value::bool(false),
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::Gt,
        vec![Value::u64(7), Value::u64(6)],
        Value::bool(true),
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::Gt,
        vec![Value::u64(5), Value::u64(5)],
        Value::bool(false),
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::Le,
        vec![Value::u64(5), Value::u64(6)],
        Value::bool(true),
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::Le,
        vec![Value::u64(5), Value::u64(5)],
        Value::bool(true),
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::Ge,
        vec![Value::u64(7), Value::u64(6)],
        Value::bool(true),
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::Ge,
        vec![Value::u64(5), Value::u64(5)],
        Value::bool(true),
    );
}

fn fake_module_with_calls(sigs: Vec<(Vec<SignatureToken>, FunctionSignature)>) -> VerifiedModule {
    let mut names: Vec<Identifier> = sigs
        .iter()
        .enumerate()
        .map(|(i, _)| ident(format!("func{}", i)))
        .collect();
    names.insert(0, ident("module"));
    let function_defs = sigs
        .iter()
        .enumerate()
        .map(|(i, _)| FunctionDefinition {
            function: FunctionHandleIndex::new(i as u16),
            flags: CodeUnit::PUBLIC,
            acquires_global_resources: vec![],
            code: CodeUnit {
                max_stack_size: 10,
                locals: LocalsSignatureIndex(i as u16),
                code: vec![],
            },
        })
        .collect();
    let function_handles = sigs
        .iter()
        .enumerate()
        .map(|(i, _)| FunctionHandle {
            name: IdentifierIndex::new((i + 1) as u16),
            signature: FunctionSignatureIndex::new(i as u16),
            module: ModuleHandleIndex::new(0),
        })
        .collect();
    let (local_sigs, function_sigs): (Vec<_>, Vec<_>) = sigs.into_iter().unzip();
    let compiled_module = CompiledModuleMut {
        function_defs,
        field_defs: vec![],
        struct_defs: vec![],

        module_handles: vec![ModuleHandle {
            address: AddressPoolIndex::new(0),
            name: IdentifierIndex::new(0),
        }],
        struct_handles: vec![],
        function_handles,
        type_signatures: vec![],
        function_signatures: function_sigs,
        locals_signatures: local_sigs.into_iter().map(LocalsSignature).collect(),
        identifiers: names,
        user_strings: vec![],
        byte_array_pool: vec![],
        address_pool: vec![AccountAddress::default()],
    }
    .freeze()
    .expect("test module should satisfy the bounds checker");

    // XXX The modules generated here don't satisfy the bytecode verifier at the moment. This should
    // probably be addressed, but it doesn't affect the validity of the test for now.
    VerifiedModule::bypass_verifier_DANGEROUS_FOR_TESTING_ONLY(compiled_module)
}

#[test]
fn test_call() {
    // Note that to pass verification, none of the signatures need to have duplicates.
    // XXX fake_module_with_calls should probably be updated to dedup signatures.
    let module = fake_module_with_calls(vec![
        // () -> (), no local
        (
            vec![],
            FunctionSignature {
                arg_types: vec![],
                return_types: vec![],
                type_formals: vec![],
            },
        ),
        // () -> (), two locals
        (
            vec![SignatureToken::U64, SignatureToken::U64],
            FunctionSignature {
                arg_types: vec![],
                return_types: vec![],
                type_formals: vec![],
            },
        ),
        // (Int, Int) -> (), two locals,
        (
            vec![SignatureToken::U64, SignatureToken::U64],
            FunctionSignature {
                arg_types: vec![SignatureToken::U64, SignatureToken::U64],
                return_types: vec![],
                type_formals: vec![],
            },
        ),
        // (Int, Int) -> (), three locals,
        (
            vec![
                SignatureToken::U64,
                SignatureToken::U64,
                SignatureToken::Bool,
            ],
            FunctionSignature {
                arg_types: vec![SignatureToken::U64, SignatureToken::U64],
                return_types: vec![],
                type_formals: vec![],
            },
        ),
    ]);

    let mod_id = module.self_id();
    let allocator = Arena::new();
    let module_cache = VMModuleCache::new_from_module(module, &allocator).unwrap();
    let fake_func = {
        let fake_mod_entry = module_cache.get_loaded_module(&mod_id).unwrap().unwrap();
        module_cache
            .resolve_function_ref(fake_mod_entry, FunctionHandleIndex::new(0))
            .unwrap()
            .unwrap()
    };
    let data_cache = FakeDataCache::new();
    let mut vm =
        TransactionExecutor::new(module_cache, &data_cache, TransactionMetadata::default());
    vm.execution_stack
        .push_frame(fake_func)
        .expect("push to empty execution stack should succeed");

    test_simple_instruction(
        &mut vm,
        Bytecode::Call(FunctionHandleIndex::new(0), NO_TYPE_ACTUALS),
        vec![],
        vec![],
        Locals::new(0),
        Locals::new(0),
        0,
    );
    test_simple_instruction(
        &mut vm,
        Bytecode::Call(FunctionHandleIndex::new(1), NO_TYPE_ACTUALS),
        vec![],
        vec![],
        Locals::new(0),
        Locals::new(2),
        0,
    );
    let mut locals_after = Locals::new(2);
    locals_after
        .store_loc(0, Value::u64(5))
        .expect("local must exist");
    locals_after
        .store_loc(1, Value::u64(4))
        .expect("local must exist");
    test_simple_instruction(
        &mut vm,
        Bytecode::Call(FunctionHandleIndex::new(2), NO_TYPE_ACTUALS),
        vec![Value::u64(5), Value::u64(4)],
        vec![],
        Locals::new(0),
        locals_after,
        0,
    );
    let mut locals_after = Locals::new(3);
    locals_after
        .store_loc(0, Value::u64(5))
        .expect("local must exist");
    locals_after
        .store_loc(1, Value::u64(4))
        .expect("local must exist");
    test_simple_instruction(
        &mut vm,
        Bytecode::Call(FunctionHandleIndex::new(3), NO_TYPE_ACTUALS),
        vec![Value::u64(5), Value::u64(4)],
        vec![],
        Locals::new(0),
        locals_after,
        0,
    );
}

#[test]
fn test_transaction_info() {
    let allocator = Arena::new();
    let module_cache = VMModuleCache::new(&allocator);
    let main_module = fake_script().into_module();
    let loaded_main = LoadedModule::new(main_module);
    let entry_func = FunctionRef::new(&loaded_main, CompiledScript::MAIN_INDEX);

    let txn_info = {
        let (_, public_key) = compat::generate_genesis_keypair();
        TransactionMetadata {
            sender: AccountAddress::default(),
            public_key,
            sequence_number: 10,
            max_gas_amount: GasUnits::new(100_000_009),
            gas_unit_price: GasPrice::new(5),
            transaction_size: AbstractMemorySize::new(100),
            channel_metadata: None,
        }
    };
    let data_cache = FakeDataCache::new();
    let mut vm = TransactionExecutor::new(module_cache, &data_cache, txn_info);

    vm.execution_stack
        .push_frame(entry_func)
        .expect("push to empty execution stack should succeed");

    test_simple_instruction(
        &mut vm,
        Bytecode::GetTxnMaxGasUnits,
        vec![],
        vec![Value::u64(100_000_009)],
        Locals::new(0),
        Locals::new(0),
        1,
    );

    test_simple_instruction(
        &mut vm,
        Bytecode::GetTxnSequenceNumber,
        vec![],
        vec![Value::u64(10)],
        Locals::new(0),
        Locals::new(0),
        1,
    );

    test_simple_instruction(
        &mut vm,
        Bytecode::GetTxnGasUnitPrice,
        vec![],
        vec![Value::u64(5)],
        Locals::new(0),
        Locals::new(0),
        1,
    );
}

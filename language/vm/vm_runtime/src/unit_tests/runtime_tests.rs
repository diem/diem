// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::{
    code_cache::module_cache::VMModuleCache, txn_executor::TransactionExecutor, value::Local,
};
use bytecode_verifier::{VerifiedModule, VerifiedScript};
use std::collections::HashMap;
use types::{access_path::AccessPath, account_address::AccountAddress, byte_array::ByteArray};
use vm::{
    file_format::{
        AddressPoolIndex, Bytecode, CodeUnit, CompiledModuleMut, CompiledScript, CompiledScriptMut,
        FunctionDefinition, FunctionHandle, FunctionHandleIndex, FunctionSignature,
        FunctionSignatureIndex, LocalsSignature, LocalsSignatureIndex, ModuleHandle,
        ModuleHandleIndex, SignatureToken, StringPoolIndex, NO_TYPE_ACTUALS,
    },
    gas_schedule::{AbstractMemorySize, GasAlgebra, GasPrice, GasUnits},
    transaction_metadata::TransactionMetadata,
};
use vm_cache_map::Arena;

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
    fn get(&self, _access_path: &AccessPath) -> Result<Option<Vec<u8>>, VMInvariantViolation> {
        Ok(None)
    }
}

fn fake_script() -> VerifiedScript {
    let compiled_script = CompiledScriptMut {
        main: FunctionDefinition {
            function: FunctionHandleIndex::new(0),
            flags: CodeUnit::PUBLIC,
            code: CodeUnit {
                max_stack_size: 10,
                locals: LocalsSignatureIndex(0),
                code: vec![Bytecode::Ret],
            },
        },
        module_handles: vec![ModuleHandle {
            address: AddressPoolIndex::new(0),
            name: StringPoolIndex::new(0),
        }],
        struct_handles: vec![],
        function_handles: vec![FunctionHandle {
            name: StringPoolIndex::new(0),
            signature: FunctionSignatureIndex::new(0),
            module: ModuleHandleIndex::new(0),
        }],
        type_signatures: vec![],
        function_signatures: vec![FunctionSignature {
            arg_types: vec![],
            return_types: vec![],
            kind_constraints: vec![],
        }],
        locals_signatures: vec![LocalsSignature(vec![])],
        string_pool: vec!["hello".to_string()],
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
    value_stack_before: Vec<Local>,
    value_stack_after: Vec<Local>,
    local_before: Vec<Local>,
    local_after: Vec<Local>,
    expected_offset: u16,
) -> VMResult<()> {
    let code = vec![instr];
    vm.execution_stack
        .top_frame_mut()?
        .set_with_states(0, local_before);
    vm.execution_stack.set_stack(value_stack_before);
    let offset = try_runtime!(vm.execute_block(code.as_slice(), 0));
    let stack_before_and_after = vm
        .execution_stack
        .get_value_stack()
        .iter()
        .zip(value_stack_after);
    for (v_before, v_after) in stack_before_and_after {
        assert!(v_before.clone().equals(v_after).unwrap())
    }
    let top_frame = vm.execution_stack.top_frame()?;
    let locals_before_and_after = top_frame.get_locals().iter().zip(local_after);
    for (l_before, l_after) in locals_before_and_after {
        assert!(l_before.clone().equals(l_after).unwrap())
    }
    assert_eq!(offset, expected_offset);
    Ok(Ok(()))
}

fn test_simple_instruction<'alloc, 'txn>(
    vm: &mut TransactionExecutor<'alloc, 'txn, VMModuleCache<'alloc>>,
    instr: Bytecode,
    value_stack_before: Vec<Local>,
    value_stack_after: Vec<Local>,
    local_before: Vec<Local>,
    local_after: Vec<Local>,
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
    .unwrap()
    .unwrap();
}

fn test_binop_instruction_impl<'alloc, 'txn>(
    vm: &mut TransactionExecutor<'alloc, 'txn, VMModuleCache<'alloc>>,
    instr: Bytecode,
    stack: Vec<Local>,
    expected_value: Local,
) -> VMResult<()> {
    test_simple_instruction_impl(vm, instr, stack, vec![expected_value], vec![], vec![], 1)
}

fn test_binop_instruction<'alloc, 'txn>(
    vm: &mut TransactionExecutor<'alloc, 'txn, VMModuleCache<'alloc>>,
    instr: Bytecode,
    stack: Vec<Local>,
    expected_value: Local,
) {
    test_binop_instruction_impl(vm, instr, stack, expected_value)
        .unwrap()
        .unwrap()
}

fn test_binop_instruction_overflow<'alloc, 'txn>(
    vm: &mut TransactionExecutor<'alloc, 'txn, VMModuleCache<'alloc>>,
    instr: Bytecode,
    stack: Vec<Local>,
) {
    assert_eq!(
        test_binop_instruction_impl(vm, instr, stack, Local::u64(0))
            .unwrap()
            .unwrap_err()
            .err,
        VMErrorKind::ArithmeticError
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
    vm.execution_stack.push_frame(entry_func);

    test_simple_instruction(
        &mut vm,
        Bytecode::Pop,
        vec![Local::u64(0)],
        vec![],
        vec![],
        vec![],
        1,
    );

    test_simple_instruction(
        &mut vm,
        Bytecode::BrTrue(100),
        vec![Local::bool(true)],
        vec![],
        vec![],
        vec![],
        100,
    );

    test_simple_instruction(
        &mut vm,
        Bytecode::BrTrue(100),
        vec![Local::bool(false)],
        vec![],
        vec![],
        vec![],
        1,
    );

    test_simple_instruction(
        &mut vm,
        Bytecode::BrFalse(100),
        vec![Local::bool(true)],
        vec![],
        vec![],
        vec![],
        1,
    );

    test_simple_instruction(
        &mut vm,
        Bytecode::BrFalse(100),
        vec![Local::bool(false)],
        vec![],
        vec![],
        vec![],
        100,
    );

    test_simple_instruction(
        &mut vm,
        Bytecode::Branch(100),
        vec![],
        vec![],
        vec![],
        vec![],
        100,
    );

    test_simple_instruction(
        &mut vm,
        Bytecode::LdConst(100),
        vec![],
        vec![Local::u64(100)],
        vec![],
        vec![],
        1,
    );

    let addr = AccountAddress::default();
    test_simple_instruction(
        &mut vm,
        Bytecode::LdAddr(AddressPoolIndex::new(0)),
        vec![],
        vec![Local::address(addr)],
        vec![],
        vec![],
        1,
    );

    test_simple_instruction(
        &mut vm,
        Bytecode::LdStr(StringPoolIndex::new(0)),
        vec![],
        vec![Local::string("hello".to_string())],
        vec![],
        vec![],
        1,
    );

    test_simple_instruction(
        &mut vm,
        Bytecode::LdTrue,
        vec![],
        vec![Local::bool(true)],
        vec![],
        vec![],
        1,
    );

    test_simple_instruction(
        &mut vm,
        Bytecode::LdFalse,
        vec![],
        vec![Local::bool(false)],
        vec![],
        vec![],
        1,
    );

    test_simple_instruction(
        &mut vm,
        Bytecode::CopyLoc(1),
        vec![],
        vec![Local::u64(10)],
        vec![Local::Invalid, Local::u64(10)],
        vec![Local::Invalid, Local::u64(10)],
        1,
    );

    test_simple_instruction(
        &mut vm,
        Bytecode::MoveLoc(1),
        vec![],
        vec![Local::u64(10)],
        vec![Local::Invalid, Local::u64(10)],
        vec![Local::Invalid, Local::Invalid],
        1,
    );

    test_simple_instruction(
        &mut vm,
        Bytecode::StLoc(0),
        vec![Local::bool(true)],
        vec![],
        vec![Local::Invalid],
        vec![Local::bool(true)],
        1,
    );

    test_simple_instruction(
        &mut vm,
        Bytecode::StLoc(1),
        vec![Local::u64(10)],
        vec![],
        vec![Local::Invalid, Local::Invalid],
        vec![Local::Invalid, Local::u64(10)],
        1,
    );

    assert_eq!(
        test_simple_instruction_impl(
            &mut vm,
            Bytecode::Abort,
            vec![Local::u64(777)],
            vec![],
            vec![],
            vec![],
            1
        )
        .unwrap()
        .unwrap_err()
        .err,
        VMErrorKind::Aborted(777)
    );
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

    vm.execution_stack.push_frame(entry_func);

    test_binop_instruction(
        &mut vm,
        Bytecode::Add,
        vec![Local::u64(1), Local::u64(2)],
        Local::u64(3),
    );
    test_binop_instruction_overflow(
        &mut vm,
        Bytecode::Add,
        vec![Local::u64(u64::max_value()), Local::u64(1)],
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::Sub,
        vec![Local::u64(10), Local::u64(2)],
        Local::u64(8),
    );
    test_binop_instruction_overflow(&mut vm, Bytecode::Sub, vec![Local::u64(0), Local::u64(1)]);

    test_binop_instruction(
        &mut vm,
        Bytecode::Mul,
        vec![Local::u64(2), Local::u64(3)],
        Local::u64(6),
    );
    test_binop_instruction_overflow(
        &mut vm,
        Bytecode::Mul,
        vec![Local::u64(u64::max_value() / 2), Local::u64(3)],
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::Mod,
        vec![Local::u64(10), Local::u64(4)],
        Local::u64(2),
    );
    test_binop_instruction_overflow(&mut vm, Bytecode::Mod, vec![Local::u64(1), Local::u64(0)]);

    test_binop_instruction(
        &mut vm,
        Bytecode::Div,
        vec![Local::u64(6), Local::u64(2)],
        Local::u64(3),
    );
    test_binop_instruction_overflow(&mut vm, Bytecode::Div, vec![Local::u64(1), Local::u64(0)]);

    test_binop_instruction(
        &mut vm,
        Bytecode::BitOr,
        vec![Local::u64(5), Local::u64(6)],
        Local::u64(7),
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::BitAnd,
        vec![Local::u64(5), Local::u64(6)],
        Local::u64(4),
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::Xor,
        vec![Local::u64(5), Local::u64(6)],
        Local::u64(3),
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::Or,
        vec![Local::bool(false), Local::bool(true)],
        Local::bool(true),
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::Or,
        vec![Local::bool(false), Local::bool(false)],
        Local::bool(false),
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::And,
        vec![Local::bool(false), Local::bool(true)],
        Local::bool(false),
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::And,
        vec![Local::bool(true), Local::bool(true)],
        Local::bool(true),
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::Eq,
        vec![Local::bool(false), Local::bool(true)],
        Local::bool(false),
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::Eq,
        vec![Local::u64(5), Local::u64(6)],
        Local::bool(false),
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::Neq,
        vec![Local::bool(false), Local::bool(true)],
        Local::bool(true),
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::Neq,
        vec![Local::u64(5), Local::u64(6)],
        Local::bool(true),
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::Lt,
        vec![Local::u64(5), Local::u64(6)],
        Local::bool(true),
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::Lt,
        vec![Local::u64(5), Local::u64(5)],
        Local::bool(false),
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::Gt,
        vec![Local::u64(7), Local::u64(6)],
        Local::bool(true),
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::Gt,
        vec![Local::u64(5), Local::u64(5)],
        Local::bool(false),
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::Le,
        vec![Local::u64(5), Local::u64(6)],
        Local::bool(true),
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::Le,
        vec![Local::u64(5), Local::u64(5)],
        Local::bool(true),
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::Ge,
        vec![Local::u64(7), Local::u64(6)],
        Local::bool(true),
    );

    test_binop_instruction(
        &mut vm,
        Bytecode::Ge,
        vec![Local::u64(5), Local::u64(5)],
        Local::bool(true),
    );
}

fn fake_module_with_calls(sigs: Vec<(Vec<SignatureToken>, FunctionSignature)>) -> VerifiedModule {
    let mut names: Vec<String> = sigs
        .iter()
        .enumerate()
        .map(|(i, _)| format!("func{}", i))
        .collect();
    names.insert(0, "module".to_string());
    let function_defs = sigs
        .iter()
        .enumerate()
        .map(|(i, _)| FunctionDefinition {
            function: FunctionHandleIndex::new(i as u16),
            flags: CodeUnit::PUBLIC,
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
            name: StringPoolIndex::new((i + 1) as u16),
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
            name: StringPoolIndex::new(0),
        }],
        struct_handles: vec![],
        function_handles,
        type_signatures: vec![],
        function_signatures: function_sigs,
        locals_signatures: local_sigs.into_iter().map(LocalsSignature).collect(),
        string_pool: names,
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
                kind_constraints: vec![],
            },
        ),
        // () -> (), two locals
        (
            vec![SignatureToken::U64, SignatureToken::U64],
            FunctionSignature {
                arg_types: vec![],
                return_types: vec![],
                kind_constraints: vec![],
            },
        ),
        // (Int, Int) -> (), two locals,
        (
            vec![SignatureToken::U64, SignatureToken::U64],
            FunctionSignature {
                arg_types: vec![SignatureToken::U64, SignatureToken::U64],
                return_types: vec![],
                kind_constraints: vec![],
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
                kind_constraints: vec![],
            },
        ),
    ]);

    let mod_id = module.self_id();
    let allocator = Arena::new();
    let module_cache = VMModuleCache::new_from_module(module, &allocator).unwrap();
    let fake_func = {
        let fake_mod_entry = module_cache
            .get_loaded_module(&mod_id)
            .unwrap()
            .unwrap()
            .unwrap();
        module_cache
            .resolve_function_ref(fake_mod_entry, FunctionHandleIndex::new(0))
            .unwrap()
            .unwrap()
            .unwrap()
    };
    let data_cache = FakeDataCache::new();
    let mut vm =
        TransactionExecutor::new(module_cache, &data_cache, TransactionMetadata::default());
    vm.execution_stack.push_frame(fake_func);

    test_simple_instruction(
        &mut vm,
        Bytecode::Call(FunctionHandleIndex::new(0), NO_TYPE_ACTUALS),
        vec![],
        vec![],
        vec![],
        vec![],
        0,
    );
    test_simple_instruction(
        &mut vm,
        Bytecode::Call(FunctionHandleIndex::new(1), NO_TYPE_ACTUALS),
        vec![],
        vec![],
        vec![],
        vec![Local::Invalid, Local::Invalid],
        0,
    );
    test_simple_instruction(
        &mut vm,
        Bytecode::Call(FunctionHandleIndex::new(2), NO_TYPE_ACTUALS),
        vec![Local::u64(5), Local::u64(4)],
        vec![],
        vec![],
        vec![Local::u64(5), Local::u64(4)],
        0,
    );
    test_simple_instruction(
        &mut vm,
        Bytecode::Call(FunctionHandleIndex::new(3), NO_TYPE_ACTUALS),
        vec![Local::u64(5), Local::u64(4)],
        vec![],
        vec![],
        vec![Local::u64(5), Local::u64(4), Local::Invalid],
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
        let (_, public_key) = crypto::signing::generate_genesis_keypair();
        TransactionMetadata {
            sender: AccountAddress::default(),
            public_key,
            sequence_number: 10,
            max_gas_amount: GasUnits::new(100_000_009),
            gas_unit_price: GasPrice::new(5),
            transaction_size: AbstractMemorySize::new(100),
        }
    };
    let data_cache = FakeDataCache::new();
    let mut vm = TransactionExecutor::new(module_cache, &data_cache, txn_info);

    vm.execution_stack.push_frame(entry_func);

    test_simple_instruction(
        &mut vm,
        Bytecode::GetTxnMaxGasUnits,
        vec![],
        vec![Local::u64(100_000_009)],
        vec![],
        vec![],
        1,
    );

    test_simple_instruction(
        &mut vm,
        Bytecode::GetTxnSequenceNumber,
        vec![],
        vec![Local::u64(10)],
        vec![],
        vec![],
        1,
    );

    test_simple_instruction(
        &mut vm,
        Bytecode::GetTxnGasUnitPrice,
        vec![],
        vec![Local::u64(5)],
        vec![],
        vec![],
        1,
    );
}

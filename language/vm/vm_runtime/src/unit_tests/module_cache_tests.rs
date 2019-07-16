// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::{
    code_cache::{module_adapter::FakeFetcher, module_cache::ModuleCache},
    loaded_data::function::{FunctionRef, FunctionReference},
};
use assert_matches::assert_matches;
use bytecode_verifier::VerifiedScript;
use compiler::Compiler;
use hex;
use types::account_address::AccountAddress;
use vm::{
    file_format::*,
    gas_schedule::{GasAlgebra, GasUnits},
};
use vm_cache_map::Arena;

fn test_module(name: String) -> VerifiedModule {
    let compiled_module = CompiledModuleMut {
        module_handles: vec![ModuleHandle {
            name: StringPoolIndex::new(0),
            address: AddressPoolIndex::new(0),
        }],
        struct_handles: vec![],
        function_handles: vec![
            FunctionHandle {
                module: ModuleHandleIndex::new(0),
                name: StringPoolIndex::new(1),
                signature: FunctionSignatureIndex::new(0),
            },
            FunctionHandle {
                module: ModuleHandleIndex::new(0),
                name: StringPoolIndex::new(2),
                signature: FunctionSignatureIndex::new(1),
            },
        ],

        struct_defs: vec![],
        field_defs: vec![],
        function_defs: vec![
            FunctionDefinition {
                function: FunctionHandleIndex::new(0),
                flags: CodeUnit::PUBLIC,
                code: CodeUnit {
                    max_stack_size: 10,
                    locals: LocalsSignatureIndex::new(0),
                    code: vec![Bytecode::LdTrue, Bytecode::Pop, Bytecode::Ret],
                },
            },
            FunctionDefinition {
                function: FunctionHandleIndex::new(1),
                flags: CodeUnit::PUBLIC,
                code: CodeUnit {
                    max_stack_size: 10,
                    locals: LocalsSignatureIndex::new(0),
                    code: vec![Bytecode::Ret],
                },
            },
        ],
        type_signatures: vec![],
        function_signatures: vec![
            FunctionSignature {
                return_types: vec![],
                arg_types: vec![],
                kind_constraints: vec![],
            },
            FunctionSignature {
                return_types: vec![],
                arg_types: vec![SignatureToken::U64],
                kind_constraints: vec![],
            },
        ],
        locals_signatures: vec![LocalsSignature(vec![])],
        string_pool: vec![name, "func1".to_string(), "func2".to_string()],
        byte_array_pool: vec![],
        address_pool: vec![AccountAddress::default()],
    }
    .freeze()
    .expect("test module should satisfy bounds checker");
    VerifiedModule::new(compiled_module).expect("test module should satisfy bytecode verifier")
}

fn test_script() -> VerifiedScript {
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
        module_handles: vec![
            ModuleHandle {
                address: AddressPoolIndex::new(0),
                name: StringPoolIndex::new(0),
            },
            ModuleHandle {
                address: AddressPoolIndex::new(0),
                name: StringPoolIndex::new(1),
            },
        ],
        struct_handles: vec![],
        function_handles: vec![
            FunctionHandle {
                name: StringPoolIndex::new(4),
                signature: FunctionSignatureIndex::new(0),
                module: ModuleHandleIndex::new(0),
            },
            FunctionHandle {
                name: StringPoolIndex::new(2),
                signature: FunctionSignatureIndex::new(0),
                module: ModuleHandleIndex::new(1),
            },
            FunctionHandle {
                name: StringPoolIndex::new(3),
                signature: FunctionSignatureIndex::new(1),
                module: ModuleHandleIndex::new(1),
            },
        ],
        type_signatures: vec![],
        function_signatures: vec![
            FunctionSignature {
                return_types: vec![],
                arg_types: vec![],
                kind_constraints: vec![],
            },
            FunctionSignature {
                return_types: vec![],
                arg_types: vec![SignatureToken::U64],
                kind_constraints: vec![],
            },
        ],
        locals_signatures: vec![LocalsSignature(vec![])],
        string_pool: vec![
            "hello".to_string(),
            "module".to_string(),
            "func1".to_string(),
            "func2".to_string(),
            "main".to_string(),
        ],
        byte_array_pool: vec![],
        address_pool: vec![AccountAddress::default()],
    }
    .freeze()
    .expect("test script should satisfy bounds checker");
    VerifiedScript::new(compiled_script).expect("test script should satisfy bytecode verifier")
}

#[test]
fn test_loader_one_module() {
    // This test tests the linking of function within a single module: We have a module that defines
    // two functions, each with different name and signature. This test will make sure that we
    // link the function handle with the right function definition within the same module.
    let module = test_module("module".to_string());
    let mod_id = module.self_id();

    let allocator = Arena::new();
    let loaded_program = VMModuleCache::new(&allocator);
    loaded_program.cache_module(module);
    let module_ref = loaded_program
        .get_loaded_module(&mod_id)
        .unwrap()
        .unwrap()
        .unwrap();

    // Get the function reference of the first two function handles.
    let func1_ref = loaded_program
        .resolve_function_ref(module_ref, FunctionHandleIndex::new(0))
        .unwrap()
        .unwrap()
        .unwrap();
    let func2_ref = loaded_program
        .resolve_function_ref(module_ref, FunctionHandleIndex::new(1))
        .unwrap()
        .unwrap()
        .unwrap();

    // The two references should refer to the same module
    assert_eq!(
        func2_ref.module() as *const LoadedModule,
        func1_ref.module() as *const LoadedModule
    );

    assert_eq!(func1_ref.arg_count(), 0);
    assert_eq!(func1_ref.return_count(), 0);
    assert_eq!(
        func1_ref.code_definition(),
        vec![Bytecode::LdTrue, Bytecode::Pop, Bytecode::Ret].as_slice()
    );

    assert_eq!(func2_ref.arg_count(), 1);
    assert_eq!(func2_ref.return_count(), 0);
    assert_eq!(func2_ref.code_definition(), vec![Bytecode::Ret].as_slice());
}

#[test]
fn test_loader_cross_modules() {
    let script = test_script();
    let module = test_module("module".to_string());

    let allocator = Arena::new();
    let loaded_program = VMModuleCache::new(&allocator);
    loaded_program.cache_module(module);

    let owned_entry_module = script.into_module();
    let loaded_main = LoadedModule::new(owned_entry_module);
    let entry_func = FunctionRef::new(&loaded_main, CompiledScript::MAIN_INDEX);
    let entry_module = entry_func.module();
    let func1 = loaded_program
        .resolve_function_ref(entry_module, FunctionHandleIndex::new(1))
        .unwrap()
        .unwrap()
        .unwrap();
    let func2 = loaded_program
        .resolve_function_ref(entry_module, FunctionHandleIndex::new(2))
        .unwrap()
        .unwrap()
        .unwrap();

    assert_eq!(
        func2.module() as *const LoadedModule,
        func1.module() as *const LoadedModule
    );

    assert_eq!(func1.arg_count(), 0);
    assert_eq!(func1.return_count(), 0);
    assert_eq!(
        func1.code_definition(),
        vec![Bytecode::LdTrue, Bytecode::Pop, Bytecode::Ret].as_slice()
    );

    assert_eq!(func2.arg_count(), 1);
    assert_eq!(func2.return_count(), 0);
    assert_eq!(func2.code_definition(), vec![Bytecode::Ret].as_slice());
}

#[test]
fn test_cache_with_storage() {
    let allocator = Arena::new();

    let owned_entry_module = test_script().into_module();
    let loaded_main = LoadedModule::new(owned_entry_module);
    let entry_func = FunctionRef::new(&loaded_main, CompiledScript::MAIN_INDEX);
    let entry_module = entry_func.module();
    println!("MODULE: {}", entry_module.as_module());

    let vm_cache = VMModuleCache::new(&allocator);

    // Function is not defined locally.
    assert!(vm_cache
        .resolve_function_ref(entry_module, FunctionHandleIndex::new(1))
        .unwrap()
        .unwrap()
        .is_none());

    {
        let fetcher = FakeFetcher::new(vec![test_module("module".to_string()).into_inner()]);
        let mut block_cache = BlockModuleCache::new(&vm_cache, fetcher);

        // Make sure the block cache fetches the code from the view.
        let func1 = block_cache
            .resolve_function_ref(entry_module, FunctionHandleIndex::new(1))
            .unwrap()
            .unwrap()
            .unwrap();
        let func2 = block_cache
            .resolve_function_ref(entry_module, FunctionHandleIndex::new(2))
            .unwrap()
            .unwrap()
            .unwrap();

        assert_eq!(
            func2.module() as *const LoadedModule,
            func1.module() as *const LoadedModule
        );

        assert_eq!(func1.arg_count(), 0);
        assert_eq!(func1.return_count(), 0);
        assert_eq!(
            func1.code_definition(),
            vec![Bytecode::LdTrue, Bytecode::Pop, Bytecode::Ret].as_slice()
        );

        assert_eq!(func2.arg_count(), 1);
        assert_eq!(func2.return_count(), 0);
        assert_eq!(func2.code_definition(), vec![Bytecode::Ret].as_slice());

        // Clean the fetcher so that there's nothing in the fetcher.
        block_cache.storage.clear();

        let func1 = block_cache
            .resolve_function_ref(entry_module, FunctionHandleIndex::new(1))
            .unwrap()
            .unwrap()
            .unwrap();
        let func2 = block_cache
            .resolve_function_ref(entry_module, FunctionHandleIndex::new(2))
            .unwrap()
            .unwrap()
            .unwrap();

        assert_eq!(
            func2.module() as *const LoadedModule,
            func1.module() as *const LoadedModule
        );

        assert_eq!(func1.arg_count(), 0);
        assert_eq!(func1.return_count(), 0);
        assert_eq!(
            func1.code_definition(),
            vec![Bytecode::LdTrue, Bytecode::Pop, Bytecode::Ret].as_slice()
        );

        assert_eq!(func2.arg_count(), 1);
        assert_eq!(func2.return_count(), 0);
        assert_eq!(func2.code_definition(), vec![Bytecode::Ret].as_slice());
    }

    // Even if the block cache goes out of scope, we should still be able to read the fetched
    // definition
    let func1 = vm_cache
        .resolve_function_ref(entry_module, FunctionHandleIndex::new(1))
        .unwrap()
        .unwrap()
        .unwrap();
    let func2 = vm_cache
        .resolve_function_ref(entry_module, FunctionHandleIndex::new(2))
        .unwrap()
        .unwrap()
        .unwrap();

    assert_eq!(
        func2.module() as *const LoadedModule,
        func1.module() as *const LoadedModule
    );

    assert_eq!(func1.arg_count(), 0);
    assert_eq!(func1.return_count(), 0);
    assert_eq!(
        func1.code_definition(),
        vec![Bytecode::LdTrue, Bytecode::Pop, Bytecode::Ret].as_slice()
    );

    assert_eq!(func2.arg_count(), 1);
    assert_eq!(func2.return_count(), 0);
    assert_eq!(func2.code_definition(), vec![Bytecode::Ret].as_slice());
}

#[test]
fn test_multi_level_cache_write_back() {
    let allocator = Arena::new();
    let vm_cache = VMModuleCache::new(&allocator);

    // Put an existing module in the cache.
    let module = test_module("existing_module".to_string());
    vm_cache.cache_module(module);

    // Create a new script that refers to both published and unpublished modules.
    let script = CompiledScriptMut {
        main: FunctionDefinition {
            function: FunctionHandleIndex::new(0),
            flags: CodeUnit::PUBLIC,
            code: CodeUnit {
                max_stack_size: 10,
                locals: LocalsSignatureIndex(0),
                code: vec![Bytecode::Ret],
            },
        },
        module_handles: vec![
            // Self
            ModuleHandle {
                address: AddressPoolIndex::new(0),
                name: StringPoolIndex::new(0),
            },
            // To-be-published Module
            ModuleHandle {
                address: AddressPoolIndex::new(0),
                name: StringPoolIndex::new(1),
            },
            // Existing module on chain
            ModuleHandle {
                address: AddressPoolIndex::new(0),
                name: StringPoolIndex::new(2),
            },
        ],
        struct_handles: vec![],
        function_handles: vec![
            // main
            FunctionHandle {
                name: StringPoolIndex::new(5),
                signature: FunctionSignatureIndex::new(0),
                module: ModuleHandleIndex::new(0),
            },
            // Func2 defined in the new module
            FunctionHandle {
                name: StringPoolIndex::new(4),
                signature: FunctionSignatureIndex::new(0),
                module: ModuleHandleIndex::new(1),
            },
            // Func1 defined in the old module
            FunctionHandle {
                name: StringPoolIndex::new(3),
                signature: FunctionSignatureIndex::new(1),
                module: ModuleHandleIndex::new(2),
            },
        ],
        type_signatures: vec![],
        function_signatures: vec![
            FunctionSignature {
                return_types: vec![],
                arg_types: vec![],
                kind_constraints: vec![],
            },
            FunctionSignature {
                return_types: vec![],
                arg_types: vec![SignatureToken::U64],
                kind_constraints: vec![],
            },
        ],
        locals_signatures: vec![LocalsSignature(vec![])],
        string_pool: vec![
            "hello".to_string(),
            "module".to_string(),
            "existing_module".to_string(),
            "func1".to_string(),
            "func2".to_string(),
            "main".to_string(),
        ],
        byte_array_pool: vec![],
        address_pool: vec![AccountAddress::default()],
    }
    .freeze()
    .expect("test script should satisfy bounds checker");
    let script = VerifiedScript::new(script).expect("test script should satisfy bytecode verifier");

    let owned_entry_module = script.into_module();
    let loaded_main = LoadedModule::new(owned_entry_module);
    let entry_func = FunctionRef::new(&loaded_main, CompiledScript::MAIN_INDEX);
    let entry_module = entry_func.module();

    {
        let txn_allocator = Arena::new();
        {
            let txn_cache = TransactionModuleCache::new(&vm_cache, &txn_allocator);

            // We should be able to read existing modules in both cache.
            let func1_vm_ref = vm_cache
                .resolve_function_ref(entry_module, FunctionHandleIndex::new(2))
                .unwrap()
                .unwrap();
            let func1_txn_ref = txn_cache
                .resolve_function_ref(entry_module, FunctionHandleIndex::new(2))
                .unwrap()
                .unwrap();
            assert_eq!(func1_vm_ref, func1_txn_ref);

            txn_cache.cache_module(test_module("module".to_string()));

            // We should not read the new module in the vm cache, but we should read it from the txn
            // cache.
            assert!(vm_cache
                .resolve_function_ref(entry_module, FunctionHandleIndex::new(1))
                .unwrap()
                .unwrap()
                .is_none());
            let func2_txn_ref = txn_cache
                .resolve_function_ref(entry_module, FunctionHandleIndex::new(1))
                .unwrap()
                .unwrap()
                .unwrap();
            assert_eq!(func2_txn_ref.arg_count(), 1);
            assert_eq!(func2_txn_ref.return_count(), 0);
            assert_eq!(
                func2_txn_ref.code_definition(),
                vec![Bytecode::Ret].as_slice()
            );
        }

        // Drop the transactional arena
        vm_cache.reclaim_cached_module(txn_allocator.into_vec());
    }

    // After reclaiming we should see it from the
    let func2_ref = vm_cache
        .resolve_function_ref(entry_module, FunctionHandleIndex::new(1))
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(func2_ref.arg_count(), 1);
    assert_eq!(func2_ref.return_count(), 0);
    assert_eq!(func2_ref.code_definition(), vec![Bytecode::Ret].as_slice());
}

fn parse_and_compile_modules(s: impl AsRef<str>) -> Vec<CompiledModule> {
    let compiler = Compiler {
        code: s.as_ref(),
        skip_stdlib_deps: true,
        ..Compiler::default()
    };
    compiler
        .into_compiled_program()
        .expect("Failed to compile program")
        .modules
}

#[test]
fn test_same_module_struct_resolution() {
    let allocator = Arena::new();
    let vm_cache = VMModuleCache::new(&allocator);

    let code = "
        modules:
        module M1 {
            struct X {}
            struct T { i: u64, x: V#Self.X }
        }
        script:
        main() {
            return;
        }
        ";

    let module = parse_and_compile_modules(code);
    let fetcher = FakeFetcher::new(module);
    let block_cache = BlockModuleCache::new(&vm_cache, fetcher);
    {
        let module_id = ModuleId::new(AccountAddress::default(), "M1".to_string());
        let module_ref = block_cache
            .get_loaded_module(&module_id)
            .unwrap()
            .unwrap()
            .unwrap();
        let gas = GasMeter::new(GasUnits::new(100_000_000));
        let struct_x = block_cache
            .resolve_struct_def(module_ref, StructDefinitionIndex::new(0), &gas)
            .unwrap()
            .unwrap()
            .unwrap();
        let struct_t = block_cache
            .resolve_struct_def(module_ref, StructDefinitionIndex::new(1), &gas)
            .unwrap()
            .unwrap()
            .unwrap();
        assert_eq!(struct_x, StructDef::new(vec![]));
        assert_eq!(
            struct_t,
            StructDef::new(vec![Type::U64, Type::Struct(StructDef::new(vec![]))]),
        );
    }
}

#[test]
fn test_multi_module_struct_resolution() {
    let allocator = Arena::new();
    let vm_cache = VMModuleCache::new(&allocator);

    let code = format!(
        "
        modules:
        module M1 {{
            struct X {{}}
        }}
        module M2 {{
            import 0x{0}.M1;
            struct T {{ i: u64, x: V#M1.X }}
        }}
        script:
        main() {{
            return;
        }}
        ",
        hex::encode(AccountAddress::default())
    );

    let module = parse_and_compile_modules(&code);
    let fetcher = FakeFetcher::new(module);
    let block_cache = BlockModuleCache::new(&vm_cache, fetcher);
    {
        let module_id_2 = ModuleId::new(AccountAddress::default(), "M2".to_string());
        let module2_ref = block_cache
            .get_loaded_module(&module_id_2)
            .unwrap()
            .unwrap()
            .unwrap();

        let gas = GasMeter::new(GasUnits::new(100_000_000));
        let struct_t = block_cache
            .resolve_struct_def(module2_ref, StructDefinitionIndex::new(0), &gas)
            .unwrap()
            .unwrap()
            .unwrap();
        assert_eq!(
            struct_t,
            StructDef::new(vec![Type::U64, Type::Struct(StructDef::new(vec![]))]),
        );
    }
}

#[test]
fn test_field_offset_resolution() {
    let allocator = Arena::new();
    let vm_cache = VMModuleCache::new(&allocator);

    let code = "
        modules:
        module M1 {
            struct X { f: u64, g: bool}
            struct T { i: u64, x: V#Self.X, y: u64 }
        }
        script:
        main() {
            return;
        }
        ";

    let module = parse_and_compile_modules(code);
    let fetcher = FakeFetcher::new(module);
    let block_cache = BlockModuleCache::new(&vm_cache, fetcher);
    {
        let module_id = ModuleId::new(AccountAddress::default(), "M1".to_string());
        let module_ref = block_cache
            .get_loaded_module(&module_id)
            .unwrap()
            .unwrap()
            .unwrap();

        let f_idx = module_ref.field_defs_table.get("f").unwrap();
        assert_eq!(module_ref.get_field_offset(*f_idx).unwrap(), 0);

        let g_idx = module_ref.field_defs_table.get("g").unwrap();
        assert_eq!(module_ref.get_field_offset(*g_idx).unwrap(), 1);

        let i_idx = module_ref.field_defs_table.get("i").unwrap();
        assert_eq!(module_ref.get_field_offset(*i_idx).unwrap(), 0);

        let x_idx = module_ref.field_defs_table.get("x").unwrap();
        assert_eq!(module_ref.get_field_offset(*x_idx).unwrap(), 1);

        let y_idx = module_ref.field_defs_table.get("y").unwrap();
        assert_eq!(module_ref.get_field_offset(*y_idx).unwrap(), 2);
    }
}

#[test]
fn test_dependency_fails_verification() {
    let allocator = Arena::new();
    let vm_cache = VMModuleCache::new(&allocator);

    // This module has a struct inside a resource, which should fail verification. But assume that
    // it made its way onto the chain somehow (e.g. there was a bug in an older version of the
    // bytecode verifier).
    let code = "
    modules:
    module Test {
        resource R1 { }
        struct S1 { r1: R#Self.R1 }

        public new_S1(): V#Self.S1 {
            let s: V#Self.S1;
            let r: R#Self.R1;
            r = R1 {};
            s = S1 { r1: move(r) };
            return move(s);
        }
    }

    script:
    main() {
    }
    ";

    let module = parse_and_compile_modules(code);
    let fetcher = FakeFetcher::new(module);
    let block_cache = BlockModuleCache::new(&vm_cache, fetcher);

    let module_id = ModuleId::new(AccountAddress::default(), "Test".to_string());
    let VMRuntimeError { err, .. } = block_cache
        .get_loaded_module(&module_id)
        .unwrap()
        .unwrap_err();
    let errors = match err {
        VMErrorKind::Verification(errors) => errors,
        other => panic!("Unexpected error: {:?}", other),
    };
    assert_matches!(
        &errors[0],
        VerificationStatus::Dependency(module_id, _)
            if module_id.address() == &AccountAddress::default() && module_id.name() == "Test"
    );
}

// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::{
    code_cache::module_cache::VMModuleCache,
    loaded_data::{
        function::{FunctionRef, FunctionReference},
        loaded_module::LoadedModule,
    },
};
use anyhow::{format_err, Result};
use bytecode_verifier::{VerifiedModule, VerifiedScript};
use compiler::Compiler;
use libra_state_view::StateView;
use libra_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    language_storage::ModuleId,
    vm_error::{StatusCode, StatusType},
};
use move_core_types::identifier::Identifier;
use move_vm_cache::Arena;
use move_vm_state::{
    data_cache::{BlockDataCache, RemoteCache},
    execution_context::{SystemExecutionContext, TransactionExecutionContext},
};
use move_vm_types::loaded_data::types::{StructType, Type};
use std::collections::HashMap;
use vm::{
    access::ModuleAccess,
    errors::VMResult,
    file_format::*,
    gas_schedule::{GasAlgebra, GasUnits},
};

struct NullStateView;

impl StateView for NullStateView {
    fn get(&self, _ap: &AccessPath) -> Result<Option<Vec<u8>>> {
        Err(format_err!("no get on null state view"))
    }

    fn multi_get(&self, _ap: &[AccessPath]) -> Result<Vec<Option<Vec<u8>>>> {
        Err(format_err!("no get on null state view"))
    }

    fn is_genesis(&self) -> bool {
        false
    }
}

#[derive(Debug, Default)]
struct FakeDataCache {
    data: HashMap<AccessPath, Vec<u8>>,
}

impl FakeDataCache {
    fn get(&self, access_path: &AccessPath) -> VMResult<Option<Vec<u8>>> {
        Ok(self.data.get(access_path).cloned())
    }

    fn set(&mut self, module: CompiledModule) {
        let ap: AccessPath = (&module.self_id()).into();
        let mut blob: Vec<u8> = vec![];
        module.serialize(&mut blob).expect("Module must serialize");
        self.data.insert(ap, blob);
    }
}

impl RemoteCache for FakeDataCache {
    fn get(&self, access_path: &AccessPath) -> VMResult<Option<Vec<u8>>> {
        FakeDataCache::get(self, access_path)
    }
}

fn test_module(name: &'static str) -> VerifiedModule {
    let compiled_module = CompiledModuleMut {
        module_handles: vec![ModuleHandle {
            name: IdentifierIndex(0),
            address: AddressPoolIndex(0),
        }],
        struct_handles: vec![],
        signatures: vec![Signature(vec![]), Signature(vec![SignatureToken::U64])],
        function_handles: vec![
            FunctionHandle {
                module: ModuleHandleIndex(0),
                name: IdentifierIndex(1),
                return_: SignatureIndex(0),
                parameters: SignatureIndex(0),
                type_parameters: vec![],
            },
            FunctionHandle {
                module: ModuleHandleIndex(0),
                name: IdentifierIndex(2),
                return_: SignatureIndex(0),
                parameters: SignatureIndex(1),
                type_parameters: vec![],
            },
        ],
        field_handles: vec![],
        struct_def_instantiations: vec![],
        function_instantiations: vec![],
        field_instantiations: vec![],

        struct_defs: vec![],
        function_defs: vec![
            FunctionDefinition {
                function: FunctionHandleIndex(0),
                flags: CodeUnit::PUBLIC,
                acquires_global_resources: vec![],
                code: CodeUnit {
                    max_stack_size: 10,
                    locals: SignatureIndex(0),
                    code: vec![Bytecode::LdTrue, Bytecode::Pop, Bytecode::Ret],
                },
            },
            FunctionDefinition {
                function: FunctionHandleIndex(1),
                flags: CodeUnit::PUBLIC,
                acquires_global_resources: vec![],
                code: CodeUnit {
                    max_stack_size: 10,
                    locals: SignatureIndex(1),
                    code: vec![Bytecode::Ret],
                },
            },
        ],
        identifiers: idents(vec![name, "func1", "func2"]),
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
            function: FunctionHandleIndex(0),
            flags: CodeUnit::PUBLIC,
            acquires_global_resources: vec![],
            code: CodeUnit {
                max_stack_size: 10,
                locals: SignatureIndex(0),
                code: vec![Bytecode::Ret],
            },
        },
        module_handles: vec![
            ModuleHandle {
                address: AddressPoolIndex(0),
                name: IdentifierIndex(0),
            },
            ModuleHandle {
                address: AddressPoolIndex(0),
                name: IdentifierIndex(1),
            },
        ],
        struct_handles: vec![],
        signatures: vec![Signature(vec![]), Signature(vec![SignatureToken::U64])],
        function_handles: vec![
            FunctionHandle {
                name: IdentifierIndex(4),
                module: ModuleHandleIndex(0),
                parameters: SignatureIndex(0),
                return_: SignatureIndex(0),
                type_parameters: vec![],
            },
            FunctionHandle {
                name: IdentifierIndex(2),
                module: ModuleHandleIndex(1),
                parameters: SignatureIndex(0),
                return_: SignatureIndex(0),
                type_parameters: vec![],
            },
            FunctionHandle {
                name: IdentifierIndex(3),
                module: ModuleHandleIndex(1),
                parameters: SignatureIndex(1),
                return_: SignatureIndex(0),
                type_parameters: vec![],
            },
        ],
        function_instantiations: vec![],
        identifiers: idents(vec!["hello", "module", "func1", "func2", "main"]),
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
    let module = test_module("module");
    let mod_id = module.self_id();

    let data_cache = FakeDataCache::default();
    let ctx = SystemExecutionContext::new(&data_cache, GasUnits::new(0));

    let allocator = Arena::new();
    let loaded_program = VMModuleCache::new(&allocator);
    loaded_program.cache_module(module);
    let module_ref = loaded_program.get_loaded_module(&mod_id, &ctx).unwrap();

    // Get the function reference of the first two function handles.
    let func1_ref = loaded_program
        .resolve_function_ref(module_ref, FunctionHandleIndex(0), &ctx)
        .unwrap();
    let func2_ref = loaded_program
        .resolve_function_ref(module_ref, FunctionHandleIndex(1), &ctx)
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
    let module = test_module("module");

    let allocator = Arena::new();
    let loaded_program = VMModuleCache::new(&allocator);
    loaded_program.cache_module(module);

    let data_cache = FakeDataCache::default();
    let ctx = SystemExecutionContext::new(&data_cache, GasUnits::new(0));

    let owned_entry_module = script.into_module();
    let loaded_main = LoadedModule::new(owned_entry_module);
    let entry_func = FunctionRef::new(&loaded_main, CompiledScript::MAIN_INDEX);
    let entry_module = entry_func.module();
    let func1 = loaded_program
        .resolve_function_ref(entry_module, FunctionHandleIndex(1), &ctx)
        .unwrap();
    let func2 = loaded_program
        .resolve_function_ref(entry_module, FunctionHandleIndex(2), &ctx)
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

    let vm_cache = VMModuleCache::new(&allocator);

    let data_cache = FakeDataCache::default();
    let ctx = SystemExecutionContext::new(&data_cache, GasUnits::new(0));

    // Function is not defined locally.
    assert!(vm_cache
        .resolve_function_ref(entry_module, FunctionHandleIndex(1), &ctx)
        .is_err());

    let mut data_cache = FakeDataCache::default();
    data_cache.set(test_module("module").into_inner());
    let ctx = SystemExecutionContext::new(&data_cache, GasUnits::new(0));

    // Make sure the block cache fetches the code from the view.
    let func1 = vm_cache
        .resolve_function_ref(entry_module, FunctionHandleIndex(1), &ctx)
        .unwrap();
    let func2 = vm_cache
        .resolve_function_ref(entry_module, FunctionHandleIndex(2), &ctx)
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
    let data_cache = FakeDataCache::default();
    let ctx = SystemExecutionContext::new(&data_cache, GasUnits::new(0));

    let func1 = vm_cache
        .resolve_function_ref(entry_module, FunctionHandleIndex(1), &ctx)
        .unwrap();
    let func2 = vm_cache
        .resolve_function_ref(entry_module, FunctionHandleIndex(2), &ctx)
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
    let module = test_module("existing_module");
    vm_cache.cache_module(module);

    // Create a new script that refers to both published and unpublished modules.
    let script = CompiledScriptMut {
        main: FunctionDefinition {
            function: FunctionHandleIndex(0),
            flags: CodeUnit::PUBLIC,
            acquires_global_resources: vec![],
            code: CodeUnit {
                max_stack_size: 10,
                locals: SignatureIndex(0),
                code: vec![Bytecode::Ret],
            },
        },
        module_handles: vec![
            // Self
            ModuleHandle {
                address: AddressPoolIndex(0),
                name: IdentifierIndex(0),
            },
            // To-be-published Module
            ModuleHandle {
                address: AddressPoolIndex(0),
                name: IdentifierIndex(1),
            },
            // Existing module on chain
            ModuleHandle {
                address: AddressPoolIndex(0),
                name: IdentifierIndex(2),
            },
        ],
        struct_handles: vec![],
        signatures: vec![Signature(vec![]), Signature(vec![SignatureToken::U64])],
        function_handles: vec![
            // main
            FunctionHandle {
                name: IdentifierIndex(5),
                module: ModuleHandleIndex(0),
                parameters: SignatureIndex(0),
                return_: SignatureIndex(0),
                type_parameters: vec![],
            },
            // Func2 defined in the new module
            FunctionHandle {
                name: IdentifierIndex(4),
                module: ModuleHandleIndex(1),
                parameters: SignatureIndex(0),
                return_: SignatureIndex(0),
                type_parameters: vec![],
            },
            // Func1 defined in the old module
            FunctionHandle {
                name: IdentifierIndex(3),
                module: ModuleHandleIndex(2),
                parameters: SignatureIndex(1),
                return_: SignatureIndex(0),
                type_parameters: vec![],
            },
        ],
        function_instantiations: vec![],
        identifiers: idents(vec![
            "hello",
            "module",
            "existing_module",
            "func1",
            "func2",
            "main",
        ]),
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

    vm_cache.cache_module(test_module("module"));

    // After reclaiming we should see it from the cache
    let data_cache = FakeDataCache::default();
    let ctx = SystemExecutionContext::new(&data_cache, GasUnits::new(0));
    let func2_ref = vm_cache
        .resolve_function_ref(entry_module, FunctionHandleIndex(1), &ctx)
        .unwrap();
    assert_eq!(func2_ref.arg_count(), 1);
    assert_eq!(func2_ref.return_count(), 0);
    assert_eq!(func2_ref.code_definition(), vec![Bytecode::Ret].as_slice());
}

fn parse_and_compile_module(s: impl AsRef<str>, deps: Vec<CompiledModule>) -> CompiledModule {
    let extra_deps = deps
        .iter()
        .map(|m| VerifiedModule::bypass_verifier_DANGEROUS_FOR_TESTING_ONLY(m.clone()))
        .collect();
    let compiler = Compiler {
        skip_stdlib_deps: true,
        extra_deps,
        ..Compiler::default()
    };
    compiler
        .into_compiled_module("file_name", s.as_ref())
        .expect("Failed to compile module")
}

#[test]
fn test_same_module_struct_resolution() {
    let allocator = Arena::new();
    let vm_cache = VMModuleCache::new(&allocator);

    let code = "
        module M1 {
            struct X { b: bool }
            struct T { i: u64, x: Self.X }
        }
        ";

    let mut data_cache = FakeDataCache::default();
    let module = parse_and_compile_module(code, vec![]);
    data_cache.set(module);
    let ctx = SystemExecutionContext::new(&data_cache, GasUnits::new(0));

    let module_id = ModuleId::new(AccountAddress::default(), ident("M1"));
    let module_ref = vm_cache.get_loaded_module(&module_id, &ctx).unwrap();

    let block_data_cache = BlockDataCache::new(&NullStateView);
    let context = TransactionExecutionContext::new(GasUnits::new(100_000_000), &block_data_cache);
    let struct_x = vm_cache
        .resolve_struct_def(module_ref, StructDefinitionIndex::new(0), &[], &context)
        .unwrap();
    let struct_t = vm_cache
        .resolve_struct_def(module_ref, StructDefinitionIndex::new(1), &[], &context)
        .unwrap();
    let struct_x_expected_ty = StructType {
        address: AccountAddress::from_hex_literal("0x0").unwrap(),
        module: Identifier::new("M1").unwrap(),
        name: Identifier::new("X").unwrap(),
        is_resource: false,
        ty_args: vec![],
        layout: vec![Type::Bool],
    };
    assert_eq!(struct_x, struct_x_expected_ty);
    let struct_t_expected_ty = StructType {
        address: AccountAddress::from_hex_literal("0x0").unwrap(),
        module: Identifier::new("M1").unwrap(),
        name: Identifier::new("T").unwrap(),
        is_resource: false,
        ty_args: vec![],
        layout: vec![Type::U64, Type::Struct(Box::new(struct_x_expected_ty))],
    };
    assert_eq!(struct_t, struct_t_expected_ty);
}

#[test]
fn test_multi_module_struct_resolution() {
    let allocator = Arena::new();
    let vm_cache = VMModuleCache::new(&allocator);

    let code1 = "
        module M1 {
            struct X { b: bool }
        }
        ";
    let code2 = format!(
        "
        module M2 {{
            import 0x{0}.M1;
            struct T {{ i: u64, x: M1.X }}
        }}
        ",
        hex::encode(AccountAddress::default())
    );

    let mut data_cache = FakeDataCache::default();
    let module1 = parse_and_compile_module(&code1, vec![]);
    data_cache.set(module1.clone());
    let module2 = parse_and_compile_module(&code2, vec![module1]);
    data_cache.set(module2);
    let ctx = SystemExecutionContext::new(&data_cache, GasUnits::new(0));

    // load both modules in the cache
    let module_id_1 = ModuleId::new(AccountAddress::default(), ident("M1"));
    vm_cache
        .get_loaded_module(&module_id_1, &ctx)
        .expect("M1 must be found");
    let module_id_2 = ModuleId::new(AccountAddress::default(), ident("M2"));
    let module2_ref = vm_cache
        .get_loaded_module(&module_id_2, &ctx)
        .expect("M2 must be found");

    let block_data_cache = BlockDataCache::new(&NullStateView);
    let context = TransactionExecutionContext::new(GasUnits::new(100_000_000), &block_data_cache);

    let struct_t = vm_cache
        .resolve_struct_def(module2_ref, StructDefinitionIndex::new(0), &[], &context)
        .unwrap();

    let struct_x_expected_ty = StructType {
        address: AccountAddress::from_hex_literal("0x0").unwrap(),
        module: Identifier::new("M1").unwrap(),
        name: Identifier::new("X").unwrap(),
        is_resource: false,
        ty_args: vec![],
        layout: vec![Type::Bool],
    };
    let struct_t_expected_ty = StructType {
        address: AccountAddress::from_hex_literal("0x0").unwrap(),
        module: Identifier::new("M2").unwrap(),
        name: Identifier::new("T").unwrap(),
        is_resource: false,
        ty_args: vec![],
        layout: vec![Type::U64, Type::Struct(Box::new(struct_x_expected_ty))],
    };
    assert_eq!(struct_t, struct_t_expected_ty,);
}

#[test]
fn test_dependency_fails_verification() {
    let allocator = Arena::new();
    let vm_cache = VMModuleCache::new(&allocator);

    // This module has a struct inside a resource, which should fail verification. But assume that
    // it made its way onto the chain somehow (e.g. there was a bug in an older version of the
    // bytecode verifier).
    let code = "
    module Test {
        resource R1 { b: bool }
        struct S1 { r1: Self.R1 }

        public new_S1(): Self.S1 {
            let s: Self.S1;
            let r: Self.R1;
            r = R1 { b: true };
            s = S1 { r1: move(r) };
            return move(s);
        }
    }
    ";

    let mut data_cache = FakeDataCache::default();
    let module = parse_and_compile_module(code, vec![]);
    data_cache.set(module);
    let ctx = SystemExecutionContext::new(&data_cache, GasUnits::new(0));

    let module_id = ModuleId::new(AccountAddress::default(), ident("Test"));
    let err = vm_cache.get_loaded_module(&module_id, &ctx).unwrap_err();
    assert!(err.is(StatusType::Verification));
    assert!(err.major_status == StatusCode::INVALID_RESOURCE_FIELD);
}

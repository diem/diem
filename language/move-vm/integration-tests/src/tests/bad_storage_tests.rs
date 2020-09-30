// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::compiler::compile_units;
use move_core_types::{
    account_address::AccountAddress,
    gas_schedule::{GasAlgebra, GasUnits},
    identifier::Identifier,
    language_storage::{ModuleId, StructTag},
    vm_status::StatusType,
};
use move_lang::compiled_unit::CompiledUnit;
use move_vm_runtime::move_vm::MoveVM;
use move_vm_test_utils::{convert_txn_effects_to_move_changeset_and_events, InMemoryStorage};
use move_vm_types::gas_schedule::{zero_cost_schedule, CostStrategy};
use vm::file_format::{CompiledModule, CompiledScript};

fn as_module(unit: CompiledUnit) -> CompiledModule {
    match unit {
        CompiledUnit::Module { module, .. } => module,
        CompiledUnit::Script { .. } => panic!("expected module got script"),
    }
}

fn as_script(unit: CompiledUnit) -> CompiledScript {
    match unit {
        CompiledUnit::Module { .. } => panic!("expected script got module"),
        CompiledUnit::Script { script, .. } => script,
    }
}

const TEST_ADDR: AccountAddress = AccountAddress::new([42; AccountAddress::LENGTH]);

#[test]
fn test_malformed_resource() {
    // Compile the modules and scripts.
    // TODO: find a better way to include the Signer module.
    let code = r#"
        address 0x1 {
            module Signer {
                native public fun borrow_address(s: &signer): &address;

                public fun address_of(s: &signer): address {
                    *borrow_address(s)
                }
            }
        }

        module M {
            use 0x1::Signer;

            resource struct Foo { x: u64, y: bool }

            public fun publish(s: &signer) {
                move_to(s, Foo { x: 123, y : false });
            }

            public fun check(s: &signer) acquires Foo {
                let foo = borrow_global<Foo>(Signer::address_of(s));
                assert(foo.x == 123 && foo.y == false, 42);
            }
        }

        script {
            use {{ADDR}}::M;

            fun main(s: &signer) {
                M::publish(s);
            }
        }

        script {
            use {{ADDR}}::M;

            fun main(s: &signer) {
                M::check(s);
            }
        }
    "#;
    let code = code.replace("{{ADDR}}", &format!("0x{}", TEST_ADDR.to_string()));
    let mut units = compile_units(TEST_ADDR, &code).unwrap();

    let s2 = as_script(units.pop().unwrap());
    let s1 = as_script(units.pop().unwrap());
    let m = as_module(units.pop().unwrap());
    let ms = as_module(units.pop().unwrap());

    let mut storage = InMemoryStorage::new();

    // Publish module Signer and module M.
    let mut blob = vec![];
    ms.serialize(&mut blob).unwrap();
    storage.publish_or_overwrite_module(ms.self_id(), blob);

    let mut blob = vec![];
    m.serialize(&mut blob).unwrap();
    storage.publish_or_overwrite_module(m.self_id(), blob);

    let vm = MoveVM::new();

    let cost_table = zero_cost_schedule();
    let mut cost_strategy = CostStrategy::system(&cost_table, GasUnits::new(0));

    // Execute the first script to publish a resource Foo.
    let mut script_blob = vec![];
    s1.serialize(&mut script_blob).unwrap();
    let mut sess = vm.new_session(&storage);
    sess.execute_script(
        script_blob,
        vec![],
        vec![],
        vec![TEST_ADDR],
        &mut cost_strategy,
    )
    .unwrap();
    let (changeset, _) =
        convert_txn_effects_to_move_changeset_and_events(sess.finish().unwrap()).unwrap();
    storage.apply(changeset).unwrap();

    // Execut the second script and make sure it succeeds. This script simply checks
    // that the published resource is what we expect it to be. This inital run is to ensure
    // the testing environment is indeed free of errors without external interference.
    let mut script_blob = vec![];
    s2.serialize(&mut script_blob).unwrap();
    {
        let mut sess = vm.new_session(&storage);
        sess.execute_script(
            script_blob.clone(),
            vec![],
            vec![],
            vec![TEST_ADDR],
            &mut cost_strategy,
        )
        .unwrap();
    }

    // Corrupt the resource in the storage.
    storage.publish_or_overwrite_resource(
        TEST_ADDR,
        StructTag {
            address: TEST_ADDR,
            module: Identifier::new("M").unwrap(),
            name: Identifier::new("Foo").unwrap(),
            type_params: vec![],
        },
        vec![0x3, 0x4, 0x5],
    );

    // Run the second script again.
    // The test will be successful if it fails with an invariant violation.
    {
        let mut sess = vm.new_session(&storage);
        let err = sess
            .execute_script(
                script_blob,
                vec![],
                vec![],
                vec![TEST_ADDR],
                &mut cost_strategy,
            )
            .unwrap_err();
        assert!(err.status_type() == StatusType::InvariantViolation);
    }
}

#[test]
fn test_malformed_module() {
    // Compile module M.
    let code = r#"
        module M {
            public fun foo() {}
        }
    "#;

    let code = code.replace("{{ADDR}}", &format!("0x{}", TEST_ADDR.to_string()));
    let mut units = compile_units(TEST_ADDR, &code).unwrap();

    let m = as_module(units.pop().unwrap());

    let mut blob = vec![];
    m.serialize(&mut blob).unwrap();

    let module_id = ModuleId::new(TEST_ADDR, Identifier::new("M").unwrap());
    let fun_name = Identifier::new("foo").unwrap();
    let cost_table = zero_cost_schedule();
    let mut cost_strategy = CostStrategy::system(&cost_table, GasUnits::new(0));

    // Publish M and call M::foo. No errors should be thrown.
    {
        let mut storage = InMemoryStorage::new();
        storage.publish_or_overwrite_module(m.self_id(), blob.clone());
        let vm = MoveVM::new();
        let mut sess = vm.new_session(&storage);
        sess.execute_function(
            &module_id,
            &fun_name,
            vec![],
            vec![],
            TEST_ADDR,
            &mut cost_strategy,
        )
        .unwrap();
    }

    // Start over with a fresh storage and publish a corrupted version of M.
    // A fresh VM needs to be used whenever the storage has been modified or otherwise the
    // loader cache gets out of sync.
    //
    // Try to call M::foo again and the module should fail to load, causing an
    // invariant violation error.
    {
        blob[0] = 0xde;
        blob[1] = 0xad;
        blob[2] = 0xbe;
        blob[3] = 0xef;
        let mut storage = InMemoryStorage::new();
        storage.publish_or_overwrite_module(m.self_id(), blob);
        let vm = MoveVM::new();
        let mut sess = vm.new_session(&storage);
        let err = sess
            .execute_function(
                &module_id,
                &fun_name,
                vec![],
                vec![],
                TEST_ADDR,
                &mut cost_strategy,
            )
            .unwrap_err();
        assert!(err.status_type() == StatusType::InvariantViolation);
    }
}

#[test]
fn test_unverifiable_module() {
    // Compile module M.
    let code = r#"
        module M {
            public fun foo() {}
        }
    "#;

    let mut units = compile_units(TEST_ADDR, &code).unwrap();
    let m = as_module(units.pop().unwrap());

    let cost_table = zero_cost_schedule();
    let mut cost_strategy = CostStrategy::system(&cost_table, GasUnits::new(0));
    let module_id = ModuleId::new(TEST_ADDR, Identifier::new("M").unwrap());
    let fun_name = Identifier::new("foo").unwrap();

    // Publish M and call M::foo to make sure it works.
    {
        let mut storage = InMemoryStorage::new();

        let mut blob = vec![];
        m.serialize(&mut blob).unwrap();
        storage.publish_or_overwrite_module(m.self_id(), blob);

        let vm = MoveVM::new();
        let mut sess = vm.new_session(&storage);

        sess.execute_function(
            &module_id,
            &fun_name,
            vec![],
            vec![],
            TEST_ADDR,
            &mut cost_strategy,
        )
        .unwrap();
    }

    // Erase the body of M::foo to make it fail verification.
    // Publish this modified version of M and the VM should fail to load it.
    {
        let mut storage = InMemoryStorage::new();

        let mut m = m.into_inner();
        m.function_defs[0].code.as_mut().unwrap().code = vec![];
        let m = m.freeze().unwrap();
        let mut blob = vec![];
        m.serialize(&mut blob).unwrap();
        storage.publish_or_overwrite_module(m.self_id(), blob);

        let vm = MoveVM::new();
        let mut sess = vm.new_session(&storage);

        let err = sess
            .execute_function(
                &module_id,
                &fun_name,
                vec![],
                vec![],
                TEST_ADDR,
                &mut cost_strategy,
            )
            .unwrap_err();

        assert!(err.status_type() == StatusType::InvariantViolation);
    }
}

#[test]
fn test_missing_module_dependency() {
    // Compile two modules M, N where N depends on M.
    let code = r#"
        module M {
            public fun foo() {}
        }

        module N {
            use {{ADDR}}::M;

            public fun bar() { M::foo(); }
        }
    "#;
    let code = code.replace("{{ADDR}}", &format!("0x{}", TEST_ADDR.to_string()));
    let mut units = compile_units(TEST_ADDR, &code).unwrap();
    let n = as_module(units.pop().unwrap());
    let m = as_module(units.pop().unwrap());

    let mut blob_m = vec![];
    m.serialize(&mut blob_m).unwrap();
    let mut blob_n = vec![];
    n.serialize(&mut blob_n).unwrap();

    let cost_table = zero_cost_schedule();
    let mut cost_strategy = CostStrategy::system(&cost_table, GasUnits::new(0));

    let module_id = ModuleId::new(TEST_ADDR, Identifier::new("N").unwrap());
    let fun_name = Identifier::new("bar").unwrap();

    // Publish M and N and call N::bar. Everything should work.
    {
        let mut storage = InMemoryStorage::new();

        storage.publish_or_overwrite_module(m.self_id(), blob_m);
        storage.publish_or_overwrite_module(n.self_id(), blob_n.clone());

        let vm = MoveVM::new();
        let mut sess = vm.new_session(&storage);

        sess.execute_function(
            &module_id,
            &fun_name,
            vec![],
            vec![],
            TEST_ADDR,
            &mut cost_strategy,
        )
        .unwrap();
    }

    // Publish only N and try to call N::bar. The VM should fail to find M and raise
    // an invariant violation.
    {
        let mut storage = InMemoryStorage::new();
        storage.publish_or_overwrite_module(n.self_id(), blob_n);

        let vm = MoveVM::new();
        let mut sess = vm.new_session(&storage);

        let err = sess
            .execute_function(
                &module_id,
                &fun_name,
                vec![],
                vec![],
                TEST_ADDR,
                &mut cost_strategy,
            )
            .unwrap_err();

        assert!(err.status_type() == StatusType::InvariantViolation);
    }
}

#[test]
fn test_malformed_module_denpency() {
    // Compile two modules M, N where N depends on M.
    let code = r#"
        module M {
            public fun foo() {}
        }

        module N {
            use {{ADDR}}::M;

            public fun bar() { M::foo(); }
        }
    "#;
    let code = code.replace("{{ADDR}}", &format!("0x{}", TEST_ADDR.to_string()));
    let mut units = compile_units(TEST_ADDR, &code).unwrap();
    let n = as_module(units.pop().unwrap());
    let m = as_module(units.pop().unwrap());

    let mut blob_m = vec![];
    m.serialize(&mut blob_m).unwrap();
    let mut blob_n = vec![];
    n.serialize(&mut blob_n).unwrap();

    let cost_table = zero_cost_schedule();
    let mut cost_strategy = CostStrategy::system(&cost_table, GasUnits::new(0));

    let module_id = ModuleId::new(TEST_ADDR, Identifier::new("N").unwrap());
    let fun_name = Identifier::new("bar").unwrap();

    // Publish M and N and call N::bar. Everything should work.
    {
        let mut storage = InMemoryStorage::new();

        storage.publish_or_overwrite_module(m.self_id(), blob_m.clone());
        storage.publish_or_overwrite_module(n.self_id(), blob_n.clone());

        let vm = MoveVM::new();
        let mut sess = vm.new_session(&storage);

        sess.execute_function(
            &module_id,
            &fun_name,
            vec![],
            vec![],
            TEST_ADDR,
            &mut cost_strategy,
        )
        .unwrap();
    }

    // Publish N and a corrupted version of M and try to call N::bar, the VM should fail to load M.
    {
        blob_m[0] = 0xde;
        blob_m[1] = 0xad;
        blob_m[2] = 0xbe;
        blob_m[3] = 0xef;

        let mut storage = InMemoryStorage::new();

        storage.publish_or_overwrite_module(m.self_id(), blob_m);
        storage.publish_or_overwrite_module(n.self_id(), blob_n);

        let vm = MoveVM::new();
        let mut sess = vm.new_session(&storage);

        let err = sess
            .execute_function(
                &module_id,
                &fun_name,
                vec![],
                vec![],
                TEST_ADDR,
                &mut cost_strategy,
            )
            .unwrap_err();

        assert!(err.status_type() == StatusType::InvariantViolation);
    }
}

#[test]
fn test_unverifiable_module_dependency() {
    // Compile two modules M, N where N depends on M.
    let code = r#"
        module M {
            public fun foo() {}
        }

        module N {
            use {{ADDR}}::M;

            public fun bar() { M::foo(); }
        }
    "#;
    let code = code.replace("{{ADDR}}", &format!("0x{}", TEST_ADDR.to_string()));
    let mut units = compile_units(TEST_ADDR, &code).unwrap();
    let n = as_module(units.pop().unwrap());
    let m = as_module(units.pop().unwrap());

    let mut blob_n = vec![];
    n.serialize(&mut blob_n).unwrap();

    let cost_table = zero_cost_schedule();
    let mut cost_strategy = CostStrategy::system(&cost_table, GasUnits::new(0));

    let module_id = ModuleId::new(TEST_ADDR, Identifier::new("N").unwrap());
    let fun_name = Identifier::new("bar").unwrap();

    // Publish M and N and call N::bar. Everything should work.
    {
        let mut blob_m = vec![];
        m.serialize(&mut blob_m).unwrap();

        let mut storage = InMemoryStorage::new();

        storage.publish_or_overwrite_module(m.self_id(), blob_m);
        storage.publish_or_overwrite_module(n.self_id(), blob_n.clone());

        let vm = MoveVM::new();
        let mut sess = vm.new_session(&storage);

        sess.execute_function(
            &module_id,
            &fun_name,
            vec![],
            vec![],
            TEST_ADDR,
            &mut cost_strategy,
        )
        .unwrap();
    }

    // Publish N and an unverifiable version of M and try to call N::bar, the VM should fail to load M.
    {
        let mut m = m.into_inner();
        m.function_defs[0].code.as_mut().unwrap().code = vec![];
        let m = m.freeze().unwrap();
        let mut blob_m = vec![];
        m.serialize(&mut blob_m).unwrap();

        let mut storage = InMemoryStorage::new();

        storage.publish_or_overwrite_module(m.self_id(), blob_m);
        storage.publish_or_overwrite_module(n.self_id(), blob_n);

        let vm = MoveVM::new();
        let mut sess = vm.new_session(&storage);

        let err = sess
            .execute_function(
                &module_id,
                &fun_name,
                vec![],
                vec![],
                TEST_ADDR,
                &mut cost_strategy,
            )
            .unwrap_err();

        assert!(err.status_type() == StatusType::InvariantViolation);
    }
}

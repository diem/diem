// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::compiler::{as_module, compile_units};
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::ModuleId,
    value::{serialize_values, MoveValue},
};
use move_vm_runtime::move_vm::MoveVM;
use move_vm_test_utils::InMemoryStorage;
use move_vm_types::gas_schedule::GasStatus;

const TEST_ADDR: AccountAddress = AccountAddress::new([42; AccountAddress::LENGTH]);

#[test]
fn mutated_accounts() {
    let code = r#"
        module {{ADDR}}::M {
            struct Foo has key { a: bool }
            public fun get(addr: address): bool acquires Foo {
                borrow_global<Foo>(addr).a
            }
            public fun flip(addr: address) acquires Foo {
                let f_ref = borrow_global_mut<Foo>(addr);
                f_ref.a = !f_ref.a;
            }
            public fun publish(addr: &signer) {
                move_to(addr, Foo { a: true} )
            }
        }
    "#;

    let code = code.replace("{{ADDR}}", &format!("0x{}", TEST_ADDR.to_string()));
    let mut units = compile_units(&code).unwrap();
    let m = as_module(units.pop().unwrap());
    let mut blob = vec![];
    m.serialize(&mut blob).unwrap();

    let mut storage = InMemoryStorage::new();
    let module_id = ModuleId::new(TEST_ADDR, Identifier::new("M").unwrap());
    storage.publish_or_overwrite_module(module_id.clone(), blob);

    let vm = MoveVM::new(vec![]).unwrap();
    let mut sess = vm.new_session(&storage);

    let mut gas_status = GasStatus::new_unmetered();

    let publish = Identifier::new("publish").unwrap();
    let flip = Identifier::new("flip").unwrap();
    let get = Identifier::new("get").unwrap();

    let account1 = AccountAddress::random();

    sess.execute_function(
        &module_id,
        &publish,
        vec![],
        serialize_values(&vec![MoveValue::Signer(account1)]),
        &mut gas_status,
    )
    .unwrap();

    // The resource was published to "account1" and the sender's account
    // (TEST_ADDR) is assumed to be mutated as well (e.g., in a subsequent
    // transaction epilogue).
    assert_eq!(sess.num_mutated_accounts(&TEST_ADDR), 2);

    sess.execute_function(
        &module_id,
        &get,
        vec![],
        serialize_values(&vec![MoveValue::Address(account1)]),
        &mut gas_status,
    )
    .unwrap();

    assert_eq!(sess.num_mutated_accounts(&TEST_ADDR), 2);

    sess.execute_function(
        &module_id,
        &flip,
        vec![],
        serialize_values(&vec![MoveValue::Address(account1)]),
        &mut gas_status,
    )
    .unwrap();
    assert_eq!(sess.num_mutated_accounts(&TEST_ADDR), 2);

    let (changes, _) = sess.finish().unwrap();
    storage.apply(changes).unwrap();

    let mut sess = vm.new_session(&storage);
    sess.execute_function(
        &module_id,
        &get,
        vec![],
        serialize_values(&vec![MoveValue::Address(account1)]),
        &mut gas_status,
    )
    .unwrap();

    // Only the sender's account (TEST_ADDR) should have been modified.
    assert_eq!(sess.num_mutated_accounts(&TEST_ADDR), 1);
}

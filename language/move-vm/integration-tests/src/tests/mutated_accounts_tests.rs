// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::compiler::{as_module, compile_units};
use move_core_types::{
    account_address::AccountAddress,
    gas_schedule::{GasAlgebra, GasUnits},
    identifier::Identifier,
    language_storage::ModuleId,
};
use move_vm_runtime::{logging::NoContextLog, move_vm::MoveVM};
use move_vm_test_utils::{convert_txn_effects_to_move_changeset_and_events, InMemoryStorage};
use move_vm_types::{
    gas_schedule::{zero_cost_schedule, CostStrategy},
    values::Value,
};

const TEST_ADDR: AccountAddress = AccountAddress::new([42; AccountAddress::LENGTH]);

#[test]
fn mutated_accounts() {
    let code = r#"
        module M {
            resource struct Foo { a: bool }
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

    let mut units = compile_units(TEST_ADDR, &code).unwrap();
    let m = as_module(units.pop().unwrap());
    let mut blob = vec![];
    m.serialize(&mut blob).unwrap();

    let mut storage = InMemoryStorage::new();
    let module_id = ModuleId::new(TEST_ADDR, Identifier::new("M").unwrap());
    storage.publish_or_overwrite_module(module_id.clone(), blob);

    let vm = MoveVM::new();
    let mut sess = vm.new_session(&storage);

    let cost_table = zero_cost_schedule();
    let mut cost_strategy = CostStrategy::system(&cost_table, GasUnits::new(0));
    let context = NoContextLog::new();

    let publish = Identifier::new("publish").unwrap();
    let flip = Identifier::new("flip").unwrap();
    let get = Identifier::new("get").unwrap();

    let account1 = AccountAddress::random();

    sess.execute_function(
        &module_id,
        &publish,
        vec![],
        vec![Value::transaction_argument_signer_reference(account1)],
        TEST_ADDR,
        &mut cost_strategy,
        &context,
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
        vec![Value::address(account1)],
        TEST_ADDR,
        &mut cost_strategy,
        &context,
    )
    .unwrap();

    assert_eq!(sess.num_mutated_accounts(&TEST_ADDR), 2);

    sess.execute_function(
        &module_id,
        &flip,
        vec![],
        vec![Value::address(account1)],
        TEST_ADDR,
        &mut cost_strategy,
        &context,
    )
    .unwrap();
    assert_eq!(sess.num_mutated_accounts(&TEST_ADDR), 2);

    let (changes, _) =
        convert_txn_effects_to_move_changeset_and_events(sess.finish().unwrap()).unwrap();
    storage.apply(changes).unwrap();

    let mut sess = vm.new_session(&storage);
    sess.execute_function(
        &module_id,
        &get,
        vec![],
        vec![Value::address(account1)],
        TEST_ADDR,
        &mut cost_strategy,
        &context,
    )
    .unwrap();

    // Only the sender's account (TEST_ADDR) should have been modified.
    assert_eq!(sess.num_mutated_accounts(&TEST_ADDR), 1);
}

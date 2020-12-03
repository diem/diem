// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::compiler::{as_module, compile_units};
use move_core_types::{
    account_address::AccountAddress,
    gas_schedule::{GasAlgebra, GasUnits},
    identifier::Identifier,
    language_storage::ModuleId,
    vm_status::StatusType,
};
use move_vm_runtime::{logging::NoContextLog, move_vm::MoveVM};
use move_vm_test_utils::{BlankStorage, InMemoryStorage};
use move_vm_types::gas_schedule::{zero_cost_schedule, CostStrategy};

const TEST_ADDR: AccountAddress = AccountAddress::new([42; AccountAddress::LENGTH]);

#[test]
fn call_non_existent_module() {
    let vm = MoveVM::new();
    let storage = BlankStorage;

    let mut sess = vm.new_session(&storage);
    let module_id = ModuleId::new(TEST_ADDR, Identifier::new("M").unwrap());
    let fun_name = Identifier::new("foo").unwrap();
    let cost_table = zero_cost_schedule();
    let mut cost_strategy = CostStrategy::system(&cost_table, GasUnits::new(0));
    let context = NoContextLog::new();

    let err = sess
        .execute_function(
            &module_id,
            &fun_name,
            vec![],
            vec![],
            TEST_ADDR,
            &mut cost_strategy,
            &context,
        )
        .unwrap_err();

    assert_eq!(err.status_type(), StatusType::InvariantViolation);
}

#[test]
fn call_non_existent_function() {
    let code = r#"
        module M {}
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

    let fun_name = Identifier::new("foo").unwrap();
    let cost_table = zero_cost_schedule();
    let mut cost_strategy = CostStrategy::system(&cost_table, GasUnits::new(0));
    let context = NoContextLog::new();

    let err = sess
        .execute_function(
            &module_id,
            &fun_name,
            vec![],
            vec![],
            TEST_ADDR,
            &mut cost_strategy,
            &context,
        )
        .unwrap_err();

    assert_eq!(err.status_type(), StatusType::InvariantViolation);
}

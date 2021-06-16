// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::compiler::{as_module, compile_units};
use move_binary_format::errors::VMResult;
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
    value::{MoveTypeLayout, MoveValue},
    vm_status::StatusCode,
};
use move_vm_runtime::move_vm::MoveVM;
use move_vm_test_utils::InMemoryStorage;
use move_vm_types::gas_schedule::GasStatus;

const TEST_ADDR: AccountAddress = AccountAddress::new([42; AccountAddress::LENGTH]);

fn run(
    structs: &[&str],
    fun_sig: &str,
    fun_body: &str,
    ty_args: Vec<TypeTag>,
    args: Vec<MoveValue>,
) -> VMResult<Vec<Vec<u8>>> {
    let structs = structs.to_vec().join("\n");

    let code = format!(
        r#"
        module 0x{}::M {{
            {}

            fun foo{} {{
                {}
            }}
        }}
    "#,
        TEST_ADDR, structs, fun_sig, fun_body
    );

    let mut units = compile_units(&code).unwrap();
    let m = as_module(units.pop().unwrap());
    let mut blob = vec![];
    m.serialize(&mut blob).unwrap();

    let mut storage = InMemoryStorage::new();
    let module_id = ModuleId::new(TEST_ADDR, Identifier::new("M").unwrap());
    storage.publish_or_overwrite_module(module_id.clone(), blob);

    let vm = MoveVM::new(vec![]).unwrap();
    let mut sess = vm.new_session(&storage);

    let fun_name = Identifier::new("foo").unwrap();
    let mut gas_status = GasStatus::new_unmetered();

    let args: Vec<_> = args
        .into_iter()
        .map(|val| val.simple_serialize().unwrap())
        .collect();

    let return_vals =
        sess.execute_function(&module_id, &fun_name, ty_args, args, &mut gas_status)?;

    Ok(return_vals)
}

fn expect_success(
    structs: &[&str],
    fun_sig: &str,
    fun_body: &str,
    ty_args: Vec<TypeTag>,
    args: Vec<MoveValue>,
    expected_layouts: &[MoveTypeLayout],
) {
    let return_vals = run(structs, fun_sig, fun_body, ty_args, args).unwrap();
    assert!(return_vals.len() == expected_layouts.len());

    for (blob, layout) in return_vals.iter().zip(expected_layouts.iter()) {
        MoveValue::simple_deserialize(blob, layout).unwrap();
    }
}

fn expect_failure(
    structs: &[&str],
    fun_sig: &str,
    fun_body: &str,
    ty_args: Vec<TypeTag>,
    args: Vec<MoveValue>,
    expected_status: StatusCode,
) {
    assert_eq!(
        run(structs, fun_sig, fun_body, ty_args, args)
            .unwrap_err()
            .major_status(),
        expected_status
    );
}

#[test]
fn return_nothing() {
    expect_success(&[], "()", "", vec![], vec![], &[])
}

#[test]
fn return_u64() {
    expect_success(&[], "(): u64", "42", vec![], vec![], &[MoveTypeLayout::U64])
}

#[test]
fn return_u64_bool() {
    expect_success(
        &[],
        "(): (u64, bool)",
        "(42, true)",
        vec![],
        vec![],
        &[MoveTypeLayout::U64, MoveTypeLayout::Bool],
    )
}

#[test]
fn return_signer_ref() {
    expect_failure(
        &[],
        "(s: &signer): &signer",
        "s",
        vec![],
        vec![MoveValue::Signer(TEST_ADDR)],
        StatusCode::INTERNAL_TYPE_ERROR,
    )
}

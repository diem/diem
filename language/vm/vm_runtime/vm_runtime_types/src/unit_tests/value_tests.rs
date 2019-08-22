// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[test]
fn test_simple_mutate() {
    let v = Local::u64(1);
    let v_ref = v.borrow_local().unwrap();
    let v2 = Local::u64(2);
    v_ref.mutate_reference(v2.value().unwrap());
    assert!(v.equals(Local::u64(2)).unwrap());
}

#[test]
fn test_cloned_value() {
    let v = Local::u64(1);
    let v2 = v.clone();

    let v_ref = v.borrow_local().unwrap();
    let v3 = Local::u64(2);
    v_ref.mutate_reference(v3.value().unwrap());
    assert!(v.equals(Local::u64(2)).unwrap());
    assert!(v2.equals(Local::u64(1)).unwrap());
}

#[test]
fn test_cloned_references() {
    let v = Local::u64(1);

    let v_ref = v.borrow_local().unwrap();
    let v_ref_clone = v_ref.clone();

    let v3 = Local::u64(2);
    v_ref.mutate_reference(v3.value().unwrap());
    assert!(v.equals(Local::u64(2)).unwrap());
    assert!(v_ref_clone
        .read_reference()
        .unwrap()
        .equals(Local::u64(2))
        .unwrap());
}

#[test]
fn test_mutate_struct() {
    let v_ref = Local::Ref(MutVal::new(Value::Struct(
        vec![Local::u64(1), Local::u64(2)]
            .into_iter()
            .map(|v| v.value().unwrap())
            .collect(),
    )));

    let field_ref = v_ref.borrow_field(1).unwrap();

    let v2 = Local::u64(3);
    field_ref.mutate_reference(v2.value().unwrap());

    let v_after = Local::struct_(
        vec![Local::u64(1), Local::u64(3)]
            .into_iter()
            .map(|v| v.value().unwrap())
            .collect(),
    );
    assert!(v_ref
        .read_reference()
        .expect("must be a reference")
        .equals(v_after)
        .unwrap());
}

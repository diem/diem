// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use std::rc::Rc;
use types::access_path::AccessPath;

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

#[test]
fn test_simple_global_ref() {
    // make a global ref to a struct
    let v = Value::Struct(vec![
        MutVal::new(Value::U64(1)),
        MutVal::new(Value::U64(10)),
        MutVal::new(Value::Bool(true)),
    ]);
    let ap = AccessPath::new(AccountAddress::new([1; 32]), vec![]);
    let v_ref = MutVal::new(v);
    // make a root
    let root = GlobalRef::make_root(ap, v_ref);
    assert_eq!(Rc::strong_count(&root.root), 1);
    assert_eq!(root.root.borrow().ref_count, 0);
    // get a reference to the root (BorrowGlobal)
    let global_ref = root.shallow_clone();
    assert_eq!(Rc::strong_count(&root.root), 2);
    assert_eq!(root.root.borrow().ref_count, 1);

    // get a copy, drop it and verify ref count in the process
    let global_ref1 = global_ref.shallow_clone();
    assert_eq!(Rc::strong_count(&global_ref1.root), 3);
    assert_eq!(Rc::strong_count(&global_ref.root), 3);
    assert_eq!(root.root.borrow().ref_count, 2);
    global_ref1
        .release_reference()
        .expect("ref count must not be 0");
    assert_eq!(Rc::strong_count(&global_ref.root), 2);
    assert_eq!(root.root.borrow().ref_count, 1);
    assert_eq!(global_ref.is_dirty(), false);

    // get references to 2 fields and verify ref count
    let field0_ref: GlobalRef;
    {
        let global_ref1 = global_ref.shallow_clone();
        field0_ref = global_ref1.borrow_field(0).expect("field must exist");
    }
    // ref count to 3 because global_ref1 is dropped at the end of the block
    assert_eq!(Rc::strong_count(&global_ref.root), 3);
    assert_eq!(root.root.borrow().ref_count, 2);
    let field1_ref: GlobalRef;
    {
        let global_ref1 = global_ref.shallow_clone();
        field1_ref = global_ref1.borrow_field(1).expect("field must exist");
    }
    // ref count to 4 because global_ref1 is dropped at the end of the block
    assert_eq!(Rc::strong_count(&global_ref.root), 4);
    assert_eq!(root.root.borrow().ref_count, 3);

    // read reference to first field, verify value and ref count. read_reference() drops reference
    let field0_val = field0_ref.read_reference();
    match &*field0_val.peek() {
        Value::U64(i) => assert_eq!(*i, 1),
        _ => unreachable!("value must be int"),
    }
    assert_eq!(Rc::strong_count(&global_ref.root), 3);
    assert_eq!(root.root.borrow().ref_count, 2);
    assert_eq!(global_ref.is_dirty(), false);

    // write reference to second field, verify value and ref count.
    // mutate_reference() drops reference
    field1_ref.mutate_reference(MutVal::new(Value::U64(100)));
    assert_eq!(Rc::strong_count(&global_ref.root), 2);
    assert_eq!(root.root.borrow().ref_count, 1);
    assert_eq!(global_ref.is_dirty(), true);
    let field1_ref: GlobalRef;
    {
        let global_ref1 = global_ref.shallow_clone();
        field1_ref = global_ref1.borrow_field(1).expect("field must exist");
    }
    assert_eq!(Rc::strong_count(&global_ref.root), 3);
    assert_eq!(root.root.borrow().ref_count, 2);
    let field1_val = field1_ref.read_reference();
    match &*field1_val.peek() {
        Value::U64(i) => assert_eq!(*i, 100),
        _ => unreachable!("value must be int"),
    }
    // 1 reference left and dirty flag true
    assert_eq!(Rc::strong_count(&global_ref.root), 2);
    assert_eq!(root.root.borrow().ref_count, 1);
    assert_eq!(global_ref.is_dirty(), true);

    // drop last reference (ReleaseRef)
    global_ref
        .release_reference()
        .expect("ref count must not be 0");
    assert_eq!(Rc::strong_count(&root.root), 1);
    assert_eq!(root.root.borrow().ref_count, 0);
    assert_eq!(root.is_dirty(), true);
}

#[test]
fn test_simple_global_ref_err() {
    // make a global ref to a struct
    let v = Value::Struct(vec![
        MutVal::new(Value::U64(1)),
        MutVal::new(Value::U64(10)),
        MutVal::new(Value::Bool(true)),
    ]);
    let ap = AccessPath::new(AccountAddress::new([1; 32]), vec![]);
    let v_ref = MutVal::new(v);
    // make a root
    let root = GlobalRef::make_root(ap, v_ref);
    assert_eq!(Rc::strong_count(&root.root), 1);
    assert_eq!(root.root.borrow().ref_count, 0);
    // get a reference to the root (BorrowGlobal)
    let global_ref = root.shallow_clone();
    assert_eq!(Rc::strong_count(&root.root), 2);
    assert_eq!(root.root.borrow().ref_count, 1);

    // drop last reference (ReleaseRef)
    global_ref
        .release_reference()
        .expect("ref count must not be 0");
    assert_eq!(Rc::strong_count(&root.root), 1);
    assert_eq!(root.root.borrow().ref_count, 0);

    // error on another ReleaseRef
    assert!(root.release_reference().is_err());
}

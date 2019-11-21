// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use libra_types::{account_address::AccountAddress, byte_array::ByteArray};

macro_rules! assert_value_eq {
    ($v1: expr, $v2: expr $(,)?) => {{
        let v1 = &($v1);
        let v2 = &($v2);
        #[allow(clippy::match_wild_err_arm)]
        match v1.equals(v2) {
            Ok(b) => {
                if !b {
                    panic!("unequal values: {:?} != {:?}", v1, v2);
                }
            }
            Err(_) => {
                panic!("incomparable values: {:?}, {:?}", v1, v2);
            }
        }
    }};
}

macro_rules! assert_value_ne {
    ($v1: expr, $v2: expr $(,)?) => {
        let v1 = &($v1);
        let v2 = &($v2);
        #[allow(clippy::match_wild_err_arm)]
        match v1.equals(v2) {
            Ok(b) => {
                if b {
                    panic!("equal values: {:?} != {:?}", v1, v2);
                }
            }
            Err(_) => {
                panic!("incomparable values: {:?}, {:?}", v1, v2);
            }
        }
    };
}

macro_rules! assert_value_incomparable {
    ($v1: expr, $v2: expr $(,)?) => {{
        let v1 = &($v1);
        let v2 = &($v2);
        assert!(v1.equals(v2).is_err());
    }};
}

#[test]
fn test_value() {
    // creation, unwrapping and comparison of common values
    assert_eq!(
        Value::u64(10).value_as::<u64>().expect("must find u64"),
        10u64,
    );
    let addr = AccountAddress::random();
    assert_eq!(
        Value::address(addr)
            .value_as::<AccountAddress>()
            .expect("must find AccountAddress"),
        addr,
    );
    assert_eq!(
        Value::bool(true)
            .value_as::<bool>()
            .expect("must find bool"),
        true,
    );
    let ba = ByteArray::new(vec![0, 1, 2, 3]);
    assert_eq!(
        Value::byte_array(ba.clone())
            .value_as::<ByteArray>()
            .expect("must find ByteArray"),
        ba,
    );
    let struct_ = Struct::new(vec![Value::u64(10), Value::address(addr)]);
    assert_value_eq!(
        Value::struct_(struct_.clone())
            .value_as::<Struct>()
            .expect("must find Struct"),
        struct_.clone(),
    );
    let struct1 = Struct::new(vec![Value::u64(10), Value::struct_(struct_.clone())]);
    assert_value_eq!(
        Value::struct_(struct1.clone())
            .value_as::<Struct>()
            .expect("must find Struct"),
        struct1.clone(),
    );

    // equal, not equal
    assert_value_eq!(
        Value::struct_(struct_.clone()),
        Value::struct_(struct_.clone()),
    );
    let struct_eq = Struct::new(vec![Value::u64(10), Value::address(addr)]);
    assert_value_eq!(Value::struct_(struct_.clone()), Value::struct_(struct_eq),);
    assert_value_eq!(Value::u64(100), Value::u64(100));
    assert_value_eq!(Value::bool(true), Value::bool(true));
    assert_value_incomparable!(Value::struct_(struct_), Value::struct_(struct1));
    assert_value_ne!(Value::u64(100), Value::u64(200));
    assert_value_incomparable!(Value::bool(true), Value::u64(200));
    assert_value_ne!(Value::bool(true), Value::bool(false));
}

#[test]
fn test_locals() {
    let invalid = ValueImpl::Invalid;

    let mut locals = Locals::new(5);
    for local in &locals.0 {
        assert_value_eq!(local, &invalid);
    }
    locals
        .store_loc(0, Value::u64(10))
        .expect("local 0 must exist");
    locals
        .store_loc(1, Value::bool(true))
        .expect("local 1 must exist");
    assert_value_eq!(
        locals.copy_loc(0).expect("local 0 must be valid"),
        Value::u64(10),
    );
    assert_value_eq!(
        locals.copy_loc(1).expect("local 1 must be valid"),
        Value::bool(true),
    );
    for idx in 2..5 {
        match locals.copy_loc(idx) {
            Ok(_) => panic!("local cannot be accessed"),
            Err(err) => assert_eq!(err.major_status, StatusCode::INTERNAL_TYPE_ERROR),
        }
        match locals.move_loc(idx) {
            Ok(_) => panic!("local cannot be accessed"),
            Err(err) => assert_eq!(err.major_status, StatusCode::INTERNAL_TYPE_ERROR),
        }
        match locals.borrow_loc(idx) {
            Ok(_) => panic!("local cannot be accessed"),
            Err(err) => assert_eq!(err.major_status, StatusCode::INTERNAL_TYPE_ERROR),
        }
    }
    for idx in 5..10 {
        match locals.copy_loc(idx) {
            Ok(_) => panic!("local cannot be accessed"),
            Err(err) => assert_eq!(err.major_status, StatusCode::INDEX_OUT_OF_BOUNDS),
        }
        match locals.move_loc(idx) {
            Ok(_) => panic!("local cannot be accessed"),
            Err(err) => assert_eq!(err.major_status, StatusCode::INDEX_OUT_OF_BOUNDS),
        }
        match locals.borrow_loc(idx) {
            Ok(_) => panic!("local cannot be accessed"),
            Err(err) => assert_eq!(err.major_status, StatusCode::INDEX_OUT_OF_BOUNDS),
        }
    }
    assert_value_eq!(
        locals.move_loc(0).expect("local 0 must be valid"),
        Value::u64(10)
    );
    match locals.copy_loc(0) {
        Ok(_) => panic!("local cannot be accessed"),
        Err(err) => assert_eq!(err.major_status, StatusCode::INTERNAL_TYPE_ERROR),
    }
    match locals.move_loc(0) {
        Ok(_) => panic!("local cannot be accessed"),
        Err(err) => assert_eq!(err.major_status, StatusCode::INTERNAL_TYPE_ERROR),
    }
    match locals.borrow_loc(0) {
        Ok(_) => panic!("local cannot be accessed"),
        Err(err) => assert_eq!(err.major_status, StatusCode::INTERNAL_TYPE_ERROR),
    }
    locals
        .store_loc(0, Value::u64(100))
        .expect("local 0 must exist");
    assert_value_eq!(
        locals.move_loc(0).expect("local 0 must be valid"),
        Value::u64(100),
    );
    assert_value_eq!(
        locals.move_loc(1).expect("local 1 must be valid"),
        Value::bool(true),
    );
    for local in &locals.0 {
        assert_value_eq!(local, &invalid);
    }
    locals
        .store_loc(0, Value::u64(100))
        .expect("local 0 must exist");
    assert_value_eq!(
        locals.move_loc(0).expect("local 0 must be valid"),
        Value::u64(100),
    );
    locals
        .store_loc(0, Value::u64(1000))
        .expect("local 0 must exist");
    assert_value_eq!(
        locals.move_loc(0).expect("local 0 must be valid"),
        Value::u64(1000),
    );

    let mut locals = Locals::new(0);
    match locals.store_loc(0, Value::u64(100)) {
        Ok(_) => panic!("local cannot be accessed"),
        Err(err) => assert_eq!(err.major_status, StatusCode::INDEX_OUT_OF_BOUNDS),
    }
    match locals.copy_loc(0) {
        Ok(_) => panic!("local cannot be accessed"),
        Err(err) => assert_eq!(err.major_status, StatusCode::INDEX_OUT_OF_BOUNDS),
    }
}

#[test]
fn test_references() {
    let invalid = ValueImpl::Invalid;

    // make 5 locals and initialize 4 of them
    let mut locals = Locals::new(5);
    for local in &locals.0 {
        assert_value_eq!(local, &invalid);
    }
    locals
        .store_loc(0, Value::u64(10))
        .expect("local 0 must exist");
    locals
        .store_loc(1, Value::bool(true))
        .expect("local 1 must exist");
    let addr = AccountAddress::random();
    let struct_inner = Struct::new(vec![Value::u64(20), Value::bool(false)]);
    let struct_outer = Struct::new(vec![
        Value::u64(10),
        Value::address(addr),
        Value::struct_(struct_inner),
    ]);
    locals
        .store_loc(2, Value::struct_(struct_outer))
        .expect("local 2 must exist");
    match locals.borrow_loc(3) {
        Ok(_) => panic!("local cannot be accessed"),
        Err(err) => assert_eq!(err.major_status, StatusCode::INTERNAL_TYPE_ERROR),
    }

    // check, change and check again 1st local
    let ref0 = locals.borrow_loc(0).expect("local 0 must exist");
    assert_value_eq!(
        ref0.value_as::<Reference>()
            .expect("value must be a reference")
            .read_ref()
            .expect("reference must be valid"),
        Value::u64(10),
    );
    let ref0 = locals.borrow_loc(0).expect("local 0 must exist");
    ref0.value_as::<Reference>()
        .expect("value must be a reference")
        .write_ref(Value::u64(100))
        .expect("write ref must succeed");
    let ref0 = locals.borrow_loc(0).expect("local 0 must exist");
    assert_value_eq!(
        ref0.value_as::<Reference>()
            .expect("value must be a reference")
            .read_ref()
            .expect("reference must be valid"),
        Value::u64(100),
    );

    // check failure in borrow field on 1st local, move it out and check failure in borrow local
    let ref0 = locals.borrow_loc(0).expect("local 0 must exist");
    match ref0
        .value_as::<DirectRef>()
        .expect("value must be a reference")
        .borrow_field(0)
    {
        Ok(_) => panic!("reference not a Struct"),
        Err(err) => assert_eq!(err.major_status, StatusCode::INTERNAL_TYPE_ERROR),
    }
    assert_value_eq!(
        locals.move_loc(0).expect("local 0 must be valid"),
        Value::u64(100),
    );
    match locals.borrow_loc(0) {
        Ok(_) => panic!("local cannot be accessed"),
        Err(err) => assert_eq!(err.major_status, StatusCode::INTERNAL_TYPE_ERROR),
    }

    // check 2nd local
    let ref1 = locals.borrow_loc(1).expect("local 1 must exist");
    assert_value_eq!(
        ref1.value_as::<Reference>()
            .expect("value must be a reference")
            .read_ref()
            .expect("reference must be valid"),
        Value::bool(true),
    );

    // check 3rd local
    let ref3 = locals.borrow_loc(2).expect("local 2 must exist");
    let struct_inner = Struct::new(vec![Value::u64(20), Value::bool(false)]);
    let struct_outer = Struct::new(vec![
        Value::u64(10),
        Value::address(addr),
        Value::struct_(struct_inner),
    ]);
    assert_value_eq!(
        ref3.value_as::<Reference>()
            .expect("value must be a reference")
            .read_ref()
            .expect("reference must be valid"),
        Value::struct_(struct_outer),
    );

    // check, change and check again field 0 in 3rd local
    let ref3 = locals.borrow_loc(2).expect("local 2 must exist");
    let field_ref = ref3
        .value_as::<DirectRef>()
        .expect("value must be a reference")
        .borrow_field(0)
        .expect("field 0 must exist");
    assert_value_eq!(
        field_ref
            .value_as::<Reference>()
            .expect("value must be a reference")
            .read_ref()
            .expect("reference must be valid"),
        Value::u64(10),
    );
    let ref3 = locals.borrow_loc(2).expect("local 2 must exist");
    let field_ref = ref3
        .value_as::<DirectRef>()
        .expect("value must be a reference")
        .borrow_field(0)
        .expect("field 0 must exist");
    field_ref
        .value_as::<Reference>()
        .expect("value must be a reference")
        .write_ref(Value::u64(100))
        .expect("write ref must succeed");
    let ref3 = locals.borrow_loc(2).expect("local 2 must exist");
    let field_ref = ref3
        .value_as::<DirectRef>()
        .expect("value must be a reference")
        .borrow_field(0)
        .expect("field 0 must exist");
    assert_value_eq!(
        field_ref
            .value_as::<Reference>()
            .expect("value must be a reference")
            .read_ref()
            .expect("reference must be valid"),
        Value::u64(100),
    );

    // check and change field 1 in the inner struct of the 3rd local
    let ref3 = locals.borrow_loc(2).expect("local 2 must exist");
    let field_ref = ref3
        .value_as::<DirectRef>()
        .expect("value must be a reference")
        .borrow_field(2)
        .expect("field 2 must exist");
    let inner_field_ref = field_ref
        .value_as::<DirectRef>()
        .expect("value must be a reference")
        .borrow_field(1)
        .expect("field 1 must exist");
    assert_value_eq!(
        inner_field_ref
            .value_as::<Reference>()
            .expect("value must be a reference")
            .read_ref()
            .expect("reference must be valid"),
        Value::bool(false),
    );
    let ref3 = locals.borrow_loc(2).expect("local 2 must exist");
    let field_ref = ref3
        .value_as::<DirectRef>()
        .expect("value must be a reference")
        .borrow_field(2)
        .expect("field 2 must exist");
    let inner_field_ref = field_ref
        .value_as::<DirectRef>()
        .expect("value must be a reference")
        .borrow_field(1)
        .expect("field 1 must exist");
    inner_field_ref
        .value_as::<Reference>()
        .expect("value must be a reference")
        .write_ref(Value::bool(false))
        .expect("write ref must succeed");

    // verify struct in 3rd local is changed
    let ref3 = locals.borrow_loc(2).expect("local 2 must exist");
    let struct_inner = Struct::new(vec![Value::u64(20), Value::bool(false)]);
    let struct_outer = Struct::new(vec![
        Value::u64(100),
        Value::address(addr),
        Value::struct_(struct_inner),
    ]);
    assert_value_eq!(
        ref3.value_as::<Reference>()
            .expect("value must be a reference")
            .read_ref()
            .expect("reference must be valid"),
        Value::struct_(struct_outer.clone()),
    );
    assert_value_eq!(
        locals.move_loc(2).expect("local 2 must exist"),
        Value::struct_(struct_outer),
    );
}

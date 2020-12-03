// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::values::*;
use vm::errors::*;

#[test]
fn locals() -> PartialVMResult<()> {
    const LEN: usize = 4;
    let mut locals = Locals::new(LEN);
    for i in 0..LEN {
        assert!(locals.copy_loc(i).is_err());
        assert!(locals.move_loc(i).is_err());
        assert!(locals.borrow_loc(i).is_err());
    }
    locals.store_loc(1, Value::u64(42))?;

    assert!(locals.copy_loc(1)?.equals(&Value::u64(42))?);
    let r = locals.borrow_loc(1)?.value_as::<Reference>()?;
    assert!(r.read_ref()?.equals(&Value::u64(42))?);
    assert!(locals.move_loc(1)?.equals(&Value::u64(42))?);

    assert!(locals.copy_loc(1).is_err());
    assert!(locals.move_loc(1).is_err());
    assert!(locals.borrow_loc(1).is_err());

    assert!(locals.copy_loc(LEN + 1).is_err());
    assert!(locals.move_loc(LEN + 1).is_err());
    assert!(locals.borrow_loc(LEN + 1).is_err());

    Ok(())
}

#[test]
fn struct_pack_and_unpack() -> PartialVMResult<()> {
    let vals = vec![Value::u8(10), Value::u64(20), Value::u128(30)];
    let s = Struct::pack(vec![Value::u8(10), Value::u64(20), Value::u128(30)], false);
    let unpacked: Vec<_> = s.unpack()?.collect();

    assert!(vals.len() == unpacked.len());
    for (v1, v2) in vals.iter().zip(unpacked.iter()) {
        assert!(v1.equals(v2)?);
    }

    Ok(())
}

#[test]
fn struct_borrow_field() -> PartialVMResult<()> {
    let mut locals = Locals::new(1);
    locals.store_loc(
        0,
        Value::struct_(Struct::pack(vec![Value::u8(10), Value::bool(false)], false)),
    )?;
    let r: StructRef = locals.borrow_loc(0)?.value_as()?;

    {
        let f: Reference = r.borrow_field(1)?.value_as()?;
        assert!(f.read_ref()?.equals(&Value::bool(false))?);
    }

    {
        let f: Reference = r.borrow_field(1)?.value_as()?;
        f.write_ref(Value::bool(true))?;
    }

    {
        let f: Reference = r.borrow_field(1)?.value_as()?;
        assert!(f.read_ref()?.equals(&Value::bool(true))?);
    }

    Ok(())
}

#[test]
fn struct_borrow_nested() -> PartialVMResult<()> {
    let mut locals = Locals::new(1);

    fn inner(x: u64) -> Value {
        Value::struct_(Struct::pack(vec![Value::u64(x)], false))
    }
    fn outer(x: u64) -> Value {
        Value::struct_(Struct::pack(vec![Value::u8(10), inner(x)], false))
    }

    locals.store_loc(0, outer(20))?;
    let r1: StructRef = locals.borrow_loc(0)?.value_as()?;
    let r2: StructRef = r1.borrow_field(1)?.value_as()?;

    {
        let r3: Reference = r2.borrow_field(0)?.value_as()?;
        assert!(r3.read_ref()?.equals(&Value::u64(20))?);
    }

    {
        let r3: Reference = r2.borrow_field(0)?.value_as()?;
        r3.write_ref(Value::u64(30))?;
    }

    {
        let r3: Reference = r2.borrow_field(0)?.value_as()?;
        assert!(r3.read_ref()?.equals(&Value::u64(30))?);
    }

    assert!(r2.read_ref()?.equals(&inner(30))?);
    assert!(r1.read_ref()?.equals(&outer(30))?);

    Ok(())
}

#[test]
fn global_value_non_struct() -> PartialVMResult<()> {
    assert!(GlobalValue::cached(Value::u64(100)).is_err());
    assert!(GlobalValue::cached(Value::bool(false)).is_err());

    let mut locals = Locals::new(1);
    locals.store_loc(0, Value::u8(0))?;
    let r = locals.borrow_loc(0)?;
    assert!(GlobalValue::cached(r).is_err());

    Ok(())
}

fn dummy_resource() -> Value {
    Value::struct_(Struct::pack(vec![Value::bool(true)], true))
}

#[test]
fn cannot_copy_resource() -> PartialVMResult<()> {
    let v = dummy_resource();
    assert!(v.copy_value().is_err());
    Ok(())
}

#[test]
fn container_ref_cannot_read_resource() -> PartialVMResult<()> {
    let gv = GlobalValue::cached(dummy_resource())?;
    let r: Reference = gv.borrow_global()?.value_as()?;
    assert!(r.read_ref().is_err());
    Ok(())
}

#[test]
fn container_ref_cannot_overwrite_resource() -> PartialVMResult<()> {
    let gv = GlobalValue::cached(dummy_resource())?;
    let r: Reference = gv.borrow_global()?.value_as()?;
    assert!(r.write_ref(dummy_resource()).is_err());
    Ok(())
}

#[test]
fn locals_indexed_ref_cannot_read_resource() -> PartialVMResult<()> {
    let mut locals = Locals::new(1);
    locals.store_loc(0, dummy_resource())?;
    let r: Reference = locals.borrow_loc(0)?.value_as()?;
    assert!(r.read_ref().is_err());
    Ok(())
}

#[test]
fn locals_indexed_ref_cannot_overwrite_resource() -> PartialVMResult<()> {
    let mut locals = Locals::new(1);
    locals.store_loc(0, dummy_resource())?;
    let r: Reference = locals.borrow_loc(0)?.value_as()?;
    assert!(r.write_ref(dummy_resource()).is_err());
    Ok(())
}

// TODO: consider adding tests for vector_indexed_ref here once we cleanup the vector APIs.

#[test]
fn locals_cannot_copy_but_can_move_resource() -> PartialVMResult<()> {
    let mut locals = Locals::new(1);
    let v = dummy_resource();
    locals.store_loc(0, v)?;
    assert!(locals.copy_loc(0).is_err());
    assert!(locals.move_loc(0).is_ok());
    Ok(())
}

#[test]
fn locals_cannot_overwrite_resource() -> PartialVMResult<()> {
    let mut locals = Locals::new(1);
    let v1 = dummy_resource();
    let v2 = dummy_resource();
    locals.store_loc(0, v1)?;
    assert!(locals.store_loc(0, v2).is_err());
    Ok(())
}

#[test]
fn locals_check_resources() -> PartialVMResult<()> {
    let mut locals = Locals::new(1);
    locals.store_loc(0, dummy_resource())?;
    assert!(locals.check_resources_for_return().is_err());
    Ok(())
}

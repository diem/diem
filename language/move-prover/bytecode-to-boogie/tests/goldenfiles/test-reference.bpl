

// everything below is auto generated

const unique TestReference_T: TypeName;
const unique TestReference_T_value: FieldName;

procedure {:inline 1} Pack_TestReference_T(v0: Value) returns (v: Value)
{
    assert is#Integer(v0);
    v := Map(DefaultMap[Field(TestReference_T_value) := v0]);
}

procedure {:inline 1} Unpack_TestReference_T(v: Value) returns (v0: Value)
{
    assert is#Map(v);
    v0 := m#Map(v)[Field(TestReference_T_value)];
}

procedure {:inline 1} Eq_TestReference_T(v1: Value, v2: Value) returns (res: Value)
{
    var b0: Value;
    assert is#Map(v1) && is#Map(v2);
    call b0 := Eq_int(m#Map(v1)[Field(TestReference_T_value)], m#Map(v2)[Field(TestReference_T_value)]);
    res := Boolean(true && b#Boolean(b0));
}

procedure {:inline 1} Neq_TestReference_T(v1: Value, v2: Value) returns (res: Value)
{
    var res_val: Value;
    var res_bool: bool;
    assert is#Map(v1) && is#Map(v2);
    call res_val := Eq_TestReference_T(v1, v2);
    res := Boolean(!b#Boolean(res_val));
}

procedure {:inline 1} ReadValue0(p: Path, i: int, v: Value) returns (v': Value)
{
    var e: Edge;
    if (i == size#Path(p)) {
        v' := v;
    } else {
        assert false;
    }
}

procedure {:inline 1} ReadValue1(p: Path, i: int, v: Value) returns (v': Value)
{
    var e: Edge;
    if (i == size#Path(p)) {
        v' := v;
    } else {
        e := p#Path(p)[i];
        v' := m#Map(v)[e];
        if (is#Vector(v)) { v' := v#Vector(v)[e]; }
        call v' := ReadValue0(p, i+1, v');
    }
}

procedure {:inline 1} ReadValue2(p: Path, i: int, v: Value) returns (v': Value)
{
    var e: Edge;
    if (i == size#Path(p)) {
        v' := v;
    } else {
        e := p#Path(p)[i];
        v' := m#Map(v)[e];
        if (is#Vector(v)) { v' := v#Vector(v)[e]; }
        call v' := ReadValue1(p, i+1, v');
    }
}

procedure {:inline 1} ReadValueMax(p: Path, i: int, v: Value) returns (v': Value)
{
    var e: Edge;
    if (i == size#Path(p)) {
        v' := v;
    } else {
        e := p#Path(p)[i];
        v' := m#Map(v)[e];
        if (is#Vector(v)) { v' := v#Vector(v)[e]; }
        call v' := ReadValue2(p, i+1, v');
    }
}

procedure {:inline 1} UpdateValue0(p: Path, i: int, v: Value, new_v: Value) returns (v': Value)
{
    var e: Edge;
    if (i == size#Path(p)) {
        v' := new_v;
    } else {
        assert false;
    }
}

procedure {:inline 1} UpdateValue1(p: Path, i: int, v: Value, new_v: Value) returns (v': Value)
{
    var e: Edge;
    if (i == size#Path(p)) {
        v' := new_v;
    } else {
        e := p#Path(p)[i];
        v' := m#Map(v)[e];
        if (is#Vector(v)) { v' := v#Vector(v)[e]; }
        call v' := UpdateValue0(p, i+1, v', new_v);
        if (is#Map(v)) { v' := Map(m#Map(v)[e := v']);}
        if (is#Vector(v)) { v' := Vector(v#Vector(v)[e := v'], l#Vector(v));}
    }
}

procedure {:inline 1} UpdateValue2(p: Path, i: int, v: Value, new_v: Value) returns (v': Value)
{
    var e: Edge;
    if (i == size#Path(p)) {
        v' := new_v;
    } else {
        e := p#Path(p)[i];
        v' := m#Map(v)[e];
        if (is#Vector(v)) { v' := v#Vector(v)[e]; }
        call v' := UpdateValue1(p, i+1, v', new_v);
        if (is#Map(v)) { v' := Map(m#Map(v)[e := v']);}
        if (is#Vector(v)) { v' := Vector(v#Vector(v)[e := v'], l#Vector(v));}
    }
}

procedure {:inline 1} UpdateValueMax(p: Path, i: int, v: Value, new_v: Value) returns (v': Value)
{
    var e: Edge;
    if (i == size#Path(p)) {
        v' := new_v;
    } else {
        e := p#Path(p)[i];
        v' := m#Map(v)[e];
        if (is#Vector(v)) { v' := v#Vector(v)[e]; }
        call v' := UpdateValue2(p, i+1, v', new_v);
        if (is#Map(v)) { v' := Map(m#Map(v)[e := v']);}
        if (is#Vector(v)) { v' := Vector(v#Vector(v)[e := v'], l#Vector(v));}
    }
}

procedure {:inline 1} TestReference_mut_b (arg0: Reference) returns ()
{
    // declare local variables
    var t0: Reference; // int_ref
    var t1: Value; // int
    var t2: Reference; // int_ref

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 3;
    t0 := arg0;

    // bytecode translation starts here
    call tmp := LdConst(10);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call t2 := CopyOrMoveRef(t0);

    call WriteRef(t2, contents#Memory(m)[old_size+1]);

    return;

}

procedure TestReference_mut_b_verify (arg0: Reference) returns ()
{
    call TestReference_mut_b(arg0);
}

procedure {:inline 1} TestReference_mut_ref () returns ()
{
    // declare local variables
    var t0: Value; // int
    var t1: Reference; // int_ref
    var t2: Value; // int
    var t3: Reference; // int_ref
    var t4: Reference; // int_ref
    var t5: Reference; // int_ref
    var t6: Value; // int
    var t7: Value; // int
    var t8: Value; // int
    var t9: Value; // bool
    var t10: Value; // bool
    var t11: Value; // int

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 12;

    // bytecode translation starts here
    call tmp := LdConst(20);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size := tmp]);

    call t3 := BorrowLoc(old_size+0);

    call t1 := CopyOrMoveRef(t3);

    call t4 := CopyOrMoveRef(t1);

    call TestReference_mut_b(t4);

    call t5 := CopyOrMoveRef(t1);

    call tmp := ReadRef(t5);
    assume is#Integer(tmp);

    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+6]);
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    call tmp := LdConst(10);
    m := Memory(domain#Memory(m)[8+old_size := true], contents#Memory(m)[8+old_size := tmp]);

    call tmp := Eq(contents#Memory(m)[old_size+7], contents#Memory(m)[old_size+8]);
    m := Memory(domain#Memory(m)[9+old_size := true], contents#Memory(m)[9+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+9]);
    m := Memory(domain#Memory(m)[10+old_size := true], contents#Memory(m)[10+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 10];
if (!b#Boolean(tmp)) { goto Label_16; }

    call tmp := LdConst(42);
    m := Memory(domain#Memory(m)[11+old_size := true], contents#Memory(m)[11+old_size := tmp]);

    assert false;

Label_16:
    return;

}

procedure TestReference_mut_ref_verify () returns ()
{
    call TestReference_mut_ref();
}

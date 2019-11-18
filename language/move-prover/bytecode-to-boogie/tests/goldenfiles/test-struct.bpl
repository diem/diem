

// everything below is auto generated

const unique TestStruct_B: TypeName;
const unique TestStruct_B_addr: FieldName;
const unique TestStruct_B_val: FieldName;

procedure {:inline 1} Pack_TestStruct_B(v0: Value, v1: Value) returns (v: Value)
{
    assert is#Address(v0);
    assert is#Integer(v1);
    v := Map(DefaultMap[Field(TestStruct_B_addr) := v0][Field(TestStruct_B_val) := v1]);
}

procedure {:inline 1} Unpack_TestStruct_B(v: Value) returns (v0: Value, v1: Value)
{
    assert is#Map(v);
    v0 := m#Map(v)[Field(TestStruct_B_addr)];
    v1 := m#Map(v)[Field(TestStruct_B_val)];
}

procedure {:inline 1} Eq_TestStruct_B(v1: Value, v2: Value) returns (res: Value)
{
    var b0: Value;
    var b1: Value;
    assert is#Map(v1) && is#Map(v2);
    call b0 := Eq_address(m#Map(v1)[Field(TestStruct_B_addr)], m#Map(v2)[Field(TestStruct_B_addr)]);
    call b1 := Eq_int(m#Map(v1)[Field(TestStruct_B_val)], m#Map(v2)[Field(TestStruct_B_val)]);
    res := Boolean(true && b#Boolean(b0) && b#Boolean(b1));
}

procedure {:inline 1} Neq_TestStruct_B(v1: Value, v2: Value) returns (res: Value)
{
    var res_val: Value;
    var res_bool: bool;
    assert is#Map(v1) && is#Map(v2);
    call res_val := Eq_TestStruct_B(v1, v2);
    res := Boolean(!b#Boolean(res_val));
}

const unique TestStruct_A: TypeName;
const unique TestStruct_A_b: FieldName;
const unique TestStruct_A_val: FieldName;

procedure {:inline 1} Pack_TestStruct_A(v0: Value, v1: Value) returns (v: Value)
{
    assert is#Map(v0);
    assert is#Integer(v1);
    v := Map(DefaultMap[Field(TestStruct_A_b) := v0][Field(TestStruct_A_val) := v1]);
}

procedure {:inline 1} Unpack_TestStruct_A(v: Value) returns (v0: Value, v1: Value)
{
    assert is#Map(v);
    v0 := m#Map(v)[Field(TestStruct_A_b)];
    v1 := m#Map(v)[Field(TestStruct_A_val)];
}

procedure {:inline 1} Eq_TestStruct_A(v1: Value, v2: Value) returns (res: Value)
{
    var b0: Value;
    var b1: Value;
    assert is#Map(v1) && is#Map(v2);
    call b0 := Eq_TestStruct_B(m#Map(v1)[Field(TestStruct_A_b)], m#Map(v2)[Field(TestStruct_A_b)]);
    call b1 := Eq_int(m#Map(v1)[Field(TestStruct_A_val)], m#Map(v2)[Field(TestStruct_A_val)]);
    res := Boolean(true && b#Boolean(b0) && b#Boolean(b1));
}

procedure {:inline 1} Neq_TestStruct_A(v1: Value, v2: Value) returns (res: Value)
{
    var res_val: Value;
    var res_bool: bool;
    assert is#Map(v1) && is#Map(v2);
    call res_val := Eq_TestStruct_A(v1, v2);
    res := Boolean(!b#Boolean(res_val));
}

const unique TestStruct_C: TypeName;
const unique TestStruct_C_b: FieldName;
const unique TestStruct_C_val: FieldName;

procedure {:inline 1} Pack_TestStruct_C(v0: Value, v1: Value) returns (v: Value)
{
    assert is#Map(v0);
    assert is#Integer(v1);
    v := Map(DefaultMap[Field(TestStruct_C_b) := v0][Field(TestStruct_C_val) := v1]);
}

procedure {:inline 1} Unpack_TestStruct_C(v: Value) returns (v0: Value, v1: Value)
{
    assert is#Map(v);
    v0 := m#Map(v)[Field(TestStruct_C_b)];
    v1 := m#Map(v)[Field(TestStruct_C_val)];
}

procedure {:inline 1} Eq_TestStruct_C(v1: Value, v2: Value) returns (res: Value)
{
    var b0: Value;
    var b1: Value;
    assert is#Map(v1) && is#Map(v2);
    call b0 := Eq_TestStruct_A(m#Map(v1)[Field(TestStruct_C_b)], m#Map(v2)[Field(TestStruct_C_b)]);
    call b1 := Eq_int(m#Map(v1)[Field(TestStruct_C_val)], m#Map(v2)[Field(TestStruct_C_val)]);
    res := Boolean(true && b#Boolean(b0) && b#Boolean(b1));
}

procedure {:inline 1} Neq_TestStruct_C(v1: Value, v2: Value) returns (res: Value)
{
    var res_val: Value;
    var res_bool: bool;
    assert is#Map(v1) && is#Map(v2);
    call res_val := Eq_TestStruct_C(v1, v2);
    res := Boolean(!b#Boolean(res_val));
}

const unique TestStruct_T: TypeName;
const unique TestStruct_T_x: FieldName;

procedure {:inline 1} Pack_TestStruct_T(v0: Value) returns (v: Value)
{
    assert is#Integer(v0);
    v := Map(DefaultMap[Field(TestStruct_T_x) := v0]);
}

procedure {:inline 1} Unpack_TestStruct_T(v: Value) returns (v0: Value)
{
    assert is#Map(v);
    v0 := m#Map(v)[Field(TestStruct_T_x)];
}

procedure {:inline 1} Eq_TestStruct_T(v1: Value, v2: Value) returns (res: Value)
{
    var b0: Value;
    assert is#Map(v1) && is#Map(v2);
    call b0 := Eq_int(m#Map(v1)[Field(TestStruct_T_x)], m#Map(v2)[Field(TestStruct_T_x)]);
    res := Boolean(true && b#Boolean(b0));
}

procedure {:inline 1} Neq_TestStruct_T(v1: Value, v2: Value) returns (res: Value)
{
    var res_val: Value;
    var res_bool: bool;
    assert is#Map(v1) && is#Map(v2);
    call res_val := Eq_TestStruct_T(v1, v2);
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

procedure {:inline 1} ReadValue3(p: Path, i: int, v: Value) returns (v': Value)
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

procedure {:inline 1} ReadValueMax(p: Path, i: int, v: Value) returns (v': Value)
{
    var e: Edge;
    if (i == size#Path(p)) {
        v' := v;
    } else {
        e := p#Path(p)[i];
        v' := m#Map(v)[e];
        if (is#Vector(v)) { v' := v#Vector(v)[e]; }
        call v' := ReadValue3(p, i+1, v');
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

procedure {:inline 1} UpdateValue3(p: Path, i: int, v: Value, new_v: Value) returns (v': Value)
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

procedure {:inline 1} UpdateValueMax(p: Path, i: int, v: Value, new_v: Value) returns (v': Value)
{
    var e: Edge;
    if (i == size#Path(p)) {
        v' := new_v;
    } else {
        e := p#Path(p)[i];
        v' := m#Map(v)[e];
        if (is#Vector(v)) { v' := v#Vector(v)[e]; }
        call v' := UpdateValue3(p, i+1, v', new_v);
        if (is#Map(v)) { v' := Map(m#Map(v)[e := v']);}
        if (is#Vector(v)) { v' := Vector(v#Vector(v)[e := v'], l#Vector(v));}
    }
}

procedure {:inline 1} TestStruct_identity (arg0: Value, arg1: Value) returns (ret0: Value, ret1: Value)
{
    // declare local variables
    var t0: Value; // TestStruct_A
    var t1: Value; // TestStruct_C
    var t2: Value; // TestStruct_A
    var t3: Value; // TestStruct_C

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Map(arg0);
    assume is#Map(arg1);

    old_size := m_size;
    m_size := m_size + 4;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+2];
    ret1 := contents#Memory(m)[old_size+3];
    return;

}

procedure TestStruct_identity_verify (arg0: Value, arg1: Value) returns (ret0: Value, ret1: Value)
{
    call ret0, ret1 := TestStruct_identity(arg0, arg1);
}

procedure {:inline 1} TestStruct_module_builtins (arg0: Value) returns (ret0: Value)
{
    // declare local variables
    var t0: Value; // address
    var t1: Value; // TestStruct_T
    var t2: Reference; // TestStruct_T_ref
    var t3: Reference; // TestStruct_T_ref
    var t4: Value; // bool
    var t5: Value; // address
    var t6: Value; // bool
    var t7: Value; // bool
    var t8: Value; // bool
    var t9: Value; // int
    var t10: Value; // address
    var t11: Reference; // TestStruct_T_ref
    var t12: Reference; // TestStruct_T_ref
    var t13: Value; // address
    var t14: Reference; // TestStruct_T_ref
    var t15: Reference; // TestStruct_T_ref
    var t16: Value; // address
    var t17: Value; // TestStruct_T
    var t18: Value; // TestStruct_T
    var t19: Value; // bool

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Address(arg0);

    old_size := m_size;
    m_size := m_size + 20;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call tmp := Exists(contents#Memory(m)[old_size+5], TestStruct_T);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+6]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+4]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+7]);
    m := Memory(domain#Memory(m)[8+old_size := true], contents#Memory(m)[8+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 8];
if (!b#Boolean(tmp)) { goto Label_8; }

    call tmp := LdConst(42);
    m := Memory(domain#Memory(m)[9+old_size := true], contents#Memory(m)[9+old_size := tmp]);

    assert false;

Label_8:
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[10+old_size := true], contents#Memory(m)[10+old_size := tmp]);

    call t11 := BorrowGlobal(contents#Memory(m)[old_size+10], TestStruct_T);

    call t2 := CopyOrMoveRef(t11);

    call t12 := CopyOrMoveRef(t2);

    // unimplemented instruction

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[13+old_size := true], contents#Memory(m)[13+old_size := tmp]);

    call t14 := BorrowGlobal(contents#Memory(m)[old_size+13], TestStruct_T);

    call t3 := CopyOrMoveRef(t14);

    call t15 := CopyOrMoveRef(t3);

    // unimplemented instruction

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[16+old_size := true], contents#Memory(m)[16+old_size := tmp]);

    call tmp := MoveFrom(contents#Memory(m)[old_size+16], TestStruct_T);
    m := Memory(domain#Memory(m)[17+old_size := true], contents#Memory(m)[17+old_size := tmp]);
    assume is#Map(t17);


    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+17]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[18+old_size := true], contents#Memory(m)[18+old_size := tmp]);

    call MoveToSender(TestStruct_T, contents#Memory(m)[old_size+18]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+4]);
    m := Memory(domain#Memory(m)[19+old_size := true], contents#Memory(m)[19+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+19];
    return;

}

procedure TestStruct_module_builtins_verify (arg0: Value) returns (ret0: Value)
{
    call ret0 := TestStruct_module_builtins(arg0);
}

procedure {:inline 1} TestStruct_nested_struct (arg0: Value) returns (ret0: Value)
{
    // declare local variables
    var t0: Value; // address
    var t1: Value; // TestStruct_A
    var t2: Value; // TestStruct_B
    var t3: Reference; // TestStruct_B_ref
    var t4: Reference; // int_ref
    var t5: Value; // int
    var t6: Value; // bool
    var t7: Value; // address
    var t8: Value; // int
    var t9: Value; // TestStruct_B
    var t10: Value; // address
    var t11: Value; // int
    var t12: Value; // TestStruct_B
    var t13: Reference; // TestStruct_B_ref
    var t14: Reference; // TestStruct_B_ref
    var t15: Reference; // int_ref
    var t16: Reference; // int_ref
    var t17: Value; // int
    var t18: Value; // int
    var t19: Value; // int
    var t20: Value; // bool
    var t21: Value; // bool
    var t22: Value; // int
    var t23: Value; // TestStruct_B

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Address(arg0);

    old_size := m_size;
    m_size := m_size + 24;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := LdFalse();
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 6];
if (!b#Boolean(tmp)) { goto Label_7; }

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[8+old_size := true], contents#Memory(m)[8+old_size := tmp]);

    assume is#Address(contents#Memory(m)[old_size+7]);

    assume is#Integer(contents#Memory(m)[old_size+8]);

    call tmp := Pack_TestStruct_B(contents#Memory(m)[old_size+7], contents#Memory(m)[old_size+8]);
    m := Memory(domain#Memory(m)[9+old_size := true], contents#Memory(m)[9+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+9]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    goto Label_11;

Label_7:
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[10+old_size := true], contents#Memory(m)[10+old_size := tmp]);

    call tmp := LdConst(42);
    m := Memory(domain#Memory(m)[11+old_size := true], contents#Memory(m)[11+old_size := tmp]);

    assume is#Address(contents#Memory(m)[old_size+10]);

    assume is#Integer(contents#Memory(m)[old_size+11]);

    call tmp := Pack_TestStruct_B(contents#Memory(m)[old_size+10], contents#Memory(m)[old_size+11]);
    m := Memory(domain#Memory(m)[12+old_size := true], contents#Memory(m)[12+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+12]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

Label_11:
    call t13 := BorrowLoc(old_size+2);

    call t3 := CopyOrMoveRef(t13);

    call t14 := CopyOrMoveRef(t3);

    call t15 := BorrowField(t14, TestStruct_B_val);

    call t4 := CopyOrMoveRef(t15);

    call t16 := CopyOrMoveRef(t4);

    call tmp := ReadRef(t16);
    assume is#Integer(tmp);

    m := Memory(domain#Memory(m)[17+old_size := true], contents#Memory(m)[17+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+17]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[18+old_size := true], contents#Memory(m)[18+old_size := tmp]);

    call tmp := LdConst(42);
    m := Memory(domain#Memory(m)[19+old_size := true], contents#Memory(m)[19+old_size := tmp]);

    call tmp := Eq(contents#Memory(m)[old_size+18], contents#Memory(m)[old_size+19]);
    m := Memory(domain#Memory(m)[20+old_size := true], contents#Memory(m)[20+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+20]);
    m := Memory(domain#Memory(m)[21+old_size := true], contents#Memory(m)[21+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 21];
if (!b#Boolean(tmp)) { goto Label_26; }

    call tmp := LdConst(42);
    m := Memory(domain#Memory(m)[22+old_size := true], contents#Memory(m)[22+old_size := tmp]);

    assert false;

Label_26:
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[23+old_size := true], contents#Memory(m)[23+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+23];
    return;

}

procedure TestStruct_nested_struct_verify (arg0: Value) returns (ret0: Value)
{
    call ret0 := TestStruct_nested_struct(arg0);
}

procedure {:inline 1} TestStruct_try_unpack (arg0: Value) returns (ret0: Value)
{
    // declare local variables
    var t0: Value; // address
    var t1: Value; // int
    var t2: Value; // TestStruct_B
    var t3: Value; // address
    var t4: Value; // address
    var t5: Value; // int
    var t6: Value; // TestStruct_B
    var t7: Value; // TestStruct_B
    var t8: Value; // address
    var t9: Value; // int
    var t10: Value; // address
    var t11: Value; // address
    var t12: Value; // bool
    var t13: Value; // bool
    var t14: Value; // int
    var t15: Value; // int

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Address(arg0);

    old_size := m_size;
    m_size := m_size + 16;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call tmp := LdConst(42);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    assume is#Address(contents#Memory(m)[old_size+4]);

    assume is#Integer(contents#Memory(m)[old_size+5]);

    call tmp := Pack_TestStruct_B(contents#Memory(m)[old_size+4], contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+6]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    call t8, t9 := Unpack_TestStruct_B(contents#Memory(m)[old_size+7]);
    assume is#Address(t8);

    assume is#Integer(t9);

    m := Memory(domain#Memory(m)[old_size+8 := true], contents#Memory(m)[old_size+8 := t8]);
    m := Memory(domain#Memory(m)[old_size+9 := true], contents#Memory(m)[old_size+9 := t9]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+9]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+8]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[10+old_size := true], contents#Memory(m)[10+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[11+old_size := true], contents#Memory(m)[11+old_size := tmp]);

    call tmp := Eq(contents#Memory(m)[old_size+10], contents#Memory(m)[old_size+11]);
    m := Memory(domain#Memory(m)[12+old_size := true], contents#Memory(m)[12+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+12]);
    m := Memory(domain#Memory(m)[13+old_size := true], contents#Memory(m)[13+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 13];
if (!b#Boolean(tmp)) { goto Label_15; }

    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[14+old_size := true], contents#Memory(m)[14+old_size := tmp]);

    assert false;

Label_15:
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[15+old_size := true], contents#Memory(m)[15+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+15];
    return;

}

procedure TestStruct_try_unpack_verify (arg0: Value) returns (ret0: Value)
{
    call ret0 := TestStruct_try_unpack(arg0);
}



// everything below is auto generated

const unique Test3_T: TypeName;
const unique Test3_T_f: FieldName;
const unique Test3_T_g: FieldName;

procedure {:inline 1} Pack_Test3_T(v0: Value, v1: Value) returns (v: Value)
{
    assert is#Integer(v0);
    assert is#Integer(v1);
    v := Map(DefaultMap[Field(Test3_T_f) := v0][Field(Test3_T_g) := v1]);
}

procedure {:inline 1} Unpack_Test3_T(v: Value) returns (v0: Value, v1: Value)
{
    assert is#Map(v);
    v0 := m#Map(v)[Field(Test3_T_f)];
    v1 := m#Map(v)[Field(Test3_T_g)];
}

procedure {:inline 1} Eq_Test3_T(v1: Value, v2: Value) returns (res: Value)
{
    var b0: Value;
    var b1: Value;
    assert is#Map(v1) && is#Map(v2);
    call b0 := Eq_int(m#Map(v1)[Field(Test3_T_f)], m#Map(v2)[Field(Test3_T_f)]);
    call b1 := Eq_int(m#Map(v1)[Field(Test3_T_g)], m#Map(v2)[Field(Test3_T_g)]);
    res := Boolean(true && b#Boolean(b0) && b#Boolean(b1));
}

procedure {:inline 1} Neq_Test3_T(v1: Value, v2: Value) returns (res: Value)
{
    var res_val: Value;
    var res_bool: bool;
    assert is#Map(v1) && is#Map(v2);
    call res_val := Eq_Test3_T(v1, v2);
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

procedure {:inline 1} Test3_test3 (arg0: Value) returns ()
{
    // declare local variables
    var t0: Value; // bool
    var t1: Value; // Test3_T
    var t2: Reference; // Test3_T_ref
    var t3: Reference; // int_ref
    var t4: Reference; // int_ref
    var t5: Reference; // int_ref
    var t6: Reference; // int_ref
    var t7: Value; // int
    var t8: Value; // int
    var t9: Value; // int
    var t10: Value; // int
    var t11: Value; // Test3_T
    var t12: Reference; // Test3_T_ref
    var t13: Value; // bool
    var t14: Reference; // Test3_T_ref
    var t15: Reference; // int_ref
    var t16: Reference; // Test3_T_ref
    var t17: Reference; // int_ref
    var t18: Value; // int
    var t19: Reference; // int_ref
    var t20: Value; // bool
    var t21: Value; // bool
    var t22: Reference; // Test3_T_ref
    var t23: Reference; // int_ref
    var t24: Reference; // Test3_T_ref
    var t25: Reference; // int_ref
    var t26: Value; // int
    var t27: Reference; // int_ref
    var t28: Reference; // Test3_T_ref
    var t29: Reference; // int_ref
    var t30: Reference; // Test3_T_ref
    var t31: Reference; // int_ref
    var t32: Reference; // int_ref
    var t33: Value; // int
    var t34: Reference; // int_ref
    var t35: Value; // int
    var t36: Value; // bool
    var t37: Value; // int
    var t38: Value; // int
    var t39: Value; // bool
    var t40: Value; // bool
    var t41: Value; // int
    var t42: Value; // int
    var t43: Value; // int
    var t44: Value; // bool
    var t45: Value; // bool
    var t46: Value; // int
    var t47: Value; // int
    var t48: Value; // int
    var t49: Value; // bool
    var t50: Value; // bool
    var t51: Value; // int
    var t52: Value; // int
    var t53: Value; // int
    var t54: Value; // bool
    var t55: Value; // bool
    var t56: Value; // int

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume is#Boolean(arg0);

    old_size := m_size;
    m_size := m_size + 57;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[9+old_size := true], contents#Memory(m)[9+old_size := tmp]);

    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[10+old_size := true], contents#Memory(m)[10+old_size := tmp]);

    assume is#Integer(contents#Memory(m)[old_size+9]);

    assume is#Integer(contents#Memory(m)[old_size+10]);

    call tmp := Pack_Test3_T(contents#Memory(m)[old_size+9], contents#Memory(m)[old_size+10]);
    m := Memory(domain#Memory(m)[11+old_size := true], contents#Memory(m)[11+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+11]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call t12 := BorrowLoc(old_size+1);

    call t2 := CopyOrMoveRef(t12);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[13+old_size := true], contents#Memory(m)[13+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 13];
if (!b#Boolean(tmp)) { goto Label_12; }

    call t14 := CopyOrMoveRef(t2);

    call t15 := BorrowField(t14, Test3_T_f);

    call t3 := CopyOrMoveRef(t15);

    goto Label_15;

Label_12:
    call t16 := CopyOrMoveRef(t2);

    call t17 := BorrowField(t16, Test3_T_g);

    call t3 := CopyOrMoveRef(t17);

Label_15:
    call tmp := LdConst(10);
    m := Memory(domain#Memory(m)[18+old_size := true], contents#Memory(m)[18+old_size := tmp]);

    call t19 := CopyOrMoveRef(t3);

    call WriteRef(t19, contents#Memory(m)[old_size+18]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[20+old_size := true], contents#Memory(m)[20+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+20]);
    m := Memory(domain#Memory(m)[21+old_size := true], contents#Memory(m)[21+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 21];
if (!b#Boolean(tmp)) { goto Label_25; }

    call t22 := CopyOrMoveRef(t2);

    call t23 := BorrowField(t22, Test3_T_f);

    call t4 := CopyOrMoveRef(t23);

    goto Label_28;

Label_25:
    call t24 := CopyOrMoveRef(t2);

    call t25 := BorrowField(t24, Test3_T_g);

    call t4 := CopyOrMoveRef(t25);

Label_28:
    call tmp := LdConst(20);
    m := Memory(domain#Memory(m)[26+old_size := true], contents#Memory(m)[26+old_size := tmp]);

    call t27 := CopyOrMoveRef(t4);

    call WriteRef(t27, contents#Memory(m)[old_size+26]);

    call t28 := CopyOrMoveRef(t2);

    call t29 := BorrowField(t28, Test3_T_f);

    call t5 := CopyOrMoveRef(t29);

    call t30 := CopyOrMoveRef(t2);

    call t31 := BorrowField(t30, Test3_T_g);

    call t6 := CopyOrMoveRef(t31);

    call t32 := CopyOrMoveRef(t5);

    call tmp := ReadRef(t32);
    assume is#Integer(tmp);

    m := Memory(domain#Memory(m)[33+old_size := true], contents#Memory(m)[33+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+33]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    call t34 := CopyOrMoveRef(t6);

    call tmp := ReadRef(t34);
    assume is#Integer(tmp);

    m := Memory(domain#Memory(m)[35+old_size := true], contents#Memory(m)[35+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+35]);
    m := Memory(domain#Memory(m)[8+old_size := true], contents#Memory(m)[8+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[36+old_size := true], contents#Memory(m)[36+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 36];
if (!b#Boolean(tmp)) { goto Label_60; }

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+7]);
    m := Memory(domain#Memory(m)[37+old_size := true], contents#Memory(m)[37+old_size := tmp]);

    call tmp := LdConst(10);
    m := Memory(domain#Memory(m)[38+old_size := true], contents#Memory(m)[38+old_size := tmp]);

    call tmp := Eq(contents#Memory(m)[old_size+37], contents#Memory(m)[old_size+38]);
    m := Memory(domain#Memory(m)[39+old_size := true], contents#Memory(m)[39+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+39]);
    m := Memory(domain#Memory(m)[40+old_size := true], contents#Memory(m)[40+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 40];
if (!b#Boolean(tmp)) { goto Label_52; }

    call tmp := LdConst(42);
    m := Memory(domain#Memory(m)[41+old_size := true], contents#Memory(m)[41+old_size := tmp]);

    assert false;

Label_52:
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+8]);
    m := Memory(domain#Memory(m)[42+old_size := true], contents#Memory(m)[42+old_size := tmp]);

    call tmp := LdConst(20);
    m := Memory(domain#Memory(m)[43+old_size := true], contents#Memory(m)[43+old_size := tmp]);

    call tmp := Eq(contents#Memory(m)[old_size+42], contents#Memory(m)[old_size+43]);
    m := Memory(domain#Memory(m)[44+old_size := true], contents#Memory(m)[44+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+44]);
    m := Memory(domain#Memory(m)[45+old_size := true], contents#Memory(m)[45+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 45];
if (!b#Boolean(tmp)) { goto Label_59; }

    call tmp := LdConst(42);
    m := Memory(domain#Memory(m)[46+old_size := true], contents#Memory(m)[46+old_size := tmp]);

    assert false;

Label_59:
    goto Label_74;

Label_60:
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+7]);
    m := Memory(domain#Memory(m)[47+old_size := true], contents#Memory(m)[47+old_size := tmp]);

    call tmp := LdConst(20);
    m := Memory(domain#Memory(m)[48+old_size := true], contents#Memory(m)[48+old_size := tmp]);

    call tmp := Eq(contents#Memory(m)[old_size+47], contents#Memory(m)[old_size+48]);
    m := Memory(domain#Memory(m)[49+old_size := true], contents#Memory(m)[49+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+49]);
    m := Memory(domain#Memory(m)[50+old_size := true], contents#Memory(m)[50+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 50];
if (!b#Boolean(tmp)) { goto Label_67; }

    call tmp := LdConst(42);
    m := Memory(domain#Memory(m)[51+old_size := true], contents#Memory(m)[51+old_size := tmp]);

    assert false;

Label_67:
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+8]);
    m := Memory(domain#Memory(m)[52+old_size := true], contents#Memory(m)[52+old_size := tmp]);

    call tmp := LdConst(10);
    m := Memory(domain#Memory(m)[53+old_size := true], contents#Memory(m)[53+old_size := tmp]);

    call tmp := Eq(contents#Memory(m)[old_size+52], contents#Memory(m)[old_size+53]);
    m := Memory(domain#Memory(m)[54+old_size := true], contents#Memory(m)[54+old_size := tmp]);

    call tmp := Not(contents#Memory(m)[old_size+54]);
    m := Memory(domain#Memory(m)[55+old_size := true], contents#Memory(m)[55+old_size := tmp]);

    tmp := contents#Memory(m)[old_size + 55];
if (!b#Boolean(tmp)) { goto Label_74; }

    call tmp := LdConst(42);
    m := Memory(domain#Memory(m)[56+old_size := true], contents#Memory(m)[56+old_size := tmp]);

    assert false;

Label_74:
    return;

}

procedure Test3_test3_verify (arg0: Value) returns ()
{
    call Test3_test3(arg0);
}

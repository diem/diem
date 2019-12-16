

// ** structs of module Test3

const unique Test3_T: TypeName;
const Test3_T_f: FieldName;
axiom Test3_T_f == 0;
const Test3_T_g: FieldName;
axiom Test3_T_g == 1;
function Test3_T_type_value(): TypeValue {
    StructType(Test3_T, TypeValueArray(DefaultTypeMap[0 := IntegerType()][1 := IntegerType()], 2))
}

procedure {:inline 1} Pack_Test3_T(v0: Value, v1: Value) returns (v: Value)
{
    assume has_type(IntegerType(), v0);
    assume has_type(IntegerType(), v1);
    v := Struct(ValueArray(DefaultIntMap[Test3_T_f := v0][Test3_T_g := v1], 2));
    assume has_type(Test3_T_type_value(), v);
}

procedure {:inline 1} Unpack_Test3_T(v: Value) returns (v0: Value, v1: Value)
{
    assert is#Struct(v);
    v0 := smap(v)[Test3_T_f];
    v1 := smap(v)[Test3_T_g];
}



// ** stratified functions

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
        if (is#Struct(v)) { v' := smap(v)[f#Field(e)]; }
        if (is#Vector(v)) { v' := vmap(v)[i#Index(e)]; }
        call v' := ReadValue0(p, i+1, v');
    }
}

procedure {:inline 1} ReadValueMax(p: Path, i: int, v: Value) returns (v': Value)
{
    var e: Edge;
    if (i == size#Path(p)) {
        v' := v;
    } else {
        e := p#Path(p)[i];
        if (is#Struct(v)) { v' := smap(v)[f#Field(e)]; }
        if (is#Vector(v)) { v' := vmap(v)[i#Index(e)]; }
        call v' := ReadValue1(p, i+1, v');
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
        if (is#Struct(v)) { v' := smap(v)[f#Field(e)]; }
        if (is#Vector(v)) { v' := vmap(v)[i#Index(e)]; }
        call v' := UpdateValue0(p, i+1, v', new_v);
        if (is#Struct(v)) { v' := mk_struct(smap(v)[f#Field(e) := v'], slen(v));}
        if (is#Vector(v)) { v' := mk_vector(vmap(v)[i#Index(e) := v'], vlen(v));}
    }
}

procedure {:inline 1} UpdateValueMax(p: Path, i: int, v: Value, new_v: Value) returns (v': Value)
{
    var e: Edge;
    if (i == size#Path(p)) {
        v' := new_v;
    } else {
        e := p#Path(p)[i];
        if (is#Struct(v)) { v' := smap(v)[f#Field(e)]; }
        if (is#Vector(v)) { v' := vmap(v)[i#Index(e)]; }
        call v' := UpdateValue1(p, i+1, v', new_v);
        if (is#Struct(v)) { v' := mk_struct(smap(v)[f#Field(e) := v'], slen(v));}
        if (is#Vector(v)) { v' := mk_vector(vmap(v)[i#Index(e) := v'], vlen(v));}
    }
}



// ** functions of module Test3

procedure {:inline 1} Test3_test3 (arg0: Value) returns ()
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // BooleanType()
    var t1: Value; // Test3_T_type_value()
    var t2: Reference; // ReferenceType(Test3_T_type_value())
    var t3: Reference; // ReferenceType(IntegerType())
    var t4: Reference; // ReferenceType(IntegerType())
    var t5: Reference; // ReferenceType(IntegerType())
    var t6: Reference; // ReferenceType(IntegerType())
    var t7: Value; // IntegerType()
    var t8: Value; // IntegerType()
    var t9: Value; // IntegerType()
    var t10: Value; // IntegerType()
    var t11: Value; // Test3_T_type_value()
    var t12: Reference; // ReferenceType(Test3_T_type_value())
    var t13: Value; // BooleanType()
    var t14: Reference; // ReferenceType(Test3_T_type_value())
    var t15: Reference; // ReferenceType(IntegerType())
    var t16: Reference; // ReferenceType(Test3_T_type_value())
    var t17: Reference; // ReferenceType(IntegerType())
    var t18: Value; // IntegerType()
    var t19: Reference; // ReferenceType(IntegerType())
    var t20: Value; // BooleanType()
    var t21: Value; // BooleanType()
    var t22: Reference; // ReferenceType(Test3_T_type_value())
    var t23: Reference; // ReferenceType(IntegerType())
    var t24: Reference; // ReferenceType(Test3_T_type_value())
    var t25: Reference; // ReferenceType(IntegerType())
    var t26: Value; // IntegerType()
    var t27: Reference; // ReferenceType(IntegerType())
    var t28: Reference; // ReferenceType(Test3_T_type_value())
    var t29: Reference; // ReferenceType(IntegerType())
    var t30: Reference; // ReferenceType(Test3_T_type_value())
    var t31: Reference; // ReferenceType(IntegerType())
    var t32: Reference; // ReferenceType(IntegerType())
    var t33: Value; // IntegerType()
    var t34: Reference; // ReferenceType(IntegerType())
    var t35: Value; // IntegerType()
    var t36: Value; // BooleanType()
    var t37: Value; // IntegerType()
    var t38: Value; // IntegerType()
    var t39: Value; // BooleanType()
    var t40: Value; // BooleanType()
    var t41: Value; // IntegerType()
    var t42: Value; // IntegerType()
    var t43: Value; // IntegerType()
    var t44: Value; // BooleanType()
    var t45: Value; // BooleanType()
    var t46: Value; // IntegerType()
    var t47: Value; // IntegerType()
    var t48: Value; // IntegerType()
    var t49: Value; // BooleanType()
    var t50: Value; // BooleanType()
    var t51: Value; // IntegerType()
    var t52: Value; // IntegerType()
    var t53: Value; // IntegerType()
    var t54: Value; // BooleanType()
    var t55: Value; // BooleanType()
    var t56: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(BooleanType(), arg0);

    old_size := m_size;
    m_size := m_size + 57;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[9+old_size := true], contents#Memory(m)[9+old_size := tmp]);

    call tmp := LdConst(0);
    m := Memory(domain#Memory(m)[10+old_size := true], contents#Memory(m)[10+old_size := tmp]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+9]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+10]);

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
    assume has_type(IntegerType(), tmp);

    m := Memory(domain#Memory(m)[33+old_size := true], contents#Memory(m)[33+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+33]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    call t34 := CopyOrMoveRef(t6);

    call tmp := ReadRef(t34);
    assume has_type(IntegerType(), tmp);

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

    tmp := Boolean(is_equal(IntegerType(), contents#Memory(m)[old_size+37], contents#Memory(m)[old_size+38]));
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

    tmp := Boolean(is_equal(IntegerType(), contents#Memory(m)[old_size+42], contents#Memory(m)[old_size+43]));
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

    tmp := Boolean(is_equal(IntegerType(), contents#Memory(m)[old_size+47], contents#Memory(m)[old_size+48]));
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

    tmp := Boolean(is_equal(IntegerType(), contents#Memory(m)[old_size+52], contents#Memory(m)[old_size+53]));
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

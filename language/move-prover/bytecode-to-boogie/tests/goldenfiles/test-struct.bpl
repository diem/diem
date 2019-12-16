

// ** structs of module TestStruct

const unique TestStruct_B: TypeName;
const TestStruct_B_addr: FieldName;
axiom TestStruct_B_addr == 0;
const TestStruct_B_val: FieldName;
axiom TestStruct_B_val == 1;
function TestStruct_B_type_value(): TypeValue {
    StructType(TestStruct_B, TypeValueArray(DefaultTypeMap[0 := AddressType()][1 := IntegerType()], 2))
}

procedure {:inline 1} Pack_TestStruct_B(v0: Value, v1: Value) returns (v: Value)
{
    assume has_type(AddressType(), v0);
    assume has_type(IntegerType(), v1);
    v := Struct(ValueArray(DefaultIntMap[TestStruct_B_addr := v0][TestStruct_B_val := v1], 2));
    assume has_type(TestStruct_B_type_value(), v);
}

procedure {:inline 1} Unpack_TestStruct_B(v: Value) returns (v0: Value, v1: Value)
{
    assert is#Struct(v);
    v0 := smap(v)[TestStruct_B_addr];
    v1 := smap(v)[TestStruct_B_val];
}

const unique TestStruct_A: TypeName;
const TestStruct_A_val: FieldName;
axiom TestStruct_A_val == 0;
const TestStruct_A_b: FieldName;
axiom TestStruct_A_b == 1;
function TestStruct_A_type_value(): TypeValue {
    StructType(TestStruct_A, TypeValueArray(DefaultTypeMap[0 := IntegerType()][1 := TestStruct_B_type_value()], 2))
}

procedure {:inline 1} Pack_TestStruct_A(v0: Value, v1: Value) returns (v: Value)
{
    assume has_type(IntegerType(), v0);
    assume has_type(TestStruct_B_type_value(), v1);
    v := Struct(ValueArray(DefaultIntMap[TestStruct_A_val := v0][TestStruct_A_b := v1], 2));
    assume has_type(TestStruct_A_type_value(), v);
}

procedure {:inline 1} Unpack_TestStruct_A(v: Value) returns (v0: Value, v1: Value)
{
    assert is#Struct(v);
    v0 := smap(v)[TestStruct_A_val];
    v1 := smap(v)[TestStruct_A_b];
}

const unique TestStruct_C: TypeName;
const TestStruct_C_val: FieldName;
axiom TestStruct_C_val == 0;
const TestStruct_C_b: FieldName;
axiom TestStruct_C_b == 1;
function TestStruct_C_type_value(): TypeValue {
    StructType(TestStruct_C, TypeValueArray(DefaultTypeMap[0 := IntegerType()][1 := TestStruct_A_type_value()], 2))
}

procedure {:inline 1} Pack_TestStruct_C(v0: Value, v1: Value) returns (v: Value)
{
    assume has_type(IntegerType(), v0);
    assume has_type(TestStruct_A_type_value(), v1);
    v := Struct(ValueArray(DefaultIntMap[TestStruct_C_val := v0][TestStruct_C_b := v1], 2));
    assume has_type(TestStruct_C_type_value(), v);
}

procedure {:inline 1} Unpack_TestStruct_C(v: Value) returns (v0: Value, v1: Value)
{
    assert is#Struct(v);
    v0 := smap(v)[TestStruct_C_val];
    v1 := smap(v)[TestStruct_C_b];
}

const unique TestStruct_T: TypeName;
const TestStruct_T_x: FieldName;
axiom TestStruct_T_x == 0;
function TestStruct_T_type_value(): TypeValue {
    StructType(TestStruct_T, TypeValueArray(DefaultTypeMap[0 := IntegerType()], 1))
}

procedure {:inline 1} Pack_TestStruct_T(v0: Value) returns (v: Value)
{
    assume has_type(IntegerType(), v0);
    v := Struct(ValueArray(DefaultIntMap[TestStruct_T_x := v0], 1));
    assume has_type(TestStruct_T_type_value(), v);
}

procedure {:inline 1} Unpack_TestStruct_T(v: Value) returns (v0: Value)
{
    assert is#Struct(v);
    v0 := smap(v)[TestStruct_T_x];
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

procedure {:inline 1} ReadValue2(p: Path, i: int, v: Value) returns (v': Value)
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

procedure {:inline 1} ReadValue3(p: Path, i: int, v: Value) returns (v': Value)
{
    var e: Edge;
    if (i == size#Path(p)) {
        v' := v;
    } else {
        e := p#Path(p)[i];
        if (is#Struct(v)) { v' := smap(v)[f#Field(e)]; }
        if (is#Vector(v)) { v' := vmap(v)[i#Index(e)]; }
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
        if (is#Struct(v)) { v' := smap(v)[f#Field(e)]; }
        if (is#Vector(v)) { v' := vmap(v)[i#Index(e)]; }
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
        if (is#Struct(v)) { v' := smap(v)[f#Field(e)]; }
        if (is#Vector(v)) { v' := vmap(v)[i#Index(e)]; }
        call v' := UpdateValue0(p, i+1, v', new_v);
        if (is#Struct(v)) { v' := mk_struct(smap(v)[f#Field(e) := v'], slen(v));}
        if (is#Vector(v)) { v' := mk_vector(vmap(v)[i#Index(e) := v'], vlen(v));}
    }
}

procedure {:inline 1} UpdateValue2(p: Path, i: int, v: Value, new_v: Value) returns (v': Value)
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

procedure {:inline 1} UpdateValue3(p: Path, i: int, v: Value, new_v: Value) returns (v': Value)
{
    var e: Edge;
    if (i == size#Path(p)) {
        v' := new_v;
    } else {
        e := p#Path(p)[i];
        if (is#Struct(v)) { v' := smap(v)[f#Field(e)]; }
        if (is#Vector(v)) { v' := vmap(v)[i#Index(e)]; }
        call v' := UpdateValue2(p, i+1, v', new_v);
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
        call v' := UpdateValue3(p, i+1, v', new_v);
        if (is#Struct(v)) { v' := mk_struct(smap(v)[f#Field(e) := v'], slen(v));}
        if (is#Vector(v)) { v' := mk_vector(vmap(v)[i#Index(e) := v'], vlen(v));}
    }
}



// ** functions of module TestStruct

procedure {:inline 1} TestStruct_identity (arg0: Value, arg1: Value) returns (ret0: Value, ret1: Value)
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // TestStruct_A_type_value()
    var t1: Value; // TestStruct_C_type_value()
    var t2: Value; // TestStruct_A_type_value()
    var t3: Value; // TestStruct_C_type_value()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(TestStruct_A_type_value(), arg0);
    assume has_type(TestStruct_C_type_value(), arg1);

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
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // TestStruct_T_type_value()
    var t2: Reference; // ReferenceType(TestStruct_T_type_value())
    var t3: Reference; // ReferenceType(TestStruct_T_type_value())
    var t4: Value; // BooleanType()
    var t5: Value; // AddressType()
    var t6: Value; // BooleanType()
    var t7: Value; // BooleanType()
    var t8: Value; // BooleanType()
    var t9: Value; // IntegerType()
    var t10: Value; // AddressType()
    var t11: Reference; // ReferenceType(TestStruct_T_type_value())
    var t12: Reference; // ReferenceType(TestStruct_T_type_value())
    var t13: Value; // AddressType()
    var t14: Reference; // ReferenceType(TestStruct_T_type_value())
    var t15: Reference; // ReferenceType(TestStruct_T_type_value())
    var t16: Value; // AddressType()
    var t17: Value; // TestStruct_T_type_value()
    var t18: Value; // TestStruct_T_type_value()
    var t19: Value; // BooleanType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(AddressType(), arg0);

    old_size := m_size;
    m_size := m_size + 20;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call tmp := Exists(contents#Memory(m)[old_size+5], TestStruct_T_type_value());
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

    call t11 := BorrowGlobal(contents#Memory(m)[old_size+10], TestStruct_T_type_value());

    call t2 := CopyOrMoveRef(t11);

    call t12 := CopyOrMoveRef(t2);

    // unimplemented instruction

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[13+old_size := true], contents#Memory(m)[13+old_size := tmp]);

    call t14 := BorrowGlobal(contents#Memory(m)[old_size+13], TestStruct_T_type_value());

    call t3 := CopyOrMoveRef(t14);

    call t15 := CopyOrMoveRef(t3);

    // unimplemented instruction

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[16+old_size := true], contents#Memory(m)[16+old_size := tmp]);

    call tmp := MoveFrom(contents#Memory(m)[old_size+16], TestStruct_T_type_value());
    m := Memory(domain#Memory(m)[17+old_size := true], contents#Memory(m)[17+old_size := tmp]);
    assume has_type(TestStruct_T_type_value(), t17);


    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+17]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[18+old_size := true], contents#Memory(m)[18+old_size := tmp]);

    call MoveToSender(TestStruct_T_type_value(), contents#Memory(m)[old_size+18]);

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
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // TestStruct_A_type_value()
    var t2: Value; // TestStruct_B_type_value()
    var t3: Reference; // ReferenceType(TestStruct_B_type_value())
    var t4: Reference; // ReferenceType(IntegerType())
    var t5: Value; // IntegerType()
    var t6: Value; // BooleanType()
    var t7: Value; // AddressType()
    var t8: Value; // IntegerType()
    var t9: Value; // TestStruct_B_type_value()
    var t10: Value; // AddressType()
    var t11: Value; // IntegerType()
    var t12: Value; // TestStruct_B_type_value()
    var t13: Reference; // ReferenceType(TestStruct_B_type_value())
    var t14: Reference; // ReferenceType(TestStruct_B_type_value())
    var t15: Reference; // ReferenceType(IntegerType())
    var t16: Reference; // ReferenceType(IntegerType())
    var t17: Value; // IntegerType()
    var t18: Value; // IntegerType()
    var t19: Value; // IntegerType()
    var t20: Value; // BooleanType()
    var t21: Value; // BooleanType()
    var t22: Value; // IntegerType()
    var t23: Value; // TestStruct_B_type_value()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(AddressType(), arg0);

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

    assume has_type(AddressType(), contents#Memory(m)[old_size+7]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+8]);

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

    assume has_type(AddressType(), contents#Memory(m)[old_size+10]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+11]);

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
    assume has_type(IntegerType(), tmp);

    m := Memory(domain#Memory(m)[17+old_size := true], contents#Memory(m)[17+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+17]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[18+old_size := true], contents#Memory(m)[18+old_size := tmp]);

    call tmp := LdConst(42);
    m := Memory(domain#Memory(m)[19+old_size := true], contents#Memory(m)[19+old_size := tmp]);

    tmp := Boolean(is_equal(IntegerType(), contents#Memory(m)[old_size+18], contents#Memory(m)[old_size+19]));
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
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // AddressType()
    var t1: Value; // IntegerType()
    var t2: Value; // TestStruct_B_type_value()
    var t3: Value; // AddressType()
    var t4: Value; // AddressType()
    var t5: Value; // IntegerType()
    var t6: Value; // TestStruct_B_type_value()
    var t7: Value; // TestStruct_B_type_value()
    var t8: Value; // AddressType()
    var t9: Value; // IntegerType()
    var t10: Value; // AddressType()
    var t11: Value; // AddressType()
    var t12: Value; // BooleanType()
    var t13: Value; // BooleanType()
    var t14: Value; // IntegerType()
    var t15: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(AddressType(), arg0);

    old_size := m_size;
    m_size := m_size + 16;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call tmp := LdConst(42);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    assume has_type(AddressType(), contents#Memory(m)[old_size+4]);

    assume has_type(IntegerType(), contents#Memory(m)[old_size+5]);

    call tmp := Pack_TestStruct_B(contents#Memory(m)[old_size+4], contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+6]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    call t8, t9 := Unpack_TestStruct_B(contents#Memory(m)[old_size+7]);
    assume has_type(AddressType(), t8);

    assume has_type(IntegerType(), t9);

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

    tmp := Boolean(is_equal(AddressType(), contents#Memory(m)[old_size+10], contents#Memory(m)[old_size+11]));
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

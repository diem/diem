

// ** structs of module TestGenerics

const unique TestGenerics_R: TypeName;
const TestGenerics_R_v: FieldName;
axiom TestGenerics_R_v == 0;
function TestGenerics_R_type_value(): TypeValue {
    StructType(TestGenerics_R, TypeValueArray(DefaultTypeMap[0 := Vector_T_type_value(IntegerType())], 1))
}

procedure {:inline 1} Pack_TestGenerics_R(v0: Value) returns (v: Value)
{
    assume has_type(Vector_T_type_value(IntegerType()), v0);
    v := Struct(ValueArray(DefaultIntMap[TestGenerics_R_v := v0], 1));
    assume has_type(TestGenerics_R_type_value(), v);
}

procedure {:inline 1} Unpack_TestGenerics_R(v: Value) returns (v0: Value)
{
    assert is#Struct(v);
    v0 := smap(v)[TestGenerics_R_v];
}

const unique TestGenerics_T: TypeName;
const TestGenerics_T_v: FieldName;
axiom TestGenerics_T_v == 0;
function TestGenerics_T_type_value(tv0: TypeValue): TypeValue {
    StructType(TestGenerics_T, TypeValueArray(DefaultTypeMap[0 := Vector_T_type_value(tv0)], 1))
}

procedure {:inline 1} Pack_TestGenerics_T(tv0: TypeValue, v0: Value) returns (v: Value)
{
    assume has_type(Vector_T_type_value(tv0), v0);
    v := Struct(ValueArray(DefaultIntMap[TestGenerics_T_v := v0], 1));
    assume has_type(TestGenerics_T_type_value(tv0), v);
}

procedure {:inline 1} Unpack_TestGenerics_T(v: Value) returns (v0: Value)
{
    assert is#Struct(v);
    v0 := smap(v)[TestGenerics_T_v];
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



// ** functions of module TestGenerics

procedure {:inline 1} TestGenerics_move2 (arg0: Value, arg1: Value) returns ()
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // IntegerType()
    var t1: Value; // IntegerType()
    var t2: Value; // Vector_T_type_value(IntegerType())
    var t3: Value; // TestGenerics_R_type_value()
    var t4: Value; // Vector_T_type_value(IntegerType())
    var t5: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var t6: Value; // IntegerType()
    var t7: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var t8: Value; // IntegerType()
    var t9: Value; // Vector_T_type_value(IntegerType())
    var t10: Value; // TestGenerics_R_type_value()
    var t11: Value; // TestGenerics_R_type_value()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(IntegerType(), arg0);
    assume has_type(IntegerType(), arg1);

    old_size := m_size;
    m_size := m_size + 12;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);

    // bytecode translation starts here
    call t4 := Vector_empty(IntegerType());
    assume has_type(Vector_T_type_value(IntegerType()), t4);

    m := Memory(domain#Memory(m)[old_size+4 := true], contents#Memory(m)[old_size+4 := t4]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+4]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call t5 := BorrowLoc(old_size+2);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    call Vector_push_back(IntegerType(), t5, contents#Memory(m)[old_size+6]);

    call t7 := BorrowLoc(old_size+2);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[8+old_size := true], contents#Memory(m)[8+old_size := tmp]);

    call Vector_push_back(IntegerType(), t7, contents#Memory(m)[old_size+8]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[9+old_size := true], contents#Memory(m)[9+old_size := tmp]);

    assume has_type(Vector_T_type_value(IntegerType()), contents#Memory(m)[old_size+9]);

    call tmp := Pack_TestGenerics_R(contents#Memory(m)[old_size+9]);
    m := Memory(domain#Memory(m)[10+old_size := true], contents#Memory(m)[10+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+10]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[11+old_size := true], contents#Memory(m)[11+old_size := tmp]);

    call MoveToSender(TestGenerics_R_type_value(), contents#Memory(m)[old_size+11]);

    return;

}

procedure TestGenerics_move2_verify (arg0: Value, arg1: Value) returns ()
{
    call TestGenerics_move2(arg0, arg1);
}

procedure {:inline 1} TestGenerics_create (tv0: TypeValue, arg0: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // tv0
    var t1: Value; // Vector_T_type_value(tv0)
    var t2: Value; // Vector_T_type_value(tv0)
    var t3: Reference; // ReferenceType(Vector_T_type_value(tv0))
    var t4: Value; // tv0
    var t5: Value; // Vector_T_type_value(tv0)
    var t6: Value; // TestGenerics_T_type_value(tv0)

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(tv0, arg0);

    old_size := m_size;
    m_size := m_size + 7;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call t2 := Vector_empty(tv0);
    assume has_type(Vector_T_type_value(tv0), t2);

    m := Memory(domain#Memory(m)[old_size+2 := true], contents#Memory(m)[old_size+2 := t2]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call t3 := BorrowLoc(old_size+1);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call Vector_push_back(tv0, t3, contents#Memory(m)[old_size+4]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    assume has_type(Vector_T_type_value(tv0), contents#Memory(m)[old_size+5]);

    call tmp := Pack_TestGenerics_T(tv0, contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+6];
    return;

}

procedure TestGenerics_create_verify (tv0: TypeValue, arg0: Value) returns (ret0: Value)
{
    call ret0 := TestGenerics_create(tv0: TypeValue, arg0);
}

procedure {:inline 1} TestGenerics_overcomplicated_equals (tv0: TypeValue, arg0: Value, arg1: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // tv0
    var t1: Value; // tv0
    var t2: Value; // BooleanType()
    var t3: Value; // TestGenerics_T_type_value(tv0)
    var t4: Value; // TestGenerics_T_type_value(tv0)
    var t5: Value; // tv0
    var t6: Value; // TestGenerics_T_type_value(tv0)
    var t7: Value; // tv0
    var t8: Value; // TestGenerics_T_type_value(tv0)
    var t9: Value; // TestGenerics_T_type_value(tv0)
    var t10: Value; // TestGenerics_T_type_value(tv0)
    var t11: Value; // BooleanType()
    var t12: Value; // BooleanType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(tv0, arg0);
    assume has_type(tv0, arg1);

    old_size := m_size;
    m_size := m_size + 13;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call t6 := TestGenerics_create(tv0, contents#Memory(m)[old_size+5]);
    assume has_type(TestGenerics_T_type_value(tv0), t6);

    m := Memory(domain#Memory(m)[old_size+6 := true], contents#Memory(m)[old_size+6 := t6]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+6]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[7+old_size := true], contents#Memory(m)[7+old_size := tmp]);

    call t8 := TestGenerics_create(tv0, contents#Memory(m)[old_size+7]);
    assume has_type(TestGenerics_T_type_value(tv0), t8);

    m := Memory(domain#Memory(m)[old_size+8 := true], contents#Memory(m)[old_size+8 := t8]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+8]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[9+old_size := true], contents#Memory(m)[9+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+4]);
    m := Memory(domain#Memory(m)[10+old_size := true], contents#Memory(m)[10+old_size := tmp]);

    tmp := Boolean(is_equal(TestGenerics_T_type_value(tv0), contents#Memory(m)[old_size+9], contents#Memory(m)[old_size+10]));
    m := Memory(domain#Memory(m)[11+old_size := true], contents#Memory(m)[11+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+11]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[12+old_size := true], contents#Memory(m)[12+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+12];
    return;

}

procedure TestGenerics_overcomplicated_equals_verify (tv0: TypeValue, arg0: Value, arg1: Value) returns (ret0: Value)
{
    call ret0 := TestGenerics_overcomplicated_equals(tv0: TypeValue, arg0, arg1);
}

procedure {:inline 1} TestGenerics_test () returns (ret0: Value)
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // BooleanType()
    var t1: Value; // IntegerType()
    var t2: Value; // IntegerType()
    var t3: Value; // BooleanType()
    var t4: Value; // BooleanType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 5;

    // bytecode translation starts here
    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    call tmp := LdConst(1);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call t3 := TestGenerics_overcomplicated_equals(IntegerType(), contents#Memory(m)[old_size+1], contents#Memory(m)[old_size+2]);
    assume has_type(BooleanType(), t3);

    m := Memory(domain#Memory(m)[old_size+3 := true], contents#Memory(m)[old_size+3 := t3]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+3]);
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+4];
    return;

}

procedure TestGenerics_test_verify () returns (ret0: Value)
{
    call ret0 := TestGenerics_test();
}

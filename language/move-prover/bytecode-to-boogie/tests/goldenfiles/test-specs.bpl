

// ** structs of module TestSpecs

const unique TestSpecs_S: TypeName;
const TestSpecs_S_a: FieldName;
axiom TestSpecs_S_a == 0;
function TestSpecs_S_type_value(): TypeValue {
    StructType(TestSpecs_S, TypeValueArray(DefaultTypeMap[0 := AddressType()], 1))
}

procedure {:inline 1} Pack_TestSpecs_S(v0: Value) returns (v: Value)
{
    assume has_type(AddressType(), v0);
    v := Struct(ValueArray(DefaultIntMap[TestSpecs_S_a := v0], 1));
    assume has_type(TestSpecs_S_type_value(), v);
}

procedure {:inline 1} Unpack_TestSpecs_S(v: Value) returns (v0: Value)
{
    assert is#Struct(v);
    v0 := smap(v)[TestSpecs_S_a];
}

const unique TestSpecs_R: TypeName;
const TestSpecs_R_x: FieldName;
axiom TestSpecs_R_x == 0;
const TestSpecs_R_s: FieldName;
axiom TestSpecs_R_s == 1;
function TestSpecs_R_type_value(): TypeValue {
    StructType(TestSpecs_R, TypeValueArray(DefaultTypeMap[0 := IntegerType()][1 := TestSpecs_S_type_value()], 2))
}

procedure {:inline 1} Pack_TestSpecs_R(v0: Value, v1: Value) returns (v: Value)
{
    assume has_type(IntegerType(), v0);
    assume has_type(TestSpecs_S_type_value(), v1);
    v := Struct(ValueArray(DefaultIntMap[TestSpecs_R_x := v0][TestSpecs_R_s := v1], 2));
    assume has_type(TestSpecs_R_type_value(), v);
}

procedure {:inline 1} Unpack_TestSpecs_R(v: Value) returns (v0: Value, v1: Value)
{
    assert is#Struct(v);
    v0 := smap(v)[TestSpecs_R_x];
    v1 := smap(v)[TestSpecs_R_s];
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

procedure {:inline 1} ReadValueMax(p: Path, i: int, v: Value) returns (v': Value)
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

procedure {:inline 1} UpdateValueMax(p: Path, i: int, v: Value, new_v: Value) returns (v': Value)
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



// ** functions of module TestSpecs

procedure {:inline 1} TestSpecs_div (arg0: Value, arg1: Value) returns (ret0: Value)
requires b#Boolean(Boolean(i#Integer(arg1) > i#Integer(Integer(0))));
requires ExistsTxnSenderAccount();
ensures !old(b#Boolean(Boolean(i#Integer(arg0) > i#Integer(Integer(1))))) ==> abort_flag;
ensures !abort_flag ==> b#Boolean(Boolean((ret0) == (Integer(i#Integer(arg0) * i#Integer(arg1)))));
ensures (b#Boolean(Boolean(i#Integer(arg0) > i#Integer(Integer(1))))) && !(b#Boolean(Boolean(i#Integer(arg0) <= i#Integer(Integer(0))))) ==> !abort_flag;
{
    // declare local variables
    var t0: Value; // IntegerType()
    var t1: Value; // IntegerType()
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()
    var t4: Value; // IntegerType()
    var t5: Value; // IntegerType()
    var t6: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(IntegerType(), arg0);
    assume has_type(IntegerType(), arg1);

    old_size := m_size;
    m_size := m_size + 7;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size :=  arg1]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[3+old_size := true], contents#Memory(m)[3+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+1]);
    m := Memory(domain#Memory(m)[4+old_size := true], contents#Memory(m)[4+old_size := tmp]);

    call tmp := Div(contents#Memory(m)[old_size+3], contents#Memory(m)[old_size+4]);
    m := Memory(domain#Memory(m)[5+old_size := true], contents#Memory(m)[5+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+5]);
    m := Memory(domain#Memory(m)[2+old_size := true], contents#Memory(m)[2+old_size := tmp]);

    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+2]);
    m := Memory(domain#Memory(m)[6+old_size := true], contents#Memory(m)[6+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+6];
    return;

}

procedure TestSpecs_div_verify (arg0: Value, arg1: Value) returns (ret0: Value)
{
    call ret0 := TestSpecs_div(arg0, arg1);
}

procedure {:inline 1} TestSpecs_create_resource () returns ()
requires ExistsTxnSenderAccount();
ensures !abort_flag ==> b#Boolean(ExistsResource(gs, a#Address(Address(1)), TestSpecs_R));
ensures !(b#Boolean(ExistsResource(gs, a#Address(Address(1)), TestSpecs_R))) ==> !abort_flag;
{
    // declare local variables

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 0;

    // bytecode translation starts here
    return;

}

procedure TestSpecs_create_resource_verify () returns ()
{
    call TestSpecs_create_resource();
}

procedure {:inline 1} TestSpecs_select_from_global_resource () returns ()
requires b#Boolean(Boolean(i#Integer(SelectField(Dereference(m, GetResourceReference(gs, a#Address(Address(1)), TestSpecs_R)), TestSpecs_R_x)) > i#Integer(Integer(0))));
requires ExistsTxnSenderAccount();
{
    // declare local variables

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 0;

    // bytecode translation starts here
    return;

}

procedure TestSpecs_select_from_global_resource_verify () returns ()
{
    call TestSpecs_select_from_global_resource();
}

procedure {:inline 1} TestSpecs_select_from_resource (arg0: Value) returns (ret0: Value)
requires b#Boolean(Boolean(i#Integer(SelectField(arg0, TestSpecs_R_x)) > i#Integer(Integer(0))));
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // TestSpecs_R_type_value()
    var t1: Value; // TestSpecs_R_type_value()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(TestSpecs_R_type_value(), arg0);

    old_size := m_size;
    m_size := m_size + 2;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+1];
    return;

}

procedure TestSpecs_select_from_resource_verify (arg0: Value) returns (ret0: Value)
{
    call ret0 := TestSpecs_select_from_resource(arg0);
}

procedure {:inline 1} TestSpecs_select_from_resource_nested (arg0: Value) returns (ret0: Value)
requires b#Boolean(Boolean((SelectField(SelectField(arg0, TestSpecs_R_s), TestSpecs_S_a)) == (Address(1))));
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // TestSpecs_R_type_value()
    var t1: Value; // TestSpecs_R_type_value()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(TestSpecs_R_type_value(), arg0);

    old_size := m_size;
    m_size := m_size + 2;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+1];
    return;

}

procedure TestSpecs_select_from_resource_nested_verify (arg0: Value) returns (ret0: Value)
{
    call ret0 := TestSpecs_select_from_resource_nested(arg0);
}

procedure {:inline 1} TestSpecs_select_from_global_resource_dynamic_address (arg0: Value) returns (ret0: Value)
requires b#Boolean(Boolean(i#Integer(SelectField(Dereference(m, GetResourceReference(gs, a#Address(SelectField(SelectField(arg0, TestSpecs_R_s), TestSpecs_S_a)), TestSpecs_R)), TestSpecs_R_x)) > i#Integer(Integer(0))));
requires ExistsTxnSenderAccount();
{
    // declare local variables
    var t0: Value; // TestSpecs_R_type_value()
    var t1: Value; // TestSpecs_R_type_value()

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types
    assume has_type(TestSpecs_R_type_value(), arg0);

    old_size := m_size;
    m_size := m_size + 2;
    m := Memory(domain#Memory(m)[0+old_size := true], contents#Memory(m)[0+old_size :=  arg0]);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(contents#Memory(m)[old_size+0]);
    m := Memory(domain#Memory(m)[1+old_size := true], contents#Memory(m)[1+old_size := tmp]);

    ret0 := contents#Memory(m)[old_size+1];
    return;

}

procedure TestSpecs_select_from_global_resource_dynamic_address_verify (arg0: Value) returns (ret0: Value)
{
    call ret0 := TestSpecs_select_from_global_resource_dynamic_address(arg0);
}

procedure {:inline 1} TestSpecs_select_from_reference (arg0: Reference) returns ()
requires b#Boolean(Boolean((SelectField(SelectField(Dereference(m, arg0), TestSpecs_R_s), TestSpecs_S_a)) == (Address(1))));
requires ExistsTxnSenderAccount();
ensures b#Boolean(Boolean((SelectField(SelectField(Dereference(m, arg0), TestSpecs_R_s), TestSpecs_S_a)) == (old(SelectField(SelectField(Dereference(m, arg0), TestSpecs_R_s), TestSpecs_S_a)))));
{
    // declare local variables
    var t0: Reference; // ReferenceType(TestSpecs_R_type_value())

    var tmp: Value;
    var old_size: int;
    assume !abort_flag;

    // assume arguments are of correct types

    old_size := m_size;
    m_size := m_size + 1;
    t0 := arg0;

    // bytecode translation starts here
    return;

}

procedure TestSpecs_select_from_reference_verify (arg0: Reference) returns ()
{
    call TestSpecs_select_from_reference(arg0);
}

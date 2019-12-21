

// ** structs of module TestSpecs

const unique TestSpecs_S: TypeName;
const TestSpecs_S_a: FieldName;
axiom TestSpecs_S_a == 0;
function TestSpecs_S_type_value(): TypeValue {
    StructType(TestSpecs_S, ExtendTypeValueArray(EmptyTypeValueArray, AddressType()))
}

procedure {:inline 1} Pack_TestSpecs_S(v0: Value) returns (v: Value)
{
    assume is#Address(v0);
    v := Vector(ExtendValueArray(EmptyValueArray, v0));

}

procedure {:inline 1} Unpack_TestSpecs_S(v: Value) returns (v0: Value)
{
    assume is#Vector(v);
    v0 := SelectField(v, TestSpecs_S_a);
}

const unique TestSpecs_R: TypeName;
const TestSpecs_R_x: FieldName;
axiom TestSpecs_R_x == 0;
const TestSpecs_R_s: FieldName;
axiom TestSpecs_R_s == 1;
function TestSpecs_R_type_value(): TypeValue {
    StructType(TestSpecs_R, ExtendTypeValueArray(ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()), TestSpecs_S_type_value()))
}

procedure {:inline 1} Pack_TestSpecs_R(v0: Value, v1: Value) returns (v: Value)
{
    assume is#Integer(v0);
    assume is#Vector(v1);
    v := Vector(ExtendValueArray(ExtendValueArray(EmptyValueArray, v0), v1));

}

procedure {:inline 1} Unpack_TestSpecs_R(v: Value) returns (v0: Value, v1: Value)
{
    assume is#Vector(v);
    v0 := SelectField(v, TestSpecs_R_x);
    v1 := SelectField(v, TestSpecs_R_s);
}



// ** functions of module TestSpecs

procedure {:inline 1} TestSpecs_div (arg0: Value, arg1: Value) returns (ret0: Value)
requires b#Boolean(Boolean(i#Integer(arg1) > i#Integer(Integer(0))));
requires ExistsTxnSenderAccount(m, txn);
ensures !abort_flag ==> b#Boolean(Boolean((ret0) == (Integer(i#Integer(arg0) * i#Integer(arg1)))));
ensures old(!(b#Boolean(Boolean(i#Integer(arg0) <= i#Integer(Integer(0))))) && (b#Boolean(Boolean(i#Integer(arg0) > i#Integer(Integer(1)))))) ==> !abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(arg0) <= i#Integer(Integer(0))))) ==> abort_flag;
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

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Integer(arg0);
    assume is#Integer(arg1);

    old_size := local_counter;
    local_counter := local_counter + 7;
    m := UpdateLocal(m, old_size + 0, arg0);
    m := UpdateLocal(m, old_size + 1, arg1);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 3, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 1));
    m := UpdateLocal(m, old_size + 4, tmp);

    call tmp := Div(GetLocal(m, old_size + 3), GetLocal(m, old_size + 4));
    if (abort_flag) { goto Label_Abort; }
    m := UpdateLocal(m, old_size + 5, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 5));
    m := UpdateLocal(m, old_size + 2, tmp);

    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 2));
    m := UpdateLocal(m, old_size + 6, tmp);

    ret0 := GetLocal(m, old_size + 6);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure TestSpecs_div_verify (arg0: Value, arg1: Value) returns (ret0: Value)
{
    call ret0 := TestSpecs_div(arg0, arg1);
}

procedure {:inline 1} TestSpecs_create_resource () returns ()
requires ExistsTxnSenderAccount(m, txn);
ensures !abort_flag ==> b#Boolean(ExistsResource(m, TestSpecs_R_type_value(), a#Address(Address(1))));
ensures old(!(b#Boolean(ExistsResource(m, TestSpecs_R_type_value(), a#Address(Address(1)))))) ==> !abort_flag;
ensures old(b#Boolean(ExistsResource(m, TestSpecs_R_type_value(), a#Address(Address(1))))) ==> abort_flag;
{
    // declare local variables

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types

    old_size := local_counter;
    local_counter := local_counter + 0;

    // bytecode translation starts here
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure TestSpecs_create_resource_verify () returns ()
{
    call TestSpecs_create_resource();
}

procedure {:inline 1} TestSpecs_select_from_global_resource () returns ()
requires b#Boolean(Boolean(i#Integer(SelectField(Dereference(m, GetResourceReference(TestSpecs_R_type_value(), a#Address(Address(1)))), TestSpecs_R_x)) > i#Integer(Integer(0))));
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types

    old_size := local_counter;
    local_counter := local_counter + 0;

    // bytecode translation starts here
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure TestSpecs_select_from_global_resource_verify () returns ()
{
    call TestSpecs_select_from_global_resource();
}

procedure {:inline 1} TestSpecs_select_from_resource (arg0: Value) returns (ret0: Value)
requires b#Boolean(Boolean(i#Integer(SelectField(arg0, TestSpecs_R_x)) > i#Integer(Integer(0))));
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Value; // TestSpecs_R_type_value()
    var t1: Value; // TestSpecs_R_type_value()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(arg0);

    old_size := local_counter;
    local_counter := local_counter + 2;
    m := UpdateLocal(m, old_size + 0, arg0);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 1, tmp);

    ret0 := GetLocal(m, old_size + 1);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure TestSpecs_select_from_resource_verify (arg0: Value) returns (ret0: Value)
{
    call ret0 := TestSpecs_select_from_resource(arg0);
}

procedure {:inline 1} TestSpecs_select_from_resource_nested (arg0: Value) returns (ret0: Value)
requires b#Boolean(Boolean((SelectField(SelectField(arg0, TestSpecs_R_s), TestSpecs_S_a)) == (Address(1))));
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Value; // TestSpecs_R_type_value()
    var t1: Value; // TestSpecs_R_type_value()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(arg0);

    old_size := local_counter;
    local_counter := local_counter + 2;
    m := UpdateLocal(m, old_size + 0, arg0);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 1, tmp);

    ret0 := GetLocal(m, old_size + 1);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure TestSpecs_select_from_resource_nested_verify (arg0: Value) returns (ret0: Value)
{
    call ret0 := TestSpecs_select_from_resource_nested(arg0);
}

procedure {:inline 1} TestSpecs_select_from_global_resource_dynamic_address (arg0: Value) returns (ret0: Value)
requires b#Boolean(Boolean(i#Integer(SelectField(Dereference(m, GetResourceReference(TestSpecs_R_type_value(), a#Address(SelectField(SelectField(arg0, TestSpecs_R_s), TestSpecs_S_a)))), TestSpecs_R_x)) > i#Integer(Integer(0))));
requires ExistsTxnSenderAccount(m, txn);
{
    // declare local variables
    var t0: Value; // TestSpecs_R_type_value()
    var t1: Value; // TestSpecs_R_type_value()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume is#Vector(arg0);

    old_size := local_counter;
    local_counter := local_counter + 2;
    m := UpdateLocal(m, old_size + 0, arg0);

    // bytecode translation starts here
    call tmp := CopyOrMoveValue(GetLocal(m, old_size + 0));
    m := UpdateLocal(m, old_size + 1, tmp);

    ret0 := GetLocal(m, old_size + 1);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
}

procedure TestSpecs_select_from_global_resource_dynamic_address_verify (arg0: Value) returns (ret0: Value)
{
    call ret0 := TestSpecs_select_from_global_resource_dynamic_address(arg0);
}

procedure {:inline 1} TestSpecs_select_from_reference (arg0: Reference) returns ()
requires b#Boolean(Boolean((SelectField(SelectField(Dereference(m, arg0), TestSpecs_R_s), TestSpecs_S_a)) == (Address(1))));
requires ExistsTxnSenderAccount(m, txn);
ensures b#Boolean(Boolean((SelectField(SelectField(Dereference(m, arg0), TestSpecs_R_s), TestSpecs_S_a)) == (old(SelectField(SelectField(Dereference(m, arg0), TestSpecs_R_s), TestSpecs_S_a)))));
{
    // declare local variables
    var t0: Reference; // ReferenceType(TestSpecs_R_type_value())

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types
    assume IsValidReferenceParameter(local_counter, arg0);

    old_size := local_counter;
    local_counter := local_counter + 1;
    t0 := arg0;

    // bytecode translation starts here
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
}

procedure TestSpecs_select_from_reference_verify (arg0: Reference) returns ()
{
    call TestSpecs_select_from_reference(arg0);
}

procedure {:inline 1} TestSpecs_ret_values () returns (ret0: Value, ret1: Value, ret2: Value)
requires ExistsTxnSenderAccount(m, txn);
ensures b#Boolean(Boolean((ret0) == (Integer(7))));
ensures b#Boolean(Boolean((ret1) == (Boolean(false))));
ensures b#Boolean(Boolean((ret2) == (Integer(10))));
{
    // declare local variables
    var t0: Value; // IntegerType()
    var t1: Value; // BooleanType()
    var t2: Value; // IntegerType()

    var tmp: Value;
    var old_size: int;

    var saved_m: Memory;
    assume !abort_flag;
    saved_m := m;

    // assume arguments are of correct types

    old_size := local_counter;
    local_counter := local_counter + 3;

    // bytecode translation starts here
    call tmp := LdConst(7);
    m := UpdateLocal(m, old_size + 0, tmp);

    call tmp := LdFalse();
    m := UpdateLocal(m, old_size + 1, tmp);

    call tmp := LdConst(10);
    m := UpdateLocal(m, old_size + 2, tmp);

    ret0 := GetLocal(m, old_size + 0);
    ret1 := GetLocal(m, old_size + 1);
    ret2 := GetLocal(m, old_size + 2);
    return;

Label_Abort:
    abort_flag := true;
    m := saved_m;
    ret0 := DefaultValue;
    ret1 := DefaultValue;
    ret2 := DefaultValue;
}

procedure TestSpecs_ret_values_verify () returns (ret0: Value, ret1: Value, ret2: Value)
{
    call ret0, ret1, ret2 := TestSpecs_ret_values();
}


// ** helpers from test_mvir/test-specs-translate.prover.bpl
// Boogie helper functions for test-specs-translate.mvir

function {:inline} number_in_range(x: Value): Value {
  Boolean(i#Integer(x) >= 0 && i#Integer(x) < 128)
}


// ** structs of module TestSpecs

const unique TestSpecs_S: TypeName;
const TestSpecs_S_a: FieldName;
axiom TestSpecs_S_a == 0;
function TestSpecs_S_type_value(): TypeValue {
    StructType(TestSpecs_S, ExtendTypeValueArray(EmptyTypeValueArray, AddressType()))
}
procedure {:inline 1} Pack_TestSpecs_S(a: Value) returns (_struct: Value)
{
    assume is#Address(a);
    _struct := Vector(ExtendValueArray(EmptyValueArray, a));
}

procedure {:inline 1} Unpack_TestSpecs_S(_struct: Value) returns (a: Value)
{
    assume is#Vector(_struct);
    a := SelectField(_struct, TestSpecs_S_a);
    assume is#Address(a);
}

const unique TestSpecs_R: TypeName;
const TestSpecs_R_x: FieldName;
axiom TestSpecs_R_x == 0;
const TestSpecs_R_s: FieldName;
axiom TestSpecs_R_s == 1;
function TestSpecs_R_type_value(): TypeValue {
    StructType(TestSpecs_R, ExtendTypeValueArray(ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()), TestSpecs_S_type_value()))
}
procedure {:inline 1} Pack_TestSpecs_R(x: Value, s: Value) returns (_struct: Value)
{
    assume IsValidU64(x);
    assume is#Vector(s);
    _struct := Vector(ExtendValueArray(ExtendValueArray(EmptyValueArray, x), s));
}

procedure {:inline 1} Unpack_TestSpecs_R(_struct: Value) returns (x: Value, s: Value)
{
    assume is#Vector(_struct);
    x := SelectField(_struct, TestSpecs_R_x);
    assume IsValidU64(x);
    s := SelectField(_struct, TestSpecs_R_s);
    assume is#Vector(s);
}



// ** functions of module TestSpecs

procedure {:inline 1} TestSpecs_div (x1: Value, x2: Value) returns (__ret0: Value)
requires b#Boolean(Boolean(i#Integer(x2) > i#Integer(Integer(0))));
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(__ret0, Integer(i#Integer(x1) * i#Integer(x2)))));
ensures old(!(b#Boolean(Boolean(i#Integer(x1) <= i#Integer(Integer(0))))) && (b#Boolean(Boolean(i#Integer(x1) > i#Integer(Integer(1)))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(x1) <= i#Integer(Integer(0))))) ==> __abort_flag;

{
    // declare local variables
    var r: Value; // IntegerType()
    var __t3: Value; // IntegerType()
    var __t4: Value; // IntegerType()
    var __t5: Value; // IntegerType()
    var __t6: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#TestSpecs#div#0#x1: [Position]Value;
    var debug#TestSpecs#div#1#x2: [Position]Value;
    var debug#TestSpecs#div#2#r: [Position]Value;
    var debug#TestSpecs#div#3#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 7;
    debug#TestSpecs#div#0#x1 := EmptyPositionMap;
    debug#TestSpecs#div#1#x2 := EmptyPositionMap;
    debug#TestSpecs#div#2#r := EmptyPositionMap;
    debug#TestSpecs#div#3#__ret := EmptyPositionMap;

    // process and type check arguments
    assume IsValidU64(x1);
    __m := UpdateLocal(__m, __frame + 0, x1);
    debug#TestSpecs#div#0#x1 := debug#TestSpecs#div#0#x1[Position(293) := x1];
    assume IsValidU64(x2);
    __m := UpdateLocal(__m, __frame + 1, x2);
    debug#TestSpecs#div#1#x2 := debug#TestSpecs#div#1#x2[Position(293) := x2];

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := Div(GetLocal(__m, __frame + 3), GetLocal(__m, __frame + 4));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    debug#TestSpecs#div#2#r := debug#TestSpecs#div#2#r[Position(461) := __tmp];

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    __ret0 := GetLocal(__m, __frame + 6);
    debug#TestSpecs#div#3#__ret := debug#TestSpecs#div#3#__ret[Position(494) := __ret0];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#TestSpecs#div#3#__ret := debug#TestSpecs#div#3#__ret[Position(514) := __ret0];
}

procedure TestSpecs_div_verify (x1: Value, x2: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := TestSpecs_div(x1, x2);
}

procedure {:inline 1} TestSpecs_create_resource () returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(ExistsResource(__m, TestSpecs_R_type_value(), a#Address(Address(1))));
ensures old(!(b#Boolean(ExistsResource(__m, TestSpecs_R_type_value(), a#Address(Address(1)))))) ==> !__abort_flag;
ensures old(b#Boolean(ExistsResource(__m, TestSpecs_R_type_value(), a#Address(Address(1))))) ==> __abort_flag;

{
    // declare local variables
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 0;

    // process and type check arguments

    // bytecode translation starts here
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure TestSpecs_create_resource_verify () returns ()
{
    call InitVerification();
    call TestSpecs_create_resource();
}

procedure {:inline 1} TestSpecs_select_from_global_resource () returns ()
requires b#Boolean(Boolean(i#Integer(SelectField(Dereference(__m, GetResourceReference(TestSpecs_R_type_value(), a#Address(Address(1)))), TestSpecs_R_x)) > i#Integer(Integer(0))));
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 0;

    // process and type check arguments

    // bytecode translation starts here
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure TestSpecs_select_from_global_resource_verify () returns ()
{
    call InitVerification();
    call TestSpecs_select_from_global_resource();
}

procedure {:inline 1} TestSpecs_select_from_resource (r: Value) returns (__ret0: Value)
requires b#Boolean(Boolean(i#Integer(SelectField(r, TestSpecs_R_x)) > i#Integer(Integer(0))));
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var __t1: Value; // TestSpecs_R_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#TestSpecs#select_from_resource#0#r: [Position]Value;
    var debug#TestSpecs#select_from_resource#1#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 2;
    debug#TestSpecs#select_from_resource#0#r := EmptyPositionMap;
    debug#TestSpecs#select_from_resource#1#__ret := EmptyPositionMap;

    // process and type check arguments
    assume is#Vector(r);
    __m := UpdateLocal(__m, __frame + 0, r);
    debug#TestSpecs#select_from_resource#0#r := debug#TestSpecs#select_from_resource#0#r[Position(770) := r];

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    __ret0 := GetLocal(__m, __frame + 1);
    debug#TestSpecs#select_from_resource#1#__ret := debug#TestSpecs#select_from_resource#1#__ret[Position(852) := __ret0];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#TestSpecs#select_from_resource#1#__ret := debug#TestSpecs#select_from_resource#1#__ret[Position(872) := __ret0];
}

procedure TestSpecs_select_from_resource_verify (r: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := TestSpecs_select_from_resource(r);
}

procedure {:inline 1} TestSpecs_select_from_resource_nested (r: Value) returns (__ret0: Value)
requires b#Boolean(Boolean(IsEqual(SelectField(SelectField(r, TestSpecs_R_s), TestSpecs_S_a), Address(1))));
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var __t1: Value; // TestSpecs_R_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#TestSpecs#select_from_resource_nested#0#r: [Position]Value;
    var debug#TestSpecs#select_from_resource_nested#1#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 2;
    debug#TestSpecs#select_from_resource_nested#0#r := EmptyPositionMap;
    debug#TestSpecs#select_from_resource_nested#1#__ret := EmptyPositionMap;

    // process and type check arguments
    assume is#Vector(r);
    __m := UpdateLocal(__m, __frame + 0, r);
    debug#TestSpecs#select_from_resource_nested#0#r := debug#TestSpecs#select_from_resource_nested#0#r[Position(879) := r];

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    __ret0 := GetLocal(__m, __frame + 1);
    debug#TestSpecs#select_from_resource_nested#1#__ret := debug#TestSpecs#select_from_resource_nested#1#__ret[Position(973) := __ret0];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#TestSpecs#select_from_resource_nested#1#__ret := debug#TestSpecs#select_from_resource_nested#1#__ret[Position(993) := __ret0];
}

procedure TestSpecs_select_from_resource_nested_verify (r: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := TestSpecs_select_from_resource_nested(r);
}

procedure {:inline 1} TestSpecs_select_from_global_resource_dynamic_address (r: Value) returns (__ret0: Value)
requires b#Boolean(Boolean(i#Integer(SelectField(Dereference(__m, GetResourceReference(TestSpecs_R_type_value(), a#Address(SelectField(SelectField(r, TestSpecs_R_s), TestSpecs_S_a)))), TestSpecs_R_x)) > i#Integer(Integer(0))));
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var __t1: Value; // TestSpecs_R_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#TestSpecs#select_from_global_resource_dynamic_address#0#r: [Position]Value;
    var debug#TestSpecs#select_from_global_resource_dynamic_address#1#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 2;
    debug#TestSpecs#select_from_global_resource_dynamic_address#0#r := EmptyPositionMap;
    debug#TestSpecs#select_from_global_resource_dynamic_address#1#__ret := EmptyPositionMap;

    // process and type check arguments
    assume is#Vector(r);
    __m := UpdateLocal(__m, __frame + 0, r);
    debug#TestSpecs#select_from_global_resource_dynamic_address#0#r := debug#TestSpecs#select_from_global_resource_dynamic_address#0#r[Position(1000) := r];

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    __ret0 := GetLocal(__m, __frame + 1);
    debug#TestSpecs#select_from_global_resource_dynamic_address#1#__ret := debug#TestSpecs#select_from_global_resource_dynamic_address#1#__ret[Position(1125) := __ret0];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#TestSpecs#select_from_global_resource_dynamic_address#1#__ret := debug#TestSpecs#select_from_global_resource_dynamic_address#1#__ret[Position(1145) := __ret0];
}

procedure TestSpecs_select_from_global_resource_dynamic_address_verify (r: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := TestSpecs_select_from_global_resource_dynamic_address(r);
}

procedure {:inline 1} TestSpecs_select_from_reference (r: Reference) returns ()
requires b#Boolean(Boolean(IsEqual(SelectField(SelectField(Dereference(__m, r), TestSpecs_R_s), TestSpecs_S_a), Address(1))));
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(SelectField(SelectField(Dereference(__m, r), TestSpecs_R_s), TestSpecs_S_a), old(SelectField(SelectField(Dereference(__m, r), TestSpecs_R_s), TestSpecs_S_a)))));
{
    // declare local variables
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#TestSpecs#select_from_reference#0#r: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 1;
    debug#TestSpecs#select_from_reference#0#r := EmptyPositionMap;

    // process and type check arguments
    assume is#Vector(Dereference(__m, r));
    assume IsValidReferenceParameter(__m, __frame, r);
    debug#TestSpecs#select_from_reference#0#r := debug#TestSpecs#select_from_reference#0#r[Position(1152) := Dereference(__m, r)];

    // bytecode translation starts here
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure TestSpecs_select_from_reference_verify (r: Reference) returns ()
{
    call InitVerification();
    call TestSpecs_select_from_reference(r);
}

procedure {:inline 1} TestSpecs_ret_values () returns (__ret0: Value, __ret1: Value, __ret2: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(__ret0, Integer(7))));
ensures b#Boolean(Boolean(IsEqual(__ret1, Boolean(false))));
ensures b#Boolean(Boolean(IsEqual(__ret2, Integer(10))));
{
    // declare local variables
    var __t0: Value; // IntegerType()
    var __t1: Value; // BooleanType()
    var __t2: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#TestSpecs#ret_values#0#__ret: [Position]Value;
    var debug#TestSpecs#ret_values#1#__ret: [Position]Value;
    var debug#TestSpecs#ret_values#2#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 3;
    debug#TestSpecs#ret_values#0#__ret := EmptyPositionMap;
    debug#TestSpecs#ret_values#1#__ret := EmptyPositionMap;
    debug#TestSpecs#ret_values#2#__ret := EmptyPositionMap;

    // process and type check arguments

    // bytecode translation starts here
    call __tmp := LdConst(7);
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call __tmp := LdFalse();
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := LdConst(10);
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    __ret0 := GetLocal(__m, __frame + 0);
    debug#TestSpecs#ret_values#0#__ret := debug#TestSpecs#ret_values#0#__ret[Position(1425) := __ret0];
    __ret1 := GetLocal(__m, __frame + 1);
    debug#TestSpecs#ret_values#1#__ret := debug#TestSpecs#ret_values#1#__ret[Position(1425) := __ret1];
    __ret2 := GetLocal(__m, __frame + 2);
    debug#TestSpecs#ret_values#2#__ret := debug#TestSpecs#ret_values#2#__ret[Position(1425) := __ret2];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#TestSpecs#ret_values#0#__ret := debug#TestSpecs#ret_values#0#__ret[Position(1452) := __ret0];
    __ret1 := DefaultValue;
    debug#TestSpecs#ret_values#1#__ret := debug#TestSpecs#ret_values#1#__ret[Position(1452) := __ret1];
    __ret2 := DefaultValue;
    debug#TestSpecs#ret_values#2#__ret := debug#TestSpecs#ret_values#2#__ret[Position(1452) := __ret2];
}

procedure TestSpecs_ret_values_verify () returns (__ret0: Value, __ret1: Value, __ret2: Value)
{
    call InitVerification();
    call __ret0, __ret1, __ret2 := TestSpecs_ret_values();
}

procedure {:inline 1} TestSpecs_helper_function (x: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(b#Boolean(number_in_range(x)) && b#Boolean(Boolean(i#Integer(x) < i#Integer(max_u64())))));
{
    // declare local variables
    var __t1: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#TestSpecs#helper_function#0#x: [Position]Value;
    var debug#TestSpecs#helper_function#1#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 2;
    debug#TestSpecs#helper_function#0#x := EmptyPositionMap;
    debug#TestSpecs#helper_function#1#__ret := EmptyPositionMap;

    // process and type check arguments
    assume IsValidU64(x);
    __m := UpdateLocal(__m, __frame + 0, x);
    debug#TestSpecs#helper_function#0#x := debug#TestSpecs#helper_function#0#x[Position(1459) := x];

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    __ret0 := GetLocal(__m, __frame + 1);
    debug#TestSpecs#helper_function#1#__ret := debug#TestSpecs#helper_function#1#__ret[Position(1559) := __ret0];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#TestSpecs#helper_function#1#__ret := debug#TestSpecs#helper_function#1#__ret[Position(1579) := __ret0];
}

procedure TestSpecs_helper_function_verify (x: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := TestSpecs_helper_function(x);
}

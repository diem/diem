
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

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume IsValidU64(x1);
    __m := UpdateLocal(__m, __frame + 0, x1);
    assume $DebugTrackLocal(0, 0, 0, 293, x1);
    assume IsValidU64(x2);
    __m := UpdateLocal(__m, __frame + 1, x2);
    assume $DebugTrackLocal(0, 0, 1, 293, x2);

    // increase the local counter
    __local_counter := __local_counter + 7;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := Div(GetLocal(__m, __frame + 3), GetLocal(__m, __frame + 4));
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 0, 465);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    assume $DebugTrackLocal(0, 0, 2, 461, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    __ret0 := GetLocal(__m, __frame + 6);
    assume $DebugTrackLocal(0, 0, 3, 494, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
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

    // process and type check arguments

    // increase the local counter
    __local_counter := __local_counter + 0;

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

    // process and type check arguments

    // increase the local counter
    __local_counter := __local_counter + 0;

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

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume is#Vector(r);
    __m := UpdateLocal(__m, __frame + 0, r);
    assume $DebugTrackLocal(0, 3, 0, 770, r);

    // increase the local counter
    __local_counter := __local_counter + 2;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    __ret0 := GetLocal(__m, __frame + 1);
    assume $DebugTrackLocal(0, 3, 1, 852, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
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

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume is#Vector(r);
    __m := UpdateLocal(__m, __frame + 0, r);
    assume $DebugTrackLocal(0, 4, 0, 879, r);

    // increase the local counter
    __local_counter := __local_counter + 2;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    __ret0 := GetLocal(__m, __frame + 1);
    assume $DebugTrackLocal(0, 4, 1, 973, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
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

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume is#Vector(r);
    __m := UpdateLocal(__m, __frame + 0, r);
    assume $DebugTrackLocal(0, 5, 0, 1000, r);

    // increase the local counter
    __local_counter := __local_counter + 2;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    __ret0 := GetLocal(__m, __frame + 1);
    assume $DebugTrackLocal(0, 5, 1, 1125, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
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

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume is#Vector(Dereference(__m, r));
    assume IsValidReferenceParameter(__m, __local_counter, r);
    assume is#Vector(Dereference(__m, r));
    assume $DebugTrackLocal(0, 6, 0, 1152, Dereference(__m, r));

    // increase the local counter
    __local_counter := __local_counter + 1;

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

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments

    // increase the local counter
    __local_counter := __local_counter + 3;

    // bytecode translation starts here
    call __tmp := LdConst(7);
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call __tmp := LdFalse();
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := LdConst(10);
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    __ret0 := GetLocal(__m, __frame + 0);
    assume $DebugTrackLocal(0, 7, 0, 1425, __ret0);
    __ret1 := GetLocal(__m, __frame + 1);
    assume $DebugTrackLocal(0, 7, 1, 1425, __ret1);
    __ret2 := GetLocal(__m, __frame + 2);
    assume $DebugTrackLocal(0, 7, 2, 1425, __ret2);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    __ret1 := DefaultValue;
    __ret2 := DefaultValue;
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

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume IsValidU64(x);
    __m := UpdateLocal(__m, __frame + 0, x);
    assume $DebugTrackLocal(0, 8, 0, 1459, x);

    // increase the local counter
    __local_counter := __local_counter + 2;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    __ret0 := GetLocal(__m, __frame + 1);
    assume $DebugTrackLocal(0, 8, 1, 1559, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure TestSpecs_helper_function_verify (x: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := TestSpecs_helper_function(x);
}

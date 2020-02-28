

// ** synthetics of module TestInvariants



// ** structs of module TestInvariants

const unique TestInvariants_T: TypeName;
const TestInvariants_T_i: FieldName;
axiom TestInvariants_T_i == 0;
function TestInvariants_T_type_value(): TypeValue {
    StructType(TestInvariants_T, ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()))
}
function {:inline 1} $TestInvariants_T_is_well_formed(__this: Value): bool {
    is#Vector(__this)
        && IsValidU64(SelectField(__this, TestInvariants_T_i))
        && b#Boolean(Boolean(i#Integer(SelectField(__this, TestInvariants_T_i)) > i#Integer(Integer(0))))
}

procedure {:inline 1} $TestInvariants_T_update_inv(__before: Value, __after: Value) {
    assert b#Boolean(Boolean(i#Integer(SelectField(__after, TestInvariants_T_i)) > i#Integer(SelectField(__before, TestInvariants_T_i))));
    assert b#Boolean(Boolean(i#Integer(SelectField(__after, TestInvariants_T_i)) > i#Integer(Integer(0))));
}

procedure {:inline 1} Pack_TestInvariants_T(module_idx: int, func_idx: int, var_idx: int, code_idx: int, i: Value) returns (_struct: Value)
{
    assume IsValidU64(i);
    _struct := Vector(ExtendValueArray(EmptyValueArray, i));
    if (code_idx > 0) { assume $DebugTrackLocal(module_idx, func_idx, var_idx, code_idx, _struct); }
    assert b#Boolean(Boolean(i#Integer(SelectField(_struct, TestInvariants_T_i)) > i#Integer(Integer(0))));
}

procedure {:inline 1} Unpack_TestInvariants_T(_struct: Value) returns (i: Value)
{
    assume is#Vector(_struct);
    i := SelectField(_struct, TestInvariants_T_i);
    assume IsValidU64(i);
}



// ** functions of module TestInvariants

procedure {:inline 1} TestInvariants_valid_T () returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(SelectField(__ret0, TestInvariants_T_i), Integer(1))));
{
    // declare local variables
    var t: Value; // TestInvariants_T_type_value()
    var __t1: Value; // IntegerType()
    var __t2: Value; // TestInvariants_T_type_value()
    var __t3: Value; // TestInvariants_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments

    // increase the local counter
    __local_counter := __local_counter + 4;

    // bytecode translation starts here
    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := Pack_TestInvariants_T(0, 0, 0, 686, GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(0, 0, 0, 682, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    __ret0 := GetLocal(__m, __frame + 3);
    assume $DebugTrackLocal(0, 0, 1, 700, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure TestInvariants_valid_T_verify () returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := TestInvariants_valid_T();
}

procedure {:inline 1} TestInvariants_invalid_T () returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(SelectField(__ret0, TestInvariants_T_i), Integer(0))));
{
    // declare local variables
    var t: Value; // TestInvariants_T_type_value()
    var __t1: Value; // IntegerType()
    var __t2: Value; // TestInvariants_T_type_value()
    var __t3: Value; // TestInvariants_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments

    // increase the local counter
    __local_counter := __local_counter + 4;

    // bytecode translation starts here
    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := Pack_TestInvariants_T(0, 1, 0, 814, GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(0, 1, 0, 810, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    __ret0 := GetLocal(__m, __frame + 3);
    assume $DebugTrackLocal(0, 1, 1, 828, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure TestInvariants_invalid_T_verify () returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := TestInvariants_invalid_T();
}

procedure {:inline 1} TestInvariants_valid_T_update () returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(SelectField(__ret0, TestInvariants_T_i), Integer(3))));
{
    // declare local variables
    var t: Value; // TestInvariants_T_type_value()
    var r: Reference; // ReferenceType(TestInvariants_T_type_value())
    var __t2: Value; // IntegerType()
    var __t3: Value; // TestInvariants_T_type_value()
    var __t4: Reference; // ReferenceType(TestInvariants_T_type_value())
    var __t5: Value; // IntegerType()
    var __t6: Value; // TestInvariants_T_type_value()
    var __t7: Reference; // ReferenceType(TestInvariants_T_type_value())
    var __t8: Value; // TestInvariants_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var __before_borrow_0: Value;
    var __before_borrow_0_ref: Reference;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments

    // increase the local counter
    __local_counter := __local_counter + 9;

    // bytecode translation starts here
    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := Pack_TestInvariants_T(0, 2, 0, 973, GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(0, 2, 0, 969, __tmp);

    call __t4 := BorrowLoc(__frame + 0, TestInvariants_T_type_value());
    __before_borrow_0 := Dereference(__m, __t4);
    __before_borrow_0_ref := __t4;

    call r := CopyOrMoveRef(__t4);
    assume $TestInvariants_T_is_well_formed(Dereference(__m, r));
    assume $DebugTrackLocal(0, 2, 1, 987, Dereference(__m, r));

    call __tmp := LdConst(3);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := Pack_TestInvariants_T(0, 0, 0, 0, GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __t7 := CopyOrMoveRef(r);

    call WriteRef(__t7, GetLocal(__m, __frame + 6));
    assume $DebugTrackLocal(0, 2, 0, 1005, GetLocal(__m, __frame + 0));

    call $TestInvariants_T_update_inv(__before_borrow_0, Dereference(__m, __before_borrow_0_ref));
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    __ret0 := GetLocal(__m, __frame + 8);
    assume $DebugTrackLocal(0, 2, 2, 1030, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure TestInvariants_valid_T_update_verify () returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := TestInvariants_valid_T_update();
}

procedure {:inline 1} TestInvariants_invalid_T_update () returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(SelectField(__ret0, TestInvariants_T_i), Integer(2))));
{
    // declare local variables
    var t: Value; // TestInvariants_T_type_value()
    var r: Reference; // ReferenceType(TestInvariants_T_type_value())
    var __t2: Value; // IntegerType()
    var __t3: Value; // TestInvariants_T_type_value()
    var __t4: Reference; // ReferenceType(TestInvariants_T_type_value())
    var __t5: Value; // IntegerType()
    var __t6: Value; // TestInvariants_T_type_value()
    var __t7: Reference; // ReferenceType(TestInvariants_T_type_value())
    var __t8: Value; // TestInvariants_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var __before_borrow_0: Value;
    var __before_borrow_0_ref: Reference;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments

    // increase the local counter
    __local_counter := __local_counter + 9;

    // bytecode translation starts here
    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := Pack_TestInvariants_T(0, 3, 0, 1177, GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(0, 3, 0, 1173, __tmp);

    call __t4 := BorrowLoc(__frame + 0, TestInvariants_T_type_value());
    __before_borrow_0 := Dereference(__m, __t4);
    __before_borrow_0_ref := __t4;

    call r := CopyOrMoveRef(__t4);
    assume $TestInvariants_T_is_well_formed(Dereference(__m, r));
    assume $DebugTrackLocal(0, 3, 1, 1191, Dereference(__m, r));

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := Pack_TestInvariants_T(0, 0, 0, 0, GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __t7 := CopyOrMoveRef(r);

    call WriteRef(__t7, GetLocal(__m, __frame + 6));
    assume $DebugTrackLocal(0, 3, 0, 1209, GetLocal(__m, __frame + 0));

    call $TestInvariants_T_update_inv(__before_borrow_0, Dereference(__m, __before_borrow_0_ref));
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    __ret0 := GetLocal(__m, __frame + 8);
    assume $DebugTrackLocal(0, 3, 2, 1234, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure TestInvariants_invalid_T_update_verify () returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := TestInvariants_invalid_T_update();
}

procedure {:inline 1} TestInvariants_invalid_T_update_indirectly () returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(SelectField(__ret0, TestInvariants_T_i), Integer(2))));
{
    // declare local variables
    var t: Value; // TestInvariants_T_type_value()
    var r: Reference; // ReferenceType(TestInvariants_T_type_value())
    var __t2: Value; // IntegerType()
    var __t3: Value; // TestInvariants_T_type_value()
    var __t4: Reference; // ReferenceType(TestInvariants_T_type_value())
    var __t5: Value; // IntegerType()
    var __t6: Reference; // ReferenceType(TestInvariants_T_type_value())
    var __t7: Reference; // ReferenceType(IntegerType())
    var __t8: Value; // TestInvariants_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var __before_borrow_0: Value;
    var __before_borrow_0_ref: Reference;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments

    // increase the local counter
    __local_counter := __local_counter + 9;

    // bytecode translation starts here
    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := Pack_TestInvariants_T(0, 4, 0, 1392, GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(0, 4, 0, 1388, __tmp);

    call __t4 := BorrowLoc(__frame + 0, TestInvariants_T_type_value());
    __before_borrow_0 := Dereference(__m, __t4);
    __before_borrow_0_ref := __t4;

    call r := CopyOrMoveRef(__t4);
    assume $TestInvariants_T_is_well_formed(Dereference(__m, r));
    assume $DebugTrackLocal(0, 4, 1, 1406, Dereference(__m, r));

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __t6 := CopyOrMoveRef(r);

    call __t7 := BorrowField(__t6, TestInvariants_T_i);

    call WriteRef(__t7, GetLocal(__m, __frame + 5));
    assume $DebugTrackLocal(0, 4, 0, 1424, GetLocal(__m, __frame + 0));

    call $TestInvariants_T_update_inv(__before_borrow_0, Dereference(__m, __before_borrow_0_ref));
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    __ret0 := GetLocal(__m, __frame + 8);
    assume $DebugTrackLocal(0, 4, 2, 1451, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure TestInvariants_invalid_T_update_indirectly_verify () returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := TestInvariants_invalid_T_update_indirectly();
}

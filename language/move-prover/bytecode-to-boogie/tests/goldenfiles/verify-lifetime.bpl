

// ** synthetics of module TestLifetime



// ** structs of module TestLifetime

const unique TestLifetime_S: TypeName;
const TestLifetime_S_x: FieldName;
axiom TestLifetime_S_x == 0;
function TestLifetime_S_type_value(): TypeValue {
    StructType(TestLifetime_S, ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()))
}
function {:inline 1} $TestLifetime_S_is_well_formed(__this: Value): bool {
    is#Vector(__this)
        && IsValidU64(SelectField(__this, TestLifetime_S_x))
        && b#Boolean(Boolean(i#Integer(SelectField(__this, TestLifetime_S_x)) > i#Integer(Integer(1))))
}

procedure {:inline 1} $TestLifetime_S_update_inv(__before: Value, __after: Value) {
    assert b#Boolean(Boolean(i#Integer(SelectField(__after, TestLifetime_S_x)) > i#Integer(Integer(1))));
}

procedure {:inline 1} Pack_TestLifetime_S(module_idx: int, func_idx: int, var_idx: int, code_idx: int, x: Value) returns (_struct: Value)
{
    assume IsValidU64(x);
    _struct := Vector(ExtendValueArray(EmptyValueArray, x));
    if (code_idx > 0) { assume $DebugTrackLocal(module_idx, func_idx, var_idx, code_idx, _struct); }
    assert b#Boolean(Boolean(i#Integer(SelectField(_struct, TestLifetime_S_x)) > i#Integer(Integer(1))));
}

procedure {:inline 1} Unpack_TestLifetime_S(_struct: Value) returns (x: Value)
{
    assume is#Vector(_struct);
    x := SelectField(_struct, TestLifetime_S_x);
    assume IsValidU64(x);
}

const unique TestLifetime_T: TypeName;
const TestLifetime_T_x: FieldName;
axiom TestLifetime_T_x == 0;
function TestLifetime_T_type_value(): TypeValue {
    StructType(TestLifetime_T, ExtendTypeValueArray(EmptyTypeValueArray, IntegerType()))
}
function {:inline 1} $TestLifetime_T_is_well_formed(__this: Value): bool {
    is#Vector(__this)
        && IsValidU64(SelectField(__this, TestLifetime_T_x))
        && b#Boolean(Boolean(i#Integer(SelectField(__this, TestLifetime_T_x)) > i#Integer(Integer(1))))
}

procedure {:inline 1} $TestLifetime_T_update_inv(__before: Value, __after: Value) {
    assert b#Boolean(Boolean(i#Integer(SelectField(__after, TestLifetime_T_x)) > i#Integer(Integer(1))));
}

procedure {:inline 1} Pack_TestLifetime_T(module_idx: int, func_idx: int, var_idx: int, code_idx: int, x: Value) returns (_struct: Value)
{
    assume IsValidU64(x);
    _struct := Vector(ExtendValueArray(EmptyValueArray, x));
    if (code_idx > 0) { assume $DebugTrackLocal(module_idx, func_idx, var_idx, code_idx, _struct); }
    assert b#Boolean(Boolean(i#Integer(SelectField(_struct, TestLifetime_T_x)) > i#Integer(Integer(1))));
}

procedure {:inline 1} Unpack_TestLifetime_T(_struct: Value) returns (x: Value)
{
    assume is#Vector(_struct);
    x := SelectField(_struct, TestLifetime_T_x);
    assume IsValidU64(x);
}



// ** functions of module TestLifetime

procedure {:inline 1} TestLifetime_invalid_S () returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var s: Value; // TestLifetime_S_type_value()
    var s_ref: Reference; // ReferenceType(TestLifetime_S_type_value())
    var x_ref: Reference; // ReferenceType(IntegerType())
    var __t3: Value; // IntegerType()
    var __t4: Value; // TestLifetime_S_type_value()
    var __t5: Reference; // ReferenceType(TestLifetime_S_type_value())
    var __t6: Reference; // ReferenceType(TestLifetime_S_type_value())
    var __t7: Reference; // ReferenceType(IntegerType())
    var __t8: Value; // IntegerType()
    var __t9: Reference; // ReferenceType(IntegerType())
    var __t10: Reference; // ReferenceType(TestLifetime_S_type_value())
    var __t11: Reference; // ReferenceType(TestLifetime_S_type_value())
    var __t12: Reference; // ReferenceType(IntegerType())
    var __t13: Value; // IntegerType()
    var __t14: Reference; // ReferenceType(IntegerType())
    var __t15: Value; // TestLifetime_S_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var __before_borrow_0: Value;
    var __before_borrow_0_ref: Reference;
    var __before_borrow_1: Value;
    var __before_borrow_1_ref: Reference;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments

    // increase the local counter
    __local_counter := __local_counter + 16;

    // bytecode translation starts here
    call __tmp := LdConst(3);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := Pack_TestLifetime_S(0, 0, 0, 367, GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(0, 0, 0, 363, __tmp);

    call __t5 := BorrowLoc(__frame + 0, TestLifetime_S_type_value());
    __before_borrow_0 := Dereference(__m, __t5);
    __before_borrow_0_ref := __t5;

    call s_ref := CopyOrMoveRef(__t5);
    assume $TestLifetime_S_is_well_formed(Dereference(__m, s_ref));
    assume $DebugTrackLocal(0, 0, 1, 383, Dereference(__m, s_ref));

    call __t6 := CopyOrMoveRef(s_ref);

    call __t7 := BorrowField(__t6, TestLifetime_S_x);

    call x_ref := CopyOrMoveRef(__t7);
    assume IsValidU64(Dereference(__m, x_ref));
    assume $DebugTrackLocal(0, 0, 2, 407, Dereference(__m, x_ref));

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __t9 := CopyOrMoveRef(x_ref);

    call WriteRef(__t9, GetLocal(__m, __frame + 8));
    assume $DebugTrackLocal(0, 0, 0, 443, GetLocal(__m, __frame + 0));

    call $TestLifetime_S_update_inv(__before_borrow_0, Dereference(__m, __before_borrow_0_ref));
    call __t10 := BorrowLoc(__frame + 0, TestLifetime_S_type_value());
    __before_borrow_1 := Dereference(__m, __t10);
    __before_borrow_1_ref := __t10;

    call s_ref := CopyOrMoveRef(__t10);
    assume $TestLifetime_S_is_well_formed(Dereference(__m, s_ref));
    assume $DebugTrackLocal(0, 0, 1, 502, Dereference(__m, s_ref));

    call __t11 := CopyOrMoveRef(s_ref);

    call __t12 := BorrowField(__t11, TestLifetime_S_x);

    call x_ref := CopyOrMoveRef(__t12);
    assume IsValidU64(Dereference(__m, x_ref));
    assume $DebugTrackLocal(0, 0, 2, 526, Dereference(__m, x_ref));

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __t14 := CopyOrMoveRef(x_ref);

    call WriteRef(__t14, GetLocal(__m, __frame + 13));
    assume $DebugTrackLocal(0, 0, 0, 562, GetLocal(__m, __frame + 0));

    call $TestLifetime_S_update_inv(__before_borrow_1, Dereference(__m, __before_borrow_1_ref));
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 15, __tmp);

    __ret0 := GetLocal(__m, __frame + 15);
    assume $DebugTrackLocal(0, 0, 3, 589, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure TestLifetime_invalid_S_verify () returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := TestLifetime_invalid_S();
}

procedure {:inline 1} TestLifetime_invalid_bool (cond: Value) returns (__ret0: Value, __ret1: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var s_ref: Reference; // ReferenceType(TestLifetime_S_type_value())
    var s: Value; // TestLifetime_S_type_value()
    var t_ref: Reference; // ReferenceType(TestLifetime_T_type_value())
    var t: Value; // TestLifetime_T_type_value()
    var x_ref: Reference; // ReferenceType(IntegerType())
    var __t6: Value; // IntegerType()
    var __t7: Value; // TestLifetime_S_type_value()
    var __t8: Value; // IntegerType()
    var __t9: Value; // TestLifetime_T_type_value()
    var __t10: Reference; // ReferenceType(TestLifetime_S_type_value())
    var __t11: Reference; // ReferenceType(TestLifetime_T_type_value())
    var __t12: Value; // BooleanType()
    var __t13: Reference; // ReferenceType(TestLifetime_S_type_value())
    var __t14: Reference; // ReferenceType(IntegerType())
    var __t15: Reference; // ReferenceType(TestLifetime_T_type_value())
    var __t16: Reference; // ReferenceType(IntegerType())
    var __t17: Value; // BooleanType()
    var __t18: Value; // IntegerType()
    var __t19: Reference; // ReferenceType(IntegerType())
    var __t20: Value; // IntegerType()
    var __t21: Reference; // ReferenceType(IntegerType())
    var __t22: Value; // TestLifetime_S_type_value()
    var __t23: Value; // TestLifetime_T_type_value()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var __before_borrow_0: Value;
    var __before_borrow_0_ref: Reference;
    var __before_borrow_1: Value;
    var __before_borrow_1_ref: Reference;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume is#Boolean(cond);
    __m := UpdateLocal(__m, __frame + 0, cond);
    assume $DebugTrackLocal(0, 1, 0, 616, cond);

    // increase the local counter
    __local_counter := __local_counter + 24;

    // bytecode translation starts here
    call __tmp := LdConst(3);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := Pack_TestLifetime_S(0, 1, 2, 820, GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    assume $DebugTrackLocal(0, 1, 2, 816, __tmp);

    call __tmp := LdConst(4);
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __tmp := Pack_TestLifetime_T(0, 1, 4, 840, GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 9));
    __m := UpdateLocal(__m, __frame + 4, __tmp);
    assume $DebugTrackLocal(0, 1, 4, 836, __tmp);

    call __t10 := BorrowLoc(__frame + 2, TestLifetime_S_type_value());
    __before_borrow_0 := Dereference(__m, __t10);
    __before_borrow_0_ref := __t10;

    call s_ref := CopyOrMoveRef(__t10);
    assume $TestLifetime_S_is_well_formed(Dereference(__m, s_ref));
    assume $DebugTrackLocal(0, 1, 1, 856, Dereference(__m, s_ref));

    call __t11 := BorrowLoc(__frame + 4, TestLifetime_T_type_value());
    __before_borrow_1 := Dereference(__m, __t11);
    __before_borrow_1_ref := __t11;

    call t_ref := CopyOrMoveRef(__t11);
    assume $TestLifetime_T_is_well_formed(Dereference(__m, t_ref));
    assume $DebugTrackLocal(0, 1, 3, 880, Dereference(__m, t_ref));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    __tmp := GetLocal(__m, __frame + 12);
    if (!b#Boolean(__tmp)) { goto Label_16; }

    call __t13 := CopyOrMoveRef(s_ref);

    call __t14 := BorrowField(__t13, TestLifetime_S_x);

    call x_ref := CopyOrMoveRef(__t14);
    assume IsValidU64(Dereference(__m, x_ref));
    assume $DebugTrackLocal(0, 1, 5, 935, Dereference(__m, x_ref));

    goto Label_19;

Label_16:
    call __t15 := CopyOrMoveRef(t_ref);

    call __t16 := BorrowField(__t15, TestLifetime_T_x);

    call x_ref := CopyOrMoveRef(__t16);
    assume IsValidU64(Dereference(__m, x_ref));
    assume $DebugTrackLocal(0, 1, 5, 992, Dereference(__m, x_ref));

Label_19:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    __tmp := GetLocal(__m, __frame + 17);
    if (!b#Boolean(__tmp)) { goto Label_25; }

    call __tmp := LdConst(10);
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    call __t19 := CopyOrMoveRef(x_ref);

    call WriteRef(__t19, GetLocal(__m, __frame + 18));
    assume $DebugTrackLocal(0, 1, 2, 1069, GetLocal(__m, __frame + 2));
    assume $DebugTrackLocal(0, 1, 4, 1069, GetLocal(__m, __frame + 4));

    call $TestLifetime_S_update_inv(__before_borrow_0, Dereference(__m, __before_borrow_0_ref));
    call $TestLifetime_T_update_inv(__before_borrow_1, Dereference(__m, __before_borrow_1_ref));
    goto Label_28;

Label_25:
    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 20, __tmp);

    call __t21 := CopyOrMoveRef(x_ref);

    call WriteRef(__t21, GetLocal(__m, __frame + 20));
    assume $DebugTrackLocal(0, 1, 2, 1117, GetLocal(__m, __frame + 2));
    assume $DebugTrackLocal(0, 1, 4, 1117, GetLocal(__m, __frame + 4));

    call $TestLifetime_S_update_inv(__before_borrow_0, Dereference(__m, __before_borrow_0_ref));
    call $TestLifetime_T_update_inv(__before_borrow_1, Dereference(__m, __before_borrow_1_ref));
Label_28:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 22, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 23, __tmp);

    __ret0 := GetLocal(__m, __frame + 22);
    assume $DebugTrackLocal(0, 1, 6, 1187, __ret0);
    __ret1 := GetLocal(__m, __frame + 23);
    assume $DebugTrackLocal(0, 1, 7, 1187, __ret1);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    __ret1 := DefaultValue;
}

procedure TestLifetime_invalid_bool_verify (cond: Value) returns (__ret0: Value, __ret1: Value)
{
    call InitVerification();
    call __ret0, __ret1 := TestLifetime_invalid_bool(cond);
}

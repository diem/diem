

// ** synthetics of module TestInvariants

var __TestInvariants_syn : Value where IsValidU64(__TestInvariants_syn);


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
}

procedure {:inline 1} $TestInvariants_T_update_inv(__before: Value, __after: Value) {
    __TestInvariants_syn := Integer(i#Integer(Integer(i#Integer(__TestInvariants_syn) - i#Integer(SelectField(__before, TestInvariants_T_i)))) + i#Integer(SelectField(__after, TestInvariants_T_i)));
}

procedure {:inline 1} Pack_TestInvariants_T(module_idx: int, func_idx: int, var_idx: int, code_idx: int, i: Value) returns (_struct: Value)
{
    assume IsValidU64(i);
    _struct := Vector(ExtendValueArray(EmptyValueArray, i));
    if (code_idx > 0) { assume $DebugTrackLocal(module_idx, func_idx, var_idx, code_idx, _struct); }
    __TestInvariants_syn := Integer(i#Integer(__TestInvariants_syn) + i#Integer(SelectField(_struct, TestInvariants_T_i)));
}

procedure {:inline 1} Unpack_TestInvariants_T(_struct: Value) returns (i: Value)
{
    assume is#Vector(_struct);
    i := SelectField(_struct, TestInvariants_T_i);
    assume IsValidU64(i);
    __TestInvariants_syn := Integer(i#Integer(__TestInvariants_syn) - i#Integer(SelectField(_struct, TestInvariants_T_i)));
}



// ** functions of module TestInvariants

procedure {:inline 1} TestInvariants_valid () returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(__TestInvariants_syn, Integer(i#Integer(Integer(i#Integer(Integer(i#Integer(Integer(i#Integer(Integer(i#Integer(old(__TestInvariants_syn)) + i#Integer(Integer(2)))) + i#Integer(Integer(3)))) - i#Integer(Integer(3)))) + i#Integer(Integer(4)))) - i#Integer(Integer(2))))));
{
    // declare local variables
    var t: Value; // TestInvariants_T_type_value()
    var r: Value; // TestInvariants_T_type_value()
    var s: Reference; // ReferenceType(TestInvariants_T_type_value())
    var x: Value; // IntegerType()
    var __t4: Value; // IntegerType()
    var __t5: Value; // TestInvariants_T_type_value()
    var __t6: Value; // IntegerType()
    var __t7: Value; // TestInvariants_T_type_value()
    var __t8: Reference; // ReferenceType(TestInvariants_T_type_value())
    var __t9: Value; // IntegerType()
    var __t10: Reference; // ReferenceType(TestInvariants_T_type_value())
    var __t11: Reference; // ReferenceType(IntegerType())
    var __t12: Value; // TestInvariants_T_type_value()
    var __t13: Value; // IntegerType()
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
    __local_counter := __local_counter + 14;

    // bytecode translation starts here
    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := Pack_TestInvariants_T(0, 0, 0, 608, GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(0, 0, 0, 604, __tmp);

    call __tmp := LdConst(3);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := Pack_TestInvariants_T(0, 0, 1, 626, GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(0, 0, 1, 622, __tmp);

    call __t8 := BorrowLoc(__frame + 1, TestInvariants_T_type_value());
    __before_borrow_0 := Dereference(__m, __t8);
    __before_borrow_0_ref := __t8;

    call s := CopyOrMoveRef(__t8);
    assume $TestInvariants_T_is_well_formed(Dereference(__m, s));
    assume $DebugTrackLocal(0, 0, 2, 640, Dereference(__m, s));

    call __tmp := LdConst(4);
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __t10 := CopyOrMoveRef(s);

    call __t11 := BorrowField(__t10, TestInvariants_T_i);

    call WriteRef(__t11, GetLocal(__m, __frame + 9));
    assume $DebugTrackLocal(0, 0, 1, 658, GetLocal(__m, __frame + 1));

    call $TestInvariants_T_update_inv(__before_borrow_0, Dereference(__m, __before_borrow_0_ref));
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    call __t13 := Unpack_TestInvariants_T(GetLocal(__m, __frame + 12));
    __m := UpdateLocal(__m, __frame + 13, __t13);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 13));
    __m := UpdateLocal(__m, __frame + 3, __tmp);
    assume $DebugTrackLocal(0, 0, 3, 687, __tmp);

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure TestInvariants_valid_verify () returns ()
{
    call InitVerification();
    call TestInvariants_valid();
}

procedure {:inline 1} TestInvariants_invalid () returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(__TestInvariants_syn, Integer(i#Integer(old(__TestInvariants_syn)) + i#Integer(Integer(2))))));
{
    // declare local variables
    var t: Value; // TestInvariants_T_type_value()
    var r: Value; // TestInvariants_T_type_value()
    var s: Reference; // ReferenceType(TestInvariants_T_type_value())
    var x: Value; // IntegerType()
    var __t4: Value; // IntegerType()
    var __t5: Value; // TestInvariants_T_type_value()
    var __t6: Value; // IntegerType()
    var __t7: Value; // TestInvariants_T_type_value()
    var __t8: Reference; // ReferenceType(TestInvariants_T_type_value())
    var __t9: Value; // IntegerType()
    var __t10: Reference; // ReferenceType(TestInvariants_T_type_value())
    var __t11: Reference; // ReferenceType(IntegerType())
    var __t12: Value; // TestInvariants_T_type_value()
    var __t13: Value; // IntegerType()
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
    __local_counter := __local_counter + 14;

    // bytecode translation starts here
    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := Pack_TestInvariants_T(0, 1, 0, 913, GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(0, 1, 0, 909, __tmp);

    call __tmp := LdConst(3);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := Pack_TestInvariants_T(0, 1, 1, 931, GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(0, 1, 1, 927, __tmp);

    call __t8 := BorrowLoc(__frame + 1, TestInvariants_T_type_value());
    __before_borrow_0 := Dereference(__m, __t8);
    __before_borrow_0_ref := __t8;

    call s := CopyOrMoveRef(__t8);
    assume $TestInvariants_T_is_well_formed(Dereference(__m, s));
    assume $DebugTrackLocal(0, 1, 2, 945, Dereference(__m, s));

    call __tmp := LdConst(4);
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __t10 := CopyOrMoveRef(s);

    call __t11 := BorrowField(__t10, TestInvariants_T_i);

    call WriteRef(__t11, GetLocal(__m, __frame + 9));
    assume $DebugTrackLocal(0, 1, 1, 963, GetLocal(__m, __frame + 1));

    call $TestInvariants_T_update_inv(__before_borrow_0, Dereference(__m, __before_borrow_0_ref));
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    call __t13 := Unpack_TestInvariants_T(GetLocal(__m, __frame + 12));
    __m := UpdateLocal(__m, __frame + 13, __t13);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 13));
    __m := UpdateLocal(__m, __frame + 3, __tmp);
    assume $DebugTrackLocal(0, 1, 3, 992, __tmp);

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure TestInvariants_invalid_verify () returns ()
{
    call InitVerification();
    call TestInvariants_invalid();
}



// ** structs of module VerifyVector



// ** functions of module VerifyVector

procedure {:inline 1} VerifyVector_test_empty1 () returns (ret0: Value, ret1: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean((ret0) == (ret1)));
{
    // declare local variables
    var t0: Value; // Vector_T_type_value(IntegerType())
    var t1: Value; // Vector_T_type_value(IntegerType())
    var t2: Value; // Vector_T_type_value(IntegerType())
    var t3: Value; // Vector_T_type_value(IntegerType())
    var t4: Value; // Vector_T_type_value(IntegerType())
    var t5: Value; // Vector_T_type_value(IntegerType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 6;

    // process and type check arguments

    // bytecode translation starts here
    call t2 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(t2);

    __m := UpdateLocal(__m, __frame + 2, t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call t3 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(t3);

    __m := UpdateLocal(__m, __frame + 3, t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    ret0 := GetLocal(__m, __frame + 4);
    ret1 := GetLocal(__m, __frame + 5);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
    ret1 := DefaultValue;
}

procedure VerifyVector_test_empty1_verify () returns (ret0: Value, ret1: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0, ret1 := VerifyVector_test_empty1();
}

procedure {:inline 1} VerifyVector_test_empty2 () returns (ret0: Value, ret1: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean((ret0) == (ret1)));
{
    // declare local variables
    var t0: Value; // Vector_T_type_value(IntegerType())
    var t1: Value; // Vector_T_type_value(IntegerType())
    var t2: Value; // IntegerType()
    var t3: Value; // Vector_T_type_value(IntegerType())
    var t4: Value; // Vector_T_type_value(IntegerType())
    var t5: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var t6: Value; // IntegerType()
    var t7: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var t8: Value; // IntegerType()
    var t9: Value; // Vector_T_type_value(IntegerType())
    var t10: Value; // Vector_T_type_value(IntegerType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 11;

    // process and type check arguments

    // bytecode translation starts here
    call t3 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(t3);

    __m := UpdateLocal(__m, __frame + 3, t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call t4 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(t4);

    __m := UpdateLocal(__m, __frame + 4, t4);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call t5 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call Vector_push_back(IntegerType(), t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) { goto Label_Abort; }

    call t7 := BorrowLoc(__frame + 0);

    call t8 := Vector_pop_back(IntegerType(), t7);
    if (__abort_flag) { goto Label_Abort; }
    assume IsValidU64(t8);

    __m := UpdateLocal(__m, __frame + 8, t8);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    ret0 := GetLocal(__m, __frame + 9);
    ret1 := GetLocal(__m, __frame + 10);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
    ret1 := DefaultValue;
}

procedure VerifyVector_test_empty2_verify () returns (ret0: Value, ret1: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0, ret1 := VerifyVector_test_empty2();
}

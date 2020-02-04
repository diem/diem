

// ** structs of module VerifyVector



// ** functions of module VerifyVector

procedure {:inline 1} VerifyVector_test_empty1 () returns (__ret0: Value, __ret1: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(__ret0, __ret1)));
{
    // declare local variables
    var ev1: Value; // Vector_T_type_value(IntegerType())
    var ev2: Value; // Vector_T_type_value(IntegerType())
    var __t2: Value; // Vector_T_type_value(IntegerType())
    var __t3: Value; // Vector_T_type_value(IntegerType())
    var __t4: Value; // Vector_T_type_value(IntegerType())
    var __t5: Value; // Vector_T_type_value(IntegerType())
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
    call __t2 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 0, 219);
      goto Label_Abort;
    }
    assume is#Vector(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 0, 0, 213, __tmp);

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 0, 248);
      goto Label_Abort;
    }
    assume is#Vector(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 0, 1, 242, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    __ret0 := GetLocal(__m, __frame + 4);
    assume $DebugTrackLocal(1, 0, 2, 271, __ret0);
    __ret1 := GetLocal(__m, __frame + 5);
    assume $DebugTrackLocal(1, 0, 3, 271, __ret1);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    __ret1 := DefaultValue;
}

procedure VerifyVector_test_empty1_verify () returns (__ret0: Value, __ret1: Value)
{
    call InitVerification();
    call __ret0, __ret1 := VerifyVector_test_empty1();
}

procedure {:inline 1} VerifyVector_test_empty2 () returns (__ret0: Value, __ret1: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(__ret0, __ret1)));
{
    // declare local variables
    var ev1: Value; // Vector_T_type_value(IntegerType())
    var ev2: Value; // Vector_T_type_value(IntegerType())
    var x: Value; // IntegerType()
    var __t3: Value; // Vector_T_type_value(IntegerType())
    var __t4: Value; // Vector_T_type_value(IntegerType())
    var __t5: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t6: Value; // IntegerType()
    var __t7: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t8: Value; // IntegerType()
    var __t9: Value; // Vector_T_type_value(IntegerType())
    var __t10: Value; // Vector_T_type_value(IntegerType())
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
    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 1, 498);
      goto Label_Abort;
    }
    assume is#Vector(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 1, 0, 492, __tmp);

    call __t4 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 1, 527);
      goto Label_Abort;
    }
    assume is#Vector(__t4);

    __m := UpdateLocal(__m, __frame + 4, __t4);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 1, 1, 521, __tmp);

    call __t5 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call Vector_push_back(IntegerType(), __t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 1, 550);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 1, 0, 550, GetLocal(__m, __frame + 0));

    call __t7 := BorrowLoc(__frame + 0);

    call __t8 := Vector_pop_back(IntegerType(), __t7);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 1, 592);
      goto Label_Abort;
    }
    assume IsValidU64(__t8);

    __m := UpdateLocal(__m, __frame + 8, __t8);
    assume $DebugTrackLocal(1, 1, 0, 592, GetLocal(__m, __frame + 0));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    assume $DebugTrackLocal(1, 1, 2, 588, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    __ret0 := GetLocal(__m, __frame + 9);
    assume $DebugTrackLocal(1, 1, 3, 626, __ret0);
    __ret1 := GetLocal(__m, __frame + 10);
    assume $DebugTrackLocal(1, 1, 4, 626, __ret1);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    __ret1 := DefaultValue;
}

procedure VerifyVector_test_empty2_verify () returns (__ret0: Value, __ret1: Value)
{
    call InitVerification();
    call __ret0, __ret1 := VerifyVector_test_empty2();
}

procedure {:inline 1} VerifyVector_test_empty3 () returns (__ret0: Value, __ret1: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(__ret0, __ret1)));
{
    // declare local variables
    var ev1: Value; // Vector_T_type_value(IntegerType())
    var ev2: Value; // Vector_T_type_value(IntegerType())
    var __t2: Value; // Vector_T_type_value(IntegerType())
    var __t3: Value; // Vector_T_type_value(IntegerType())
    var __t4: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t5: Value; // IntegerType()
    var __t6: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t7: Value; // IntegerType()
    var __t8: Value; // Vector_T_type_value(IntegerType())
    var __t9: Value; // Vector_T_type_value(IntegerType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 10;

    // process and type check arguments

    // bytecode translation starts here
    call __t2 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 2, 838);
      goto Label_Abort;
    }
    assume is#Vector(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 2, 0, 832, __tmp);

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 2, 867);
      goto Label_Abort;
    }
    assume is#Vector(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 2, 1, 861, __tmp);

    call __t4 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call Vector_push_back(IntegerType(), __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 2, 890);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 2, 0, 890, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 2, 1, 890, GetLocal(__m, __frame + 1));

    call __t6 := BorrowLoc(__frame + 1);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call Vector_push_back(IntegerType(), __t6, GetLocal(__m, __frame + 7));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 2, 928);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 2, 0, 928, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 2, 1, 928, GetLocal(__m, __frame + 1));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    __ret0 := GetLocal(__m, __frame + 8);
    assume $DebugTrackLocal(1, 2, 2, 966, __ret0);
    __ret1 := GetLocal(__m, __frame + 9);
    assume $DebugTrackLocal(1, 2, 3, 966, __ret1);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    __ret1 := DefaultValue;
}

procedure VerifyVector_test_empty3_verify () returns (__ret0: Value, __ret1: Value)
{
    call InitVerification();
    call __ret0, __ret1 := VerifyVector_test_empty3();
}

procedure {:inline 1} VerifyVector_test_empty4 () returns (__ret0: Value, __ret1: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(!IsEqual(__ret0, __ret1)));
{
    // declare local variables
    var ev1: Value; // Vector_T_type_value(IntegerType())
    var ev2: Value; // Vector_T_type_value(IntegerType())
    var __t2: Value; // Vector_T_type_value(IntegerType())
    var __t3: Value; // Vector_T_type_value(IntegerType())
    var __t4: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t5: Value; // IntegerType()
    var __t6: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t7: Value; // IntegerType()
    var __t8: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t9: Value; // IntegerType()
    var __t10: Value; // Vector_T_type_value(IntegerType())
    var __t11: Value; // Vector_T_type_value(IntegerType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 12;

    // process and type check arguments

    // bytecode translation starts here
    call __t2 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 3, 1180);
      goto Label_Abort;
    }
    assume is#Vector(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 3, 0, 1174, __tmp);

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 3, 1209);
      goto Label_Abort;
    }
    assume is#Vector(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 3, 1, 1203, __tmp);

    call __t4 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call Vector_push_back(IntegerType(), __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 3, 1232);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 3, 0, 1232, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 3, 1, 1232, GetLocal(__m, __frame + 1));

    call __t6 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call Vector_push_back(IntegerType(), __t6, GetLocal(__m, __frame + 7));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 3, 1270);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 3, 0, 1270, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 3, 1, 1270, GetLocal(__m, __frame + 1));

    call __t8 := BorrowLoc(__frame + 1);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call Vector_push_back(IntegerType(), __t8, GetLocal(__m, __frame + 9));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 3, 1308);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 3, 0, 1308, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 3, 1, 1308, GetLocal(__m, __frame + 1));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    __ret0 := GetLocal(__m, __frame + 10);
    assume $DebugTrackLocal(1, 3, 2, 1346, __ret0);
    __ret1 := GetLocal(__m, __frame + 11);
    assume $DebugTrackLocal(1, 3, 3, 1346, __ret1);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    __ret1 := DefaultValue;
}

procedure VerifyVector_test_empty4_verify () returns (__ret0: Value, __ret1: Value)
{
    call InitVerification();
    call __ret0, __ret1 := VerifyVector_test_empty4();
}

procedure {:inline 1} VerifyVector_test_empty5 () returns (__ret0: Value, __ret1: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(!IsEqual(__ret0, __ret1)));
{
    // declare local variables
    var ev1: Value; // Vector_T_type_value(IntegerType())
    var ev2: Value; // Vector_T_type_value(IntegerType())
    var __t2: Value; // Vector_T_type_value(IntegerType())
    var __t3: Value; // Vector_T_type_value(IntegerType())
    var __t4: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t5: Value; // IntegerType()
    var __t6: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t7: Value; // IntegerType()
    var __t8: Value; // Vector_T_type_value(IntegerType())
    var __t9: Value; // Vector_T_type_value(IntegerType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 10;

    // process and type check arguments

    // bytecode translation starts here
    call __t2 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 4, 1558);
      goto Label_Abort;
    }
    assume is#Vector(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 4, 0, 1552, __tmp);

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 4, 1587);
      goto Label_Abort;
    }
    assume is#Vector(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 4, 1, 1581, __tmp);

    call __t4 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call Vector_push_back(IntegerType(), __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 4, 1610);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 4, 0, 1610, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 4, 1, 1610, GetLocal(__m, __frame + 1));

    call __t6 := BorrowLoc(__frame + 1);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call Vector_push_back(IntegerType(), __t6, GetLocal(__m, __frame + 7));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 4, 1648);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 4, 0, 1648, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 4, 1, 1648, GetLocal(__m, __frame + 1));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    __ret0 := GetLocal(__m, __frame + 8);
    assume $DebugTrackLocal(1, 4, 2, 1686, __ret0);
    __ret1 := GetLocal(__m, __frame + 9);
    assume $DebugTrackLocal(1, 4, 3, 1686, __ret1);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    __ret1 := DefaultValue;
}

procedure VerifyVector_test_empty5_verify () returns (__ret0: Value, __ret1: Value)
{
    call InitVerification();
    call __ret0, __ret1 := VerifyVector_test_empty5();
}

procedure {:inline 1} VerifyVector_test_reverse1 () returns (__ret0: Value, __ret1: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(__ret0, __ret1)));
{
    // declare local variables
    var ev1: Value; // Vector_T_type_value(IntegerType())
    var ev2: Value; // Vector_T_type_value(IntegerType())
    var __t2: Value; // Vector_T_type_value(IntegerType())
    var __t3: Value; // Vector_T_type_value(IntegerType())
    var __t4: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t5: Value; // Vector_T_type_value(IntegerType())
    var __t6: Value; // Vector_T_type_value(IntegerType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 7;

    // process and type check arguments

    // bytecode translation starts here
    call __t2 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 5, 1908);
      goto Label_Abort;
    }
    assume is#Vector(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 5, 0, 1902, __tmp);

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 5, 1937);
      goto Label_Abort;
    }
    assume is#Vector(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 5, 1, 1931, __tmp);

    call __t4 := BorrowLoc(__frame + 0);

    call Vector_reverse(IntegerType(), __t4);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 5, 1960);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 5, 0, 1960, GetLocal(__m, __frame + 0));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    __ret0 := GetLocal(__m, __frame + 5);
    assume $DebugTrackLocal(1, 5, 2, 1993, __ret0);
    __ret1 := GetLocal(__m, __frame + 6);
    assume $DebugTrackLocal(1, 5, 3, 1993, __ret1);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    __ret1 := DefaultValue;
}

procedure VerifyVector_test_reverse1_verify () returns (__ret0: Value, __ret1: Value)
{
    call InitVerification();
    call __ret0, __ret1 := VerifyVector_test_reverse1();
}

procedure {:inline 1} VerifyVector_test_reverse2 () returns (__ret0: Value, __ret1: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(__ret0, __ret1)));
{
    // declare local variables
    var ev1: Value; // Vector_T_type_value(IntegerType())
    var ev2: Value; // Vector_T_type_value(IntegerType())
    var __t2: Value; // Vector_T_type_value(IntegerType())
    var __t3: Value; // Vector_T_type_value(IntegerType())
    var __t4: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t5: Value; // IntegerType()
    var __t6: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t7: Value; // IntegerType()
    var __t8: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t9: Value; // IntegerType()
    var __t10: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t11: Value; // IntegerType()
    var __t12: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t13: Value; // Vector_T_type_value(IntegerType())
    var __t14: Value; // Vector_T_type_value(IntegerType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 15;

    // process and type check arguments

    // bytecode translation starts here
    call __t2 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 6, 2221);
      goto Label_Abort;
    }
    assume is#Vector(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 6, 0, 2215, __tmp);

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 6, 2250);
      goto Label_Abort;
    }
    assume is#Vector(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 6, 1, 2244, __tmp);

    call __t4 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call Vector_push_back(IntegerType(), __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 6, 2273);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 6, 0, 2273, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 6, 1, 2273, GetLocal(__m, __frame + 1));

    call __t6 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call Vector_push_back(IntegerType(), __t6, GetLocal(__m, __frame + 7));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 6, 2311);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 6, 0, 2311, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 6, 1, 2311, GetLocal(__m, __frame + 1));

    call __t8 := BorrowLoc(__frame + 1);

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call Vector_push_back(IntegerType(), __t8, GetLocal(__m, __frame + 9));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 6, 2349);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 6, 0, 2349, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 6, 1, 2349, GetLocal(__m, __frame + 1));

    call __t10 := BorrowLoc(__frame + 1);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call Vector_push_back(IntegerType(), __t10, GetLocal(__m, __frame + 11));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 6, 2387);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 6, 0, 2387, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 6, 1, 2387, GetLocal(__m, __frame + 1));

    call __t12 := BorrowLoc(__frame + 0);

    call Vector_reverse(IntegerType(), __t12);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 6, 2425);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 6, 0, 2425, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 6, 1, 2425, GetLocal(__m, __frame + 1));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    __ret0 := GetLocal(__m, __frame + 13);
    assume $DebugTrackLocal(1, 6, 2, 2458, __ret0);
    __ret1 := GetLocal(__m, __frame + 14);
    assume $DebugTrackLocal(1, 6, 3, 2458, __ret1);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    __ret1 := DefaultValue;
}

procedure VerifyVector_test_reverse2_verify () returns (__ret0: Value, __ret1: Value)
{
    call InitVerification();
    call __ret0, __ret1 := VerifyVector_test_reverse2();
}

procedure {:inline 1} VerifyVector_test_swap1 () returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(true)))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(true))) ==> __abort_flag;

{
    // declare local variables
    var ev1: Value; // Vector_T_type_value(IntegerType())
    var __t1: Value; // Vector_T_type_value(IntegerType())
    var __t2: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t3: Value; // IntegerType()
    var __t4: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t5: Value; // IntegerType()
    var __t6: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 7;

    // process and type check arguments

    // bytecode translation starts here
    call __t1 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 7, 2659);
      goto Label_Abort;
    }
    assume is#Vector(__t1);

    __m := UpdateLocal(__m, __frame + 1, __t1);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 7, 0, 2653, __tmp);

    call __t2 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call Vector_push_back(IntegerType(), __t2, GetLocal(__m, __frame + 3));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 7, 2682);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 7, 0, 2682, GetLocal(__m, __frame + 0));

    call __t4 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call Vector_swap(IntegerType(), __t4, GetLocal(__m, __frame + 5), GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 7, 2720);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 7, 0, 2720, GetLocal(__m, __frame + 0));

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure VerifyVector_test_swap1_verify () returns ()
{
    call InitVerification();
    call VerifyVector_test_swap1();
}

procedure {:inline 1} VerifyVector_test_swap2 () returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(true)))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(true))) ==> __abort_flag;

{
    // declare local variables
    var ev1: Value; // Vector_T_type_value(IntegerType())
    var __t1: Value; // Vector_T_type_value(IntegerType())
    var __t2: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t3: Value; // IntegerType()
    var __t4: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t5: Value; // IntegerType()
    var __t6: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 7;

    // process and type check arguments

    // bytecode translation starts here
    call __t1 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 8, 2932);
      goto Label_Abort;
    }
    assume is#Vector(__t1);

    __m := UpdateLocal(__m, __frame + 1, __t1);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 8, 0, 2926, __tmp);

    call __t2 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call Vector_push_back(IntegerType(), __t2, GetLocal(__m, __frame + 3));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 8, 2955);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 8, 0, 2955, GetLocal(__m, __frame + 0));

    call __t4 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call Vector_swap(IntegerType(), __t4, GetLocal(__m, __frame + 5), GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 8, 2993);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 8, 0, 2993, GetLocal(__m, __frame + 0));

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure VerifyVector_test_swap2_verify () returns ()
{
    call InitVerification();
    call VerifyVector_test_swap2();
}

procedure {:inline 1} VerifyVector_test_swap3 () returns (__ret0: Value, __ret1: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(__ret0, __ret1)));
{
    // declare local variables
    var ev1: Value; // Vector_T_type_value(IntegerType())
    var ev2: Value; // Vector_T_type_value(IntegerType())
    var __t2: Value; // Vector_T_type_value(IntegerType())
    var __t3: Value; // Vector_T_type_value(IntegerType())
    var __t4: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t5: Value; // IntegerType()
    var __t6: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t7: Value; // IntegerType()
    var __t8: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t9: Value; // IntegerType()
    var __t10: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t11: Value; // IntegerType()
    var __t12: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t13: Value; // IntegerType()
    var __t14: Value; // IntegerType()
    var __t15: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t16: Value; // IntegerType()
    var __t17: Value; // IntegerType()
    var __t18: Value; // Vector_T_type_value(IntegerType())
    var __t19: Value; // Vector_T_type_value(IntegerType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 20;

    // process and type check arguments

    // bytecode translation starts here
    call __t2 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 9, 3229);
      goto Label_Abort;
    }
    assume is#Vector(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 9, 0, 3223, __tmp);

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 9, 3258);
      goto Label_Abort;
    }
    assume is#Vector(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 9, 1, 3252, __tmp);

    call __t4 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call Vector_push_back(IntegerType(), __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 9, 3281);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 9, 0, 3281, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 9, 1, 3281, GetLocal(__m, __frame + 1));

    call __t6 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call Vector_push_back(IntegerType(), __t6, GetLocal(__m, __frame + 7));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 9, 3319);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 9, 0, 3319, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 9, 1, 3319, GetLocal(__m, __frame + 1));

    call __t8 := BorrowLoc(__frame + 1);

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call Vector_push_back(IntegerType(), __t8, GetLocal(__m, __frame + 9));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 9, 3357);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 9, 0, 3357, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 9, 1, 3357, GetLocal(__m, __frame + 1));

    call __t10 := BorrowLoc(__frame + 1);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call Vector_push_back(IntegerType(), __t10, GetLocal(__m, __frame + 11));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 9, 3395);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 9, 0, 3395, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 9, 1, 3395, GetLocal(__m, __frame + 1));

    call __t12 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call Vector_swap(IntegerType(), __t12, GetLocal(__m, __frame + 13), GetLocal(__m, __frame + 14));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 9, 3433);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 9, 0, 3433, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 9, 1, 3433, GetLocal(__m, __frame + 1));

    call __t15 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    call Vector_swap(IntegerType(), __t15, GetLocal(__m, __frame + 16), GetLocal(__m, __frame + 17));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 9, 3469);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 9, 0, 3469, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 9, 1, 3469, GetLocal(__m, __frame + 1));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 19, __tmp);

    __ret0 := GetLocal(__m, __frame + 18);
    assume $DebugTrackLocal(1, 9, 2, 3505, __ret0);
    __ret1 := GetLocal(__m, __frame + 19);
    assume $DebugTrackLocal(1, 9, 3, 3505, __ret1);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    __ret1 := DefaultValue;
}

procedure VerifyVector_test_swap3_verify () returns (__ret0: Value, __ret1: Value)
{
    call InitVerification();
    call __ret0, __ret1 := VerifyVector_test_swap3();
}

procedure {:inline 1} VerifyVector_test_length1 () returns (__ret0: Value, __ret1: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(__ret0, Integer(i#Integer(__ret1) + i#Integer(Integer(1))))));
{
    // declare local variables
    var ev1: Value; // Vector_T_type_value(IntegerType())
    var ev2: Value; // Vector_T_type_value(IntegerType())
    var __t2: Value; // Vector_T_type_value(IntegerType())
    var __t3: Value; // Vector_T_type_value(IntegerType())
    var __t4: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t5: Value; // IntegerType()
    var __t6: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t7: Value; // IntegerType()
    var __t8: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t9: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 10;

    // process and type check arguments

    // bytecode translation starts here
    call __t2 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 10, 3724);
      goto Label_Abort;
    }
    assume is#Vector(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 10, 0, 3718, __tmp);

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 10, 3753);
      goto Label_Abort;
    }
    assume is#Vector(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 10, 1, 3747, __tmp);

    call __t4 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call Vector_push_back(IntegerType(), __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 10, 3776);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 10, 0, 3776, GetLocal(__m, __frame + 0));

    call __t6 := BorrowLoc(__frame + 0);

    call __t7 := Vector_length(IntegerType(), __t6);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 10, 3822);
      goto Label_Abort;
    }
    assume IsValidU64(__t7);

    __m := UpdateLocal(__m, __frame + 7, __t7);

    call __t8 := BorrowLoc(__frame + 1);

    call __t9 := Vector_length(IntegerType(), __t8);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 10, 3849);
      goto Label_Abort;
    }
    assume IsValidU64(__t9);

    __m := UpdateLocal(__m, __frame + 9, __t9);

    __ret0 := GetLocal(__m, __frame + 7);
    assume $DebugTrackLocal(1, 10, 2, 3814, __ret0);
    __ret1 := GetLocal(__m, __frame + 9);
    assume $DebugTrackLocal(1, 10, 3, 3814, __ret1);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    __ret1 := DefaultValue;
}

procedure VerifyVector_test_length1_verify () returns (__ret0: Value, __ret1: Value)
{
    call InitVerification();
    call __ret0, __ret1 := VerifyVector_test_length1();
}

procedure {:inline 1} VerifyVector_test_length2 (v: Value) returns (__ret0: Value, __ret1: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(Integer(i#Integer(__ret0) + i#Integer(Integer(3))), __ret1)));
{
    // declare local variables
    var x: Value; // IntegerType()
    var y: Value; // IntegerType()
    var __t3: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t4: Value; // IntegerType()
    var __t5: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t6: Value; // IntegerType()
    var __t7: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t8: Value; // IntegerType()
    var __t9: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t10: Value; // IntegerType()
    var __t11: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t12: Value; // IntegerType()
    var __t13: Value; // IntegerType()
    var __t14: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 15;

    // process and type check arguments
    assume is#Vector(v);
    __m := UpdateLocal(__m, __frame + 0, v);
    assume $DebugTrackLocal(1, 11, 0, 3942, v);

    // bytecode translation starts here
    call __t3 := BorrowLoc(__frame + 0);

    call __t4 := Vector_length(IntegerType(), __t3);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 11, 4065);
      goto Label_Abort;
    }
    assume IsValidU64(__t4);

    __m := UpdateLocal(__m, __frame + 4, __t4);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 11, 1, 4061, __tmp);

    call __t5 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call Vector_push_back(IntegerType(), __t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 11, 4092);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 11, 0, 4092, GetLocal(__m, __frame + 0));

    call __t7 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call Vector_push_back(IntegerType(), __t7, GetLocal(__m, __frame + 8));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 11, 4128);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 11, 0, 4128, GetLocal(__m, __frame + 0));

    call __t9 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(3);
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call Vector_push_back(IntegerType(), __t9, GetLocal(__m, __frame + 10));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 11, 4164);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 11, 0, 4164, GetLocal(__m, __frame + 0));

    call __t11 := BorrowLoc(__frame + 0);

    call __t12 := Vector_length(IntegerType(), __t11);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 11, 4204);
      goto Label_Abort;
    }
    assume IsValidU64(__t12);

    __m := UpdateLocal(__m, __frame + 12, __t12);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 12));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    assume $DebugTrackLocal(1, 11, 2, 4200, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    __ret0 := GetLocal(__m, __frame + 13);
    assume $DebugTrackLocal(1, 11, 3, 4231, __ret0);
    __ret1 := GetLocal(__m, __frame + 14);
    assume $DebugTrackLocal(1, 11, 4, 4231, __ret1);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    __ret1 := DefaultValue;
}

procedure VerifyVector_test_length2_verify (v: Value) returns (__ret0: Value, __ret1: Value)
{
    call InitVerification();
    call __ret0, __ret1 := VerifyVector_test_length2(v);
}

procedure {:inline 1} VerifyVector_test_id1 (v: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(__ret0, old(v))));
{
    // declare local variables
    var __t1: Value; // Vector_T_type_value(IntegerType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 2;

    // process and type check arguments
    assume is#Vector(v);
    __m := UpdateLocal(__m, __frame + 0, v);
    assume $DebugTrackLocal(1, 12, 0, 4288, v);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    __ret0 := GetLocal(__m, __frame + 1);
    assume $DebugTrackLocal(1, 12, 1, 4375, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure VerifyVector_test_id1_verify (v: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := VerifyVector_test_id1(v);
}

procedure {:inline 1} VerifyVector_test_id2 (v: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(__ret0, old(v))));
{
    // declare local variables
    var __t1: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t2: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t3: Value; // Vector_T_type_value(IntegerType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 4;

    // process and type check arguments
    assume is#Vector(v);
    __m := UpdateLocal(__m, __frame + 0, v);
    assume $DebugTrackLocal(1, 13, 0, 4441, v);

    // bytecode translation starts here
    call __t1 := BorrowLoc(__frame + 0);

    call Vector_reverse(IntegerType(), __t1);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 13, 4528);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 13, 0, 4528, GetLocal(__m, __frame + 0));

    call __t2 := BorrowLoc(__frame + 0);

    call Vector_reverse(IntegerType(), __t2);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 13, 4559);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 13, 0, 4559, GetLocal(__m, __frame + 0));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    __ret0 := GetLocal(__m, __frame + 3);
    assume $DebugTrackLocal(1, 13, 1, 4590, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure VerifyVector_test_id2_verify (v: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := VerifyVector_test_id2(v);
}

procedure {:inline 1} VerifyVector_test_id3 (v: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(__ret0, old(v))));
{
    // declare local variables
    var l: Value; // IntegerType()
    var __t2: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t3: Value; // IntegerType()
    var __t4: Value; // IntegerType()
    var __t5: Value; // IntegerType()
    var __t6: Value; // BooleanType()
    var __t7: Value; // IntegerType()
    var __t8: Value; // IntegerType()
    var __t9: Value; // BooleanType()
    var __t10: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t11: Value; // IntegerType()
    var __t12: Value; // IntegerType()
    var __t13: Value; // IntegerType()
    var __t14: Value; // IntegerType()
    var __t15: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t16: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t17: Value; // Vector_T_type_value(IntegerType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 18;

    // process and type check arguments
    assume is#Vector(v);
    __m := UpdateLocal(__m, __frame + 0, v);
    assume $DebugTrackLocal(1, 14, 0, 4677, v);

    // bytecode translation starts here
    call __t2 := BorrowLoc(__frame + 0);

    call __t3 := Vector_length(IntegerType(), __t2);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 14, 4782);
      goto Label_Abort;
    }
    assume IsValidU64(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 14, 1, 4778, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := Le(GetLocal(__m, __frame + 4), GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    __tmp := GetLocal(__m, __frame + 6);
    if (!b#Boolean(__tmp)) { goto Label_8; }

    goto Label_21;

Label_8:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := LdConst(3);
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __tmp := Le(GetLocal(__m, __frame + 7), GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    __tmp := GetLocal(__m, __frame + 9);
    if (!b#Boolean(__tmp)) { goto Label_19; }

    call __t10 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __tmp := Sub(GetLocal(__m, __frame + 12), GetLocal(__m, __frame + 13));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 14, 4896);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call Vector_swap(IntegerType(), __t10, GetLocal(__m, __frame + 11), GetLocal(__m, __frame + 14));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 14, 4868);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 14, 0, 4868, GetLocal(__m, __frame + 0));

    goto Label_21;

Label_19:
    call __t15 := BorrowLoc(__frame + 0);

    call Vector_reverse(IntegerType(), __t15);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 14, 4927);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 14, 0, 4927, GetLocal(__m, __frame + 0));

Label_21:
    call __t16 := BorrowLoc(__frame + 0);

    call Vector_reverse(IntegerType(), __t16);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 14, 4967);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 14, 0, 4967, GetLocal(__m, __frame + 0));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    __ret0 := GetLocal(__m, __frame + 17);
    assume $DebugTrackLocal(1, 14, 2, 4998, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure VerifyVector_test_id3_verify (v: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := VerifyVector_test_id3(v);
}

procedure {:inline 1} VerifyVector_test_destroy_empty1 (v: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(__ret0, old(v))));
{
    // declare local variables
    var __t1: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t2: Value; // BooleanType()
    var __t3: Value; // Vector_T_type_value(IntegerType())
    var __t4: Value; // Vector_T_type_value(IntegerType())
    var __t5: Value; // Vector_T_type_value(IntegerType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 6;

    // process and type check arguments
    assume is#Vector(v);
    __m := UpdateLocal(__m, __frame + 0, v);
    assume $DebugTrackLocal(1, 15, 0, 5111, v);

    // bytecode translation starts here
    call __t1 := BorrowLoc(__frame + 0);

    call __t2 := Vector_is_empty(IntegerType(), __t1);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 15, 5210);
      goto Label_Abort;
    }
    assume is#Boolean(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    __tmp := GetLocal(__m, __frame + 2);
    if (!b#Boolean(__tmp)) { goto Label_7; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call Vector_destroy_empty(IntegerType(), GetLocal(__m, __frame + 3));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 15, 5242);
      goto Label_Abort;
    }

    call __t4 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 15, 5288);
      goto Label_Abort;
    }
    assume is#Vector(__t4);

    __m := UpdateLocal(__m, __frame + 4, __t4);

    __ret0 := GetLocal(__m, __frame + 4);
    assume $DebugTrackLocal(1, 15, 1, 5281, __ret0);
    return;

Label_7:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    __ret0 := GetLocal(__m, __frame + 5);
    assume $DebugTrackLocal(1, 15, 1, 5325, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure VerifyVector_test_destroy_empty1_verify (v: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := VerifyVector_test_destroy_empty1(v);
}

procedure {:inline 1} VerifyVector_test_destroy_empty2 (v: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(true)))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(true))) ==> __abort_flag;

{
    // declare local variables
    var __t1: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t2: Value; // BooleanType()
    var __t3: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t4: Value; // IntegerType()
    var __t5: Value; // IntegerType()
    var __t6: Value; // Vector_T_type_value(IntegerType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 7;

    // process and type check arguments
    assume is#Vector(v);
    __m := UpdateLocal(__m, __frame + 0, v);
    assume $DebugTrackLocal(1, 16, 0, 5479, v);

    // bytecode translation starts here
    call __t1 := BorrowLoc(__frame + 0);

    call __t2 := Vector_is_empty(IntegerType(), __t1);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 16, 5549);
      goto Label_Abort;
    }
    assume is#Boolean(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    __tmp := GetLocal(__m, __frame + 2);
    if (!b#Boolean(__tmp)) { goto Label_8; }

    call __t3 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call Vector_set(IntegerType(), __t3, GetLocal(__m, __frame + 4), GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 16, 5581);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 16, 0, 5581, GetLocal(__m, __frame + 0));

    goto Label_10;

Label_8:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call Vector_destroy_empty(IntegerType(), GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 16, 5628);
      goto Label_Abort;
    }

Label_10:
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure VerifyVector_test_destroy_empty2_verify (v: Value) returns ()
{
    call InitVerification();
    call VerifyVector_test_destroy_empty2(v);
}

procedure {:inline 1} VerifyVector_test_get_set1 (x: Value) returns (__ret0: Value, __ret1: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(__ret0, __ret1)));
{
    // declare local variables
    var ev1: Value; // Vector_T_type_value(IntegerType())
    var ev2: Value; // Vector_T_type_value(IntegerType())
    var __t3: Value; // Vector_T_type_value(IntegerType())
    var __t4: Value; // Vector_T_type_value(IntegerType())
    var __t5: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t6: Value; // IntegerType()
    var __t7: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t8: Value; // IntegerType()
    var __t9: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t10: Value; // IntegerType()
    var __t11: Value; // IntegerType()
    var __t12: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t13: Value; // IntegerType()
    var __t14: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t15: Value; // IntegerType()
    var __t16: Value; // IntegerType()
    var __t17: Value; // Vector_T_type_value(IntegerType())
    var __t18: Value; // Vector_T_type_value(IntegerType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 19;

    // process and type check arguments
    assume IsValidU64(x);
    __m := UpdateLocal(__m, __frame + 0, x);
    assume $DebugTrackLocal(1, 17, 0, 5709, x);

    // bytecode translation starts here
    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 17, 5865);
      goto Label_Abort;
    }
    assume is#Vector(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 17, 1, 5859, __tmp);

    call __t4 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 17, 5894);
      goto Label_Abort;
    }
    assume is#Vector(__t4);

    __m := UpdateLocal(__m, __frame + 4, __t4);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    assume $DebugTrackLocal(1, 17, 2, 5888, __tmp);

    call __t5 := BorrowLoc(__frame + 1);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call Vector_push_back(IntegerType(), __t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 17, 5917);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 17, 1, 5917, GetLocal(__m, __frame + 1));
    assume $DebugTrackLocal(1, 17, 2, 5917, GetLocal(__m, __frame + 2));

    call __t7 := BorrowLoc(__frame + 2);

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call Vector_push_back(IntegerType(), __t7, GetLocal(__m, __frame + 8));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 17, 5955);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 17, 1, 5955, GetLocal(__m, __frame + 1));
    assume $DebugTrackLocal(1, 17, 2, 5955, GetLocal(__m, __frame + 2));

    call __t9 := BorrowLoc(__frame + 1);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call Vector_set(IntegerType(), __t9, GetLocal(__m, __frame + 10), GetLocal(__m, __frame + 11));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 17, 5993);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 17, 1, 5993, GetLocal(__m, __frame + 1));
    assume $DebugTrackLocal(1, 17, 2, 5993, GetLocal(__m, __frame + 2));

    call __t12 := BorrowLoc(__frame + 2);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __t14 := BorrowLoc(__frame + 1);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 15, __tmp);

    call __t16 := Vector_get(IntegerType(), __t14, GetLocal(__m, __frame + 15));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 17, 6063);
      goto Label_Abort;
    }
    assume IsValidU64(__t16);

    __m := UpdateLocal(__m, __frame + 16, __t16);

    call Vector_set(IntegerType(), __t12, GetLocal(__m, __frame + 13), GetLocal(__m, __frame + 16));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 17, 6034);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 17, 1, 6034, GetLocal(__m, __frame + 1));
    assume $DebugTrackLocal(1, 17, 2, 6034, GetLocal(__m, __frame + 2));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    __ret0 := GetLocal(__m, __frame + 17);
    assume $DebugTrackLocal(1, 17, 3, 6093, __ret0);
    __ret1 := GetLocal(__m, __frame + 18);
    assume $DebugTrackLocal(1, 17, 4, 6093, __ret1);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    __ret1 := DefaultValue;
}

procedure VerifyVector_test_get_set1_verify (x: Value) returns (__ret0: Value, __ret1: Value)
{
    call InitVerification();
    call __ret0, __ret1 := VerifyVector_test_get_set1(x);
}

procedure {:inline 1} VerifyVector_test_get1 () returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(__ret0, Integer(7))));
{
    // declare local variables
    var ev1: Value; // Vector_T_type_value(IntegerType())
    var __t1: Value; // Vector_T_type_value(IntegerType())
    var __t2: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t3: Value; // IntegerType()
    var __t4: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t5: Value; // IntegerType()
    var __t6: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 7;

    // process and type check arguments

    // bytecode translation starts here
    call __t1 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 18, 6238);
      goto Label_Abort;
    }
    assume is#Vector(__t1);

    __m := UpdateLocal(__m, __frame + 1, __t1);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 18, 0, 6232, __tmp);

    call __t2 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(7);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call Vector_push_back(IntegerType(), __t2, GetLocal(__m, __frame + 3));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 18, 6261);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 18, 0, 6261, GetLocal(__m, __frame + 0));

    call __t4 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __t6 := Vector_get(IntegerType(), __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 18, 6306);
      goto Label_Abort;
    }
    assume IsValidU64(__t6);

    __m := UpdateLocal(__m, __frame + 6, __t6);

    __ret0 := GetLocal(__m, __frame + 6);
    assume $DebugTrackLocal(1, 18, 1, 6299, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure VerifyVector_test_get1_verify () returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := VerifyVector_test_get1();
}

procedure {:inline 1} VerifyVector_test_get2 () returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(true)))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(true))) ==> __abort_flag;

{
    // declare local variables
    var x: Value; // IntegerType()
    var ev1: Value; // Vector_T_type_value(IntegerType())
    var __t2: Value; // Vector_T_type_value(IntegerType())
    var __t3: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t4: Value; // IntegerType()
    var __t5: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t6: Value; // IntegerType()
    var __t7: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 8;

    // process and type check arguments

    // bytecode translation starts here
    call __t2 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 19, 6620);
      goto Label_Abort;
    }
    assume is#Vector(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 19, 1, 6614, __tmp);

    call __t3 := BorrowLoc(__frame + 1);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call Vector_push_back(IntegerType(), __t3, GetLocal(__m, __frame + 4));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 19, 6643);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 19, 1, 6643, GetLocal(__m, __frame + 1));

    call __t5 := BorrowLoc(__frame + 1);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __t7 := Vector_get(IntegerType(), __t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 19, 6685);
      goto Label_Abort;
    }
    assume IsValidU64(__t7);

    __m := UpdateLocal(__m, __frame + 7, __t7);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 19, 0, 6681, __tmp);

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure VerifyVector_test_get2_verify () returns ()
{
    call InitVerification();
    call VerifyVector_test_get2();
}

procedure {:inline 1} VerifyVector_test_borrow1 () returns (__ret0: Value, __ret1: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(__ret0, __ret1)));
{
    // declare local variables
    var ev1: Value; // Vector_T_type_value(IntegerType())
    var y: Reference; // ReferenceType(IntegerType())
    var __t2: Value; // Vector_T_type_value(IntegerType())
    var __t3: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t4: Value; // IntegerType()
    var __t5: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t6: Value; // IntegerType()
    var __t7: Reference; // ReferenceType(IntegerType())
    var __t8: Value; // IntegerType()
    var __t9: Reference; // ReferenceType(IntegerType())
    var __t10: Value; // IntegerType()
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
    call __t2 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 20, 6871);
      goto Label_Abort;
    }
    assume is#Vector(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 20, 0, 6865, __tmp);

    call __t3 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(7);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call Vector_push_back(IntegerType(), __t3, GetLocal(__m, __frame + 4));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 20, 6894);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 20, 0, 6894, GetLocal(__m, __frame + 0));

    call __t5 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __t7 := Vector_borrow(IntegerType(), __t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 20, 6936);
      goto Label_Abort;
    }
    assume IsValidU64(Dereference(__m, __t7));
    assume IsValidReferenceParameter(__m, __frame, __t7);



    call y := CopyOrMoveRef(__t7);
    assume IsValidU64(Dereference(__m, y));
    assume $DebugTrackLocal(1, 20, 1, 6932, Dereference(__m, y));

    call __tmp := LdConst(7);
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __t9 := CopyOrMoveRef(y);

    call __tmp := ReadRef(__t9);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    __ret0 := GetLocal(__m, __frame + 8);
    assume $DebugTrackLocal(1, 20, 2, 6967, __ret0);
    __ret1 := GetLocal(__m, __frame + 10);
    assume $DebugTrackLocal(1, 20, 3, 6967, __ret1);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    __ret1 := DefaultValue;
}

procedure VerifyVector_test_borrow1_verify () returns (__ret0: Value, __ret1: Value)
{
    call InitVerification();
    call __ret0, __ret1 := VerifyVector_test_borrow1();
}

procedure {:inline 1} VerifyVector_test_borrow2 () returns (__ret0: Value, __ret1: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(__ret0, __ret1)));
{
    // declare local variables
    var ev1: Value; // Vector_T_type_value(IntegerType())
    var y: Reference; // ReferenceType(IntegerType())
    var __t2: Value; // Vector_T_type_value(IntegerType())
    var __t3: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t4: Value; // IntegerType()
    var __t5: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t6: Value; // IntegerType()
    var __t7: Reference; // ReferenceType(IntegerType())
    var __t8: Value; // IntegerType()
    var __t9: Reference; // ReferenceType(IntegerType())
    var __t10: Value; // IntegerType()
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
    call __t2 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 21, 7247);
      goto Label_Abort;
    }
    assume is#Vector(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 21, 0, 7241, __tmp);

    call __t3 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(7);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call Vector_push_back(IntegerType(), __t3, GetLocal(__m, __frame + 4));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 21, 7270);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 21, 0, 7270, GetLocal(__m, __frame + 0));

    call __t5 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __t7 := Vector_borrow(IntegerType(), __t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 21, 7312);
      goto Label_Abort;
    }
    assume IsValidU64(Dereference(__m, __t7));
    assume IsValidReferenceParameter(__m, __frame, __t7);



    call y := CopyOrMoveRef(__t7);
    assume IsValidU64(Dereference(__m, y));
    assume $DebugTrackLocal(1, 21, 1, 7308, Dereference(__m, y));

    call __tmp := LdConst(7);
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __t9 := CopyOrMoveRef(y);

    call __tmp := ReadRef(__t9);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    __ret0 := GetLocal(__m, __frame + 8);
    assume $DebugTrackLocal(1, 21, 2, 7343, __ret0);
    __ret1 := GetLocal(__m, __frame + 10);
    assume $DebugTrackLocal(1, 21, 3, 7343, __ret1);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    __ret1 := DefaultValue;
}

procedure VerifyVector_test_borrow2_verify () returns (__ret0: Value, __ret1: Value)
{
    call InitVerification();
    call __ret0, __ret1 := VerifyVector_test_borrow2();
}

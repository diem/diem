

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
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    __ret0 := GetLocal(__m, __frame + 4);
    __ret1 := GetLocal(__m, __frame + 5);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    __ret1 := DefaultValue;
}

procedure VerifyVector_test_empty1_verify () returns (__ret0: Value, __ret1: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call __t4 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t4);

    __m := UpdateLocal(__m, __frame + 4, __t4);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __t5 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call Vector_push_back(IntegerType(), __t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) { goto Label_Abort; }

    call __t7 := BorrowLoc(__frame + 0);

    call __t8 := Vector_pop_back(IntegerType(), __t7);
    if (__abort_flag) { goto Label_Abort; }
    assume IsValidU64(__t8);

    __m := UpdateLocal(__m, __frame + 8, __t8);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    __ret0 := GetLocal(__m, __frame + 9);
    __ret1 := GetLocal(__m, __frame + 10);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    __ret1 := DefaultValue;
}

procedure VerifyVector_test_empty2_verify () returns (__ret0: Value, __ret1: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __t4 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call Vector_push_back(IntegerType(), __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) { goto Label_Abort; }

    call __t6 := BorrowLoc(__frame + 1);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call Vector_push_back(IntegerType(), __t6, GetLocal(__m, __frame + 7));
    if (__abort_flag) { goto Label_Abort; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    __ret0 := GetLocal(__m, __frame + 8);
    __ret1 := GetLocal(__m, __frame + 9);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    __ret1 := DefaultValue;
}

procedure VerifyVector_test_empty3_verify () returns (__ret0: Value, __ret1: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __t4 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call Vector_push_back(IntegerType(), __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) { goto Label_Abort; }

    call __t6 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call Vector_push_back(IntegerType(), __t6, GetLocal(__m, __frame + 7));
    if (__abort_flag) { goto Label_Abort; }

    call __t8 := BorrowLoc(__frame + 1);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call Vector_push_back(IntegerType(), __t8, GetLocal(__m, __frame + 9));
    if (__abort_flag) { goto Label_Abort; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    __ret0 := GetLocal(__m, __frame + 10);
    __ret1 := GetLocal(__m, __frame + 11);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    __ret1 := DefaultValue;
}

procedure VerifyVector_test_empty4_verify () returns (__ret0: Value, __ret1: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __t4 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call Vector_push_back(IntegerType(), __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) { goto Label_Abort; }

    call __t6 := BorrowLoc(__frame + 1);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call Vector_push_back(IntegerType(), __t6, GetLocal(__m, __frame + 7));
    if (__abort_flag) { goto Label_Abort; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    __ret0 := GetLocal(__m, __frame + 8);
    __ret1 := GetLocal(__m, __frame + 9);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    __ret1 := DefaultValue;
}

procedure VerifyVector_test_empty5_verify () returns (__ret0: Value, __ret1: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __t4 := BorrowLoc(__frame + 0);

    call Vector_reverse(IntegerType(), __t4);
    if (__abort_flag) { goto Label_Abort; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    __ret0 := GetLocal(__m, __frame + 5);
    __ret1 := GetLocal(__m, __frame + 6);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    __ret1 := DefaultValue;
}

procedure VerifyVector_test_reverse1_verify () returns (__ret0: Value, __ret1: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __t4 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call Vector_push_back(IntegerType(), __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) { goto Label_Abort; }

    call __t6 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call Vector_push_back(IntegerType(), __t6, GetLocal(__m, __frame + 7));
    if (__abort_flag) { goto Label_Abort; }

    call __t8 := BorrowLoc(__frame + 1);

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call Vector_push_back(IntegerType(), __t8, GetLocal(__m, __frame + 9));
    if (__abort_flag) { goto Label_Abort; }

    call __t10 := BorrowLoc(__frame + 1);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call Vector_push_back(IntegerType(), __t10, GetLocal(__m, __frame + 11));
    if (__abort_flag) { goto Label_Abort; }

    call __t12 := BorrowLoc(__frame + 0);

    call Vector_reverse(IntegerType(), __t12);
    if (__abort_flag) { goto Label_Abort; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    __ret0 := GetLocal(__m, __frame + 13);
    __ret1 := GetLocal(__m, __frame + 14);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    __ret1 := DefaultValue;
}

procedure VerifyVector_test_reverse2_verify () returns (__ret0: Value, __ret1: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t1);

    __m := UpdateLocal(__m, __frame + 1, __t1);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call __t2 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call Vector_push_back(IntegerType(), __t2, GetLocal(__m, __frame + 3));
    if (__abort_flag) { goto Label_Abort; }

    call __t4 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call Vector_swap(IntegerType(), __t4, GetLocal(__m, __frame + 5), GetLocal(__m, __frame + 6));
    if (__abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure VerifyVector_test_swap1_verify () returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t1);

    __m := UpdateLocal(__m, __frame + 1, __t1);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call __t2 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call Vector_push_back(IntegerType(), __t2, GetLocal(__m, __frame + 3));
    if (__abort_flag) { goto Label_Abort; }

    call __t4 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call Vector_swap(IntegerType(), __t4, GetLocal(__m, __frame + 5), GetLocal(__m, __frame + 6));
    if (__abort_flag) { goto Label_Abort; }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure VerifyVector_test_swap2_verify () returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __t4 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call Vector_push_back(IntegerType(), __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) { goto Label_Abort; }

    call __t6 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call Vector_push_back(IntegerType(), __t6, GetLocal(__m, __frame + 7));
    if (__abort_flag) { goto Label_Abort; }

    call __t8 := BorrowLoc(__frame + 1);

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call Vector_push_back(IntegerType(), __t8, GetLocal(__m, __frame + 9));
    if (__abort_flag) { goto Label_Abort; }

    call __t10 := BorrowLoc(__frame + 1);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call Vector_push_back(IntegerType(), __t10, GetLocal(__m, __frame + 11));
    if (__abort_flag) { goto Label_Abort; }

    call __t12 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call Vector_swap(IntegerType(), __t12, GetLocal(__m, __frame + 13), GetLocal(__m, __frame + 14));
    if (__abort_flag) { goto Label_Abort; }

    call __t15 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    call Vector_swap(IntegerType(), __t15, GetLocal(__m, __frame + 16), GetLocal(__m, __frame + 17));
    if (__abort_flag) { goto Label_Abort; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 19, __tmp);

    __ret0 := GetLocal(__m, __frame + 18);
    __ret1 := GetLocal(__m, __frame + 19);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    __ret1 := DefaultValue;
}

procedure VerifyVector_test_swap3_verify () returns (__ret0: Value, __ret1: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __t4 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call Vector_push_back(IntegerType(), __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) { goto Label_Abort; }

    call __t6 := BorrowLoc(__frame + 0);

    call __t7 := Vector_length(IntegerType(), __t6);
    if (__abort_flag) { goto Label_Abort; }
    assume IsValidU64(__t7);

    __m := UpdateLocal(__m, __frame + 7, __t7);

    call __t8 := BorrowLoc(__frame + 1);

    call __t9 := Vector_length(IntegerType(), __t8);
    if (__abort_flag) { goto Label_Abort; }
    assume IsValidU64(__t9);

    __m := UpdateLocal(__m, __frame + 9, __t9);

    __ret0 := GetLocal(__m, __frame + 7);
    __ret1 := GetLocal(__m, __frame + 9);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    __ret1 := DefaultValue;
}

procedure VerifyVector_test_length1_verify () returns (__ret0: Value, __ret1: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
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

    // bytecode translation starts here
    call __t3 := BorrowLoc(__frame + 0);

    call __t4 := Vector_length(IntegerType(), __t3);
    if (__abort_flag) { goto Label_Abort; }
    assume IsValidU64(__t4);

    __m := UpdateLocal(__m, __frame + 4, __t4);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __t5 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call Vector_push_back(IntegerType(), __t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) { goto Label_Abort; }

    call __t7 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call Vector_push_back(IntegerType(), __t7, GetLocal(__m, __frame + 8));
    if (__abort_flag) { goto Label_Abort; }

    call __t9 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(3);
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call Vector_push_back(IntegerType(), __t9, GetLocal(__m, __frame + 10));
    if (__abort_flag) { goto Label_Abort; }

    call __t11 := BorrowLoc(__frame + 0);

    call __t12 := Vector_length(IntegerType(), __t11);
    if (__abort_flag) { goto Label_Abort; }
    assume IsValidU64(__t12);

    __m := UpdateLocal(__m, __frame + 12, __t12);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 12));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    __ret0 := GetLocal(__m, __frame + 13);
    __ret1 := GetLocal(__m, __frame + 14);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    __ret1 := DefaultValue;
}

procedure VerifyVector_test_length2_verify (v: Value) returns (__ret0: Value, __ret1: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
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

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    __ret0 := GetLocal(__m, __frame + 1);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure VerifyVector_test_id1_verify (v: Value) returns (__ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
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

    // bytecode translation starts here
    call __t1 := BorrowLoc(__frame + 0);

    call Vector_reverse(IntegerType(), __t1);
    if (__abort_flag) { goto Label_Abort; }

    call __t2 := BorrowLoc(__frame + 0);

    call Vector_reverse(IntegerType(), __t2);
    if (__abort_flag) { goto Label_Abort; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    __ret0 := GetLocal(__m, __frame + 3);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure VerifyVector_test_id2_verify (v: Value) returns (__ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
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

    // bytecode translation starts here
    call __t2 := BorrowLoc(__frame + 0);

    call __t3 := Vector_length(IntegerType(), __t2);
    if (__abort_flag) { goto Label_Abort; }
    assume IsValidU64(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

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
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call Vector_swap(IntegerType(), __t10, GetLocal(__m, __frame + 11), GetLocal(__m, __frame + 14));
    if (__abort_flag) { goto Label_Abort; }

    goto Label_21;

Label_19:
    call __t15 := BorrowLoc(__frame + 0);

    call Vector_reverse(IntegerType(), __t15);
    if (__abort_flag) { goto Label_Abort; }

Label_21:
    call __t16 := BorrowLoc(__frame + 0);

    call Vector_reverse(IntegerType(), __t16);
    if (__abort_flag) { goto Label_Abort; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    __ret0 := GetLocal(__m, __frame + 17);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure VerifyVector_test_id3_verify (v: Value) returns (__ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
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

    // bytecode translation starts here
    call __t1 := BorrowLoc(__frame + 0);

    call __t2 := Vector_is_empty(IntegerType(), __t1);
    if (__abort_flag) { goto Label_Abort; }
    assume is#Boolean(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    __tmp := GetLocal(__m, __frame + 2);
    if (!b#Boolean(__tmp)) { goto Label_7; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call Vector_destroy_empty(IntegerType(), GetLocal(__m, __frame + 3));
    if (__abort_flag) { goto Label_Abort; }

    call __t4 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t4);

    __m := UpdateLocal(__m, __frame + 4, __t4);

    __ret0 := GetLocal(__m, __frame + 4);
    return;

Label_7:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    __ret0 := GetLocal(__m, __frame + 5);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure VerifyVector_test_destroy_empty1_verify (v: Value) returns (__ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
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

    // bytecode translation starts here
    call __t1 := BorrowLoc(__frame + 0);

    call __t2 := Vector_is_empty(IntegerType(), __t1);
    if (__abort_flag) { goto Label_Abort; }
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
    if (__abort_flag) { goto Label_Abort; }

    goto Label_10;

Label_8:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call Vector_destroy_empty(IntegerType(), GetLocal(__m, __frame + 6));
    if (__abort_flag) { goto Label_Abort; }

Label_10:
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure VerifyVector_test_destroy_empty2_verify (v: Value) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
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

    // bytecode translation starts here
    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __t4 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t4);

    __m := UpdateLocal(__m, __frame + 4, __t4);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __t5 := BorrowLoc(__frame + 1);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call Vector_push_back(IntegerType(), __t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) { goto Label_Abort; }

    call __t7 := BorrowLoc(__frame + 2);

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call Vector_push_back(IntegerType(), __t7, GetLocal(__m, __frame + 8));
    if (__abort_flag) { goto Label_Abort; }

    call __t9 := BorrowLoc(__frame + 1);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call Vector_set(IntegerType(), __t9, GetLocal(__m, __frame + 10), GetLocal(__m, __frame + 11));
    if (__abort_flag) { goto Label_Abort; }

    call __t12 := BorrowLoc(__frame + 2);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __t14 := BorrowLoc(__frame + 1);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 15, __tmp);

    call __t16 := Vector_get(IntegerType(), __t14, GetLocal(__m, __frame + 15));
    if (__abort_flag) { goto Label_Abort; }
    assume IsValidU64(__t16);

    __m := UpdateLocal(__m, __frame + 16, __t16);

    call Vector_set(IntegerType(), __t12, GetLocal(__m, __frame + 13), GetLocal(__m, __frame + 16));
    if (__abort_flag) { goto Label_Abort; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    __ret0 := GetLocal(__m, __frame + 17);
    __ret1 := GetLocal(__m, __frame + 18);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    __ret1 := DefaultValue;
}

procedure VerifyVector_test_get_set1_verify (x: Value) returns (__ret0: Value, __ret1: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t1);

    __m := UpdateLocal(__m, __frame + 1, __t1);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call __t2 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(7);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call Vector_push_back(IntegerType(), __t2, GetLocal(__m, __frame + 3));
    if (__abort_flag) { goto Label_Abort; }

    call __t4 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __t6 := Vector_get(IntegerType(), __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) { goto Label_Abort; }
    assume IsValidU64(__t6);

    __m := UpdateLocal(__m, __frame + 6, __t6);

    __ret0 := GetLocal(__m, __frame + 6);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure VerifyVector_test_get1_verify () returns (__ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __t3 := BorrowLoc(__frame + 1);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call Vector_push_back(IntegerType(), __t3, GetLocal(__m, __frame + 4));
    if (__abort_flag) { goto Label_Abort; }

    call __t5 := BorrowLoc(__frame + 1);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __t7 := Vector_get(IntegerType(), __t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) { goto Label_Abort; }
    assume IsValidU64(__t7);

    __m := UpdateLocal(__m, __frame + 7, __t7);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure VerifyVector_test_get2_verify () returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call __t3 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(7);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call Vector_push_back(IntegerType(), __t3, GetLocal(__m, __frame + 4));
    if (__abort_flag) { goto Label_Abort; }

    call __t5 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __t7 := Vector_borrow(IntegerType(), __t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) { goto Label_Abort; }
    assume IsValidU64(Dereference(__m, __t7));
    assume IsValidReferenceParameter(__m, __frame, __t7);


    call y := CopyOrMoveRef(__t7);

    call __tmp := LdConst(7);
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __t9 := CopyOrMoveRef(y);

    call __tmp := ReadRef(__t9);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    __ret0 := GetLocal(__m, __frame + 8);
    __ret1 := GetLocal(__m, __frame + 10);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    __ret1 := DefaultValue;
}

procedure VerifyVector_test_borrow1_verify () returns (__ret0: Value, __ret1: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call __t3 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(7);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call Vector_push_back(IntegerType(), __t3, GetLocal(__m, __frame + 4));
    if (__abort_flag) { goto Label_Abort; }

    call __t5 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __t7 := Vector_borrow(IntegerType(), __t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) { goto Label_Abort; }
    assume IsValidU64(Dereference(__m, __t7));
    assume IsValidReferenceParameter(__m, __frame, __t7);


    call y := CopyOrMoveRef(__t7);

    call __tmp := LdConst(7);
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __t9 := CopyOrMoveRef(y);

    call __tmp := ReadRef(__t9);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    __ret0 := GetLocal(__m, __frame + 8);
    __ret1 := GetLocal(__m, __frame + 10);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    __ret1 := DefaultValue;
}

procedure VerifyVector_test_borrow2_verify () returns (__ret0: Value, __ret1: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call __ret0, __ret1 := VerifyVector_test_borrow2();
}

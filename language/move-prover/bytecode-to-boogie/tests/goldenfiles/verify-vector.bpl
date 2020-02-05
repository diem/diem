

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
    var debug#VerifyVector#test_empty1#0#ev1: [Position]Value;
    var debug#VerifyVector#test_empty1#1#ev2: [Position]Value;
    var debug#VerifyVector#test_empty1#2#__ret: [Position]Value;
    var debug#VerifyVector#test_empty1#3#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 6;
    debug#VerifyVector#test_empty1#0#ev1 := EmptyPositionMap;
    debug#VerifyVector#test_empty1#1#ev2 := EmptyPositionMap;
    debug#VerifyVector#test_empty1#2#__ret := EmptyPositionMap;
    debug#VerifyVector#test_empty1#3#__ret := EmptyPositionMap;

    // process and type check arguments

    // bytecode translation starts here
    call __t2 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    debug#VerifyVector#test_empty1#0#ev1 := debug#VerifyVector#test_empty1#0#ev1[Position(213) := __tmp];

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    debug#VerifyVector#test_empty1#1#ev2 := debug#VerifyVector#test_empty1#1#ev2[Position(242) := __tmp];

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    __ret0 := GetLocal(__m, __frame + 4);
    debug#VerifyVector#test_empty1#2#__ret := debug#VerifyVector#test_empty1#2#__ret[Position(271) := __ret0];
    __ret1 := GetLocal(__m, __frame + 5);
    debug#VerifyVector#test_empty1#3#__ret := debug#VerifyVector#test_empty1#3#__ret[Position(271) := __ret1];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#VerifyVector#test_empty1#2#__ret := debug#VerifyVector#test_empty1#2#__ret[Position(306) := __ret0];
    __ret1 := DefaultValue;
    debug#VerifyVector#test_empty1#3#__ret := debug#VerifyVector#test_empty1#3#__ret[Position(306) := __ret1];
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
    var debug#VerifyVector#test_empty2#0#ev1: [Position]Value;
    var debug#VerifyVector#test_empty2#1#ev2: [Position]Value;
    var debug#VerifyVector#test_empty2#2#x: [Position]Value;
    var debug#VerifyVector#test_empty2#3#__ret: [Position]Value;
    var debug#VerifyVector#test_empty2#4#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 11;
    debug#VerifyVector#test_empty2#0#ev1 := EmptyPositionMap;
    debug#VerifyVector#test_empty2#1#ev2 := EmptyPositionMap;
    debug#VerifyVector#test_empty2#2#x := EmptyPositionMap;
    debug#VerifyVector#test_empty2#3#__ret := EmptyPositionMap;
    debug#VerifyVector#test_empty2#4#__ret := EmptyPositionMap;

    // process and type check arguments

    // bytecode translation starts here
    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    debug#VerifyVector#test_empty2#0#ev1 := debug#VerifyVector#test_empty2#0#ev1[Position(492) := __tmp];

    call __t4 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t4);

    __m := UpdateLocal(__m, __frame + 4, __t4);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    debug#VerifyVector#test_empty2#1#ev2 := debug#VerifyVector#test_empty2#1#ev2[Position(521) := __tmp];

    call __t5 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call Vector_push_back(IntegerType(), __t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_empty2#0#ev1 := debug#VerifyVector#test_empty2#0#ev1[Position(550) := GetLocal(__m, __frame + 0)];

    call __t7 := BorrowLoc(__frame + 0);

    call __t8 := Vector_pop_back(IntegerType(), __t7);
    if (__abort_flag) { goto Label_Abort; }
    assume IsValidU64(__t8);

    __m := UpdateLocal(__m, __frame + 8, __t8);
    debug#VerifyVector#test_empty2#0#ev1 := debug#VerifyVector#test_empty2#0#ev1[Position(592) := GetLocal(__m, __frame + 0)];

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    debug#VerifyVector#test_empty2#2#x := debug#VerifyVector#test_empty2#2#x[Position(588) := __tmp];

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    __ret0 := GetLocal(__m, __frame + 9);
    debug#VerifyVector#test_empty2#3#__ret := debug#VerifyVector#test_empty2#3#__ret[Position(626) := __ret0];
    __ret1 := GetLocal(__m, __frame + 10);
    debug#VerifyVector#test_empty2#4#__ret := debug#VerifyVector#test_empty2#4#__ret[Position(626) := __ret1];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#VerifyVector#test_empty2#3#__ret := debug#VerifyVector#test_empty2#3#__ret[Position(661) := __ret0];
    __ret1 := DefaultValue;
    debug#VerifyVector#test_empty2#4#__ret := debug#VerifyVector#test_empty2#4#__ret[Position(661) := __ret1];
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
    var debug#VerifyVector#test_empty3#0#ev1: [Position]Value;
    var debug#VerifyVector#test_empty3#1#ev2: [Position]Value;
    var debug#VerifyVector#test_empty3#2#__ret: [Position]Value;
    var debug#VerifyVector#test_empty3#3#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 10;
    debug#VerifyVector#test_empty3#0#ev1 := EmptyPositionMap;
    debug#VerifyVector#test_empty3#1#ev2 := EmptyPositionMap;
    debug#VerifyVector#test_empty3#2#__ret := EmptyPositionMap;
    debug#VerifyVector#test_empty3#3#__ret := EmptyPositionMap;

    // process and type check arguments

    // bytecode translation starts here
    call __t2 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    debug#VerifyVector#test_empty3#0#ev1 := debug#VerifyVector#test_empty3#0#ev1[Position(832) := __tmp];

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    debug#VerifyVector#test_empty3#1#ev2 := debug#VerifyVector#test_empty3#1#ev2[Position(861) := __tmp];

    call __t4 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call Vector_push_back(IntegerType(), __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_empty3#0#ev1 := debug#VerifyVector#test_empty3#0#ev1[Position(890) := GetLocal(__m, __frame + 0)];
    debug#VerifyVector#test_empty3#1#ev2 := debug#VerifyVector#test_empty3#1#ev2[Position(890) := GetLocal(__m, __frame + 1)];

    call __t6 := BorrowLoc(__frame + 1);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call Vector_push_back(IntegerType(), __t6, GetLocal(__m, __frame + 7));
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_empty3#0#ev1 := debug#VerifyVector#test_empty3#0#ev1[Position(928) := GetLocal(__m, __frame + 0)];
    debug#VerifyVector#test_empty3#1#ev2 := debug#VerifyVector#test_empty3#1#ev2[Position(928) := GetLocal(__m, __frame + 1)];

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    __ret0 := GetLocal(__m, __frame + 8);
    debug#VerifyVector#test_empty3#2#__ret := debug#VerifyVector#test_empty3#2#__ret[Position(966) := __ret0];
    __ret1 := GetLocal(__m, __frame + 9);
    debug#VerifyVector#test_empty3#3#__ret := debug#VerifyVector#test_empty3#3#__ret[Position(966) := __ret1];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#VerifyVector#test_empty3#2#__ret := debug#VerifyVector#test_empty3#2#__ret[Position(1001) := __ret0];
    __ret1 := DefaultValue;
    debug#VerifyVector#test_empty3#3#__ret := debug#VerifyVector#test_empty3#3#__ret[Position(1001) := __ret1];
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
    var debug#VerifyVector#test_empty4#0#ev1: [Position]Value;
    var debug#VerifyVector#test_empty4#1#ev2: [Position]Value;
    var debug#VerifyVector#test_empty4#2#__ret: [Position]Value;
    var debug#VerifyVector#test_empty4#3#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 12;
    debug#VerifyVector#test_empty4#0#ev1 := EmptyPositionMap;
    debug#VerifyVector#test_empty4#1#ev2 := EmptyPositionMap;
    debug#VerifyVector#test_empty4#2#__ret := EmptyPositionMap;
    debug#VerifyVector#test_empty4#3#__ret := EmptyPositionMap;

    // process and type check arguments

    // bytecode translation starts here
    call __t2 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    debug#VerifyVector#test_empty4#0#ev1 := debug#VerifyVector#test_empty4#0#ev1[Position(1174) := __tmp];

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    debug#VerifyVector#test_empty4#1#ev2 := debug#VerifyVector#test_empty4#1#ev2[Position(1203) := __tmp];

    call __t4 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call Vector_push_back(IntegerType(), __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_empty4#0#ev1 := debug#VerifyVector#test_empty4#0#ev1[Position(1232) := GetLocal(__m, __frame + 0)];
    debug#VerifyVector#test_empty4#1#ev2 := debug#VerifyVector#test_empty4#1#ev2[Position(1232) := GetLocal(__m, __frame + 1)];

    call __t6 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call Vector_push_back(IntegerType(), __t6, GetLocal(__m, __frame + 7));
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_empty4#0#ev1 := debug#VerifyVector#test_empty4#0#ev1[Position(1270) := GetLocal(__m, __frame + 0)];
    debug#VerifyVector#test_empty4#1#ev2 := debug#VerifyVector#test_empty4#1#ev2[Position(1270) := GetLocal(__m, __frame + 1)];

    call __t8 := BorrowLoc(__frame + 1);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call Vector_push_back(IntegerType(), __t8, GetLocal(__m, __frame + 9));
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_empty4#0#ev1 := debug#VerifyVector#test_empty4#0#ev1[Position(1308) := GetLocal(__m, __frame + 0)];
    debug#VerifyVector#test_empty4#1#ev2 := debug#VerifyVector#test_empty4#1#ev2[Position(1308) := GetLocal(__m, __frame + 1)];

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    __ret0 := GetLocal(__m, __frame + 10);
    debug#VerifyVector#test_empty4#2#__ret := debug#VerifyVector#test_empty4#2#__ret[Position(1346) := __ret0];
    __ret1 := GetLocal(__m, __frame + 11);
    debug#VerifyVector#test_empty4#3#__ret := debug#VerifyVector#test_empty4#3#__ret[Position(1346) := __ret1];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#VerifyVector#test_empty4#2#__ret := debug#VerifyVector#test_empty4#2#__ret[Position(1381) := __ret0];
    __ret1 := DefaultValue;
    debug#VerifyVector#test_empty4#3#__ret := debug#VerifyVector#test_empty4#3#__ret[Position(1381) := __ret1];
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
    var debug#VerifyVector#test_empty5#0#ev1: [Position]Value;
    var debug#VerifyVector#test_empty5#1#ev2: [Position]Value;
    var debug#VerifyVector#test_empty5#2#__ret: [Position]Value;
    var debug#VerifyVector#test_empty5#3#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 10;
    debug#VerifyVector#test_empty5#0#ev1 := EmptyPositionMap;
    debug#VerifyVector#test_empty5#1#ev2 := EmptyPositionMap;
    debug#VerifyVector#test_empty5#2#__ret := EmptyPositionMap;
    debug#VerifyVector#test_empty5#3#__ret := EmptyPositionMap;

    // process and type check arguments

    // bytecode translation starts here
    call __t2 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    debug#VerifyVector#test_empty5#0#ev1 := debug#VerifyVector#test_empty5#0#ev1[Position(1552) := __tmp];

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    debug#VerifyVector#test_empty5#1#ev2 := debug#VerifyVector#test_empty5#1#ev2[Position(1581) := __tmp];

    call __t4 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call Vector_push_back(IntegerType(), __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_empty5#0#ev1 := debug#VerifyVector#test_empty5#0#ev1[Position(1610) := GetLocal(__m, __frame + 0)];
    debug#VerifyVector#test_empty5#1#ev2 := debug#VerifyVector#test_empty5#1#ev2[Position(1610) := GetLocal(__m, __frame + 1)];

    call __t6 := BorrowLoc(__frame + 1);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call Vector_push_back(IntegerType(), __t6, GetLocal(__m, __frame + 7));
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_empty5#0#ev1 := debug#VerifyVector#test_empty5#0#ev1[Position(1648) := GetLocal(__m, __frame + 0)];
    debug#VerifyVector#test_empty5#1#ev2 := debug#VerifyVector#test_empty5#1#ev2[Position(1648) := GetLocal(__m, __frame + 1)];

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    __ret0 := GetLocal(__m, __frame + 8);
    debug#VerifyVector#test_empty5#2#__ret := debug#VerifyVector#test_empty5#2#__ret[Position(1686) := __ret0];
    __ret1 := GetLocal(__m, __frame + 9);
    debug#VerifyVector#test_empty5#3#__ret := debug#VerifyVector#test_empty5#3#__ret[Position(1686) := __ret1];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#VerifyVector#test_empty5#2#__ret := debug#VerifyVector#test_empty5#2#__ret[Position(1721) := __ret0];
    __ret1 := DefaultValue;
    debug#VerifyVector#test_empty5#3#__ret := debug#VerifyVector#test_empty5#3#__ret[Position(1721) := __ret1];
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
    var debug#VerifyVector#test_reverse1#0#ev1: [Position]Value;
    var debug#VerifyVector#test_reverse1#1#ev2: [Position]Value;
    var debug#VerifyVector#test_reverse1#2#__ret: [Position]Value;
    var debug#VerifyVector#test_reverse1#3#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 7;
    debug#VerifyVector#test_reverse1#0#ev1 := EmptyPositionMap;
    debug#VerifyVector#test_reverse1#1#ev2 := EmptyPositionMap;
    debug#VerifyVector#test_reverse1#2#__ret := EmptyPositionMap;
    debug#VerifyVector#test_reverse1#3#__ret := EmptyPositionMap;

    // process and type check arguments

    // bytecode translation starts here
    call __t2 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    debug#VerifyVector#test_reverse1#0#ev1 := debug#VerifyVector#test_reverse1#0#ev1[Position(1902) := __tmp];

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    debug#VerifyVector#test_reverse1#1#ev2 := debug#VerifyVector#test_reverse1#1#ev2[Position(1931) := __tmp];

    call __t4 := BorrowLoc(__frame + 0);

    call Vector_reverse(IntegerType(), __t4);
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_reverse1#0#ev1 := debug#VerifyVector#test_reverse1#0#ev1[Position(1960) := GetLocal(__m, __frame + 0)];

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    __ret0 := GetLocal(__m, __frame + 5);
    debug#VerifyVector#test_reverse1#2#__ret := debug#VerifyVector#test_reverse1#2#__ret[Position(1993) := __ret0];
    __ret1 := GetLocal(__m, __frame + 6);
    debug#VerifyVector#test_reverse1#3#__ret := debug#VerifyVector#test_reverse1#3#__ret[Position(1993) := __ret1];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#VerifyVector#test_reverse1#2#__ret := debug#VerifyVector#test_reverse1#2#__ret[Position(2028) := __ret0];
    __ret1 := DefaultValue;
    debug#VerifyVector#test_reverse1#3#__ret := debug#VerifyVector#test_reverse1#3#__ret[Position(2028) := __ret1];
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
    var debug#VerifyVector#test_reverse2#0#ev1: [Position]Value;
    var debug#VerifyVector#test_reverse2#1#ev2: [Position]Value;
    var debug#VerifyVector#test_reverse2#2#__ret: [Position]Value;
    var debug#VerifyVector#test_reverse2#3#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 15;
    debug#VerifyVector#test_reverse2#0#ev1 := EmptyPositionMap;
    debug#VerifyVector#test_reverse2#1#ev2 := EmptyPositionMap;
    debug#VerifyVector#test_reverse2#2#__ret := EmptyPositionMap;
    debug#VerifyVector#test_reverse2#3#__ret := EmptyPositionMap;

    // process and type check arguments

    // bytecode translation starts here
    call __t2 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    debug#VerifyVector#test_reverse2#0#ev1 := debug#VerifyVector#test_reverse2#0#ev1[Position(2215) := __tmp];

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    debug#VerifyVector#test_reverse2#1#ev2 := debug#VerifyVector#test_reverse2#1#ev2[Position(2244) := __tmp];

    call __t4 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call Vector_push_back(IntegerType(), __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_reverse2#0#ev1 := debug#VerifyVector#test_reverse2#0#ev1[Position(2273) := GetLocal(__m, __frame + 0)];
    debug#VerifyVector#test_reverse2#1#ev2 := debug#VerifyVector#test_reverse2#1#ev2[Position(2273) := GetLocal(__m, __frame + 1)];

    call __t6 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call Vector_push_back(IntegerType(), __t6, GetLocal(__m, __frame + 7));
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_reverse2#0#ev1 := debug#VerifyVector#test_reverse2#0#ev1[Position(2311) := GetLocal(__m, __frame + 0)];
    debug#VerifyVector#test_reverse2#1#ev2 := debug#VerifyVector#test_reverse2#1#ev2[Position(2311) := GetLocal(__m, __frame + 1)];

    call __t8 := BorrowLoc(__frame + 1);

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call Vector_push_back(IntegerType(), __t8, GetLocal(__m, __frame + 9));
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_reverse2#0#ev1 := debug#VerifyVector#test_reverse2#0#ev1[Position(2349) := GetLocal(__m, __frame + 0)];
    debug#VerifyVector#test_reverse2#1#ev2 := debug#VerifyVector#test_reverse2#1#ev2[Position(2349) := GetLocal(__m, __frame + 1)];

    call __t10 := BorrowLoc(__frame + 1);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call Vector_push_back(IntegerType(), __t10, GetLocal(__m, __frame + 11));
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_reverse2#0#ev1 := debug#VerifyVector#test_reverse2#0#ev1[Position(2387) := GetLocal(__m, __frame + 0)];
    debug#VerifyVector#test_reverse2#1#ev2 := debug#VerifyVector#test_reverse2#1#ev2[Position(2387) := GetLocal(__m, __frame + 1)];

    call __t12 := BorrowLoc(__frame + 0);

    call Vector_reverse(IntegerType(), __t12);
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_reverse2#0#ev1 := debug#VerifyVector#test_reverse2#0#ev1[Position(2425) := GetLocal(__m, __frame + 0)];
    debug#VerifyVector#test_reverse2#1#ev2 := debug#VerifyVector#test_reverse2#1#ev2[Position(2425) := GetLocal(__m, __frame + 1)];

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    __ret0 := GetLocal(__m, __frame + 13);
    debug#VerifyVector#test_reverse2#2#__ret := debug#VerifyVector#test_reverse2#2#__ret[Position(2458) := __ret0];
    __ret1 := GetLocal(__m, __frame + 14);
    debug#VerifyVector#test_reverse2#3#__ret := debug#VerifyVector#test_reverse2#3#__ret[Position(2458) := __ret1];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#VerifyVector#test_reverse2#2#__ret := debug#VerifyVector#test_reverse2#2#__ret[Position(2493) := __ret0];
    __ret1 := DefaultValue;
    debug#VerifyVector#test_reverse2#3#__ret := debug#VerifyVector#test_reverse2#3#__ret[Position(2493) := __ret1];
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
    var debug#VerifyVector#test_swap1#0#ev1: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 7;
    debug#VerifyVector#test_swap1#0#ev1 := EmptyPositionMap;

    // process and type check arguments

    // bytecode translation starts here
    call __t1 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t1);

    __m := UpdateLocal(__m, __frame + 1, __t1);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    debug#VerifyVector#test_swap1#0#ev1 := debug#VerifyVector#test_swap1#0#ev1[Position(2653) := __tmp];

    call __t2 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call Vector_push_back(IntegerType(), __t2, GetLocal(__m, __frame + 3));
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_swap1#0#ev1 := debug#VerifyVector#test_swap1#0#ev1[Position(2682) := GetLocal(__m, __frame + 0)];

    call __t4 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call Vector_swap(IntegerType(), __t4, GetLocal(__m, __frame + 5), GetLocal(__m, __frame + 6));
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_swap1#0#ev1 := debug#VerifyVector#test_swap1#0#ev1[Position(2720) := GetLocal(__m, __frame + 0)];

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
    var debug#VerifyVector#test_swap2#0#ev1: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 7;
    debug#VerifyVector#test_swap2#0#ev1 := EmptyPositionMap;

    // process and type check arguments

    // bytecode translation starts here
    call __t1 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t1);

    __m := UpdateLocal(__m, __frame + 1, __t1);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    debug#VerifyVector#test_swap2#0#ev1 := debug#VerifyVector#test_swap2#0#ev1[Position(2926) := __tmp];

    call __t2 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call Vector_push_back(IntegerType(), __t2, GetLocal(__m, __frame + 3));
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_swap2#0#ev1 := debug#VerifyVector#test_swap2#0#ev1[Position(2955) := GetLocal(__m, __frame + 0)];

    call __t4 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call Vector_swap(IntegerType(), __t4, GetLocal(__m, __frame + 5), GetLocal(__m, __frame + 6));
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_swap2#0#ev1 := debug#VerifyVector#test_swap2#0#ev1[Position(2993) := GetLocal(__m, __frame + 0)];

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
    var debug#VerifyVector#test_swap3#0#ev1: [Position]Value;
    var debug#VerifyVector#test_swap3#1#ev2: [Position]Value;
    var debug#VerifyVector#test_swap3#2#__ret: [Position]Value;
    var debug#VerifyVector#test_swap3#3#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 20;
    debug#VerifyVector#test_swap3#0#ev1 := EmptyPositionMap;
    debug#VerifyVector#test_swap3#1#ev2 := EmptyPositionMap;
    debug#VerifyVector#test_swap3#2#__ret := EmptyPositionMap;
    debug#VerifyVector#test_swap3#3#__ret := EmptyPositionMap;

    // process and type check arguments

    // bytecode translation starts here
    call __t2 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    debug#VerifyVector#test_swap3#0#ev1 := debug#VerifyVector#test_swap3#0#ev1[Position(3223) := __tmp];

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    debug#VerifyVector#test_swap3#1#ev2 := debug#VerifyVector#test_swap3#1#ev2[Position(3252) := __tmp];

    call __t4 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call Vector_push_back(IntegerType(), __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_swap3#0#ev1 := debug#VerifyVector#test_swap3#0#ev1[Position(3281) := GetLocal(__m, __frame + 0)];
    debug#VerifyVector#test_swap3#1#ev2 := debug#VerifyVector#test_swap3#1#ev2[Position(3281) := GetLocal(__m, __frame + 1)];

    call __t6 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call Vector_push_back(IntegerType(), __t6, GetLocal(__m, __frame + 7));
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_swap3#0#ev1 := debug#VerifyVector#test_swap3#0#ev1[Position(3319) := GetLocal(__m, __frame + 0)];
    debug#VerifyVector#test_swap3#1#ev2 := debug#VerifyVector#test_swap3#1#ev2[Position(3319) := GetLocal(__m, __frame + 1)];

    call __t8 := BorrowLoc(__frame + 1);

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call Vector_push_back(IntegerType(), __t8, GetLocal(__m, __frame + 9));
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_swap3#0#ev1 := debug#VerifyVector#test_swap3#0#ev1[Position(3357) := GetLocal(__m, __frame + 0)];
    debug#VerifyVector#test_swap3#1#ev2 := debug#VerifyVector#test_swap3#1#ev2[Position(3357) := GetLocal(__m, __frame + 1)];

    call __t10 := BorrowLoc(__frame + 1);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call Vector_push_back(IntegerType(), __t10, GetLocal(__m, __frame + 11));
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_swap3#0#ev1 := debug#VerifyVector#test_swap3#0#ev1[Position(3395) := GetLocal(__m, __frame + 0)];
    debug#VerifyVector#test_swap3#1#ev2 := debug#VerifyVector#test_swap3#1#ev2[Position(3395) := GetLocal(__m, __frame + 1)];

    call __t12 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call Vector_swap(IntegerType(), __t12, GetLocal(__m, __frame + 13), GetLocal(__m, __frame + 14));
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_swap3#0#ev1 := debug#VerifyVector#test_swap3#0#ev1[Position(3433) := GetLocal(__m, __frame + 0)];
    debug#VerifyVector#test_swap3#1#ev2 := debug#VerifyVector#test_swap3#1#ev2[Position(3433) := GetLocal(__m, __frame + 1)];

    call __t15 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    call Vector_swap(IntegerType(), __t15, GetLocal(__m, __frame + 16), GetLocal(__m, __frame + 17));
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_swap3#0#ev1 := debug#VerifyVector#test_swap3#0#ev1[Position(3469) := GetLocal(__m, __frame + 0)];
    debug#VerifyVector#test_swap3#1#ev2 := debug#VerifyVector#test_swap3#1#ev2[Position(3469) := GetLocal(__m, __frame + 1)];

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 19, __tmp);

    __ret0 := GetLocal(__m, __frame + 18);
    debug#VerifyVector#test_swap3#2#__ret := debug#VerifyVector#test_swap3#2#__ret[Position(3505) := __ret0];
    __ret1 := GetLocal(__m, __frame + 19);
    debug#VerifyVector#test_swap3#3#__ret := debug#VerifyVector#test_swap3#3#__ret[Position(3505) := __ret1];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#VerifyVector#test_swap3#2#__ret := debug#VerifyVector#test_swap3#2#__ret[Position(3540) := __ret0];
    __ret1 := DefaultValue;
    debug#VerifyVector#test_swap3#3#__ret := debug#VerifyVector#test_swap3#3#__ret[Position(3540) := __ret1];
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
    var debug#VerifyVector#test_length1#0#ev1: [Position]Value;
    var debug#VerifyVector#test_length1#1#ev2: [Position]Value;
    var debug#VerifyVector#test_length1#2#__ret: [Position]Value;
    var debug#VerifyVector#test_length1#3#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 10;
    debug#VerifyVector#test_length1#0#ev1 := EmptyPositionMap;
    debug#VerifyVector#test_length1#1#ev2 := EmptyPositionMap;
    debug#VerifyVector#test_length1#2#__ret := EmptyPositionMap;
    debug#VerifyVector#test_length1#3#__ret := EmptyPositionMap;

    // process and type check arguments

    // bytecode translation starts here
    call __t2 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    debug#VerifyVector#test_length1#0#ev1 := debug#VerifyVector#test_length1#0#ev1[Position(3718) := __tmp];

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    debug#VerifyVector#test_length1#1#ev2 := debug#VerifyVector#test_length1#1#ev2[Position(3747) := __tmp];

    call __t4 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call Vector_push_back(IntegerType(), __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_length1#0#ev1 := debug#VerifyVector#test_length1#0#ev1[Position(3776) := GetLocal(__m, __frame + 0)];

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
    debug#VerifyVector#test_length1#2#__ret := debug#VerifyVector#test_length1#2#__ret[Position(3814) := __ret0];
    __ret1 := GetLocal(__m, __frame + 9);
    debug#VerifyVector#test_length1#3#__ret := debug#VerifyVector#test_length1#3#__ret[Position(3814) := __ret1];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#VerifyVector#test_length1#2#__ret := debug#VerifyVector#test_length1#2#__ret[Position(3881) := __ret0];
    __ret1 := DefaultValue;
    debug#VerifyVector#test_length1#3#__ret := debug#VerifyVector#test_length1#3#__ret[Position(3881) := __ret1];
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
    var debug#VerifyVector#test_length2#0#v: [Position]Value;
    var debug#VerifyVector#test_length2#1#x: [Position]Value;
    var debug#VerifyVector#test_length2#2#y: [Position]Value;
    var debug#VerifyVector#test_length2#3#__ret: [Position]Value;
    var debug#VerifyVector#test_length2#4#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 15;
    debug#VerifyVector#test_length2#0#v := EmptyPositionMap;
    debug#VerifyVector#test_length2#1#x := EmptyPositionMap;
    debug#VerifyVector#test_length2#2#y := EmptyPositionMap;
    debug#VerifyVector#test_length2#3#__ret := EmptyPositionMap;
    debug#VerifyVector#test_length2#4#__ret := EmptyPositionMap;

    // process and type check arguments
    assume is#Vector(v);
    __m := UpdateLocal(__m, __frame + 0, v);
    debug#VerifyVector#test_length2#0#v := debug#VerifyVector#test_length2#0#v[Position(3942) := v];

    // bytecode translation starts here
    call __t3 := BorrowLoc(__frame + 0);

    call __t4 := Vector_length(IntegerType(), __t3);
    if (__abort_flag) { goto Label_Abort; }
    assume IsValidU64(__t4);

    __m := UpdateLocal(__m, __frame + 4, __t4);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    debug#VerifyVector#test_length2#1#x := debug#VerifyVector#test_length2#1#x[Position(4061) := __tmp];

    call __t5 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call Vector_push_back(IntegerType(), __t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_length2#0#v := debug#VerifyVector#test_length2#0#v[Position(4092) := GetLocal(__m, __frame + 0)];

    call __t7 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call Vector_push_back(IntegerType(), __t7, GetLocal(__m, __frame + 8));
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_length2#0#v := debug#VerifyVector#test_length2#0#v[Position(4128) := GetLocal(__m, __frame + 0)];

    call __t9 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(3);
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call Vector_push_back(IntegerType(), __t9, GetLocal(__m, __frame + 10));
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_length2#0#v := debug#VerifyVector#test_length2#0#v[Position(4164) := GetLocal(__m, __frame + 0)];

    call __t11 := BorrowLoc(__frame + 0);

    call __t12 := Vector_length(IntegerType(), __t11);
    if (__abort_flag) { goto Label_Abort; }
    assume IsValidU64(__t12);

    __m := UpdateLocal(__m, __frame + 12, __t12);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 12));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    debug#VerifyVector#test_length2#2#y := debug#VerifyVector#test_length2#2#y[Position(4200) := __tmp];

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    __ret0 := GetLocal(__m, __frame + 13);
    debug#VerifyVector#test_length2#3#__ret := debug#VerifyVector#test_length2#3#__ret[Position(4231) := __ret0];
    __ret1 := GetLocal(__m, __frame + 14);
    debug#VerifyVector#test_length2#4#__ret := debug#VerifyVector#test_length2#4#__ret[Position(4231) := __ret1];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#VerifyVector#test_length2#3#__ret := debug#VerifyVector#test_length2#3#__ret[Position(4262) := __ret0];
    __ret1 := DefaultValue;
    debug#VerifyVector#test_length2#4#__ret := debug#VerifyVector#test_length2#4#__ret[Position(4262) := __ret1];
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
    var debug#VerifyVector#test_id1#0#v: [Position]Value;
    var debug#VerifyVector#test_id1#1#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 2;
    debug#VerifyVector#test_id1#0#v := EmptyPositionMap;
    debug#VerifyVector#test_id1#1#__ret := EmptyPositionMap;

    // process and type check arguments
    assume is#Vector(v);
    __m := UpdateLocal(__m, __frame + 0, v);
    debug#VerifyVector#test_id1#0#v := debug#VerifyVector#test_id1#0#v[Position(4288) := v];

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    __ret0 := GetLocal(__m, __frame + 1);
    debug#VerifyVector#test_id1#1#__ret := debug#VerifyVector#test_id1#1#__ret[Position(4375) := __ret0];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#VerifyVector#test_id1#1#__ret := debug#VerifyVector#test_id1#1#__ret[Position(4397) := __ret0];
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
    var debug#VerifyVector#test_id2#0#v: [Position]Value;
    var debug#VerifyVector#test_id2#1#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 4;
    debug#VerifyVector#test_id2#0#v := EmptyPositionMap;
    debug#VerifyVector#test_id2#1#__ret := EmptyPositionMap;

    // process and type check arguments
    assume is#Vector(v);
    __m := UpdateLocal(__m, __frame + 0, v);
    debug#VerifyVector#test_id2#0#v := debug#VerifyVector#test_id2#0#v[Position(4441) := v];

    // bytecode translation starts here
    call __t1 := BorrowLoc(__frame + 0);

    call Vector_reverse(IntegerType(), __t1);
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_id2#0#v := debug#VerifyVector#test_id2#0#v[Position(4528) := GetLocal(__m, __frame + 0)];

    call __t2 := BorrowLoc(__frame + 0);

    call Vector_reverse(IntegerType(), __t2);
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_id2#0#v := debug#VerifyVector#test_id2#0#v[Position(4559) := GetLocal(__m, __frame + 0)];

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    __ret0 := GetLocal(__m, __frame + 3);
    debug#VerifyVector#test_id2#1#__ret := debug#VerifyVector#test_id2#1#__ret[Position(4590) := __ret0];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#VerifyVector#test_id2#1#__ret := debug#VerifyVector#test_id2#1#__ret[Position(4612) := __ret0];
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
    var debug#VerifyVector#test_id3#0#v: [Position]Value;
    var debug#VerifyVector#test_id3#1#l: [Position]Value;
    var debug#VerifyVector#test_id3#2#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 18;
    debug#VerifyVector#test_id3#0#v := EmptyPositionMap;
    debug#VerifyVector#test_id3#1#l := EmptyPositionMap;
    debug#VerifyVector#test_id3#2#__ret := EmptyPositionMap;

    // process and type check arguments
    assume is#Vector(v);
    __m := UpdateLocal(__m, __frame + 0, v);
    debug#VerifyVector#test_id3#0#v := debug#VerifyVector#test_id3#0#v[Position(4677) := v];

    // bytecode translation starts here
    call __t2 := BorrowLoc(__frame + 0);

    call __t3 := Vector_length(IntegerType(), __t2);
    if (__abort_flag) { goto Label_Abort; }
    assume IsValidU64(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    debug#VerifyVector#test_id3#1#l := debug#VerifyVector#test_id3#1#l[Position(4778) := __tmp];

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
    debug#VerifyVector#test_id3#0#v := debug#VerifyVector#test_id3#0#v[Position(4868) := GetLocal(__m, __frame + 0)];

    goto Label_21;

Label_19:
    call __t15 := BorrowLoc(__frame + 0);

    call Vector_reverse(IntegerType(), __t15);
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_id3#0#v := debug#VerifyVector#test_id3#0#v[Position(4927) := GetLocal(__m, __frame + 0)];

Label_21:
    call __t16 := BorrowLoc(__frame + 0);

    call Vector_reverse(IntegerType(), __t16);
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_id3#0#v := debug#VerifyVector#test_id3#0#v[Position(4967) := GetLocal(__m, __frame + 0)];

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    __ret0 := GetLocal(__m, __frame + 17);
    debug#VerifyVector#test_id3#2#__ret := debug#VerifyVector#test_id3#2#__ret[Position(4998) := __ret0];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#VerifyVector#test_id3#2#__ret := debug#VerifyVector#test_id3#2#__ret[Position(5020) := __ret0];
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
    var debug#VerifyVector#test_destroy_empty1#0#v: [Position]Value;
    var debug#VerifyVector#test_destroy_empty1#1#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 6;
    debug#VerifyVector#test_destroy_empty1#0#v := EmptyPositionMap;
    debug#VerifyVector#test_destroy_empty1#1#__ret := EmptyPositionMap;

    // process and type check arguments
    assume is#Vector(v);
    __m := UpdateLocal(__m, __frame + 0, v);
    debug#VerifyVector#test_destroy_empty1#0#v := debug#VerifyVector#test_destroy_empty1#0#v[Position(5111) := v];

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
    debug#VerifyVector#test_destroy_empty1#1#__ret := debug#VerifyVector#test_destroy_empty1#1#__ret[Position(5281) := __ret0];
    return;

Label_7:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    __ret0 := GetLocal(__m, __frame + 5);
    debug#VerifyVector#test_destroy_empty1#1#__ret := debug#VerifyVector#test_destroy_empty1#1#__ret[Position(5325) := __ret0];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#VerifyVector#test_destroy_empty1#1#__ret := debug#VerifyVector#test_destroy_empty1#1#__ret[Position(5346) := __ret0];
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
    var debug#VerifyVector#test_destroy_empty2#0#v: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 7;
    debug#VerifyVector#test_destroy_empty2#0#v := EmptyPositionMap;

    // process and type check arguments
    assume is#Vector(v);
    __m := UpdateLocal(__m, __frame + 0, v);
    debug#VerifyVector#test_destroy_empty2#0#v := debug#VerifyVector#test_destroy_empty2#0#v[Position(5479) := v];

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
    debug#VerifyVector#test_destroy_empty2#0#v := debug#VerifyVector#test_destroy_empty2#0#v[Position(5581) := GetLocal(__m, __frame + 0)];

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
    var debug#VerifyVector#test_get_set1#0#x: [Position]Value;
    var debug#VerifyVector#test_get_set1#1#ev1: [Position]Value;
    var debug#VerifyVector#test_get_set1#2#ev2: [Position]Value;
    var debug#VerifyVector#test_get_set1#3#__ret: [Position]Value;
    var debug#VerifyVector#test_get_set1#4#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 19;
    debug#VerifyVector#test_get_set1#0#x := EmptyPositionMap;
    debug#VerifyVector#test_get_set1#1#ev1 := EmptyPositionMap;
    debug#VerifyVector#test_get_set1#2#ev2 := EmptyPositionMap;
    debug#VerifyVector#test_get_set1#3#__ret := EmptyPositionMap;
    debug#VerifyVector#test_get_set1#4#__ret := EmptyPositionMap;

    // process and type check arguments
    assume IsValidU64(x);
    __m := UpdateLocal(__m, __frame + 0, x);
    debug#VerifyVector#test_get_set1#0#x := debug#VerifyVector#test_get_set1#0#x[Position(5709) := x];

    // bytecode translation starts here
    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    debug#VerifyVector#test_get_set1#1#ev1 := debug#VerifyVector#test_get_set1#1#ev1[Position(5859) := __tmp];

    call __t4 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t4);

    __m := UpdateLocal(__m, __frame + 4, __t4);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    debug#VerifyVector#test_get_set1#2#ev2 := debug#VerifyVector#test_get_set1#2#ev2[Position(5888) := __tmp];

    call __t5 := BorrowLoc(__frame + 1);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call Vector_push_back(IntegerType(), __t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_get_set1#1#ev1 := debug#VerifyVector#test_get_set1#1#ev1[Position(5917) := GetLocal(__m, __frame + 1)];
    debug#VerifyVector#test_get_set1#2#ev2 := debug#VerifyVector#test_get_set1#2#ev2[Position(5917) := GetLocal(__m, __frame + 2)];

    call __t7 := BorrowLoc(__frame + 2);

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call Vector_push_back(IntegerType(), __t7, GetLocal(__m, __frame + 8));
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_get_set1#1#ev1 := debug#VerifyVector#test_get_set1#1#ev1[Position(5955) := GetLocal(__m, __frame + 1)];
    debug#VerifyVector#test_get_set1#2#ev2 := debug#VerifyVector#test_get_set1#2#ev2[Position(5955) := GetLocal(__m, __frame + 2)];

    call __t9 := BorrowLoc(__frame + 1);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call Vector_set(IntegerType(), __t9, GetLocal(__m, __frame + 10), GetLocal(__m, __frame + 11));
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_get_set1#1#ev1 := debug#VerifyVector#test_get_set1#1#ev1[Position(5993) := GetLocal(__m, __frame + 1)];
    debug#VerifyVector#test_get_set1#2#ev2 := debug#VerifyVector#test_get_set1#2#ev2[Position(5993) := GetLocal(__m, __frame + 2)];

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
    debug#VerifyVector#test_get_set1#1#ev1 := debug#VerifyVector#test_get_set1#1#ev1[Position(6034) := GetLocal(__m, __frame + 1)];
    debug#VerifyVector#test_get_set1#2#ev2 := debug#VerifyVector#test_get_set1#2#ev2[Position(6034) := GetLocal(__m, __frame + 2)];

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    __ret0 := GetLocal(__m, __frame + 17);
    debug#VerifyVector#test_get_set1#3#__ret := debug#VerifyVector#test_get_set1#3#__ret[Position(6093) := __ret0];
    __ret1 := GetLocal(__m, __frame + 18);
    debug#VerifyVector#test_get_set1#4#__ret := debug#VerifyVector#test_get_set1#4#__ret[Position(6093) := __ret1];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#VerifyVector#test_get_set1#3#__ret := debug#VerifyVector#test_get_set1#3#__ret[Position(6128) := __ret0];
    __ret1 := DefaultValue;
    debug#VerifyVector#test_get_set1#4#__ret := debug#VerifyVector#test_get_set1#4#__ret[Position(6128) := __ret1];
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
    var debug#VerifyVector#test_get1#0#ev1: [Position]Value;
    var debug#VerifyVector#test_get1#1#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 7;
    debug#VerifyVector#test_get1#0#ev1 := EmptyPositionMap;
    debug#VerifyVector#test_get1#1#__ret := EmptyPositionMap;

    // process and type check arguments

    // bytecode translation starts here
    call __t1 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t1);

    __m := UpdateLocal(__m, __frame + 1, __t1);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    debug#VerifyVector#test_get1#0#ev1 := debug#VerifyVector#test_get1#0#ev1[Position(6232) := __tmp];

    call __t2 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(7);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call Vector_push_back(IntegerType(), __t2, GetLocal(__m, __frame + 3));
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_get1#0#ev1 := debug#VerifyVector#test_get1#0#ev1[Position(6261) := GetLocal(__m, __frame + 0)];

    call __t4 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __t6 := Vector_get(IntegerType(), __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) { goto Label_Abort; }
    assume IsValidU64(__t6);

    __m := UpdateLocal(__m, __frame + 6, __t6);

    __ret0 := GetLocal(__m, __frame + 6);
    debug#VerifyVector#test_get1#1#__ret := debug#VerifyVector#test_get1#1#__ret[Position(6299) := __ret0];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#VerifyVector#test_get1#1#__ret := debug#VerifyVector#test_get1#1#__ret[Position(6336) := __ret0];
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
    var debug#VerifyVector#test_get2#0#x: [Position]Value;
    var debug#VerifyVector#test_get2#1#ev1: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 8;
    debug#VerifyVector#test_get2#0#x := EmptyPositionMap;
    debug#VerifyVector#test_get2#1#ev1 := EmptyPositionMap;

    // process and type check arguments

    // bytecode translation starts here
    call __t2 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    debug#VerifyVector#test_get2#1#ev1 := debug#VerifyVector#test_get2#1#ev1[Position(6614) := __tmp];

    call __t3 := BorrowLoc(__frame + 1);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call Vector_push_back(IntegerType(), __t3, GetLocal(__m, __frame + 4));
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_get2#1#ev1 := debug#VerifyVector#test_get2#1#ev1[Position(6643) := GetLocal(__m, __frame + 1)];

    call __t5 := BorrowLoc(__frame + 1);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __t7 := Vector_get(IntegerType(), __t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) { goto Label_Abort; }
    assume IsValidU64(__t7);

    __m := UpdateLocal(__m, __frame + 7, __t7);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    debug#VerifyVector#test_get2#0#x := debug#VerifyVector#test_get2#0#x[Position(6681) := __tmp];

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
    var debug#VerifyVector#test_borrow1#0#ev1: [Position]Value;
    var debug#VerifyVector#test_borrow1#1#y: [Position]Value;
    var debug#VerifyVector#test_borrow1#2#__ret: [Position]Value;
    var debug#VerifyVector#test_borrow1#3#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 11;
    debug#VerifyVector#test_borrow1#0#ev1 := EmptyPositionMap;
    debug#VerifyVector#test_borrow1#1#y := EmptyPositionMap;
    debug#VerifyVector#test_borrow1#2#__ret := EmptyPositionMap;
    debug#VerifyVector#test_borrow1#3#__ret := EmptyPositionMap;

    // process and type check arguments

    // bytecode translation starts here
    call __t2 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    debug#VerifyVector#test_borrow1#0#ev1 := debug#VerifyVector#test_borrow1#0#ev1[Position(6865) := __tmp];

    call __t3 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(7);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call Vector_push_back(IntegerType(), __t3, GetLocal(__m, __frame + 4));
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_borrow1#0#ev1 := debug#VerifyVector#test_borrow1#0#ev1[Position(6894) := GetLocal(__m, __frame + 0)];

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
    debug#VerifyVector#test_borrow1#2#__ret := debug#VerifyVector#test_borrow1#2#__ret[Position(6967) := __ret0];
    __ret1 := GetLocal(__m, __frame + 10);
    debug#VerifyVector#test_borrow1#3#__ret := debug#VerifyVector#test_borrow1#3#__ret[Position(6967) := __ret1];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#VerifyVector#test_borrow1#2#__ret := debug#VerifyVector#test_borrow1#2#__ret[Position(6993) := __ret0];
    __ret1 := DefaultValue;
    debug#VerifyVector#test_borrow1#3#__ret := debug#VerifyVector#test_borrow1#3#__ret[Position(6993) := __ret1];
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
    var debug#VerifyVector#test_borrow2#0#ev1: [Position]Value;
    var debug#VerifyVector#test_borrow2#1#y: [Position]Value;
    var debug#VerifyVector#test_borrow2#2#__ret: [Position]Value;
    var debug#VerifyVector#test_borrow2#3#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 11;
    debug#VerifyVector#test_borrow2#0#ev1 := EmptyPositionMap;
    debug#VerifyVector#test_borrow2#1#y := EmptyPositionMap;
    debug#VerifyVector#test_borrow2#2#__ret := EmptyPositionMap;
    debug#VerifyVector#test_borrow2#3#__ret := EmptyPositionMap;

    // process and type check arguments

    // bytecode translation starts here
    call __t2 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    debug#VerifyVector#test_borrow2#0#ev1 := debug#VerifyVector#test_borrow2#0#ev1[Position(7241) := __tmp];

    call __t3 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(7);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call Vector_push_back(IntegerType(), __t3, GetLocal(__m, __frame + 4));
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_borrow2#0#ev1 := debug#VerifyVector#test_borrow2#0#ev1[Position(7270) := GetLocal(__m, __frame + 0)];

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
    debug#VerifyVector#test_borrow2#2#__ret := debug#VerifyVector#test_borrow2#2#__ret[Position(7343) := __ret0];
    __ret1 := GetLocal(__m, __frame + 10);
    debug#VerifyVector#test_borrow2#3#__ret := debug#VerifyVector#test_borrow2#3#__ret[Position(7343) := __ret1];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#VerifyVector#test_borrow2#2#__ret := debug#VerifyVector#test_borrow2#2#__ret[Position(7369) := __ret0];
    __ret1 := DefaultValue;
    debug#VerifyVector#test_borrow2#3#__ret := debug#VerifyVector#test_borrow2#3#__ret[Position(7369) := __ret1];
}

procedure VerifyVector_test_borrow2_verify () returns (__ret0: Value, __ret1: Value)
{
    call InitVerification();
    call __ret0, __ret1 := VerifyVector_test_borrow2();
}

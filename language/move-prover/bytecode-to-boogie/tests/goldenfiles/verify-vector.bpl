

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
    debug#VerifyVector#test_empty1#0#ev1 := debug#VerifyVector#test_empty1#0#ev1[Position(189) := __tmp];

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    debug#VerifyVector#test_empty1#1#ev2 := debug#VerifyVector#test_empty1#1#ev2[Position(217) := __tmp];

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    __ret0 := GetLocal(__m, __frame + 4);
    debug#VerifyVector#test_empty1#2#__ret := debug#VerifyVector#test_empty1#2#__ret[Position(245) := __ret0];
    __ret1 := GetLocal(__m, __frame + 5);
    debug#VerifyVector#test_empty1#3#__ret := debug#VerifyVector#test_empty1#3#__ret[Position(245) := __ret1];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#VerifyVector#test_empty1#2#__ret := debug#VerifyVector#test_empty1#2#__ret[Position(280) := __ret0];
    __ret1 := DefaultValue;
    debug#VerifyVector#test_empty1#3#__ret := debug#VerifyVector#test_empty1#3#__ret[Position(280) := __ret1];
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
    debug#VerifyVector#test_empty2#0#ev1 := debug#VerifyVector#test_empty2#0#ev1[Position(440) := __tmp];

    call __t4 := Vector_empty(IntegerType());
    if (__abort_flag) { goto Label_Abort; }
    assume is#Vector(__t4);

    __m := UpdateLocal(__m, __frame + 4, __t4);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    debug#VerifyVector#test_empty2#1#ev2 := debug#VerifyVector#test_empty2#1#ev2[Position(468) := __tmp];

    call __t5 := BorrowLoc(__frame + 0);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call Vector_push_back(IntegerType(), __t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) { goto Label_Abort; }
    debug#VerifyVector#test_empty2#0#ev1 := debug#VerifyVector#test_empty2#0#ev1[Position(496) := GetLocal(__m, __frame + 0)];

    call __t7 := BorrowLoc(__frame + 0);

    call __t8 := Vector_pop_back(IntegerType(), __t7);
    if (__abort_flag) { goto Label_Abort; }
    assume IsValidU64(__t8);

    __m := UpdateLocal(__m, __frame + 8, __t8);
    debug#VerifyVector#test_empty2#0#ev1 := debug#VerifyVector#test_empty2#0#ev1[Position(537) := GetLocal(__m, __frame + 0)];

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    debug#VerifyVector#test_empty2#2#x := debug#VerifyVector#test_empty2#2#x[Position(533) := __tmp];

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    __ret0 := GetLocal(__m, __frame + 9);
    debug#VerifyVector#test_empty2#3#__ret := debug#VerifyVector#test_empty2#3#__ret[Position(570) := __ret0];
    __ret1 := GetLocal(__m, __frame + 10);
    debug#VerifyVector#test_empty2#4#__ret := debug#VerifyVector#test_empty2#4#__ret[Position(570) := __ret1];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#VerifyVector#test_empty2#3#__ret := debug#VerifyVector#test_empty2#3#__ret[Position(605) := __ret0];
    __ret1 := DefaultValue;
    debug#VerifyVector#test_empty2#4#__ret := debug#VerifyVector#test_empty2#4#__ret[Position(605) := __ret1];
}

procedure VerifyVector_test_empty2_verify () returns (__ret0: Value, __ret1: Value)
{
    call InitVerification();
    call __ret0, __ret1 := VerifyVector_test_empty2();
}

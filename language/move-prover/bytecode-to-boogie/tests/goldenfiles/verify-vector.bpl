
// ** helpers from test_mvir/verify-vector.prover.bpl
function {:inline 1} len(v : Value) : Value {
    Integer(vlen(v))
}

// deprecated
function {:inline 1} vector_length(v : Value) : Value {
    Integer(vlen(v))
}

function {:inline 1} vector_get(v : Value, i : Value) : Value {
    select_vector(v, i#Integer(i))
}

// deprecated
function {:inline 1} vector_get2(v : Value, i : Value) : Value {
    vmap(v)[i#Integer(i)]
}

function {:inline 1} vector_update(v : Value, i : Value, e : Value) : Value {
    update_vector(v, i#Integer(i), e)
}

function {:inline 1} vector_slice(v : Value, i : Value, j : Value) : Value {
    slice_vector(v, i#Integer(i), i#Integer(j))
}


// ** synthetics of module VerifyVector



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

    // process and type check arguments

    // increase the local counter
    __local_counter := __local_counter + 6;

    // bytecode translation starts here
    call __t2 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 0, 243);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 0, 0, 237, __tmp);

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 0, 278);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 0, 1, 272, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    __ret0 := GetLocal(__m, __frame + 4);
    assume $DebugTrackLocal(1, 0, 2, 307, __ret0);
    __ret1 := GetLocal(__m, __frame + 5);
    assume $DebugTrackLocal(1, 0, 3, 307, __ret1);
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

    // process and type check arguments

    // increase the local counter
    __local_counter := __local_counter + 11;

    // bytecode translation starts here
    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 1, 561);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 1, 0, 555, __tmp);

    call __t4 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 1, 596);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t4);

    __m := UpdateLocal(__m, __frame + 4, __t4);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 1, 1, 590, __tmp);

    call __t5 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call Vector_push_back(IntegerType(), __t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 1, 625);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 1, 0, 625, GetLocal(__m, __frame + 0));

    call __t7 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __t8 := Vector_pop_back(IntegerType(), __t7);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 1, 673);
      goto Label_Abort;
    }
    assume IsValidU64(__t8);

    __m := UpdateLocal(__m, __frame + 8, __t8);
    assume $DebugTrackLocal(1, 1, 0, 673, GetLocal(__m, __frame + 0));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    assume $DebugTrackLocal(1, 1, 2, 669, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    __ret0 := GetLocal(__m, __frame + 9);
    assume $DebugTrackLocal(1, 1, 3, 713, __ret0);
    __ret1 := GetLocal(__m, __frame + 10);
    assume $DebugTrackLocal(1, 1, 4, 713, __ret1);
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

    // process and type check arguments

    // increase the local counter
    __local_counter := __local_counter + 10;

    // bytecode translation starts here
    call __t2 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 2, 949);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 2, 0, 943, __tmp);

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 2, 984);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 2, 1, 978, __tmp);

    call __t4 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call Vector_push_back(IntegerType(), __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 2, 1013);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 2, 0, 1013, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 2, 1, 1013, GetLocal(__m, __frame + 1));

    call __t6 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call Vector_push_back(IntegerType(), __t6, GetLocal(__m, __frame + 7));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 2, 1057);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 2, 0, 1057, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 2, 1, 1057, GetLocal(__m, __frame + 1));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    __ret0 := GetLocal(__m, __frame + 8);
    assume $DebugTrackLocal(1, 2, 2, 1101, __ret0);
    __ret1 := GetLocal(__m, __frame + 9);
    assume $DebugTrackLocal(1, 2, 3, 1101, __ret1);
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

    // process and type check arguments

    // increase the local counter
    __local_counter := __local_counter + 12;

    // bytecode translation starts here
    call __t2 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 3, 1339);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 3, 0, 1333, __tmp);

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 3, 1374);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 3, 1, 1368, __tmp);

    call __t4 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call Vector_push_back(IntegerType(), __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 3, 1403);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 3, 0, 1403, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 3, 1, 1403, GetLocal(__m, __frame + 1));

    call __t6 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call Vector_push_back(IntegerType(), __t6, GetLocal(__m, __frame + 7));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 3, 1447);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 3, 0, 1447, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 3, 1, 1447, GetLocal(__m, __frame + 1));

    call __t8 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call Vector_push_back(IntegerType(), __t8, GetLocal(__m, __frame + 9));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 3, 1491);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 3, 0, 1491, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 3, 1, 1491, GetLocal(__m, __frame + 1));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    __ret0 := GetLocal(__m, __frame + 10);
    assume $DebugTrackLocal(1, 3, 2, 1535, __ret0);
    __ret1 := GetLocal(__m, __frame + 11);
    assume $DebugTrackLocal(1, 3, 3, 1535, __ret1);
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

    // process and type check arguments

    // increase the local counter
    __local_counter := __local_counter + 10;

    // bytecode translation starts here
    call __t2 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 4, 1771);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 4, 0, 1765, __tmp);

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 4, 1806);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 4, 1, 1800, __tmp);

    call __t4 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call Vector_push_back(IntegerType(), __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 4, 1835);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 4, 0, 1835, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 4, 1, 1835, GetLocal(__m, __frame + 1));

    call __t6 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call Vector_push_back(IntegerType(), __t6, GetLocal(__m, __frame + 7));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 4, 1879);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 4, 0, 1879, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 4, 1, 1879, GetLocal(__m, __frame + 1));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    __ret0 := GetLocal(__m, __frame + 8);
    assume $DebugTrackLocal(1, 4, 2, 1923, __ret0);
    __ret1 := GetLocal(__m, __frame + 9);
    assume $DebugTrackLocal(1, 4, 3, 1923, __ret1);
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

    // process and type check arguments

    // increase the local counter
    __local_counter := __local_counter + 7;

    // bytecode translation starts here
    call __t2 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 5, 2169);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 5, 0, 2163, __tmp);

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 5, 2204);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 5, 1, 2198, __tmp);

    call __t4 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call Vector_reverse(IntegerType(), __t4);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 5, 2233);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 5, 0, 2233, GetLocal(__m, __frame + 0));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    __ret0 := GetLocal(__m, __frame + 5);
    assume $DebugTrackLocal(1, 5, 2, 2272, __ret0);
    __ret1 := GetLocal(__m, __frame + 6);
    assume $DebugTrackLocal(1, 5, 3, 2272, __ret1);
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

    // process and type check arguments

    // increase the local counter
    __local_counter := __local_counter + 15;

    // bytecode translation starts here
    call __t2 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 6, 2524);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 6, 0, 2518, __tmp);

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 6, 2559);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 6, 1, 2553, __tmp);

    call __t4 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call Vector_push_back(IntegerType(), __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 6, 2588);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 6, 0, 2588, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 6, 1, 2588, GetLocal(__m, __frame + 1));

    call __t6 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call Vector_push_back(IntegerType(), __t6, GetLocal(__m, __frame + 7));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 6, 2632);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 6, 0, 2632, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 6, 1, 2632, GetLocal(__m, __frame + 1));

    call __t8 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call Vector_push_back(IntegerType(), __t8, GetLocal(__m, __frame + 9));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 6, 2676);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 6, 0, 2676, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 6, 1, 2676, GetLocal(__m, __frame + 1));

    call __t10 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call Vector_push_back(IntegerType(), __t10, GetLocal(__m, __frame + 11));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 6, 2720);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 6, 0, 2720, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 6, 1, 2720, GetLocal(__m, __frame + 1));

    call __t12 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call Vector_reverse(IntegerType(), __t12);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 6, 2764);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 6, 0, 2764, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 6, 1, 2764, GetLocal(__m, __frame + 1));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    __ret0 := GetLocal(__m, __frame + 13);
    assume $DebugTrackLocal(1, 6, 2, 2803, __ret0);
    __ret1 := GetLocal(__m, __frame + 14);
    assume $DebugTrackLocal(1, 6, 3, 2803, __ret1);
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

    // process and type check arguments

    // increase the local counter
    __local_counter := __local_counter + 7;

    // bytecode translation starts here
    call __t1 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 7, 3028);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t1);

    __m := UpdateLocal(__m, __frame + 1, __t1);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 7, 0, 3022, __tmp);

    call __t2 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call Vector_push_back(IntegerType(), __t2, GetLocal(__m, __frame + 3));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 7, 3057);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 7, 0, 3057, GetLocal(__m, __frame + 0));

    call __t4 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call Vector_swap(IntegerType(), __t4, GetLocal(__m, __frame + 5), GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 7, 3101);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 7, 0, 3101, GetLocal(__m, __frame + 0));

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

    // process and type check arguments

    // increase the local counter
    __local_counter := __local_counter + 7;

    // bytecode translation starts here
    call __t1 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 8, 3346);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t1);

    __m := UpdateLocal(__m, __frame + 1, __t1);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 8, 0, 3340, __tmp);

    call __t2 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call Vector_push_back(IntegerType(), __t2, GetLocal(__m, __frame + 3));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 8, 3375);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 8, 0, 3375, GetLocal(__m, __frame + 0));

    call __t4 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call Vector_swap(IntegerType(), __t4, GetLocal(__m, __frame + 5), GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 8, 3419);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 8, 0, 3419, GetLocal(__m, __frame + 0));

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

    // process and type check arguments

    // increase the local counter
    __local_counter := __local_counter + 20;

    // bytecode translation starts here
    call __t2 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 9, 3688);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 9, 0, 3682, __tmp);

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 9, 3723);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 9, 1, 3717, __tmp);

    call __t4 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call Vector_push_back(IntegerType(), __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 9, 3752);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 9, 0, 3752, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 9, 1, 3752, GetLocal(__m, __frame + 1));

    call __t6 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call Vector_push_back(IntegerType(), __t6, GetLocal(__m, __frame + 7));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 9, 3796);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 9, 0, 3796, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 9, 1, 3796, GetLocal(__m, __frame + 1));

    call __t8 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call Vector_push_back(IntegerType(), __t8, GetLocal(__m, __frame + 9));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 9, 3840);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 9, 0, 3840, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 9, 1, 3840, GetLocal(__m, __frame + 1));

    call __t10 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call Vector_push_back(IntegerType(), __t10, GetLocal(__m, __frame + 11));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 9, 3884);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 9, 0, 3884, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 9, 1, 3884, GetLocal(__m, __frame + 1));

    call __t12 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call Vector_swap(IntegerType(), __t12, GetLocal(__m, __frame + 13), GetLocal(__m, __frame + 14));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 9, 3928);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 9, 0, 3928, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 9, 1, 3928, GetLocal(__m, __frame + 1));

    call __t15 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    call Vector_swap(IntegerType(), __t15, GetLocal(__m, __frame + 16), GetLocal(__m, __frame + 17));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 9, 3970);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 9, 0, 3970, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 9, 1, 3970, GetLocal(__m, __frame + 1));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 19, __tmp);

    __ret0 := GetLocal(__m, __frame + 18);
    assume $DebugTrackLocal(1, 9, 2, 4012, __ret0);
    __ret1 := GetLocal(__m, __frame + 19);
    assume $DebugTrackLocal(1, 9, 3, 4012, __ret1);
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

    // process and type check arguments

    // increase the local counter
    __local_counter := __local_counter + 10;

    // bytecode translation starts here
    call __t2 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 10, 4252);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 10, 0, 4246, __tmp);

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 10, 4287);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 10, 1, 4281, __tmp);

    call __t4 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call Vector_push_back(IntegerType(), __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 10, 4316);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 10, 0, 4316, GetLocal(__m, __frame + 0));

    call __t6 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __t7 := Vector_length(IntegerType(), __t6);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 10, 4368);
      goto Label_Abort;
    }
    assume IsValidU64(__t7);

    __m := UpdateLocal(__m, __frame + 7, __t7);

    call __t8 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __t9 := Vector_length(IntegerType(), __t8);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 10, 4395);
      goto Label_Abort;
    }
    assume IsValidU64(__t9);

    __m := UpdateLocal(__m, __frame + 9, __t9);

    __ret0 := GetLocal(__m, __frame + 7);
    assume $DebugTrackLocal(1, 10, 2, 4360, __ret0);
    __ret1 := GetLocal(__m, __frame + 9);
    assume $DebugTrackLocal(1, 10, 3, 4360, __ret1);
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

    // process and type check arguments
    assume $Vector_T_is_well_formed(v);
    __m := UpdateLocal(__m, __frame + 0, v);
    assume $DebugTrackLocal(1, 11, 0, 4491, v);

    // increase the local counter
    __local_counter := __local_counter + 15;

    // bytecode translation starts here
    call __t3 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __t4 := Vector_length(IntegerType(), __t3);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 11, 4632);
      goto Label_Abort;
    }
    assume IsValidU64(__t4);

    __m := UpdateLocal(__m, __frame + 4, __t4);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 11, 1, 4628, __tmp);

    call __t5 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call Vector_push_back(IntegerType(), __t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 11, 4665);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 11, 0, 4665, GetLocal(__m, __frame + 0));

    call __t7 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call Vector_push_back(IntegerType(), __t7, GetLocal(__m, __frame + 8));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 11, 4707);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 11, 0, 4707, GetLocal(__m, __frame + 0));

    call __t9 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(3);
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call Vector_push_back(IntegerType(), __t9, GetLocal(__m, __frame + 10));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 11, 4749);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 11, 0, 4749, GetLocal(__m, __frame + 0));

    call __t11 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __t12 := Vector_length(IntegerType(), __t11);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 11, 4795);
      goto Label_Abort;
    }
    assume IsValidU64(__t12);

    __m := UpdateLocal(__m, __frame + 12, __t12);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 12));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    assume $DebugTrackLocal(1, 11, 2, 4791, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    __ret0 := GetLocal(__m, __frame + 13);
    assume $DebugTrackLocal(1, 11, 3, 4828, __ret0);
    __ret1 := GetLocal(__m, __frame + 14);
    assume $DebugTrackLocal(1, 11, 4, 4828, __ret1);
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

    // process and type check arguments
    assume $Vector_T_is_well_formed(v);
    __m := UpdateLocal(__m, __frame + 0, v);
    assume $DebugTrackLocal(1, 12, 0, 4891, v);

    // increase the local counter
    __local_counter := __local_counter + 2;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    __ret0 := GetLocal(__m, __frame + 1);
    assume $DebugTrackLocal(1, 12, 1, 4984, __ret0);
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

    // process and type check arguments
    assume $Vector_T_is_well_formed(v);
    __m := UpdateLocal(__m, __frame + 0, v);
    assume $DebugTrackLocal(1, 13, 0, 5056, v);

    // increase the local counter
    __local_counter := __local_counter + 4;

    // bytecode translation starts here
    call __t1 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call Vector_reverse(IntegerType(), __t1);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 13, 5149);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 13, 0, 5149, GetLocal(__m, __frame + 0));

    call __t2 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call Vector_reverse(IntegerType(), __t2);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 13, 5186);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 13, 0, 5186, GetLocal(__m, __frame + 0));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    __ret0 := GetLocal(__m, __frame + 3);
    assume $DebugTrackLocal(1, 13, 1, 5223, __ret0);
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

    // process and type check arguments
    assume $Vector_T_is_well_formed(v);
    __m := UpdateLocal(__m, __frame + 0, v);
    assume $DebugTrackLocal(1, 14, 0, 5316, v);

    // increase the local counter
    __local_counter := __local_counter + 18;

    // bytecode translation starts here
    call __t2 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __t3 := Vector_length(IntegerType(), __t2);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 14, 5433);
      goto Label_Abort;
    }
    assume IsValidU64(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 14, 1, 5429, __tmp);

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

    call __t10 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __tmp := Sub(GetLocal(__m, __frame + 12), GetLocal(__m, __frame + 13));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 14, 5586);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call Vector_swap(IntegerType(), __t10, GetLocal(__m, __frame + 11), GetLocal(__m, __frame + 14));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 14, 5558);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 14, 0, 5558, GetLocal(__m, __frame + 0));

    goto Label_21;

Label_19:
    call __t15 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call Vector_reverse(IntegerType(), __t15);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 14, 5647);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 14, 0, 5647, GetLocal(__m, __frame + 0));

Label_21:
    call __t16 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call Vector_reverse(IntegerType(), __t16);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 14, 5708);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 14, 0, 5708, GetLocal(__m, __frame + 0));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    __ret0 := GetLocal(__m, __frame + 17);
    assume $DebugTrackLocal(1, 14, 2, 5745, __ret0);
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

    // process and type check arguments
    assume $Vector_T_is_well_formed(v);
    __m := UpdateLocal(__m, __frame + 0, v);
    assume $DebugTrackLocal(1, 15, 0, 5864, v);

    // increase the local counter
    __local_counter := __local_counter + 6;

    // bytecode translation starts here
    call __t1 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __t2 := Vector_is_empty(IntegerType(), __t1);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 15, 5972);
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
      assume $DebugTrackAbort(1, 15, 6013);
      goto Label_Abort;
    }

    call __t4 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 15, 6068);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t4);

    __m := UpdateLocal(__m, __frame + 4, __t4);

    __ret0 := GetLocal(__m, __frame + 4);
    assume $DebugTrackLocal(1, 15, 1, 6061, __ret0);
    return;

Label_7:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    __ret0 := GetLocal(__m, __frame + 5);
    assume $DebugTrackLocal(1, 15, 1, 6126, __ret0);
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

    // process and type check arguments
    assume $Vector_T_is_well_formed(v);
    __m := UpdateLocal(__m, __frame + 0, v);
    assume $DebugTrackLocal(1, 16, 0, 6295, v);

    // increase the local counter
    __local_counter := __local_counter + 7;

    // bytecode translation starts here
    call __t1 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __t2 := Vector_is_empty(IntegerType(), __t1);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 16, 6377);
      goto Label_Abort;
    }
    assume is#Boolean(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    __tmp := GetLocal(__m, __frame + 2);
    if (!b#Boolean(__tmp)) { goto Label_8; }

    call __t3 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call Vector_set(IntegerType(), __t3, GetLocal(__m, __frame + 4), GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 16, 6418);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 16, 0, 6418, GetLocal(__m, __frame + 0));

    goto Label_10;

Label_8:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call Vector_destroy_empty(IntegerType(), GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 16, 6486);
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

    // process and type check arguments
    assume IsValidU64(x);
    __m := UpdateLocal(__m, __frame + 0, x);
    assume $DebugTrackLocal(1, 17, 0, 6588, x);

    // increase the local counter
    __local_counter := __local_counter + 19;

    // bytecode translation starts here
    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 17, 6762);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 17, 1, 6756, __tmp);

    call __t4 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 17, 6797);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t4);

    __m := UpdateLocal(__m, __frame + 4, __t4);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    assume $DebugTrackLocal(1, 17, 2, 6791, __tmp);

    call __t5 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call Vector_push_back(IntegerType(), __t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 17, 6826);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 17, 1, 6826, GetLocal(__m, __frame + 1));
    assume $DebugTrackLocal(1, 17, 2, 6826, GetLocal(__m, __frame + 2));

    call __t7 := BorrowLoc(__frame + 2, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call Vector_push_back(IntegerType(), __t7, GetLocal(__m, __frame + 8));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 17, 6870);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 17, 1, 6870, GetLocal(__m, __frame + 1));
    assume $DebugTrackLocal(1, 17, 2, 6870, GetLocal(__m, __frame + 2));

    call __t9 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call Vector_set(IntegerType(), __t9, GetLocal(__m, __frame + 10), GetLocal(__m, __frame + 11));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 17, 6914);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 17, 1, 6914, GetLocal(__m, __frame + 1));
    assume $DebugTrackLocal(1, 17, 2, 6914, GetLocal(__m, __frame + 2));

    call __t12 := BorrowLoc(__frame + 2, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __t14 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 15, __tmp);

    call __t16 := Vector_get(IntegerType(), __t14, GetLocal(__m, __frame + 15));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 17, 6990);
      goto Label_Abort;
    }
    assume IsValidU64(__t16);

    __m := UpdateLocal(__m, __frame + 16, __t16);

    call Vector_set(IntegerType(), __t12, GetLocal(__m, __frame + 13), GetLocal(__m, __frame + 16));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 17, 6961);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 17, 1, 6961, GetLocal(__m, __frame + 1));
    assume $DebugTrackLocal(1, 17, 2, 6961, GetLocal(__m, __frame + 2));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    __ret0 := GetLocal(__m, __frame + 17);
    assume $DebugTrackLocal(1, 17, 3, 7026, __ret0);
    __ret1 := GetLocal(__m, __frame + 18);
    assume $DebugTrackLocal(1, 17, 4, 7026, __ret1);
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

    // process and type check arguments

    // increase the local counter
    __local_counter := __local_counter + 7;

    // bytecode translation starts here
    call __t1 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 18, 7186);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t1);

    __m := UpdateLocal(__m, __frame + 1, __t1);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 18, 0, 7180, __tmp);

    call __t2 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(7);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call Vector_push_back(IntegerType(), __t2, GetLocal(__m, __frame + 3));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 18, 7215);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 18, 0, 7215, GetLocal(__m, __frame + 0));

    call __t4 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __t6 := Vector_get(IntegerType(), __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 18, 7266);
      goto Label_Abort;
    }
    assume IsValidU64(__t6);

    __m := UpdateLocal(__m, __frame + 6, __t6);

    __ret0 := GetLocal(__m, __frame + 6);
    assume $DebugTrackLocal(1, 18, 1, 7259, __ret0);
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

    // process and type check arguments

    // increase the local counter
    __local_counter := __local_counter + 8;

    // bytecode translation starts here
    call __t2 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 19, 7482);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 19, 1, 7476, __tmp);

    call __t3 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call Vector_push_back(IntegerType(), __t3, GetLocal(__m, __frame + 4));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 19, 7511);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 19, 1, 7511, GetLocal(__m, __frame + 1));

    call __t5 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __t7 := Vector_get(IntegerType(), __t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 19, 7559);
      goto Label_Abort;
    }
    assume IsValidU64(__t7);

    __m := UpdateLocal(__m, __frame + 7, __t7);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 19, 0, 7555, __tmp);

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

    // process and type check arguments

    // increase the local counter
    __local_counter := __local_counter + 11;

    // bytecode translation starts here
    call __t2 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 20, 7775);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 20, 0, 7769, __tmp);

    call __t3 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(7);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call Vector_push_back(IntegerType(), __t3, GetLocal(__m, __frame + 4));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 20, 7804);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 20, 0, 7804, GetLocal(__m, __frame + 0));

    call __t5 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __t7 := Vector_borrow(IntegerType(), __t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 20, 7852);
      goto Label_Abort;
    }
    assume IsValidU64(Dereference(__m, __t7)) && IsValidReferenceParameter(__m, __local_counter, __t7);



    call y := CopyOrMoveRef(__t7);
    assume IsValidU64(Dereference(__m, y));
    assume $DebugTrackLocal(1, 20, 1, 7848, Dereference(__m, y));

    call __tmp := LdConst(7);
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __t9 := CopyOrMoveRef(y);

    call __tmp := ReadRef(__t9);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    __ret0 := GetLocal(__m, __frame + 8);
    assume $DebugTrackLocal(1, 20, 2, 7889, __ret0);
    __ret1 := GetLocal(__m, __frame + 10);
    assume $DebugTrackLocal(1, 20, 3, 7889, __ret1);
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
ensures !__abort_flag ==> b#Boolean(Boolean(false));
ensures old(!(b#Boolean(Boolean(true)))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(true))) ==> __abort_flag;

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

    // process and type check arguments

    // increase the local counter
    __local_counter := __local_counter + 11;

    // bytecode translation starts here
    call __t2 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 21, 8135);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 21, 0, 8129, __tmp);

    call __t3 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(7);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call Vector_push_back(IntegerType(), __t3, GetLocal(__m, __frame + 4));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 21, 8164);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 21, 0, 8164, GetLocal(__m, __frame + 0));

    call __t5 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __t7 := Vector_borrow(IntegerType(), __t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 21, 8212);
      goto Label_Abort;
    }
    assume IsValidU64(Dereference(__m, __t7)) && IsValidReferenceParameter(__m, __local_counter, __t7);



    call y := CopyOrMoveRef(__t7);
    assume IsValidU64(Dereference(__m, y));
    assume $DebugTrackLocal(1, 21, 1, 8208, Dereference(__m, y));

    call __tmp := LdConst(7);
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __t9 := CopyOrMoveRef(y);

    call __tmp := ReadRef(__t9);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    __ret0 := GetLocal(__m, __frame + 8);
    assume $DebugTrackLocal(1, 21, 2, 8249, __ret0);
    __ret1 := GetLocal(__m, __frame + 10);
    assume $DebugTrackLocal(1, 21, 3, 8249, __ret1);
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

procedure {:inline 1} VerifyVector_test_borrow3 () returns (__ret0: Value, __ret1: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(!IsEqual(__ret0, __ret1)));
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

    // process and type check arguments

    // increase the local counter
    __local_counter := __local_counter + 11;

    // bytecode translation starts here
    call __t2 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 22, 8443);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 22, 0, 8437, __tmp);

    call __t3 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(7);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call Vector_push_back(IntegerType(), __t3, GetLocal(__m, __frame + 4));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 22, 8472);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 22, 0, 8472, GetLocal(__m, __frame + 0));

    call __t5 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __t7 := Vector_borrow(IntegerType(), __t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 22, 8520);
      goto Label_Abort;
    }
    assume IsValidU64(Dereference(__m, __t7)) && IsValidReferenceParameter(__m, __local_counter, __t7);



    call y := CopyOrMoveRef(__t7);
    assume IsValidU64(Dereference(__m, y));
    assume $DebugTrackLocal(1, 22, 1, 8516, Dereference(__m, y));

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __t9 := CopyOrMoveRef(y);

    call __tmp := ReadRef(__t9);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    __ret0 := GetLocal(__m, __frame + 8);
    assume $DebugTrackLocal(1, 22, 2, 8557, __ret0);
    __ret1 := GetLocal(__m, __frame + 10);
    assume $DebugTrackLocal(1, 22, 3, 8557, __ret1);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    __ret1 := DefaultValue;
}

procedure VerifyVector_test_borrow3_verify () returns (__ret0: Value, __ret1: Value)
{
    call InitVerification();
    call __ret0, __ret1 := VerifyVector_test_borrow3();
}

procedure {:inline 1} VerifyVector_my_length1 (v: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(Integer(vlen(v)), __ret0)));
{
    // declare local variables
    var __t1: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t2: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $Vector_T_is_well_formed(v);
    __m := UpdateLocal(__m, __frame + 0, v);
    assume $DebugTrackLocal(1, 23, 0, 8607, v);

    // increase the local counter
    __local_counter := __local_counter + 3;

    // bytecode translation starts here
    call __t1 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __t2 := Vector_length(IntegerType(), __t1);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 23, 9013);
      goto Label_Abort;
    }
    assume IsValidU64(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    __ret0 := GetLocal(__m, __frame + 2);
    assume $DebugTrackLocal(1, 23, 1, 9006, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure VerifyVector_my_length1_verify (v: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := VerifyVector_my_length1(v);
}

procedure {:inline 1} VerifyVector_my_length2 (v: Reference) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(Integer(vlen(Dereference(__m, v))), __ret0)));
{
    // declare local variables
    var __t1: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t2: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $Vector_T_is_well_formed(Dereference(__m, v)) && IsValidReferenceParameter(__m, __local_counter, v);
    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 24, 0, 9114, Dereference(__m, v));

    // increase the local counter
    __local_counter := __local_counter + 3;

    // bytecode translation starts here
    call __t1 := CopyOrMoveRef(v);

    call __t2 := Vector_length(IntegerType(), __t1);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 24, 9208);
      goto Label_Abort;
    }
    assume IsValidU64(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    __ret0 := GetLocal(__m, __frame + 2);
    assume $DebugTrackLocal(1, 24, 1, 9201, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure VerifyVector_my_length2_verify (v: Reference) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := VerifyVector_my_length2(v);
}

procedure {:inline 1} VerifyVector_my_length3 (tv0: TypeValue, v: Reference) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(Integer(vlen(Dereference(__m, v))), __ret0)));
{
    // declare local variables
    var __t1: Reference; // ReferenceType(Vector_T_type_value(tv0))
    var __t2: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $Vector_T_is_well_formed(Dereference(__m, v)) && IsValidReferenceParameter(__m, __local_counter, v);
    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 25, 0, 9289, Dereference(__m, v));

    // increase the local counter
    __local_counter := __local_counter + 3;

    // bytecode translation starts here
    call __t1 := CopyOrMoveRef(v);

    call __t2 := Vector_length(tv0, __t1);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 25, 9396);
      goto Label_Abort;
    }
    assume IsValidU64(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    __ret0 := GetLocal(__m, __frame + 2);
    assume $DebugTrackLocal(1, 25, 1, 9389, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure VerifyVector_my_length3_verify (tv0: TypeValue, v: Reference) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := VerifyVector_my_length3(tv0, v);
}

procedure {:inline 1} VerifyVector_my_get1 (v: Value, i: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(__ret0, select_vector(v, i#Integer(i)))));
ensures old(!(b#Boolean(Boolean(i#Integer(i) >= i#Integer(Integer(vlen(v))))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(i) >= i#Integer(Integer(vlen(v)))))) ==> __abort_flag;

{
    // declare local variables
    var __t2: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t3: Value; // IntegerType()
    var __t4: Reference; // ReferenceType(IntegerType())
    var __t5: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $Vector_T_is_well_formed(v);
    __m := UpdateLocal(__m, __frame + 0, v);
    assume $DebugTrackLocal(1, 26, 0, 9457, v);
    assume IsValidU64(i);
    __m := UpdateLocal(__m, __frame + 1, i);
    assume $DebugTrackLocal(1, 26, 1, 9457, i);

    // increase the local counter
    __local_counter := __local_counter + 6;

    // bytecode translation starts here
    call __t2 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __t4 := Vector_borrow(IntegerType(), __t2, GetLocal(__m, __frame + 3));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 26, 9619);
      goto Label_Abort;
    }
    assume IsValidU64(Dereference(__m, __t4)) && IsValidReferenceParameter(__m, __local_counter, __t4);



    call __tmp := ReadRef(__t4);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    __ret0 := GetLocal(__m, __frame + 5);
    assume $DebugTrackLocal(1, 26, 2, 9611, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure VerifyVector_my_get1_verify (v: Value, i: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := VerifyVector_my_get1(v, i);
}

procedure {:inline 1} VerifyVector_my_get2 (v: Reference, i: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(__ret0, select_vector(Dereference(__m, v), i#Integer(i)))));
ensures old(!(b#Boolean(Boolean(i#Integer(i) >= i#Integer(Integer(vlen(Dereference(__m, v)))))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(i) >= i#Integer(Integer(vlen(Dereference(__m, v))))))) ==> __abort_flag;

{
    // declare local variables
    var __t2: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t3: Value; // IntegerType()
    var __t4: Reference; // ReferenceType(IntegerType())
    var __t5: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $Vector_T_is_well_formed(Dereference(__m, v)) && IsValidReferenceParameter(__m, __local_counter, v);
    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 27, 0, 9729, Dereference(__m, v));
    assume IsValidU64(i);
    __m := UpdateLocal(__m, __frame + 1, i);
    assume $DebugTrackLocal(1, 27, 1, 9729, i);

    // increase the local counter
    __local_counter := __local_counter + 6;

    // bytecode translation starts here
    call __t2 := CopyOrMoveRef(v);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __t4 := Vector_borrow(IntegerType(), __t2, GetLocal(__m, __frame + 3));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 27, 9894);
      goto Label_Abort;
    }
    assume IsValidU64(Dereference(__m, __t4)) && IsValidReferenceParameter(__m, __local_counter, __t4);



    call __tmp := ReadRef(__t4);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    __ret0 := GetLocal(__m, __frame + 5);
    assume $DebugTrackLocal(1, 27, 2, 9886, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure VerifyVector_my_get2_verify (v: Reference, i: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := VerifyVector_my_get2(v, i);
}

procedure {:inline 1} VerifyVector_my_get3 (v: Reference, i: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(Integer(i#Integer(__ret0) + i#Integer(Integer(1))), select_vector(Dereference(__m, v), i#Integer(i)))));
ensures old(!(b#Boolean(Boolean(i#Integer(i) >= i#Integer(Integer(vlen(Dereference(__m, v)))))) || b#Boolean(Boolean(IsEqual(select_vector(Dereference(__m, v), i#Integer(i)), Integer(0)))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(i) >= i#Integer(Integer(vlen(Dereference(__m, v)))))) || b#Boolean(Boolean(IsEqual(select_vector(Dereference(__m, v), i#Integer(i)), Integer(0))))) ==> __abort_flag;

{
    // declare local variables
    var __t2: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t3: Value; // IntegerType()
    var __t4: Reference; // ReferenceType(IntegerType())
    var __t5: Value; // IntegerType()
    var __t6: Value; // IntegerType()
    var __t7: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $Vector_T_is_well_formed(Dereference(__m, v)) && IsValidReferenceParameter(__m, __local_counter, v);
    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 28, 0, 9960, Dereference(__m, v));
    assume IsValidU64(i);
    __m := UpdateLocal(__m, __frame + 1, i);
    assume $DebugTrackLocal(1, 28, 1, 9960, i);

    // increase the local counter
    __local_counter := __local_counter + 8;

    // bytecode translation starts here
    call __t2 := CopyOrMoveRef(v);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __t4 := Vector_borrow(IntegerType(), __t2, GetLocal(__m, __frame + 3));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 28, 10150);
      goto Label_Abort;
    }
    assume IsValidU64(Dereference(__m, __t4)) && IsValidReferenceParameter(__m, __local_counter, __t4);



    call __tmp := ReadRef(__t4);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := Sub(GetLocal(__m, __frame + 5), GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 28, 10149);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    __ret0 := GetLocal(__m, __frame + 7);
    assume $DebugTrackLocal(1, 28, 2, 10142, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure VerifyVector_my_get3_verify (v: Reference, i: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := VerifyVector_my_get3(v, i);
}

procedure {:inline 1} VerifyVector_my_get4 (tv0: TypeValue, v: Reference, i: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(__ret0, select_vector(Dereference(__m, v), i#Integer(i)))));
ensures old(!(b#Boolean(Boolean(i#Integer(i) >= i#Integer(Integer(vlen(Dereference(__m, v)))))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(i) >= i#Integer(Integer(vlen(Dereference(__m, v))))))) ==> __abort_flag;

{
    // declare local variables
    var __t2: Reference; // ReferenceType(Vector_T_type_value(tv0))
    var __t3: Value; // IntegerType()
    var __t4: Reference; // ReferenceType(tv0)
    var __t5: Value; // tv0
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $Vector_T_is_well_formed(Dereference(__m, v)) && IsValidReferenceParameter(__m, __local_counter, v);
    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 29, 0, 10244, Dereference(__m, v));
    assume IsValidU64(i);
    __m := UpdateLocal(__m, __frame + 1, i);
    assume $DebugTrackLocal(1, 29, 1, 10244, i);

    // increase the local counter
    __local_counter := __local_counter + 6;

    // bytecode translation starts here
    call __t2 := CopyOrMoveRef(v);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __t4 := Vector_borrow(tv0, __t2, GetLocal(__m, __frame + 3));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 29, 10440);
      goto Label_Abort;
    }
    assume IsValidReferenceParameter(__m, __local_counter, __t4);



    call __tmp := ReadRef(__t4);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    __ret0 := GetLocal(__m, __frame + 5);
    assume $DebugTrackLocal(1, 29, 2, 10432, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure VerifyVector_my_get4_verify (tv0: TypeValue, v: Reference, i: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := VerifyVector_my_get4(tv0, v, i);
}

procedure {:inline 1} VerifyVector_my_set1 (v: Value, i: Value, e: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(__ret0, update_vector(v, i#Integer(i), e))));
ensures old(!(b#Boolean(Boolean(i#Integer(i) >= i#Integer(Integer(vlen(v))))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(i) >= i#Integer(Integer(vlen(v)))))) ==> __abort_flag;

{
    // declare local variables
    var __t3: Value; // IntegerType()
    var __t4: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t5: Value; // IntegerType()
    var __t6: Reference; // ReferenceType(IntegerType())
    var __t7: Value; // Vector_T_type_value(IntegerType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $Vector_T_is_well_formed(v);
    __m := UpdateLocal(__m, __frame + 0, v);
    assume $DebugTrackLocal(1, 30, 0, 10510, v);
    assume IsValidU64(i);
    __m := UpdateLocal(__m, __frame + 1, i);
    assume $DebugTrackLocal(1, 30, 1, 10510, i);
    assume IsValidU64(e);
    __m := UpdateLocal(__m, __frame + 2, e);
    assume $DebugTrackLocal(1, 30, 2, 10510, e);

    // increase the local counter
    __local_counter := __local_counter + 8;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __t4 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __t6 := Vector_borrow_mut(IntegerType(), __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 30, 10649);
      goto Label_Abort;
    }
    assume IsValidU64(Dereference(__m, __t6)) && IsValidReferenceParameter(__m, __local_counter, __t6);


    assume $DebugTrackLocal(1, 30, 0, 10649, GetLocal(__m, __frame + 0));

    call WriteRef(__t6, GetLocal(__m, __frame + 3));
    assume $DebugTrackLocal(1, 30, 0, 10647, GetLocal(__m, __frame + 0));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    __ret0 := GetLocal(__m, __frame + 7);
    assume $DebugTrackLocal(1, 30, 3, 10709, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure VerifyVector_my_set1_verify (v: Value, i: Value, e: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := VerifyVector_my_set1(v, i, e);
}

procedure {:inline 1} VerifyVector_my_set2 (v: Reference, i: Value, e: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(Dereference(__m, v), old(update_vector(Dereference(__m, v), i#Integer(i), e)))));
ensures old(!(b#Boolean(Boolean(i#Integer(i) >= i#Integer(Integer(vlen(Dereference(__m, v)))))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(i) >= i#Integer(Integer(vlen(Dereference(__m, v))))))) ==> __abort_flag;

{
    // declare local variables
    var __t3: Value; // IntegerType()
    var __t4: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t5: Value; // IntegerType()
    var __t6: Reference; // ReferenceType(IntegerType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $Vector_T_is_well_formed(Dereference(__m, v)) && IsValidReferenceParameter(__m, __local_counter, v);
    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 31, 0, 10802, Dereference(__m, v));
    assume IsValidU64(i);
    __m := UpdateLocal(__m, __frame + 1, i);
    assume $DebugTrackLocal(1, 31, 1, 10802, i);
    assume IsValidU64(e);
    __m := UpdateLocal(__m, __frame + 2, e);
    assume $DebugTrackLocal(1, 31, 2, 10802, e);

    // increase the local counter
    __local_counter := __local_counter + 7;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __t4 := CopyOrMoveRef(v);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __t6 := Vector_borrow_mut(IntegerType(), __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 31, 10930);
      goto Label_Abort;
    }
    assume IsValidU64(Dereference(__m, __t6)) && IsValidReferenceParameter(__m, __local_counter, __t6);


    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 31, 0, 10930, Dereference(__m, v));

    call WriteRef(__t6, GetLocal(__m, __frame + 3));
    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 31, 0, 10928, Dereference(__m, v));

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure VerifyVector_my_set2_verify (v: Reference, i: Value, e: Value) returns ()
{
    call InitVerification();
    call VerifyVector_my_set2(v, i, e);
}

procedure {:inline 1} VerifyVector_my_set3 (tv0: TypeValue, v: Reference, i: Value, e: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(Dereference(__m, v), old(update_vector(Dereference(__m, v), i#Integer(i), e)))));
ensures old(!(b#Boolean(Boolean(i#Integer(i) >= i#Integer(Integer(vlen(Dereference(__m, v)))))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(i) >= i#Integer(Integer(vlen(Dereference(__m, v))))))) ==> __abort_flag;

{
    // declare local variables
    var __t3: Value; // tv0
    var __t4: Reference; // ReferenceType(Vector_T_type_value(tv0))
    var __t5: Value; // IntegerType()
    var __t6: Reference; // ReferenceType(tv0)
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $Vector_T_is_well_formed(Dereference(__m, v)) && IsValidReferenceParameter(__m, __local_counter, v);
    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 32, 0, 11051, Dereference(__m, v));
    assume IsValidU64(i);
    __m := UpdateLocal(__m, __frame + 1, i);
    assume $DebugTrackLocal(1, 32, 1, 11051, i);
    __m := UpdateLocal(__m, __frame + 2, e);
    assume $DebugTrackLocal(1, 32, 2, 11051, e);

    // increase the local counter
    __local_counter := __local_counter + 7;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __t4 := CopyOrMoveRef(v);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __t6 := Vector_borrow_mut(tv0, __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 32, 11304);
      goto Label_Abort;
    }
    assume IsValidReferenceParameter(__m, __local_counter, __t6);


    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 32, 0, 11304, Dereference(__m, v));

    call WriteRef(__t6, GetLocal(__m, __frame + 3));
    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 32, 0, 11302, Dereference(__m, v));

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure VerifyVector_my_set3_verify (tv0: TypeValue, v: Reference, i: Value, e: Value) returns ()
{
    call InitVerification();
    call VerifyVector_my_set3(tv0, v, i, e);
}

procedure {:inline 1} VerifyVector_my_is_empty (v: Reference) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var __t1: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t2: Value; // IntegerType()
    var __t3: Value; // IntegerType()
    var __t4: Value; // BooleanType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $Vector_T_is_well_formed(Dereference(__m, v)) && IsValidReferenceParameter(__m, __local_counter, v);
    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 33, 0, 11444, Dereference(__m, v));

    // increase the local counter
    __local_counter := __local_counter + 5;

    // bytecode translation starts here
    call __t1 := CopyOrMoveRef(v);

    call __t2 := Vector_length(IntegerType(), __t1);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 33, 11676);
      goto Label_Abort;
    }
    assume IsValidU64(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    __tmp := Boolean(IsEqual(GetLocal(__m, __frame + 2), GetLocal(__m, __frame + 3)));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    __ret0 := GetLocal(__m, __frame + 4);
    assume $DebugTrackLocal(1, 33, 1, 11669, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure VerifyVector_my_is_empty_verify (v: Reference) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := VerifyVector_my_is_empty(v);
}

procedure {:inline 1} VerifyVector_test_slice1 () returns (__ret0: Value, __ret1: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(__ret0, slice_vector(__ret1, i#Integer(Integer(1)), i#Integer(Integer(3))))));
ensures b#Boolean(Boolean(!IsEqual(__ret0, slice_vector(__ret1, i#Integer(Integer(0)), i#Integer(Integer(2))))));
ensures b#Boolean(Boolean(IsEqual(__ret0, slice_vector(__ret1, i#Integer(Integer(4)), i#Integer(Integer(6))))));
ensures b#Boolean(Boolean(IsEqual(slice_vector(__ret0, i#Integer(Integer(0)), i#Integer(Integer(2))), slice_vector(__ret1, i#Integer(Integer(4)), i#Integer(Integer(6))))));
ensures b#Boolean(Boolean(IsEqual(slice_vector(__ret0, i#Integer(Integer(1)), i#Integer(Integer(2))), slice_vector(__ret1, i#Integer(Integer(2)), i#Integer(Integer(3))))));
ensures b#Boolean(Boolean(IsEqual(slice_vector(__ret1, i#Integer(Integer(1)), i#Integer(Integer(3))), slice_vector(__ret1, i#Integer(Integer(4)), i#Integer(Integer(6))))));
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
    var __t14: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t15: Value; // IntegerType()
    var __t16: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t17: Value; // IntegerType()
    var __t18: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t19: Value; // IntegerType()
    var __t20: Value; // Vector_T_type_value(IntegerType())
    var __t21: Value; // Vector_T_type_value(IntegerType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments

    // increase the local counter
    __local_counter := __local_counter + 22;

    // bytecode translation starts here
    call __t2 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 34, 12086);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 34, 0, 12080, __tmp);

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 34, 12121);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 34, 1, 12115, __tmp);

    call __t4 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call Vector_push_back(IntegerType(), __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 34, 12150);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 34, 0, 12150, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 34, 1, 12150, GetLocal(__m, __frame + 1));

    call __t6 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call Vector_push_back(IntegerType(), __t6, GetLocal(__m, __frame + 7));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 34, 12194);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 34, 0, 12194, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 34, 1, 12194, GetLocal(__m, __frame + 1));

    call __t8 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call Vector_push_back(IntegerType(), __t8, GetLocal(__m, __frame + 9));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 34, 12239);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 34, 0, 12239, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 34, 1, 12239, GetLocal(__m, __frame + 1));

    call __t10 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call Vector_push_back(IntegerType(), __t10, GetLocal(__m, __frame + 11));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 34, 12283);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 34, 0, 12283, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 34, 1, 12283, GetLocal(__m, __frame + 1));

    call __t12 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call Vector_push_back(IntegerType(), __t12, GetLocal(__m, __frame + 13));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 34, 12327);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 34, 0, 12327, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 34, 1, 12327, GetLocal(__m, __frame + 1));

    call __t14 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(3);
    __m := UpdateLocal(__m, __frame + 15, __tmp);

    call Vector_push_back(IntegerType(), __t14, GetLocal(__m, __frame + 15));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 34, 12371);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 34, 0, 12371, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 34, 1, 12371, GetLocal(__m, __frame + 1));

    call __t16 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    call Vector_push_back(IntegerType(), __t16, GetLocal(__m, __frame + 17));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 34, 12415);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 34, 0, 12415, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 34, 1, 12415, GetLocal(__m, __frame + 1));

    call __t18 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 19, __tmp);

    call Vector_push_back(IntegerType(), __t18, GetLocal(__m, __frame + 19));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 34, 12459);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 34, 0, 12459, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 34, 1, 12459, GetLocal(__m, __frame + 1));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 20, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 21, __tmp);

    __ret0 := GetLocal(__m, __frame + 20);
    assume $DebugTrackLocal(1, 34, 2, 12503, __ret0);
    __ret1 := GetLocal(__m, __frame + 21);
    assume $DebugTrackLocal(1, 34, 3, 12503, __ret1);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    __ret1 := DefaultValue;
}

procedure VerifyVector_test_slice1_verify () returns (__ret0: Value, __ret1: Value)
{
    call InitVerification();
    call __ret0, __ret1 := VerifyVector_test_slice1();
}

procedure {:inline 1} VerifyVector_my_push_back1 (v: Reference, e: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(Integer(vlen(Dereference(__m, v))), Integer(i#Integer(Integer(vlen(old(Dereference(__m, v))))) + i#Integer(Integer(1))))));
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(select_vector(Dereference(__m, v), i#Integer(Integer(i#Integer(Integer(vlen(Dereference(__m, v)))) - i#Integer(Integer(1))))), Integer(i#Integer(e) + i#Integer(Integer(1))))));
ensures old(!(b#Boolean(Boolean(IsEqual(e, Integer(9223372036854775807)))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(IsEqual(e, Integer(9223372036854775807))))) ==> __abort_flag;

{
    // declare local variables
    var x: Value; // IntegerType()
    var __t3: Value; // IntegerType()
    var __t4: Value; // IntegerType()
    var __t5: Value; // IntegerType()
    var __t6: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t7: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $Vector_T_is_well_formed(Dereference(__m, v)) && IsValidReferenceParameter(__m, __local_counter, v);
    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 35, 0, 12593, Dereference(__m, v));
    assume IsValidU64(e);
    __m := UpdateLocal(__m, __frame + 1, e);
    assume $DebugTrackLocal(1, 35, 1, 12593, e);

    // increase the local counter
    __local_counter := __local_counter + 8;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 3), GetLocal(__m, __frame + 4));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 35, 12797);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    assume $DebugTrackLocal(1, 35, 2, 12793, __tmp);

    call __t6 := CopyOrMoveRef(v);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call Vector_push_back(IntegerType(), __t6, GetLocal(__m, __frame + 7));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 35, 12818);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 35, 0, 12818, Dereference(__m, v));

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure VerifyVector_my_push_back1_verify (v: Reference, e: Value) returns ()
{
    call InitVerification();
    call VerifyVector_my_push_back1(v, e);
}

procedure {:inline 1} VerifyVector_my_push_back2 (v: Reference, e: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(Integer(vlen(Dereference(__m, v))), Integer(i#Integer(Integer(vlen(old(Dereference(__m, v))))) + i#Integer(Integer(2))))));
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(select_vector(Dereference(__m, v), i#Integer(Integer(i#Integer(Integer(vlen(Dereference(__m, v)))) - i#Integer(Integer(1))))), e)));
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(select_vector(Dereference(__m, v), i#Integer(Integer(i#Integer(Integer(vlen(Dereference(__m, v)))) - i#Integer(Integer(2))))), Integer(i#Integer(e) + i#Integer(Integer(1))))));
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(old(Dereference(__m, v)), slice_vector(Dereference(__m, v), i#Integer(Integer(0)), i#Integer(Integer(i#Integer(Integer(vlen(Dereference(__m, v)))) - i#Integer(Integer(2))))))));
ensures old(!(b#Boolean(Boolean(IsEqual(e, Integer(9223372036854775807)))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(IsEqual(e, Integer(9223372036854775807))))) ==> __abort_flag;

{
    // declare local variables
    var __t2: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t3: Value; // IntegerType()
    var __t4: Value; // IntegerType()
    var __t5: Value; // IntegerType()
    var __t6: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t7: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $Vector_T_is_well_formed(Dereference(__m, v)) && IsValidReferenceParameter(__m, __local_counter, v);
    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 36, 0, 12903, Dereference(__m, v));
    assume IsValidU64(e);
    __m := UpdateLocal(__m, __frame + 1, e);
    assume $DebugTrackLocal(1, 36, 1, 12903, e);

    // increase the local counter
    __local_counter := __local_counter + 8;

    // bytecode translation starts here
    call __t2 := CopyOrMoveRef(v);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 3), GetLocal(__m, __frame + 4));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 36, 13180);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call Vector_push_back(IntegerType(), __t2, GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 36, 13149);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 36, 0, 13149, Dereference(__m, v));

    call __t6 := CopyOrMoveRef(v);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call Vector_push_back(IntegerType(), __t6, GetLocal(__m, __frame + 7));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 36, 13200);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 36, 0, 13200, Dereference(__m, v));

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure VerifyVector_my_push_back2_verify (v: Reference, e: Value) returns ()
{
    call InitVerification();
    call VerifyVector_my_push_back2(v, e);
}

procedure {:inline 1} VerifyVector_my_push_back3 (v: Reference, e: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(Integer(vlen(Dereference(__m, v))), Integer(i#Integer(Integer(vlen(old(Dereference(__m, v))))) + i#Integer(Integer(1))))));
ensures b#Boolean(Boolean(IsEqual(select_vector(Dereference(__m, v), i#Integer(Integer(i#Integer(Integer(vlen(Dereference(__m, v)))) - i#Integer(Integer(1))))), e)));
ensures b#Boolean(Boolean(IsEqual(old(Dereference(__m, v)), slice_vector(Dereference(__m, v), i#Integer(Integer(0)), i#Integer(Integer(i#Integer(Integer(vlen(Dereference(__m, v)))) - i#Integer(Integer(1))))))));
{
    // declare local variables
    var __t2: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t3: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $Vector_T_is_well_formed(Dereference(__m, v)) && IsValidReferenceParameter(__m, __local_counter, v);
    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 37, 0, 13334, Dereference(__m, v));
    assume IsValidU64(e);
    __m := UpdateLocal(__m, __frame + 1, e);
    assume $DebugTrackLocal(1, 37, 1, 13334, e);

    // increase the local counter
    __local_counter := __local_counter + 4;

    // bytecode translation starts here
    call __t2 := CopyOrMoveRef(v);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call Vector_push_back(IntegerType(), __t2, GetLocal(__m, __frame + 3));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 37, 13509);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 37, 0, 13509, Dereference(__m, v));

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure VerifyVector_my_push_back3_verify (v: Reference, e: Value) returns ()
{
    call InitVerification();
    call VerifyVector_my_push_back3(v, e);
}

procedure {:inline 1} VerifyVector_my_push_back4 (tv0: TypeValue, v: Reference, e: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(Integer(vlen(Dereference(__m, v))), Integer(i#Integer(Integer(vlen(old(Dereference(__m, v))))) + i#Integer(Integer(1))))));
ensures b#Boolean(Boolean(IsEqual(select_vector(Dereference(__m, v), i#Integer(Integer(i#Integer(Integer(vlen(Dereference(__m, v)))) - i#Integer(Integer(1))))), e)));
ensures b#Boolean(Boolean(IsEqual(old(Dereference(__m, v)), slice_vector(Dereference(__m, v), i#Integer(Integer(0)), i#Integer(Integer(i#Integer(Integer(vlen(Dereference(__m, v)))) - i#Integer(Integer(1))))))));
{
    // declare local variables
    var __t2: Reference; // ReferenceType(Vector_T_type_value(tv0))
    var __t3: Value; // tv0
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $Vector_T_is_well_formed(Dereference(__m, v)) && IsValidReferenceParameter(__m, __local_counter, v);
    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 38, 0, 13618, Dereference(__m, v));
    __m := UpdateLocal(__m, __frame + 1, e);
    assume $DebugTrackLocal(1, 38, 1, 13618, e);

    // increase the local counter
    __local_counter := __local_counter + 4;

    // bytecode translation starts here
    call __t2 := CopyOrMoveRef(v);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call Vector_push_back(tv0, __t2, GetLocal(__m, __frame + 3));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 38, 13810);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 38, 0, 13810, Dereference(__m, v));

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure VerifyVector_my_push_back4_verify (tv0: TypeValue, v: Reference, e: Value) returns ()
{
    call InitVerification();
    call VerifyVector_my_push_back4(tv0, v, e);
}

procedure {:inline 1} VerifyVector_pop_back1 (v: Reference) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(Integer(vlen(Dereference(__m, v))), Integer(i#Integer(old(Integer(vlen(Dereference(__m, v))))) - i#Integer(Integer(1))))));
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(old(select_vector(Dereference(__m, v), i#Integer(Integer(i#Integer(Integer(vlen(Dereference(__m, v)))) - i#Integer(Integer(1)))))), __ret0)));
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(Dereference(__m, v), old(slice_vector(Dereference(__m, v), i#Integer(Integer(0)), i#Integer(Integer(i#Integer(Integer(vlen(Dereference(__m, v)))) - i#Integer(Integer(1)))))))));
ensures old(!(b#Boolean(Boolean(IsEqual(Integer(vlen(Dereference(__m, v))), Integer(0)))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(IsEqual(Integer(vlen(Dereference(__m, v))), Integer(0))))) ==> __abort_flag;

{
    // declare local variables
    var __t1: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t2: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $Vector_T_is_well_formed(Dereference(__m, v)) && IsValidReferenceParameter(__m, __local_counter, v);
    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 39, 0, 13948, Dereference(__m, v));

    // increase the local counter
    __local_counter := __local_counter + 3;

    // bytecode translation starts here
    call __t1 := CopyOrMoveRef(v);

    call __t2 := Vector_pop_back(IntegerType(), __t1);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 39, 14160);
      goto Label_Abort;
    }
    assume IsValidU64(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);
    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 39, 0, 14160, Dereference(__m, v));

    __ret0 := GetLocal(__m, __frame + 2);
    assume $DebugTrackLocal(1, 39, 1, 14153, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure VerifyVector_pop_back1_verify (v: Reference) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := VerifyVector_pop_back1(v);
}

procedure {:inline 1} VerifyVector_pop_back2 (tv0: TypeValue, v: Reference) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(Integer(vlen(Dereference(__m, v))), Integer(i#Integer(old(Integer(vlen(Dereference(__m, v))))) - i#Integer(Integer(1))))));
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(__ret0, old(select_vector(Dereference(__m, v), i#Integer(Integer(i#Integer(Integer(vlen(Dereference(__m, v)))) - i#Integer(Integer(1)))))))));
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(Dereference(__m, v), old(slice_vector(Dereference(__m, v), i#Integer(Integer(0)), i#Integer(Integer(i#Integer(Integer(vlen(Dereference(__m, v)))) - i#Integer(Integer(1)))))))));
ensures old(!(b#Boolean(Boolean(IsEqual(Integer(vlen(Dereference(__m, v))), Integer(0)))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(IsEqual(Integer(vlen(Dereference(__m, v))), Integer(0))))) ==> __abort_flag;

{
    // declare local variables
    var __t1: Reference; // ReferenceType(Vector_T_type_value(tv0))
    var __t2: Value; // tv0
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $Vector_T_is_well_formed(Dereference(__m, v)) && IsValidReferenceParameter(__m, __local_counter, v);
    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 40, 0, 14243, Dereference(__m, v));

    // increase the local counter
    __local_counter := __local_counter + 3;

    // bytecode translation starts here
    call __t1 := CopyOrMoveRef(v);

    call __t2 := Vector_pop_back(tv0, __t1);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 40, 14472);
      goto Label_Abort;
    }

    __m := UpdateLocal(__m, __frame + 2, __t2);
    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 40, 0, 14472, Dereference(__m, v));

    __ret0 := GetLocal(__m, __frame + 2);
    assume $DebugTrackLocal(1, 40, 1, 14465, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure VerifyVector_pop_back2_verify (tv0: TypeValue, v: Reference) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := VerifyVector_pop_back2(tv0, v);
}

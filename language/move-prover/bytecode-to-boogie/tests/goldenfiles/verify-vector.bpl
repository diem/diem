
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
    var __t2: Value; // Vector_T_type_value(tv0)
    var __t3: Value; // Vector_T_type_value(tv0)
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
      assume $DebugTrackAbort(1, 0, 235);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 0, 0, 229, __tmp);

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 0, 270);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 0, 1, 264, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    __ret0 := GetLocal(__m, __frame + 4);
    assume $DebugTrackLocal(1, 0, 2, 299, __ret0);
    __ret1 := GetLocal(__m, __frame + 5);
    assume $DebugTrackLocal(1, 0, 3, 299, __ret1);
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
    var __t3: Value; // Vector_T_type_value(tv0)
    var __t4: Value; // Vector_T_type_value(tv0)
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
      assume $DebugTrackAbort(1, 1, 545);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 1, 0, 539, __tmp);

    call __t4 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 1, 580);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t4);

    __m := UpdateLocal(__m, __frame + 4, __t4);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 1, 1, 574, __tmp);

    call __t5 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call Vector_push_back(IntegerType(), __t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 1, 609);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 1, 0, 609, GetLocal(__m, __frame + 0));

    call __t7 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __t8 := Vector_pop_back(IntegerType(), __t7);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 1, 657);
      goto Label_Abort;
    }
    assume IsValidU64(__t8);

    __m := UpdateLocal(__m, __frame + 8, __t8);
    assume $DebugTrackLocal(1, 1, 0, 657, GetLocal(__m, __frame + 0));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    assume $DebugTrackLocal(1, 1, 2, 653, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    __ret0 := GetLocal(__m, __frame + 9);
    assume $DebugTrackLocal(1, 1, 3, 697, __ret0);
    __ret1 := GetLocal(__m, __frame + 10);
    assume $DebugTrackLocal(1, 1, 4, 697, __ret1);
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
    var __t2: Value; // Vector_T_type_value(tv0)
    var __t3: Value; // Vector_T_type_value(tv0)
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
      assume $DebugTrackAbort(1, 2, 925);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 2, 0, 919, __tmp);

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 2, 960);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 2, 1, 954, __tmp);

    call __t4 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call Vector_push_back(IntegerType(), __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 2, 989);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 2, 0, 989, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 2, 1, 989, GetLocal(__m, __frame + 1));

    call __t6 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call Vector_push_back(IntegerType(), __t6, GetLocal(__m, __frame + 7));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 2, 1033);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 2, 0, 1033, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 2, 1, 1033, GetLocal(__m, __frame + 1));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    __ret0 := GetLocal(__m, __frame + 8);
    assume $DebugTrackLocal(1, 2, 2, 1077, __ret0);
    __ret1 := GetLocal(__m, __frame + 9);
    assume $DebugTrackLocal(1, 2, 3, 1077, __ret1);
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
    var __t2: Value; // Vector_T_type_value(tv0)
    var __t3: Value; // Vector_T_type_value(tv0)
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
      assume $DebugTrackAbort(1, 3, 1307);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 3, 0, 1301, __tmp);

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 3, 1342);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 3, 1, 1336, __tmp);

    call __t4 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call Vector_push_back(IntegerType(), __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 3, 1371);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 3, 0, 1371, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 3, 1, 1371, GetLocal(__m, __frame + 1));

    call __t6 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call Vector_push_back(IntegerType(), __t6, GetLocal(__m, __frame + 7));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 3, 1415);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 3, 0, 1415, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 3, 1, 1415, GetLocal(__m, __frame + 1));

    call __t8 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call Vector_push_back(IntegerType(), __t8, GetLocal(__m, __frame + 9));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 3, 1459);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 3, 0, 1459, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 3, 1, 1459, GetLocal(__m, __frame + 1));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    __ret0 := GetLocal(__m, __frame + 10);
    assume $DebugTrackLocal(1, 3, 2, 1503, __ret0);
    __ret1 := GetLocal(__m, __frame + 11);
    assume $DebugTrackLocal(1, 3, 3, 1503, __ret1);
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
    var __t2: Value; // Vector_T_type_value(tv0)
    var __t3: Value; // Vector_T_type_value(tv0)
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
      assume $DebugTrackAbort(1, 4, 1731);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 4, 0, 1725, __tmp);

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 4, 1766);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 4, 1, 1760, __tmp);

    call __t4 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call Vector_push_back(IntegerType(), __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 4, 1795);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 4, 0, 1795, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 4, 1, 1795, GetLocal(__m, __frame + 1));

    call __t6 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call Vector_push_back(IntegerType(), __t6, GetLocal(__m, __frame + 7));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 4, 1839);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 4, 0, 1839, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 4, 1, 1839, GetLocal(__m, __frame + 1));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    __ret0 := GetLocal(__m, __frame + 8);
    assume $DebugTrackLocal(1, 4, 2, 1883, __ret0);
    __ret1 := GetLocal(__m, __frame + 9);
    assume $DebugTrackLocal(1, 4, 3, 1883, __ret1);
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
    var __t2: Value; // Vector_T_type_value(tv0)
    var __t3: Value; // Vector_T_type_value(tv0)
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
      assume $DebugTrackAbort(1, 5, 2121);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 5, 0, 2115, __tmp);

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 5, 2156);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 5, 1, 2150, __tmp);

    call __t4 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call Vector_reverse(IntegerType(), __t4);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 5, 2185);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 5, 0, 2185, GetLocal(__m, __frame + 0));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    __ret0 := GetLocal(__m, __frame + 5);
    assume $DebugTrackLocal(1, 5, 2, 2224, __ret0);
    __ret1 := GetLocal(__m, __frame + 6);
    assume $DebugTrackLocal(1, 5, 3, 2224, __ret1);
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
    var __t2: Value; // Vector_T_type_value(tv0)
    var __t3: Value; // Vector_T_type_value(tv0)
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
      assume $DebugTrackAbort(1, 6, 2468);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 6, 0, 2462, __tmp);

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 6, 2503);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 6, 1, 2497, __tmp);

    call __t4 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call Vector_push_back(IntegerType(), __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 6, 2532);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 6, 0, 2532, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 6, 1, 2532, GetLocal(__m, __frame + 1));

    call __t6 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call Vector_push_back(IntegerType(), __t6, GetLocal(__m, __frame + 7));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 6, 2576);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 6, 0, 2576, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 6, 1, 2576, GetLocal(__m, __frame + 1));

    call __t8 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call Vector_push_back(IntegerType(), __t8, GetLocal(__m, __frame + 9));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 6, 2620);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 6, 0, 2620, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 6, 1, 2620, GetLocal(__m, __frame + 1));

    call __t10 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call Vector_push_back(IntegerType(), __t10, GetLocal(__m, __frame + 11));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 6, 2664);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 6, 0, 2664, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 6, 1, 2664, GetLocal(__m, __frame + 1));

    call __t12 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call Vector_reverse(IntegerType(), __t12);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 6, 2708);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 6, 0, 2708, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 6, 1, 2708, GetLocal(__m, __frame + 1));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    __ret0 := GetLocal(__m, __frame + 13);
    assume $DebugTrackLocal(1, 6, 2, 2747, __ret0);
    __ret1 := GetLocal(__m, __frame + 14);
    assume $DebugTrackLocal(1, 6, 3, 2747, __ret1);
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
    var __t1: Value; // Vector_T_type_value(tv0)
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
      assume $DebugTrackAbort(1, 7, 2970);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t1);

    __m := UpdateLocal(__m, __frame + 1, __t1);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 7, 0, 2964, __tmp);

    call __t2 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call Vector_push_back(IntegerType(), __t2, GetLocal(__m, __frame + 3));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 7, 2999);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 7, 0, 2999, GetLocal(__m, __frame + 0));

    call __t4 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call Vector_swap(IntegerType(), __t4, GetLocal(__m, __frame + 5), GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 7, 3043);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 7, 0, 3043, GetLocal(__m, __frame + 0));

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
    var __t1: Value; // Vector_T_type_value(tv0)
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
      assume $DebugTrackAbort(1, 8, 3286);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t1);

    __m := UpdateLocal(__m, __frame + 1, __t1);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 8, 0, 3280, __tmp);

    call __t2 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call Vector_push_back(IntegerType(), __t2, GetLocal(__m, __frame + 3));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 8, 3315);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 8, 0, 3315, GetLocal(__m, __frame + 0));

    call __t4 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call Vector_swap(IntegerType(), __t4, GetLocal(__m, __frame + 5), GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 8, 3359);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 8, 0, 3359, GetLocal(__m, __frame + 0));

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
    var __t2: Value; // Vector_T_type_value(tv0)
    var __t3: Value; // Vector_T_type_value(tv0)
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
      assume $DebugTrackAbort(1, 9, 3620);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 9, 0, 3614, __tmp);

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 9, 3655);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 9, 1, 3649, __tmp);

    call __t4 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call Vector_push_back(IntegerType(), __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 9, 3684);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 9, 0, 3684, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 9, 1, 3684, GetLocal(__m, __frame + 1));

    call __t6 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call Vector_push_back(IntegerType(), __t6, GetLocal(__m, __frame + 7));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 9, 3728);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 9, 0, 3728, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 9, 1, 3728, GetLocal(__m, __frame + 1));

    call __t8 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call Vector_push_back(IntegerType(), __t8, GetLocal(__m, __frame + 9));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 9, 3772);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 9, 0, 3772, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 9, 1, 3772, GetLocal(__m, __frame + 1));

    call __t10 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call Vector_push_back(IntegerType(), __t10, GetLocal(__m, __frame + 11));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 9, 3816);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 9, 0, 3816, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 9, 1, 3816, GetLocal(__m, __frame + 1));

    call __t12 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call Vector_swap(IntegerType(), __t12, GetLocal(__m, __frame + 13), GetLocal(__m, __frame + 14));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 9, 3860);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 9, 0, 3860, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 9, 1, 3860, GetLocal(__m, __frame + 1));

    call __t15 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    call Vector_swap(IntegerType(), __t15, GetLocal(__m, __frame + 16), GetLocal(__m, __frame + 17));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 9, 3902);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 9, 0, 3902, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 9, 1, 3902, GetLocal(__m, __frame + 1));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 19, __tmp);

    __ret0 := GetLocal(__m, __frame + 18);
    assume $DebugTrackLocal(1, 9, 2, 3944, __ret0);
    __ret1 := GetLocal(__m, __frame + 19);
    assume $DebugTrackLocal(1, 9, 3, 3944, __ret1);
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
    var __t2: Value; // Vector_T_type_value(tv0)
    var __t3: Value; // Vector_T_type_value(tv0)
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
      assume $DebugTrackAbort(1, 10, 4180);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 10, 0, 4174, __tmp);

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 10, 4215);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 10, 1, 4209, __tmp);

    call __t4 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call Vector_push_back(IntegerType(), __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 10, 4244);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 10, 0, 4244, GetLocal(__m, __frame + 0));

    call __t6 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __t7 := Vector_length(IntegerType(), __t6);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 10, 4296);
      goto Label_Abort;
    }
    assume IsValidU64(__t7);

    __m := UpdateLocal(__m, __frame + 7, __t7);

    call __t8 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __t9 := Vector_length(IntegerType(), __t8);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 10, 4323);
      goto Label_Abort;
    }
    assume IsValidU64(__t9);

    __m := UpdateLocal(__m, __frame + 9, __t9);

    __ret0 := GetLocal(__m, __frame + 7);
    assume $DebugTrackLocal(1, 10, 2, 4288, __ret0);
    __ret1 := GetLocal(__m, __frame + 9);
    assume $DebugTrackLocal(1, 10, 3, 4288, __ret1);
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
    assume $DebugTrackLocal(1, 11, 0, 4419, v);

    // increase the local counter
    __local_counter := __local_counter + 15;

    // bytecode translation starts here
    call __t3 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __t4 := Vector_length(IntegerType(), __t3);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 11, 4558);
      goto Label_Abort;
    }
    assume IsValidU64(__t4);

    __m := UpdateLocal(__m, __frame + 4, __t4);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 11, 1, 4554, __tmp);

    call __t5 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call Vector_push_back(IntegerType(), __t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 11, 4591);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 11, 0, 4591, GetLocal(__m, __frame + 0));

    call __t7 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call Vector_push_back(IntegerType(), __t7, GetLocal(__m, __frame + 8));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 11, 4633);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 11, 0, 4633, GetLocal(__m, __frame + 0));

    call __t9 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(3);
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call Vector_push_back(IntegerType(), __t9, GetLocal(__m, __frame + 10));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 11, 4675);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 11, 0, 4675, GetLocal(__m, __frame + 0));

    call __t11 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __t12 := Vector_length(IntegerType(), __t11);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 11, 4721);
      goto Label_Abort;
    }
    assume IsValidU64(__t12);

    __m := UpdateLocal(__m, __frame + 12, __t12);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 12));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    assume $DebugTrackLocal(1, 11, 2, 4717, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    __ret0 := GetLocal(__m, __frame + 13);
    assume $DebugTrackLocal(1, 11, 3, 4754, __ret0);
    __ret1 := GetLocal(__m, __frame + 14);
    assume $DebugTrackLocal(1, 11, 4, 4754, __ret1);
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
    assume $DebugTrackLocal(1, 12, 0, 4817, v);

    // increase the local counter
    __local_counter := __local_counter + 2;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    __ret0 := GetLocal(__m, __frame + 1);
    assume $DebugTrackLocal(1, 12, 1, 4906, __ret0);
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
    assume $DebugTrackLocal(1, 13, 0, 4978, v);

    // increase the local counter
    __local_counter := __local_counter + 4;

    // bytecode translation starts here
    call __t1 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call Vector_reverse(IntegerType(), __t1);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 13, 5067);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 13, 0, 5067, GetLocal(__m, __frame + 0));

    call __t2 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call Vector_reverse(IntegerType(), __t2);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 13, 5104);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 13, 0, 5104, GetLocal(__m, __frame + 0));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    __ret0 := GetLocal(__m, __frame + 3);
    assume $DebugTrackLocal(1, 13, 1, 5141, __ret0);
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
    assume $DebugTrackLocal(1, 14, 0, 5234, v);

    // increase the local counter
    __local_counter := __local_counter + 18;

    // bytecode translation starts here
    call __t2 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __t3 := Vector_length(IntegerType(), __t2);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 14, 5347);
      goto Label_Abort;
    }
    assume IsValidU64(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 14, 1, 5343, __tmp);

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
      assume $DebugTrackAbort(1, 14, 5500);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call Vector_swap(IntegerType(), __t10, GetLocal(__m, __frame + 11), GetLocal(__m, __frame + 14));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 14, 5472);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 14, 0, 5472, GetLocal(__m, __frame + 0));

    goto Label_21;

Label_19:
    call __t15 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call Vector_reverse(IntegerType(), __t15);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 14, 5561);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 14, 0, 5561, GetLocal(__m, __frame + 0));

Label_21:
    call __t16 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call Vector_reverse(IntegerType(), __t16);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 14, 5622);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 14, 0, 5622, GetLocal(__m, __frame + 0));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    __ret0 := GetLocal(__m, __frame + 17);
    assume $DebugTrackLocal(1, 14, 2, 5659, __ret0);
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
    var __t4: Value; // Vector_T_type_value(tv0)
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
    assume $DebugTrackLocal(1, 15, 0, 5778, v);

    // increase the local counter
    __local_counter := __local_counter + 6;

    // bytecode translation starts here
    call __t1 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __t2 := Vector_is_empty(IntegerType(), __t1);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 15, 5882);
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
      assume $DebugTrackAbort(1, 15, 5923);
      goto Label_Abort;
    }

    call __t4 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 15, 5978);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t4);

    __m := UpdateLocal(__m, __frame + 4, __t4);

    __ret0 := GetLocal(__m, __frame + 4);
    assume $DebugTrackLocal(1, 15, 1, 5971, __ret0);
    return;

Label_7:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    __ret0 := GetLocal(__m, __frame + 5);
    assume $DebugTrackLocal(1, 15, 1, 6036, __ret0);
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
    assume $DebugTrackLocal(1, 16, 0, 6205, v);

    // increase the local counter
    __local_counter := __local_counter + 7;

    // bytecode translation starts here
    call __t1 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __t2 := Vector_is_empty(IntegerType(), __t1);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 16, 6285);
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
      assume $DebugTrackAbort(1, 16, 6326);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 16, 0, 6326, GetLocal(__m, __frame + 0));

    goto Label_10;

Label_8:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call Vector_destroy_empty(IntegerType(), GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 16, 6394);
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
    var __t3: Value; // Vector_T_type_value(tv0)
    var __t4: Value; // Vector_T_type_value(tv0)
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
    assume $DebugTrackLocal(1, 17, 0, 6496, x);

    // increase the local counter
    __local_counter := __local_counter + 19;

    // bytecode translation starts here
    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 17, 6662);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 17, 1, 6656, __tmp);

    call __t4 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 17, 6697);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t4);

    __m := UpdateLocal(__m, __frame + 4, __t4);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    assume $DebugTrackLocal(1, 17, 2, 6691, __tmp);

    call __t5 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call Vector_push_back(IntegerType(), __t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 17, 6726);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 17, 1, 6726, GetLocal(__m, __frame + 1));
    assume $DebugTrackLocal(1, 17, 2, 6726, GetLocal(__m, __frame + 2));

    call __t7 := BorrowLoc(__frame + 2, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call Vector_push_back(IntegerType(), __t7, GetLocal(__m, __frame + 8));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 17, 6770);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 17, 1, 6770, GetLocal(__m, __frame + 1));
    assume $DebugTrackLocal(1, 17, 2, 6770, GetLocal(__m, __frame + 2));

    call __t9 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call Vector_set(IntegerType(), __t9, GetLocal(__m, __frame + 10), GetLocal(__m, __frame + 11));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 17, 6814);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 17, 1, 6814, GetLocal(__m, __frame + 1));
    assume $DebugTrackLocal(1, 17, 2, 6814, GetLocal(__m, __frame + 2));

    call __t12 := BorrowLoc(__frame + 2, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __t14 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 15, __tmp);

    call __t16 := Vector_get(IntegerType(), __t14, GetLocal(__m, __frame + 15));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 17, 6890);
      goto Label_Abort;
    }
    assume IsValidU64(__t16);

    __m := UpdateLocal(__m, __frame + 16, __t16);

    call Vector_set(IntegerType(), __t12, GetLocal(__m, __frame + 13), GetLocal(__m, __frame + 16));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 17, 6861);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 17, 1, 6861, GetLocal(__m, __frame + 1));
    assume $DebugTrackLocal(1, 17, 2, 6861, GetLocal(__m, __frame + 2));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    __ret0 := GetLocal(__m, __frame + 17);
    assume $DebugTrackLocal(1, 17, 3, 6926, __ret0);
    __ret1 := GetLocal(__m, __frame + 18);
    assume $DebugTrackLocal(1, 17, 4, 6926, __ret1);
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
    var __t1: Value; // Vector_T_type_value(tv0)
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
      assume $DebugTrackAbort(1, 18, 7084);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t1);

    __m := UpdateLocal(__m, __frame + 1, __t1);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 18, 0, 7078, __tmp);

    call __t2 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(7);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call Vector_push_back(IntegerType(), __t2, GetLocal(__m, __frame + 3));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 18, 7113);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 18, 0, 7113, GetLocal(__m, __frame + 0));

    call __t4 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __t6 := Vector_get(IntegerType(), __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 18, 7164);
      goto Label_Abort;
    }
    assume IsValidU64(__t6);

    __m := UpdateLocal(__m, __frame + 6, __t6);

    __ret0 := GetLocal(__m, __frame + 6);
    assume $DebugTrackLocal(1, 18, 1, 7157, __ret0);
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
    var __t2: Value; // Vector_T_type_value(tv0)
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
      assume $DebugTrackAbort(1, 19, 7378);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 19, 1, 7372, __tmp);

    call __t3 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call Vector_push_back(IntegerType(), __t3, GetLocal(__m, __frame + 4));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 19, 7407);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 19, 1, 7407, GetLocal(__m, __frame + 1));

    call __t5 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __t7 := Vector_get(IntegerType(), __t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 19, 7455);
      goto Label_Abort;
    }
    assume IsValidU64(__t7);

    __m := UpdateLocal(__m, __frame + 7, __t7);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 19, 0, 7451, __tmp);

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
    var __t2: Value; // Vector_T_type_value(tv0)
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
      assume $DebugTrackAbort(1, 20, 7669);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 20, 0, 7663, __tmp);

    call __t3 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(7);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call Vector_push_back(IntegerType(), __t3, GetLocal(__m, __frame + 4));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 20, 7698);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 20, 0, 7698, GetLocal(__m, __frame + 0));

    call __t5 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __t7 := Vector_borrow(IntegerType(), __t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 20, 7746);
      goto Label_Abort;
    }
    assume IsValidU64(Dereference(__m, __t7)) && IsValidReferenceParameter(__m, __local_counter, __t7);



    call y := CopyOrMoveRef(__t7);
    assume IsValidU64(Dereference(__m, y));
    assume $DebugTrackLocal(1, 20, 1, 7742, Dereference(__m, y));

    call __tmp := LdConst(7);
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __t9 := CopyOrMoveRef(y);

    call __tmp := ReadRef(__t9);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    __ret0 := GetLocal(__m, __frame + 8);
    assume $DebugTrackLocal(1, 20, 2, 7783, __ret0);
    __ret1 := GetLocal(__m, __frame + 10);
    assume $DebugTrackLocal(1, 20, 3, 7783, __ret1);
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
    var __t2: Value; // Vector_T_type_value(tv0)
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
      assume $DebugTrackAbort(1, 21, 8027);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 21, 0, 8021, __tmp);

    call __t3 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(7);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call Vector_push_back(IntegerType(), __t3, GetLocal(__m, __frame + 4));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 21, 8056);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 21, 0, 8056, GetLocal(__m, __frame + 0));

    call __t5 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __t7 := Vector_borrow(IntegerType(), __t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 21, 8104);
      goto Label_Abort;
    }
    assume IsValidU64(Dereference(__m, __t7)) && IsValidReferenceParameter(__m, __local_counter, __t7);



    call y := CopyOrMoveRef(__t7);
    assume IsValidU64(Dereference(__m, y));
    assume $DebugTrackLocal(1, 21, 1, 8100, Dereference(__m, y));

    call __tmp := LdConst(7);
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __t9 := CopyOrMoveRef(y);

    call __tmp := ReadRef(__t9);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    __ret0 := GetLocal(__m, __frame + 8);
    assume $DebugTrackLocal(1, 21, 2, 8141, __ret0);
    __ret1 := GetLocal(__m, __frame + 10);
    assume $DebugTrackLocal(1, 21, 3, 8141, __ret1);
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
    var __t2: Value; // Vector_T_type_value(tv0)
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
      assume $DebugTrackAbort(1, 22, 8333);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 22, 0, 8327, __tmp);

    call __t3 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(7);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call Vector_push_back(IntegerType(), __t3, GetLocal(__m, __frame + 4));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 22, 8362);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 22, 0, 8362, GetLocal(__m, __frame + 0));

    call __t5 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __t7 := Vector_borrow(IntegerType(), __t5, GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 22, 8410);
      goto Label_Abort;
    }
    assume IsValidU64(Dereference(__m, __t7)) && IsValidReferenceParameter(__m, __local_counter, __t7);



    call y := CopyOrMoveRef(__t7);
    assume IsValidU64(Dereference(__m, y));
    assume $DebugTrackLocal(1, 22, 1, 8406, Dereference(__m, y));

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __t9 := CopyOrMoveRef(y);

    call __tmp := ReadRef(__t9);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    __ret0 := GetLocal(__m, __frame + 8);
    assume $DebugTrackLocal(1, 22, 2, 8447, __ret0);
    __ret1 := GetLocal(__m, __frame + 10);
    assume $DebugTrackLocal(1, 22, 3, 8447, __ret1);
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

procedure {:inline 1} VerifyVector_my_empty (tv0: TypeValue) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(Integer(vlen(__ret0)), Integer(0))));
{
    // declare local variables
    var __t0: Value; // Vector_T_type_value(tv0)
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments

    // increase the local counter
    __local_counter := __local_counter + 1;

    // bytecode translation starts here
    call __t0 := Vector_empty(tv0);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 23, 8616);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t0);

    __m := UpdateLocal(__m, __frame + 0, __t0);

    __ret0 := GetLocal(__m, __frame + 0);
    assume $DebugTrackLocal(1, 23, 0, 8609, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure VerifyVector_my_empty_verify (tv0: TypeValue) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := VerifyVector_my_empty(tv0);
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
    assume $DebugTrackLocal(1, 24, 0, 8694, v);

    // increase the local counter
    __local_counter := __local_counter + 3;

    // bytecode translation starts here
    call __t1 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __t2 := Vector_length(IntegerType(), __t1);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 24, 9098);
      goto Label_Abort;
    }
    assume IsValidU64(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    __ret0 := GetLocal(__m, __frame + 2);
    assume $DebugTrackLocal(1, 24, 1, 9091, __ret0);
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
    assume $DebugTrackLocal(1, 25, 0, 9199, Dereference(__m, v));

    // increase the local counter
    __local_counter := __local_counter + 3;

    // bytecode translation starts here
    call __t1 := CopyOrMoveRef(v);

    call __t2 := Vector_length(IntegerType(), __t1);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 25, 9290);
      goto Label_Abort;
    }
    assume IsValidU64(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    __ret0 := GetLocal(__m, __frame + 2);
    assume $DebugTrackLocal(1, 25, 1, 9283, __ret0);
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
    assume $DebugTrackLocal(1, 26, 0, 9372, Dereference(__m, v));

    // increase the local counter
    __local_counter := __local_counter + 3;

    // bytecode translation starts here
    call __t1 := CopyOrMoveRef(v);

    call __t2 := Vector_length(tv0, __t1);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 26, 9476);
      goto Label_Abort;
    }
    assume IsValidU64(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    __ret0 := GetLocal(__m, __frame + 2);
    assume $DebugTrackLocal(1, 26, 1, 9469, __ret0);
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
    assume $DebugTrackLocal(1, 27, 0, 9537, v);
    assume IsValidU64(i);
    __m := UpdateLocal(__m, __frame + 1, i);
    assume $DebugTrackLocal(1, 27, 1, 9537, i);

    // increase the local counter
    __local_counter := __local_counter + 6;

    // bytecode translation starts here
    call __t2 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __t4 := Vector_borrow(IntegerType(), __t2, GetLocal(__m, __frame + 3));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 27, 9697);
      goto Label_Abort;
    }
    assume IsValidU64(Dereference(__m, __t4)) && IsValidReferenceParameter(__m, __local_counter, __t4);



    call __tmp := ReadRef(__t4);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    __ret0 := GetLocal(__m, __frame + 5);
    assume $DebugTrackLocal(1, 27, 2, 9689, __ret0);
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
    assume $DebugTrackLocal(1, 28, 0, 9807, Dereference(__m, v));
    assume IsValidU64(i);
    __m := UpdateLocal(__m, __frame + 1, i);
    assume $DebugTrackLocal(1, 28, 1, 9807, i);

    // increase the local counter
    __local_counter := __local_counter + 6;

    // bytecode translation starts here
    call __t2 := CopyOrMoveRef(v);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __t4 := Vector_borrow(IntegerType(), __t2, GetLocal(__m, __frame + 3));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 28, 9970);
      goto Label_Abort;
    }
    assume IsValidU64(Dereference(__m, __t4)) && IsValidReferenceParameter(__m, __local_counter, __t4);



    call __tmp := ReadRef(__t4);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    __ret0 := GetLocal(__m, __frame + 5);
    assume $DebugTrackLocal(1, 28, 2, 9962, __ret0);
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
    assume $DebugTrackLocal(1, 29, 0, 10036, Dereference(__m, v));
    assume IsValidU64(i);
    __m := UpdateLocal(__m, __frame + 1, i);
    assume $DebugTrackLocal(1, 29, 1, 10036, i);

    // increase the local counter
    __local_counter := __local_counter + 8;

    // bytecode translation starts here
    call __t2 := CopyOrMoveRef(v);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __t4 := Vector_borrow(IntegerType(), __t2, GetLocal(__m, __frame + 3));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 29, 10224);
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
      assume $DebugTrackAbort(1, 29, 10223);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    __ret0 := GetLocal(__m, __frame + 7);
    assume $DebugTrackLocal(1, 29, 2, 10216, __ret0);
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
    assume $DebugTrackLocal(1, 30, 0, 10318, Dereference(__m, v));
    assume IsValidU64(i);
    __m := UpdateLocal(__m, __frame + 1, i);
    assume $DebugTrackLocal(1, 30, 1, 10318, i);

    // increase the local counter
    __local_counter := __local_counter + 6;

    // bytecode translation starts here
    call __t2 := CopyOrMoveRef(v);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __t4 := Vector_borrow(tv0, __t2, GetLocal(__m, __frame + 3));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 30, 10512);
      goto Label_Abort;
    }
    assume IsValidReferenceParameter(__m, __local_counter, __t4);



    call __tmp := ReadRef(__t4);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    __ret0 := GetLocal(__m, __frame + 5);
    assume $DebugTrackLocal(1, 30, 2, 10504, __ret0);
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
    assume $DebugTrackLocal(1, 31, 0, 10582, v);
    assume IsValidU64(i);
    __m := UpdateLocal(__m, __frame + 1, i);
    assume $DebugTrackLocal(1, 31, 1, 10582, i);
    assume IsValidU64(e);
    __m := UpdateLocal(__m, __frame + 2, e);
    assume $DebugTrackLocal(1, 31, 2, 10582, e);

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
      assume $DebugTrackAbort(1, 31, 10717);
      goto Label_Abort;
    }
    assume IsValidU64(Dereference(__m, __t6)) && IsValidReferenceParameter(__m, __local_counter, __t6);


    assume $DebugTrackLocal(1, 31, 0, 10717, GetLocal(__m, __frame + 0));

    call WriteRef(__t6, GetLocal(__m, __frame + 3));
    assume $DebugTrackLocal(1, 31, 0, 10715, GetLocal(__m, __frame + 0));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    __ret0 := GetLocal(__m, __frame + 7);
    assume $DebugTrackLocal(1, 31, 3, 10777, __ret0);
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
    assume $DebugTrackLocal(1, 32, 0, 10870, Dereference(__m, v));
    assume IsValidU64(i);
    __m := UpdateLocal(__m, __frame + 1, i);
    assume $DebugTrackLocal(1, 32, 1, 10870, i);
    assume IsValidU64(e);
    __m := UpdateLocal(__m, __frame + 2, e);
    assume $DebugTrackLocal(1, 32, 2, 10870, e);

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
      assume $DebugTrackAbort(1, 32, 10996);
      goto Label_Abort;
    }
    assume IsValidU64(Dereference(__m, __t6)) && IsValidReferenceParameter(__m, __local_counter, __t6);


    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 32, 0, 10996, Dereference(__m, v));

    call WriteRef(__t6, GetLocal(__m, __frame + 3));
    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 32, 0, 10994, Dereference(__m, v));

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
    assume $DebugTrackLocal(1, 33, 0, 11117, Dereference(__m, v));
    assume IsValidU64(i);
    __m := UpdateLocal(__m, __frame + 1, i);
    assume $DebugTrackLocal(1, 33, 1, 11117, i);
    __m := UpdateLocal(__m, __frame + 2, e);
    assume $DebugTrackLocal(1, 33, 2, 11117, e);

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
      assume $DebugTrackAbort(1, 33, 11368);
      goto Label_Abort;
    }
    assume IsValidReferenceParameter(__m, __local_counter, __t6);


    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 33, 0, 11368, Dereference(__m, v));

    call WriteRef(__t6, GetLocal(__m, __frame + 3));
    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 33, 0, 11366, Dereference(__m, v));

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
    assume $DebugTrackLocal(1, 34, 0, 11508, Dereference(__m, v));

    // increase the local counter
    __local_counter := __local_counter + 5;

    // bytecode translation starts here
    call __t1 := CopyOrMoveRef(v);

    call __t2 := Vector_length(IntegerType(), __t1);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 34, 11738);
      goto Label_Abort;
    }
    assume IsValidU64(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    __tmp := Boolean(IsEqual(GetLocal(__m, __frame + 2), GetLocal(__m, __frame + 3)));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    __ret0 := GetLocal(__m, __frame + 4);
    assume $DebugTrackLocal(1, 34, 1, 11731, __ret0);
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
    var __t2: Value; // Vector_T_type_value(tv0)
    var __t3: Value; // Vector_T_type_value(tv0)
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
      assume $DebugTrackAbort(1, 35, 12140);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(1, 35, 0, 12134, __tmp);

    call __t3 := Vector_empty(IntegerType());
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 35, 12175);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(__t3);

    __m := UpdateLocal(__m, __frame + 3, __t3);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(1, 35, 1, 12169, __tmp);

    call __t4 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call Vector_push_back(IntegerType(), __t4, GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 35, 12204);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 35, 0, 12204, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 35, 1, 12204, GetLocal(__m, __frame + 1));

    call __t6 := BorrowLoc(__frame + 0, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call Vector_push_back(IntegerType(), __t6, GetLocal(__m, __frame + 7));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 35, 12248);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 35, 0, 12248, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 35, 1, 12248, GetLocal(__m, __frame + 1));

    call __t8 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call Vector_push_back(IntegerType(), __t8, GetLocal(__m, __frame + 9));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 35, 12293);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 35, 0, 12293, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 35, 1, 12293, GetLocal(__m, __frame + 1));

    call __t10 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call Vector_push_back(IntegerType(), __t10, GetLocal(__m, __frame + 11));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 35, 12337);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 35, 0, 12337, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 35, 1, 12337, GetLocal(__m, __frame + 1));

    call __t12 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call Vector_push_back(IntegerType(), __t12, GetLocal(__m, __frame + 13));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 35, 12381);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 35, 0, 12381, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 35, 1, 12381, GetLocal(__m, __frame + 1));

    call __t14 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(3);
    __m := UpdateLocal(__m, __frame + 15, __tmp);

    call Vector_push_back(IntegerType(), __t14, GetLocal(__m, __frame + 15));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 35, 12425);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 35, 0, 12425, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 35, 1, 12425, GetLocal(__m, __frame + 1));

    call __t16 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    call Vector_push_back(IntegerType(), __t16, GetLocal(__m, __frame + 17));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 35, 12469);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 35, 0, 12469, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 35, 1, 12469, GetLocal(__m, __frame + 1));

    call __t18 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 19, __tmp);

    call Vector_push_back(IntegerType(), __t18, GetLocal(__m, __frame + 19));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 35, 12513);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 35, 0, 12513, GetLocal(__m, __frame + 0));
    assume $DebugTrackLocal(1, 35, 1, 12513, GetLocal(__m, __frame + 1));

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 20, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 21, __tmp);

    __ret0 := GetLocal(__m, __frame + 20);
    assume $DebugTrackLocal(1, 35, 2, 12557, __ret0);
    __ret1 := GetLocal(__m, __frame + 21);
    assume $DebugTrackLocal(1, 35, 3, 12557, __ret1);
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
    assume $DebugTrackLocal(1, 36, 0, 12647, Dereference(__m, v));
    assume IsValidU64(e);
    __m := UpdateLocal(__m, __frame + 1, e);
    assume $DebugTrackLocal(1, 36, 1, 12647, e);

    // increase the local counter
    __local_counter := __local_counter + 8;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 3), GetLocal(__m, __frame + 4));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 36, 12849);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    assume $DebugTrackLocal(1, 36, 2, 12845, __tmp);

    call __t6 := CopyOrMoveRef(v);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call Vector_push_back(IntegerType(), __t6, GetLocal(__m, __frame + 7));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 36, 12870);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 36, 0, 12870, Dereference(__m, v));

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
    assume $DebugTrackLocal(1, 37, 0, 12955, Dereference(__m, v));
    assume IsValidU64(e);
    __m := UpdateLocal(__m, __frame + 1, e);
    assume $DebugTrackLocal(1, 37, 1, 12955, e);

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
      assume $DebugTrackAbort(1, 37, 13230);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call Vector_push_back(IntegerType(), __t2, GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 37, 13199);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 37, 0, 13199, Dereference(__m, v));

    call __t6 := CopyOrMoveRef(v);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call Vector_push_back(IntegerType(), __t6, GetLocal(__m, __frame + 7));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 37, 13250);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 37, 0, 13250, Dereference(__m, v));

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
    assume $DebugTrackLocal(1, 38, 0, 13384, Dereference(__m, v));
    assume IsValidU64(e);
    __m := UpdateLocal(__m, __frame + 1, e);
    assume $DebugTrackLocal(1, 38, 1, 13384, e);

    // increase the local counter
    __local_counter := __local_counter + 4;

    // bytecode translation starts here
    call __t2 := CopyOrMoveRef(v);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call Vector_push_back(IntegerType(), __t2, GetLocal(__m, __frame + 3));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 38, 13557);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 38, 0, 13557, Dereference(__m, v));

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
    assume $DebugTrackLocal(1, 39, 0, 13666, Dereference(__m, v));
    __m := UpdateLocal(__m, __frame + 1, e);
    assume $DebugTrackLocal(1, 39, 1, 13666, e);

    // increase the local counter
    __local_counter := __local_counter + 4;

    // bytecode translation starts here
    call __t2 := CopyOrMoveRef(v);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call Vector_push_back(tv0, __t2, GetLocal(__m, __frame + 3));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 39, 13856);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 39, 0, 13856, Dereference(__m, v));

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
    assume $DebugTrackLocal(1, 40, 0, 13994, Dereference(__m, v));

    // increase the local counter
    __local_counter := __local_counter + 3;

    // bytecode translation starts here
    call __t1 := CopyOrMoveRef(v);

    call __t2 := Vector_pop_back(IntegerType(), __t1);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 40, 14204);
      goto Label_Abort;
    }
    assume IsValidU64(__t2);

    __m := UpdateLocal(__m, __frame + 2, __t2);
    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 40, 0, 14204, Dereference(__m, v));

    __ret0 := GetLocal(__m, __frame + 2);
    assume $DebugTrackLocal(1, 40, 1, 14197, __ret0);
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
    assume $DebugTrackLocal(1, 41, 0, 14287, Dereference(__m, v));

    // increase the local counter
    __local_counter := __local_counter + 3;

    // bytecode translation starts here
    call __t1 := CopyOrMoveRef(v);

    call __t2 := Vector_pop_back(tv0, __t1);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 41, 14514);
      goto Label_Abort;
    }

    __m := UpdateLocal(__m, __frame + 2, __t2);
    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 41, 0, 14514, Dereference(__m, v));

    __ret0 := GetLocal(__m, __frame + 2);
    assume $DebugTrackLocal(1, 41, 1, 14507, __ret0);
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

procedure {:inline 1} VerifyVector_my_swap1 (v: Reference, i: Value, j: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(Dereference(__m, v), old(update_vector(update_vector(Dereference(__m, v), i#Integer(i), select_vector(Dereference(__m, v), i#Integer(j))), i#Integer(j), select_vector(Dereference(__m, v), i#Integer(i)))))));
ensures old(!(b#Boolean(Boolean(i#Integer(i) >= i#Integer(Integer(vlen(Dereference(__m, v)))))) || b#Boolean(Boolean(i#Integer(j) >= i#Integer(Integer(vlen(Dereference(__m, v)))))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(i) >= i#Integer(Integer(vlen(Dereference(__m, v)))))) || b#Boolean(Boolean(i#Integer(j) >= i#Integer(Integer(vlen(Dereference(__m, v))))))) ==> __abort_flag;

{
    // declare local variables
    var __t3: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t4: Value; // IntegerType()
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
    assume $DebugTrackLocal(1, 42, 0, 14626, Dereference(__m, v));
    assume IsValidU64(i);
    __m := UpdateLocal(__m, __frame + 1, i);
    assume $DebugTrackLocal(1, 42, 1, 14626, i);
    assume IsValidU64(j);
    __m := UpdateLocal(__m, __frame + 2, j);
    assume $DebugTrackLocal(1, 42, 2, 14626, j);

    // increase the local counter
    __local_counter := __local_counter + 6;

    // bytecode translation starts here
    call __t3 := CopyOrMoveRef(v);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call Vector_swap(IntegerType(), __t3, GetLocal(__m, __frame + 4), GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 42, 14790);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 42, 0, 14790, Dereference(__m, v));

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure VerifyVector_my_swap1_verify (v: Reference, i: Value, j: Value) returns ()
{
    call InitVerification();
    call VerifyVector_my_swap1(v, i, j);
}

procedure {:inline 1} VerifyVector_my_swap2 (tv0: TypeValue, v: Reference, i: Value, j: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(Dereference(__m, v), old(update_vector(update_vector(Dereference(__m, v), i#Integer(i), select_vector(Dereference(__m, v), i#Integer(j))), i#Integer(j), select_vector(Dereference(__m, v), i#Integer(i)))))));
ensures old(!(b#Boolean(Boolean(i#Integer(i) >= i#Integer(Integer(vlen(Dereference(__m, v)))))) || b#Boolean(Boolean(i#Integer(j) >= i#Integer(Integer(vlen(Dereference(__m, v)))))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(i) >= i#Integer(Integer(vlen(Dereference(__m, v)))))) || b#Boolean(Boolean(i#Integer(j) >= i#Integer(Integer(vlen(Dereference(__m, v))))))) ==> __abort_flag;

{
    // declare local variables
    var __t3: Reference; // ReferenceType(Vector_T_type_value(tv0))
    var __t4: Value; // IntegerType()
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
    assume $DebugTrackLocal(1, 43, 0, 14903, Dereference(__m, v));
    assume IsValidU64(i);
    __m := UpdateLocal(__m, __frame + 1, i);
    assume $DebugTrackLocal(1, 43, 1, 14903, i);
    assume IsValidU64(j);
    __m := UpdateLocal(__m, __frame + 2, j);
    assume $DebugTrackLocal(1, 43, 2, 14903, j);

    // increase the local counter
    __local_counter := __local_counter + 6;

    // bytecode translation starts here
    call __t3 := CopyOrMoveRef(v);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call Vector_swap(tv0, __t3, GetLocal(__m, __frame + 4), GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 43, 15080);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 43, 0, 15080, Dereference(__m, v));

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure VerifyVector_my_swap2_verify (tv0: TypeValue, v: Reference, i: Value, j: Value) returns ()
{
    call InitVerification();
    call VerifyVector_my_swap2(tv0, v, i, j);
}

procedure {:inline 1} VerifyVector_my_swap3 (tv0: TypeValue, v: Reference, i: Value, j: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(Dereference(__m, v), old(update_vector(update_vector(Dereference(__m, v), i#Integer(i), select_vector(Dereference(__m, v), i#Integer(j))), i#Integer(j), select_vector(Dereference(__m, v), i#Integer(i)))))));
ensures old(!(b#Boolean(Boolean(i#Integer(i) >= i#Integer(Integer(vlen(Dereference(__m, v)))))) || b#Boolean(Boolean(i#Integer(j) >= i#Integer(Integer(vlen(Dereference(__m, v)))))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(i) >= i#Integer(Integer(vlen(Dereference(__m, v)))))) || b#Boolean(Boolean(i#Integer(j) >= i#Integer(Integer(vlen(Dereference(__m, v))))))) ==> __abort_flag;

{
    // declare local variables
    var x: Value; // tv0
    var y: Value; // tv0
    var __t5: Reference; // ReferenceType(Vector_T_type_value(tv0))
    var __t6: Reference; // ReferenceType(Vector_T_type_value(tv0))
    var __t7: Value; // IntegerType()
    var __t8: Value; // tv0
    var __t9: Reference; // ReferenceType(Vector_T_type_value(tv0))
    var __t10: Reference; // ReferenceType(Vector_T_type_value(tv0))
    var __t11: Value; // IntegerType()
    var __t12: Value; // tv0
    var __t13: Reference; // ReferenceType(Vector_T_type_value(tv0))
    var __t14: Value; // IntegerType()
    var __t15: Value; // tv0
    var __t16: Reference; // ReferenceType(Vector_T_type_value(tv0))
    var __t17: Value; // IntegerType()
    var __t18: Value; // tv0
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
    assume $DebugTrackLocal(1, 44, 0, 15195, Dereference(__m, v));
    assume IsValidU64(i);
    __m := UpdateLocal(__m, __frame + 1, i);
    assume $DebugTrackLocal(1, 44, 1, 15195, i);
    assume IsValidU64(j);
    __m := UpdateLocal(__m, __frame + 2, j);
    assume $DebugTrackLocal(1, 44, 2, 15195, j);

    // increase the local counter
    __local_counter := __local_counter + 19;

    // bytecode translation starts here
    call __t5 := CopyOrMoveRef(v);

    call __t6 := FreezeRef(__t5);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __t8 := Vector_get(tv0, __t6, GetLocal(__m, __frame + 7));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 44, 15440);
      goto Label_Abort;
    }

    __m := UpdateLocal(__m, __frame + 8, __t8);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 3, __tmp);
    assume $DebugTrackLocal(1, 44, 3, 15436, __tmp);

    call __t9 := CopyOrMoveRef(v);

    call __t10 := FreezeRef(__t9);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __t12 := Vector_get(tv0, __t10, GetLocal(__m, __frame + 11));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 44, 15499);
      goto Label_Abort;
    }

    __m := UpdateLocal(__m, __frame + 12, __t12);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 12));
    __m := UpdateLocal(__m, __frame + 4, __tmp);
    assume $DebugTrackLocal(1, 44, 4, 15495, __tmp);

    call __t13 := CopyOrMoveRef(v);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 15, __tmp);

    call Vector_set(tv0, __t13, GetLocal(__m, __frame + 14), GetLocal(__m, __frame + 15));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 44, 15554);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 44, 0, 15554, Dereference(__m, v));

    call __t16 := CopyOrMoveRef(v);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    call Vector_set(tv0, __t16, GetLocal(__m, __frame + 17), GetLocal(__m, __frame + 18));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 44, 15610);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 44, 0, 15610, Dereference(__m, v));

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure VerifyVector_my_swap3_verify (tv0: TypeValue, v: Reference, i: Value, j: Value) returns ()
{
    call InitVerification();
    call VerifyVector_my_swap3(tv0, v, i, j);
}

procedure {:inline 1} VerifyVector_my_remove_unstable1 (v: Reference, i: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
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
    assume $DebugTrackLocal(1, 45, 0, 15749, Dereference(__m, v));
    assume IsValidU64(i);
    __m := UpdateLocal(__m, __frame + 1, i);
    assume $DebugTrackLocal(1, 45, 1, 15749, i);

    // increase the local counter
    __local_counter := __local_counter + 3;

    // bytecode translation starts here
    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    __ret0 := GetLocal(__m, __frame + 2);
    assume $DebugTrackLocal(1, 45, 2, 15954, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure VerifyVector_my_remove_unstable1_verify (v: Reference, i: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := VerifyVector_my_remove_unstable1(v, i);
}

procedure {:inline 1} VerifyVector_my_remove_unstable2 (tv0: TypeValue, v: Reference, i: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(Integer(vlen(Dereference(__m, v))), Integer(i#Integer(old(Integer(vlen(Dereference(__m, v))))) - i#Integer(Integer(1))))));
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(old(select_vector(Dereference(__m, v), i#Integer(i))), __ret0)));
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(Dereference(__m, v), old(slice_vector(update_vector(Dereference(__m, v), i#Integer(i), select_vector(Dereference(__m, v), i#Integer(Integer(i#Integer(Integer(vlen(Dereference(__m, v)))) - i#Integer(Integer(1)))))), i#Integer(Integer(0)), i#Integer(Integer(i#Integer(Integer(vlen(Dereference(__m, v)))) - i#Integer(Integer(1)))))))));
ensures old(!(b#Boolean(Boolean(i#Integer(i) >= i#Integer(Integer(vlen(Dereference(__m, v)))))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(i) >= i#Integer(Integer(vlen(Dereference(__m, v))))))) ==> __abort_flag;

{
    // declare local variables
    var last_index: Value; // IntegerType()
    var __t3: Reference; // ReferenceType(Vector_T_type_value(tv0))
    var __t4: Reference; // ReferenceType(Vector_T_type_value(tv0))
    var __t5: Value; // BooleanType()
    var __t6: Value; // IntegerType()
    var __t7: Reference; // ReferenceType(Vector_T_type_value(tv0))
    var __t8: Reference; // ReferenceType(Vector_T_type_value(tv0))
    var __t9: Value; // IntegerType()
    var __t10: Value; // IntegerType()
    var __t11: Value; // IntegerType()
    var __t12: Value; // IntegerType()
    var __t13: Value; // IntegerType()
    var __t14: Value; // BooleanType()
    var __t15: Reference; // ReferenceType(Vector_T_type_value(tv0))
    var __t16: Value; // IntegerType()
    var __t17: Value; // IntegerType()
    var __t18: Reference; // ReferenceType(Vector_T_type_value(tv0))
    var __t19: Value; // tv0
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
    assume $DebugTrackLocal(1, 46, 0, 16041, Dereference(__m, v));
    assume IsValidU64(i);
    __m := UpdateLocal(__m, __frame + 1, i);
    assume $DebugTrackLocal(1, 46, 1, 16041, i);

    // increase the local counter
    __local_counter := __local_counter + 20;

    // bytecode translation starts here
    call __t3 := CopyOrMoveRef(v);

    call __t4 := FreezeRef(__t3);

    call __t5 := Vector_is_empty(tv0, __t4);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 46, 16354);
      goto Label_Abort;
    }
    assume is#Boolean(__t5);

    __m := UpdateLocal(__m, __frame + 5, __t5);

    __tmp := GetLocal(__m, __frame + 5);
    if (!b#Boolean(__tmp)) { goto Label_6; }

    call __tmp := LdConst(10);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    if (true) { assume $DebugTrackAbort(1, 46, 16449); }
    goto Label_Abort;

Label_6:
    call __t7 := CopyOrMoveRef(v);

    call __t8 := FreezeRef(__t7);

    call __t9 := Vector_length(tv0, __t8);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 46, 16492);
      goto Label_Abort;
    }
    assume IsValidU64(__t9);

    __m := UpdateLocal(__m, __frame + 9, __t9);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __tmp := Sub(GetLocal(__m, __frame + 9), GetLocal(__m, __frame + 10));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 46, 16492);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 11));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    assume $DebugTrackLocal(1, 46, 2, 16479, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    __tmp := Boolean(!IsEqual(GetLocal(__m, __frame + 12), GetLocal(__m, __frame + 13)));
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    __tmp := GetLocal(__m, __frame + 14);
    if (!b#Boolean(__tmp)) { goto Label_20; }

    call __t15 := CopyOrMoveRef(v);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    call Vector_swap(tv0, __t15, GetLocal(__m, __frame + 16), GetLocal(__m, __frame + 17));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 46, 16592);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 46, 0, 16592, Dereference(__m, v));

Label_20:
    call __t18 := CopyOrMoveRef(v);

    call __t19 := Vector_pop_back(tv0, __t18);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 46, 16736);
      goto Label_Abort;
    }

    __m := UpdateLocal(__m, __frame + 19, __t19);
    assume $Vector_T_is_well_formed(Dereference(__m, v));
    assume $DebugTrackLocal(1, 46, 0, 16736, Dereference(__m, v));

    __ret0 := GetLocal(__m, __frame + 19);
    assume $DebugTrackLocal(1, 46, 3, 16729, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure VerifyVector_my_remove_unstable2_verify (tv0: TypeValue, v: Reference, i: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := VerifyVector_my_remove_unstable2(tv0, v, i);
}

procedure {:inline 1} VerifyVector_my_append1 (lhs: Reference, other: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(Integer(vlen(Dereference(__m, lhs))), old(Integer(i#Integer(Integer(vlen(Dereference(__m, lhs)))) + i#Integer(Integer(vlen(other))))))));
ensures b#Boolean(Boolean(IsEqual(slice_vector(Dereference(__m, lhs), i#Integer(Integer(0)), i#Integer(old(Integer(vlen(Dereference(__m, lhs)))))), old(Dereference(__m, lhs)))));
ensures b#Boolean(Boolean(IsEqual(slice_vector(Dereference(__m, lhs), i#Integer(old(Integer(vlen(Dereference(__m, lhs))))), i#Integer(Integer(vlen(Dereference(__m, lhs))))), old(other))));
{
    // declare local variables
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
    assume $Vector_T_is_well_formed(Dereference(__m, lhs)) && IsValidReferenceParameter(__m, __local_counter, lhs);
    assume $Vector_T_is_well_formed(Dereference(__m, lhs));
    assume $DebugTrackLocal(1, 47, 0, 16848, Dereference(__m, lhs));
    assume $Vector_T_is_well_formed(other);
    __m := UpdateLocal(__m, __frame + 1, other);
    assume $DebugTrackLocal(1, 47, 1, 16848, other);

    // increase the local counter
    __local_counter := __local_counter + 4;

    // bytecode translation starts here
    call __t2 := CopyOrMoveRef(lhs);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call Vector_append(IntegerType(), __t2, GetLocal(__m, __frame + 3));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 47, 17081);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(Dereference(__m, lhs));
    assume $DebugTrackLocal(1, 47, 0, 17081, Dereference(__m, lhs));

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure VerifyVector_my_append1_verify (lhs: Reference, other: Value) returns ()
{
    call InitVerification();
    call VerifyVector_my_append1(lhs, other);
}

procedure {:inline 1} VerifyVector_my_append2 (tv0: TypeValue, lhs: Reference, other: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures b#Boolean(Boolean(IsEqual(Integer(vlen(Dereference(__m, lhs))), old(Integer(i#Integer(Integer(vlen(Dereference(__m, lhs)))) + i#Integer(Integer(vlen(other))))))));
ensures b#Boolean(Boolean(IsEqual(slice_vector(Dereference(__m, lhs), i#Integer(Integer(0)), i#Integer(old(Integer(vlen(Dereference(__m, lhs)))))), old(Dereference(__m, lhs)))));
ensures b#Boolean(Boolean(IsEqual(slice_vector(Dereference(__m, lhs), i#Integer(old(Integer(vlen(Dereference(__m, lhs))))), i#Integer(Integer(vlen(Dereference(__m, lhs))))), old(other))));
{
    // declare local variables
    var __t2: Reference; // ReferenceType(Vector_T_type_value(tv0))
    var __t3: Value; // Vector_T_type_value(tv0)
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $Vector_T_is_well_formed(Dereference(__m, lhs)) && IsValidReferenceParameter(__m, __local_counter, lhs);
    assume $Vector_T_is_well_formed(Dereference(__m, lhs));
    assume $DebugTrackLocal(1, 48, 0, 17193, Dereference(__m, lhs));
    assume $Vector_T_is_well_formed(other);
    __m := UpdateLocal(__m, __frame + 1, other);
    assume $DebugTrackLocal(1, 48, 1, 17193, other);

    // increase the local counter
    __local_counter := __local_counter + 4;

    // bytecode translation starts here
    call __t2 := CopyOrMoveRef(lhs);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call Vector_append(tv0, __t2, GetLocal(__m, __frame + 3));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 48, 17443);
      goto Label_Abort;
    }
    assume $Vector_T_is_well_formed(Dereference(__m, lhs));
    assume $DebugTrackLocal(1, 48, 0, 17443, Dereference(__m, lhs));

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure VerifyVector_my_append2_verify (tv0: TypeValue, lhs: Reference, other: Value) returns ()
{
    call InitVerification();
    call VerifyVector_my_append2(tv0, lhs, other);
}

procedure {:inline 1} VerifyVector_my_append3 (lhs: Reference, other: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var __t2: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t3: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t4: Value; // BooleanType()
    var __t5: Value; // BooleanType()
    var __t6: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t7: Reference; // ReferenceType(Vector_T_type_value(IntegerType()))
    var __t8: Value; // IntegerType()
    var __t9: Value; // Vector_T_type_value(IntegerType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume $Vector_T_is_well_formed(Dereference(__m, lhs)) && IsValidReferenceParameter(__m, __local_counter, lhs);
    assume $Vector_T_is_well_formed(Dereference(__m, lhs));
    assume $DebugTrackLocal(1, 49, 0, 17577, Dereference(__m, lhs));
    assume $Vector_T_is_well_formed(other);
    __m := UpdateLocal(__m, __frame + 1, other);
    assume $DebugTrackLocal(1, 49, 1, 17577, other);

    // increase the local counter
    __local_counter := __local_counter + 10;

    // bytecode translation starts here
    call __t2 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call Vector_reverse(IntegerType(), __t2);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 49, 17816);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 49, 1, 17816, GetLocal(__m, __frame + 1));
    assume $Vector_T_is_well_formed(Dereference(__m, lhs));
    assume $DebugTrackLocal(1, 49, 0, 17816, Dereference(__m, lhs));

Label_2:
    call __t3 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __t4 := Vector_is_empty(IntegerType(), __t3);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 49, 17866);
      goto Label_Abort;
    }
    assume is#Boolean(__t4);

    __m := UpdateLocal(__m, __frame + 4, __t4);

    call __tmp := Not(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    __tmp := GetLocal(__m, __frame + 5);
    if (!b#Boolean(__tmp)) { goto Label_11; }

    call __t6 := CopyOrMoveRef(lhs);

    call __t7 := BorrowLoc(__frame + 1, Vector_T_type_value(IntegerType()));

    call __t8 := Vector_pop_back(IntegerType(), __t7);
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 49, 17976);
      goto Label_Abort;
    }
    assume IsValidU64(__t8);

    __m := UpdateLocal(__m, __frame + 8, __t8);
    assume $DebugTrackLocal(1, 49, 1, 17976, GetLocal(__m, __frame + 1));
    assume $Vector_T_is_well_formed(Dereference(__m, lhs));
    assume $DebugTrackLocal(1, 49, 0, 17976, Dereference(__m, lhs));

    call Vector_push_back(IntegerType(), __t6, GetLocal(__m, __frame + 8));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 49, 17910);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(1, 49, 1, 17910, GetLocal(__m, __frame + 1));
    assume $Vector_T_is_well_formed(Dereference(__m, lhs));
    assume $DebugTrackLocal(1, 49, 0, 17910, Dereference(__m, lhs));

    goto Label_2;

Label_11:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call Vector_destroy_empty(IntegerType(), GetLocal(__m, __frame + 9));
    if (__abort_flag) {
      assume $DebugTrackAbort(1, 49, 18043);
      goto Label_Abort;
    }

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure VerifyVector_my_append3_verify (lhs: Reference, other: Value) returns ()
{
    call InitVerification();
    call VerifyVector_my_append3(lhs, other);
}

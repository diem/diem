

// ** synthetics of module TestSpecs



// ** structs of module TestSpecs



// ** functions of module TestSpecs

procedure {:inline 1} TestSpecs_mut_b (b: Reference) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var __t1: Value; // IntegerType()
    var __t2: Reference; // ReferenceType(IntegerType())
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume IsValidU64(Dereference(__m, b)) && IsValidReferenceParameter(__m, __local_counter, b);
    assume IsValidU64(Dereference(__m, b));
    assume $DebugTrackLocal(0, 0, 0, 24, Dereference(__m, b));

    // increase the local counter
    __local_counter := __local_counter + 3;

    // bytecode translation starts here
    call __tmp := LdConst(10);
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __t2 := CopyOrMoveRef(b);

    call WriteRef(__t2, GetLocal(__m, __frame + 1));
    assume IsValidU64(Dereference(__m, b));
    assume $DebugTrackLocal(0, 0, 0, 59, Dereference(__m, b));

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure TestSpecs_mut_b_verify (b: Reference) returns ()
{
    call InitVerification();
    call TestSpecs_mut_b(b);
}

procedure {:inline 1} TestSpecs_mut_ref () returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(b#Boolean(Boolean(true))) ==> !__abort_flag;
{
    // declare local variables
    var b: Value; // IntegerType()
    var b_ref: Reference; // ReferenceType(IntegerType())
    var __t2: Value; // IntegerType()
    var __t3: Reference; // ReferenceType(IntegerType())
    var __t4: Reference; // ReferenceType(IntegerType())
    var __t5: Reference; // ReferenceType(IntegerType())
    var __t6: Value; // IntegerType()
    var __t7: Value; // IntegerType()
    var __t8: Value; // IntegerType()
    var __t9: Value; // BooleanType()
    var __t10: Value; // BooleanType()
    var __t11: Value; // IntegerType()
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
    call __tmp := LdConst(20);
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(0, 1, 0, 252, __tmp);

    call __t3 := BorrowLoc(__frame + 0, IntegerType());

    call b_ref := CopyOrMoveRef(__t3);
    assume IsValidU64(Dereference(__m, b_ref));
    assume $DebugTrackLocal(0, 1, 1, 268, Dereference(__m, b_ref));

    call __t4 := CopyOrMoveRef(b_ref);

    call TestSpecs_mut_b(__t4);
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 1, 349);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(0, 1, 0, 349, GetLocal(__m, __frame + 0));

    call __t5 := CopyOrMoveRef(b_ref);

    call __tmp := ReadRef(__t5);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(0, 1, 0, 382, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := LdConst(10);
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    __tmp := Boolean(IsEqual(GetLocal(__m, __frame + 7), GetLocal(__m, __frame + 8)));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 9));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    __tmp := GetLocal(__m, __frame + 10);
    if (!b#Boolean(__tmp)) { goto Label_16; }

    call __tmp := LdConst(42);
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    if (true) { assume $DebugTrackAbort(0, 1, 430); }
    goto Label_Abort;

Label_16:
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure TestSpecs_mut_ref_verify () returns ()
{
    call InitVerification();
    call TestSpecs_mut_ref();
}

procedure {:inline 1} TestSpecs_mut_ref_failure () returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(b#Boolean(Boolean(true))) ==> !__abort_flag;
{
    // declare local variables
    var b: Value; // IntegerType()
    var b_ref: Reference; // ReferenceType(IntegerType())
    var __t2: Value; // IntegerType()
    var __t3: Reference; // ReferenceType(IntegerType())
    var __t4: Reference; // ReferenceType(IntegerType())
    var __t5: Reference; // ReferenceType(IntegerType())
    var __t6: Value; // IntegerType()
    var __t7: Value; // IntegerType()
    var __t8: Value; // IntegerType()
    var __t9: Value; // BooleanType()
    var __t10: Value; // BooleanType()
    var __t11: Value; // IntegerType()
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
    call __tmp := LdConst(20);
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(0, 2, 0, 621, __tmp);

    call __t3 := BorrowLoc(__frame + 0, IntegerType());

    call b_ref := CopyOrMoveRef(__t3);
    assume IsValidU64(Dereference(__m, b_ref));
    assume $DebugTrackLocal(0, 2, 1, 637, Dereference(__m, b_ref));

    call __t4 := CopyOrMoveRef(b_ref);

    call TestSpecs_mut_b(__t4);
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 2, 661);
      goto Label_Abort;
    }
    assume $DebugTrackLocal(0, 2, 0, 661, GetLocal(__m, __frame + 0));

    call __t5 := CopyOrMoveRef(b_ref);

    call __tmp := ReadRef(__t5);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(0, 2, 0, 694, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := LdConst(9);
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    __tmp := Boolean(IsEqual(GetLocal(__m, __frame + 7), GetLocal(__m, __frame + 8)));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 9));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    __tmp := GetLocal(__m, __frame + 10);
    if (!b#Boolean(__tmp)) { goto Label_16; }

    call __tmp := LdConst(42);
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    if (true) { assume $DebugTrackAbort(0, 2, 801); }
    goto Label_Abort;

Label_16:
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure TestSpecs_mut_ref_failure_verify () returns ()
{
    call InitVerification();
    call TestSpecs_mut_ref_failure();
}

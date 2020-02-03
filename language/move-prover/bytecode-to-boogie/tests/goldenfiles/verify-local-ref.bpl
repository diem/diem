

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
    __local_counter := __local_counter + 3;

    // process and type check arguments
    assume IsValidU64(Dereference(__m, b));
    assume IsValidReferenceParameter(__m, __frame, b);

    // bytecode translation starts here
    call __tmp := LdConst(10);
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __t2 := CopyOrMoveRef(b);

    call WriteRef(__t2, GetLocal(__m, __frame + 1));

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure TestSpecs_mut_b_verify (b: Reference) returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 12;

    // process and type check arguments

    // bytecode translation starts here
    call __tmp := LdConst(20);
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call __t3 := BorrowLoc(__frame + 0);

    call b_ref := CopyOrMoveRef(__t3);

    call __t4 := CopyOrMoveRef(b_ref);

    call TestSpecs_mut_b(__t4);
    if (__abort_flag) { goto Label_Abort; }

    call __t5 := CopyOrMoveRef(b_ref);

    call __tmp := ReadRef(__t5);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 0, __tmp);

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

    goto Label_Abort;

Label_16:
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure TestSpecs_mut_ref_verify () returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
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
    __local_counter := __local_counter + 12;

    // process and type check arguments

    // bytecode translation starts here
    call __tmp := LdConst(20);
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);

    call __t3 := BorrowLoc(__frame + 0);

    call b_ref := CopyOrMoveRef(__t3);

    call __t4 := CopyOrMoveRef(b_ref);

    call TestSpecs_mut_b(__t4);
    if (__abort_flag) { goto Label_Abort; }

    call __t5 := CopyOrMoveRef(b_ref);

    call __tmp := ReadRef(__t5);
    assume IsValidU64(__tmp);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 0, __tmp);

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

    goto Label_Abort;

Label_16:
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure TestSpecs_mut_ref_failure_verify () returns ()
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call TestSpecs_mut_ref_failure();
}

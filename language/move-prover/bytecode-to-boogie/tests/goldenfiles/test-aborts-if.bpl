

// ** structs of module TestAbortIf



// ** functions of module TestAbortIf

procedure {:inline 1} TestAbortIf_abort1 (x: Value, y: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(i#Integer(x) <= i#Integer(y))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(x) <= i#Integer(y)))) ==> __abort_flag;

{
    // declare local variables
    var __t2: Value; // IntegerType()
    var __t3: Value; // IntegerType()
    var __t4: Value; // BooleanType()
    var __t5: Value; // BooleanType()
    var __t6: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#TestAbortIf#abort1#0#x: [Position]Value;
    var debug#TestAbortIf#abort1#1#y: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 7;
    debug#TestAbortIf#abort1#0#x := EmptyPositionMap;
    debug#TestAbortIf#abort1#1#y := EmptyPositionMap;

    // process and type check arguments
    assume IsValidU64(x);
    __m := UpdateLocal(__m, __frame + 0, x);
    debug#TestAbortIf#abort1#0#x := debug#TestAbortIf#abort1#0#x[Position(66) := x];
    assume IsValidU64(y);
    __m := UpdateLocal(__m, __frame + 1, y);
    debug#TestAbortIf#abort1#1#y := debug#TestAbortIf#abort1#1#y[Position(66) := y];

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := Gt(GetLocal(__m, __frame + 2), GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    __tmp := GetLocal(__m, __frame + 5);
    if (!b#Boolean(__tmp)) { goto Label_7; }

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    goto Label_Abort;

Label_7:
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure TestAbortIf_abort1_verify (x: Value, y: Value) returns ()
{
    call InitVerification();
    call TestAbortIf_abort1(x, y);
}

procedure {:inline 1} TestAbortIf_abort2 (x: Value, y: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(i#Integer(x) <= i#Integer(y))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(x) <= i#Integer(y)))) ==> __abort_flag;

{
    // declare local variables
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#TestAbortIf#abort2#0#x: [Position]Value;
    var debug#TestAbortIf#abort2#1#y: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 2;
    debug#TestAbortIf#abort2#0#x := EmptyPositionMap;
    debug#TestAbortIf#abort2#1#y := EmptyPositionMap;

    // process and type check arguments
    assume IsValidU64(x);
    __m := UpdateLocal(__m, __frame + 0, x);
    debug#TestAbortIf#abort2#0#x := debug#TestAbortIf#abort2#0#x[Position(283) := x];
    assume IsValidU64(y);
    __m := UpdateLocal(__m, __frame + 1, y);
    debug#TestAbortIf#abort2#1#y := debug#TestAbortIf#abort2#1#y[Position(283) := y];

    // bytecode translation starts here
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure TestAbortIf_abort2_verify (x: Value, y: Value) returns ()
{
    call InitVerification();
    call TestAbortIf_abort2(x, y);
}

procedure {:inline 1} TestAbortIf_abort3 (x: Value, y: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(i#Integer(x) <= i#Integer(y))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(x) <= i#Integer(y)))) ==> __abort_flag;

{
    // declare local variables
    var __t2: Value; // BooleanType()
    var __t3: Value; // BooleanType()
    var __t4: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#TestAbortIf#abort3#0#x: [Position]Value;
    var debug#TestAbortIf#abort3#1#y: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 5;
    debug#TestAbortIf#abort3#0#x := EmptyPositionMap;
    debug#TestAbortIf#abort3#1#y := EmptyPositionMap;

    // process and type check arguments
    assume IsValidU64(x);
    __m := UpdateLocal(__m, __frame + 0, x);
    debug#TestAbortIf#abort3#0#x := debug#TestAbortIf#abort3#0#x[Position(476) := x];
    assume IsValidU64(y);
    __m := UpdateLocal(__m, __frame + 1, y);
    debug#TestAbortIf#abort3#1#y := debug#TestAbortIf#abort3#1#y[Position(476) := y];

    // bytecode translation starts here
    call __tmp := LdFalse();
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    __tmp := GetLocal(__m, __frame + 3);
    if (!b#Boolean(__tmp)) { goto Label_5; }

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    goto Label_Abort;

Label_5:
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure TestAbortIf_abort3_verify (x: Value, y: Value) returns ()
{
    call InitVerification();
    call TestAbortIf_abort3(x, y);
}

procedure {:inline 1} TestAbortIf_abort4 (x: Value, y: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(i#Integer(x) < i#Integer(y))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(x) < i#Integer(y)))) ==> __abort_flag;

{
    // declare local variables
    var __t2: Value; // IntegerType()
    var __t3: Value; // IntegerType()
    var __t4: Value; // BooleanType()
    var __t5: Value; // BooleanType()
    var __t6: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#TestAbortIf#abort4#0#x: [Position]Value;
    var debug#TestAbortIf#abort4#1#y: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 7;
    debug#TestAbortIf#abort4#0#x := EmptyPositionMap;
    debug#TestAbortIf#abort4#1#y := EmptyPositionMap;

    // process and type check arguments
    assume IsValidU64(x);
    __m := UpdateLocal(__m, __frame + 0, x);
    debug#TestAbortIf#abort4#0#x := debug#TestAbortIf#abort4#0#x[Position(717) := x];
    assume IsValidU64(y);
    __m := UpdateLocal(__m, __frame + 1, y);
    debug#TestAbortIf#abort4#1#y := debug#TestAbortIf#abort4#1#y[Position(717) := y];

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := Gt(GetLocal(__m, __frame + 2), GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    __tmp := GetLocal(__m, __frame + 5);
    if (!b#Boolean(__tmp)) { goto Label_7; }

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    goto Label_Abort;

Label_7:
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure TestAbortIf_abort4_verify (x: Value, y: Value) returns ()
{
    call InitVerification();
    call TestAbortIf_abort4(x, y);
}

procedure {:inline 1} TestAbortIf_abort5 (x: Value, y: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(i#Integer(x) <= i#Integer(y)))) && (b#Boolean(Boolean(i#Integer(x) > i#Integer(y))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(x) <= i#Integer(y)))) ==> __abort_flag;

{
    // declare local variables
    var __t2: Value; // IntegerType()
    var __t3: Value; // IntegerType()
    var __t4: Value; // BooleanType()
    var __t5: Value; // BooleanType()
    var __t6: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#TestAbortIf#abort5#0#x: [Position]Value;
    var debug#TestAbortIf#abort5#1#y: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 7;
    debug#TestAbortIf#abort5#0#x := EmptyPositionMap;
    debug#TestAbortIf#abort5#1#y := EmptyPositionMap;

    // process and type check arguments
    assume IsValidU64(x);
    __m := UpdateLocal(__m, __frame + 0, x);
    debug#TestAbortIf#abort5#0#x := debug#TestAbortIf#abort5#0#x[Position(954) := x];
    assume IsValidU64(y);
    __m := UpdateLocal(__m, __frame + 1, y);
    debug#TestAbortIf#abort5#1#y := debug#TestAbortIf#abort5#1#y[Position(954) := y];

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := Gt(GetLocal(__m, __frame + 2), GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    __tmp := GetLocal(__m, __frame + 5);
    if (!b#Boolean(__tmp)) { goto Label_7; }

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    goto Label_Abort;

Label_7:
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure TestAbortIf_abort5_verify (x: Value, y: Value) returns ()
{
    call InitVerification();
    call TestAbortIf_abort5(x, y);
}

procedure {:inline 1} TestAbortIf_abort6 (x: Value, y: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(i#Integer(x) < i#Integer(y)))) && (b#Boolean(Boolean(i#Integer(x) > i#Integer(y))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(x) < i#Integer(y)))) ==> __abort_flag;

{
    // declare local variables
    var __t2: Value; // IntegerType()
    var __t3: Value; // IntegerType()
    var __t4: Value; // BooleanType()
    var __t5: Value; // BooleanType()
    var __t6: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#TestAbortIf#abort6#0#x: [Position]Value;
    var debug#TestAbortIf#abort6#1#y: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 7;
    debug#TestAbortIf#abort6#0#x := EmptyPositionMap;
    debug#TestAbortIf#abort6#1#y := EmptyPositionMap;

    // process and type check arguments
    assume IsValidU64(x);
    __m := UpdateLocal(__m, __frame + 0, x);
    debug#TestAbortIf#abort6#0#x := debug#TestAbortIf#abort6#0#x[Position(1199) := x];
    assume IsValidU64(y);
    __m := UpdateLocal(__m, __frame + 1, y);
    debug#TestAbortIf#abort6#1#y := debug#TestAbortIf#abort6#1#y[Position(1199) := y];

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := Gt(GetLocal(__m, __frame + 2), GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    __tmp := GetLocal(__m, __frame + 5);
    if (!b#Boolean(__tmp)) { goto Label_7; }

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    goto Label_Abort;

Label_7:
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure TestAbortIf_abort6_verify (x: Value, y: Value) returns ()
{
    call InitVerification();
    call TestAbortIf_abort6(x, y);
}

procedure {:inline 1} TestAbortIf_abort7 (x: Value, y: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(i#Integer(x) < i#Integer(y)))) && (b#Boolean(Boolean(i#Integer(x) >= i#Integer(y))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(x) < i#Integer(y)))) ==> __abort_flag;

{
    // declare local variables
    var __t2: Value; // IntegerType()
    var __t3: Value; // IntegerType()
    var __t4: Value; // BooleanType()
    var __t5: Value; // BooleanType()
    var __t6: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#TestAbortIf#abort7#0#x: [Position]Value;
    var debug#TestAbortIf#abort7#1#y: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 7;
    debug#TestAbortIf#abort7#0#x := EmptyPositionMap;
    debug#TestAbortIf#abort7#1#y := EmptyPositionMap;

    // process and type check arguments
    assume IsValidU64(x);
    __m := UpdateLocal(__m, __frame + 0, x);
    debug#TestAbortIf#abort7#0#x := debug#TestAbortIf#abort7#0#x[Position(1423) := x];
    assume IsValidU64(y);
    __m := UpdateLocal(__m, __frame + 1, y);
    debug#TestAbortIf#abort7#1#y := debug#TestAbortIf#abort7#1#y[Position(1423) := y];

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := Gt(GetLocal(__m, __frame + 2), GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    __tmp := GetLocal(__m, __frame + 5);
    if (!b#Boolean(__tmp)) { goto Label_7; }

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    goto Label_Abort;

Label_7:
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure TestAbortIf_abort7_verify (x: Value, y: Value) returns ()
{
    call InitVerification();
    call TestAbortIf_abort7(x, y);
}

procedure {:inline 1} TestAbortIf_abort8 (x: Value, y: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(__ret0, Boolean(true))));
ensures old(!(b#Boolean(Boolean(i#Integer(x) < i#Integer(y)))) && (b#Boolean(Boolean(i#Integer(x) > i#Integer(y))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(x) < i#Integer(y)))) ==> __abort_flag;

{
    // declare local variables
    var __t2: Value; // IntegerType()
    var __t3: Value; // IntegerType()
    var __t4: Value; // BooleanType()
    var __t5: Value; // BooleanType()
    var __t6: Value; // IntegerType()
    var __t7: Value; // IntegerType()
    var __t8: Value; // IntegerType()
    var __t9: Value; // IntegerType()
    var __t10: Value; // BooleanType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#TestAbortIf#abort8#0#x: [Position]Value;
    var debug#TestAbortIf#abort8#1#y: [Position]Value;
    var debug#TestAbortIf#abort8#2#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 11;
    debug#TestAbortIf#abort8#0#x := EmptyPositionMap;
    debug#TestAbortIf#abort8#1#y := EmptyPositionMap;
    debug#TestAbortIf#abort8#2#__ret := EmptyPositionMap;

    // process and type check arguments
    assume IsValidU64(x);
    __m := UpdateLocal(__m, __frame + 0, x);
    debug#TestAbortIf#abort8#0#x := debug#TestAbortIf#abort8#0#x[Position(1706) := x];
    assume IsValidU64(y);
    __m := UpdateLocal(__m, __frame + 1, y);
    debug#TestAbortIf#abort8#1#y := debug#TestAbortIf#abort8#1#y[Position(1706) := y];

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := Gt(GetLocal(__m, __frame + 2), GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    __tmp := GetLocal(__m, __frame + 5);
    if (!b#Boolean(__tmp)) { goto Label_7; }

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    goto Label_Abort;

Label_7:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    debug#TestAbortIf#abort8#1#y := debug#TestAbortIf#abort8#1#y[Position(1900) := __tmp];

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    __tmp := Boolean(IsEqual(GetLocal(__m, __frame + 8), GetLocal(__m, __frame + 9)));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    __ret0 := GetLocal(__m, __frame + 10);
    debug#TestAbortIf#abort8#2#__ret := debug#TestAbortIf#abort8#2#__ret[Position(1935) := __ret0];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#TestAbortIf#abort8#2#__ret := debug#TestAbortIf#abort8#2#__ret[Position(1971) := __ret0];
}

procedure TestAbortIf_abort8_verify (x: Value, y: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := TestAbortIf_abort8(x, y);
}

procedure {:inline 1} TestAbortIf_abort9 (x: Value, y: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
ensures !__abort_flag ==> b#Boolean(Boolean(IsEqual(x, y)));
ensures old(!(b#Boolean(Boolean(i#Integer(x) > i#Integer(y))) || b#Boolean(Boolean(i#Integer(x) < i#Integer(y))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(x) > i#Integer(y))) || b#Boolean(Boolean(i#Integer(x) < i#Integer(y)))) ==> __abort_flag;

{
    // declare local variables
    var __t2: Value; // IntegerType()
    var __t3: Value; // IntegerType()
    var __t4: Value; // BooleanType()
    var __t5: Value; // BooleanType()
    var __t6: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#TestAbortIf#abort9#0#x: [Position]Value;
    var debug#TestAbortIf#abort9#1#y: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 7;
    debug#TestAbortIf#abort9#0#x := EmptyPositionMap;
    debug#TestAbortIf#abort9#1#y := EmptyPositionMap;

    // process and type check arguments
    assume IsValidU64(x);
    __m := UpdateLocal(__m, __frame + 0, x);
    debug#TestAbortIf#abort9#0#x := debug#TestAbortIf#abort9#0#x[Position(2218) := x];
    assume IsValidU64(y);
    __m := UpdateLocal(__m, __frame + 1, y);
    debug#TestAbortIf#abort9#1#y := debug#TestAbortIf#abort9#1#y[Position(2218) := y];

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := Gt(GetLocal(__m, __frame + 2), GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 4));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    __tmp := GetLocal(__m, __frame + 5);
    if (!b#Boolean(__tmp)) { goto Label_7; }

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    goto Label_Abort;

Label_7:
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure TestAbortIf_abort9_verify (x: Value, y: Value) returns ()
{
    call InitVerification();
    call TestAbortIf_abort9(x, y);
}

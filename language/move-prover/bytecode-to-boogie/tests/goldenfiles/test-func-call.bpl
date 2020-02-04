

// ** structs of module TestFuncCall



// ** functions of module TestFuncCall

procedure {:inline 1} TestFuncCall_f (x: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var __t1: Value; // IntegerType()
    var __t2: Value; // IntegerType()
    var __t3: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#TestFuncCall#f#0#x#25: Value;
    var debug#TestFuncCall#f#1#__ret#52: Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 4;

    // process and type check arguments
    assume IsValidU64(x);
    __m := UpdateLocal(__m, __frame + 0, x);
    assume (debug#TestFuncCall#f#0#x#25) == (x);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 1), GetLocal(__m, __frame + 2));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    __ret0 := GetLocal(__m, __frame + 3);
    assume (debug#TestFuncCall#f#1#__ret#52) == (__ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure TestFuncCall_f_verify (x: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := TestFuncCall_f(x);
}

procedure {:inline 1} TestFuncCall_g (x: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var __t1: Value; // IntegerType()
    var __t2: Value; // IntegerType()
    var __t3: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#TestFuncCall#g#0#x#75: Value;
    var debug#TestFuncCall#g#1#__ret#102: Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 4;

    // process and type check arguments
    assume IsValidU64(x);
    __m := UpdateLocal(__m, __frame + 0, x);
    assume (debug#TestFuncCall#g#0#x#75) == (x);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 1), GetLocal(__m, __frame + 2));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    __ret0 := GetLocal(__m, __frame + 3);
    assume (debug#TestFuncCall#g#1#__ret#102) == (__ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure TestFuncCall_g_verify (x: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := TestFuncCall_g(x);
}

procedure {:inline 1} TestFuncCall_h (b: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(false)))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(false))) ==> __abort_flag;

{
    // declare local variables
    var x: Value; // IntegerType()
    var y: Value; // IntegerType()
    var __t3: Value; // IntegerType()
    var __t4: Value; // BooleanType()
    var __t5: Value; // IntegerType()
    var __t6: Value; // IntegerType()
    var __t7: Value; // IntegerType()
    var __t8: Value; // IntegerType()
    var __t9: Value; // BooleanType()
    var __t10: Value; // IntegerType()
    var __t11: Value; // IntegerType()
    var __t12: Value; // BooleanType()
    var __t13: Value; // BooleanType()
    var __t14: Value; // BooleanType()
    var __t15: Value; // BooleanType()
    var __t16: Value; // IntegerType()
    var __t17: Value; // IntegerType()
    var __t18: Value; // BooleanType()
    var __t19: Value; // BooleanType()
    var __t20: Value; // BooleanType()
    var __t21: Value; // BooleanType()
    var __t22: Value; // IntegerType()
    var __t23: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;
    var debug#TestFuncCall#h#0#b#125: Value;
    var debug#TestFuncCall#h#1#x#199: Value;
    var debug#TestFuncCall#h#2#y#226: Value;
    var debug#TestFuncCall#h#2#y#264: Value;
    var debug#TestFuncCall#h#3#__ret#366: Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 24;

    // process and type check arguments
    assume is#Boolean(b);
    __m := UpdateLocal(__m, __frame + 0, b);
    assume (debug#TestFuncCall#h#0#b#125) == (b);

    // bytecode translation starts here
    call __tmp := LdConst(3);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume (debug#TestFuncCall#h#1#x#199) == (__tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    __tmp := GetLocal(__m, __frame + 4);
    if (!b#Boolean(__tmp)) { goto Label_8; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __t6 := TestFuncCall_f(GetLocal(__m, __frame + 5));
    if (__abort_flag) { goto Label_Abort; }
    assume IsValidU64(__t6);

    __m := UpdateLocal(__m, __frame + 6, __t6);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    assume (debug#TestFuncCall#h#2#y#226) == (__tmp);

    goto Label_11;

Label_8:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __t8 := TestFuncCall_g(GetLocal(__m, __frame + 7));
    if (__abort_flag) { goto Label_Abort; }
    assume IsValidU64(__t8);

    __m := UpdateLocal(__m, __frame + 8, __t8);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    assume (debug#TestFuncCall#h#2#y#264) == (__tmp);

Label_11:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __tmp := LdConst(4);
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    __tmp := Boolean(IsEqual(GetLocal(__m, __frame + 10), GetLocal(__m, __frame + 11)));
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    call __tmp := And(GetLocal(__m, __frame + 9), GetLocal(__m, __frame + 12));
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 14));
    __m := UpdateLocal(__m, __frame + 15, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __tmp := LdConst(5);
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    __tmp := Boolean(IsEqual(GetLocal(__m, __frame + 16), GetLocal(__m, __frame + 17)));
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    call __tmp := And(GetLocal(__m, __frame + 15), GetLocal(__m, __frame + 18));
    __m := UpdateLocal(__m, __frame + 19, __tmp);

    call __tmp := Or(GetLocal(__m, __frame + 13), GetLocal(__m, __frame + 19));
    __m := UpdateLocal(__m, __frame + 20, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 20));
    __m := UpdateLocal(__m, __frame + 21, __tmp);

    __tmp := GetLocal(__m, __frame + 21);
    if (!b#Boolean(__tmp)) { goto Label_27; }

    call __tmp := LdConst(42);
    __m := UpdateLocal(__m, __frame + 22, __tmp);

    goto Label_Abort;

Label_27:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 23, __tmp);

    __ret0 := GetLocal(__m, __frame + 23);
    assume (debug#TestFuncCall#h#3#__ret#366) == (__ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure TestFuncCall_h_verify (b: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := TestFuncCall_h(b);
}

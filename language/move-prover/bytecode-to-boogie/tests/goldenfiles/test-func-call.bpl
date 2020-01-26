

// ** structs of module TestFuncCall



// ** functions of module TestFuncCall

procedure {:inline 1} TestFuncCall_f (x: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t1: Value; // IntegerType()
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 4;

    // process and type check arguments
    assume IsValidU64(x);
    __m := UpdateLocal(__m, __frame + 0, x);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 1), GetLocal(__m, __frame + 2));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    ret0 := GetLocal(__m, __frame + 3);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure TestFuncCall_f_verify (x: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := TestFuncCall_f(x);
}

procedure {:inline 1} TestFuncCall_g (x: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var t1: Value; // IntegerType()
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 4;

    // process and type check arguments
    assume IsValidU64(x);
    __m := UpdateLocal(__m, __frame + 0, x);

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 1), GetLocal(__m, __frame + 2));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    ret0 := GetLocal(__m, __frame + 3);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure TestFuncCall_g_verify (x: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := TestFuncCall_g(x);
}

procedure {:inline 1} TestFuncCall_h (b: Value) returns (ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(false)))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(false))) ==> __abort_flag;

{
    // declare local variables
    var t1: Value; // IntegerType()
    var t2: Value; // IntegerType()
    var t3: Value; // IntegerType()
    var t4: Value; // BooleanType()
    var t5: Value; // IntegerType()
    var t6: Value; // IntegerType()
    var t7: Value; // IntegerType()
    var t8: Value; // IntegerType()
    var t9: Value; // BooleanType()
    var t10: Value; // IntegerType()
    var t11: Value; // IntegerType()
    var t12: Value; // BooleanType()
    var t13: Value; // BooleanType()
    var t14: Value; // BooleanType()
    var t15: Value; // BooleanType()
    var t16: Value; // IntegerType()
    var t17: Value; // IntegerType()
    var t18: Value; // BooleanType()
    var t19: Value; // BooleanType()
    var t20: Value; // BooleanType()
    var t21: Value; // BooleanType()
    var t22: Value; // IntegerType()
    var t23: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 24;

    // process and type check arguments
    assume is#Boolean(b);
    __m := UpdateLocal(__m, __frame + 0, b);

    // bytecode translation starts here
    call __tmp := LdConst(3);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    __tmp := GetLocal(__m, __frame + 4);
    if (!b#Boolean(__tmp)) { goto Label_8; }

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call t6 := TestFuncCall_f(GetLocal(__m, __frame + 5));
    if (__abort_flag) { goto Label_Abort; }
    assume IsValidU64(t6);

    __m := UpdateLocal(__m, __frame + 6, t6);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    goto Label_11;

Label_8:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call t8 := TestFuncCall_g(GetLocal(__m, __frame + 7));
    if (__abort_flag) { goto Label_Abort; }
    assume IsValidU64(t8);

    __m := UpdateLocal(__m, __frame + 8, t8);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

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

    ret0 := GetLocal(__m, __frame + 23);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    ret0 := DefaultValue;
}

procedure TestFuncCall_h_verify (b: Value) returns (ret0: Value)
{
    assume ExistsTxnSenderAccount(__m, __txn);
    call ret0 := TestFuncCall_h(b);
}

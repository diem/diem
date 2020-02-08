

// ** structs of module TestArithmetic



// ** functions of module TestArithmetic

procedure {:inline 1} TestArithmetic_add_two_number (x: Value, y: Value) returns (__ret0: Value, __ret1: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var res: Value; // IntegerType()
    var z: Value; // IntegerType()
    var __t4: Value; // IntegerType()
    var __t5: Value; // IntegerType()
    var __t6: Value; // IntegerType()
    var __t7: Value; // IntegerType()
    var __t8: Value; // IntegerType()
    var __t9: Value; // IntegerType()
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
    assume $DebugTrackLocal(0, 0, 0, 25, x);
    assume IsValidU64(y);
    __m := UpdateLocal(__m, __frame + 1, y);
    assume $DebugTrackLocal(0, 0, 1, 25, y);

    // increase the local counter
    __local_counter := __local_counter + 10;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 4), GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 0, 112);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 6));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    assume $DebugTrackLocal(0, 0, 2, 106, __tmp);

    call __tmp := LdConst(3);
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 7));
    __m := UpdateLocal(__m, __frame + 3, __tmp);
    assume $DebugTrackLocal(0, 0, 3, 133, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    __ret0 := GetLocal(__m, __frame + 8);
    assume $DebugTrackLocal(0, 0, 4, 142, __ret0);
    __ret1 := GetLocal(__m, __frame + 9);
    assume $DebugTrackLocal(0, 0, 5, 142, __ret1);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    __ret1 := DefaultValue;
}

procedure TestArithmetic_add_two_number_verify (x: Value, y: Value) returns (__ret0: Value, __ret1: Value)
{
    call InitVerification();
    call __ret0, __ret1 := TestArithmetic_add_two_number(x, y);
}

procedure {:inline 1} TestArithmetic_multiple_ops (x: Value, y: Value, z: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var res: Value; // IntegerType()
    var __t4: Value; // IntegerType()
    var __t5: Value; // IntegerType()
    var __t6: Value; // IntegerType()
    var __t7: Value; // IntegerType()
    var __t8: Value; // IntegerType()
    var __t9: Value; // IntegerType()
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
    assume $DebugTrackLocal(0, 1, 0, 173, x);
    assume IsValidU64(y);
    __m := UpdateLocal(__m, __frame + 1, y);
    assume $DebugTrackLocal(0, 1, 1, 173, y);
    assume IsValidU64(z);
    __m := UpdateLocal(__m, __frame + 2, z);
    assume $DebugTrackLocal(0, 1, 2, 173, z);

    // increase the local counter
    __local_counter := __local_counter + 10;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 4), GetLocal(__m, __frame + 5));
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 1, 249);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := MulU64(GetLocal(__m, __frame + 6), GetLocal(__m, __frame + 7));
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 1, 248);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 3, __tmp);
    assume $DebugTrackLocal(0, 1, 3, 242, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    __ret0 := GetLocal(__m, __frame + 9);
    assume $DebugTrackLocal(0, 1, 4, 281, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure TestArithmetic_multiple_ops_verify (x: Value, y: Value, z: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := TestArithmetic_multiple_ops(x, y, z);
}

procedure {:inline 1} TestArithmetic_bool_ops (a: Value, b: Value) returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var c: Value; // BooleanType()
    var d: Value; // BooleanType()
    var __t4: Value; // IntegerType()
    var __t5: Value; // IntegerType()
    var __t6: Value; // BooleanType()
    var __t7: Value; // IntegerType()
    var __t8: Value; // IntegerType()
    var __t9: Value; // BooleanType()
    var __t10: Value; // BooleanType()
    var __t11: Value; // IntegerType()
    var __t12: Value; // IntegerType()
    var __t13: Value; // BooleanType()
    var __t14: Value; // IntegerType()
    var __t15: Value; // IntegerType()
    var __t16: Value; // BooleanType()
    var __t17: Value; // BooleanType()
    var __t18: Value; // BooleanType()
    var __t19: Value; // BooleanType()
    var __t20: Value; // BooleanType()
    var __t21: Value; // BooleanType()
    var __t22: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume IsValidU64(a);
    __m := UpdateLocal(__m, __frame + 0, a);
    assume $DebugTrackLocal(0, 2, 0, 304, a);
    assume IsValidU64(b);
    __m := UpdateLocal(__m, __frame + 1, b);
    assume $DebugTrackLocal(0, 2, 1, 304, b);

    // increase the local counter
    __local_counter := __local_counter + 23;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := Gt(GetLocal(__m, __frame + 4), GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __tmp := Ge(GetLocal(__m, __frame + 7), GetLocal(__m, __frame + 8));
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __tmp := And(GetLocal(__m, __frame + 6), GetLocal(__m, __frame + 9));
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 10));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    assume $DebugTrackLocal(0, 2, 2, 382, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    call __tmp := Lt(GetLocal(__m, __frame + 11), GetLocal(__m, __frame + 12));
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 15, __tmp);

    call __tmp := Le(GetLocal(__m, __frame + 14), GetLocal(__m, __frame + 15));
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __tmp := Or(GetLocal(__m, __frame + 13), GetLocal(__m, __frame + 16));
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 17));
    __m := UpdateLocal(__m, __frame + 3, __tmp);
    assume $DebugTrackLocal(0, 2, 3, 437, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 3));
    __m := UpdateLocal(__m, __frame + 19, __tmp);

    __tmp := Boolean(!IsEqual(GetLocal(__m, __frame + 18), GetLocal(__m, __frame + 19)));
    __m := UpdateLocal(__m, __frame + 20, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 20));
    __m := UpdateLocal(__m, __frame + 21, __tmp);

    __tmp := GetLocal(__m, __frame + 21);
    if (!b#Boolean(__tmp)) { goto Label_23; }

    call __tmp := LdConst(42);
    __m := UpdateLocal(__m, __frame + 22, __tmp);

    goto Label_Abort;

Label_23:
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure TestArithmetic_bool_ops_verify (a: Value, b: Value) returns ()
{
    call InitVerification();
    call TestArithmetic_bool_ops(a, b);
}

procedure {:inline 1} TestArithmetic_arithmetic_ops (a: Value, b: Value) returns (__ret0: Value, __ret1: Value)
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var c: Value; // IntegerType()
    var __t3: Value; // IntegerType()
    var __t4: Value; // IntegerType()
    var __t5: Value; // IntegerType()
    var __t6: Value; // IntegerType()
    var __t7: Value; // IntegerType()
    var __t8: Value; // IntegerType()
    var __t9: Value; // IntegerType()
    var __t10: Value; // IntegerType()
    var __t11: Value; // IntegerType()
    var __t12: Value; // IntegerType()
    var __t13: Value; // IntegerType()
    var __t14: Value; // IntegerType()
    var __t15: Value; // IntegerType()
    var __t16: Value; // BooleanType()
    var __t17: Value; // BooleanType()
    var __t18: Value; // IntegerType()
    var __t19: Value; // IntegerType()
    var __t20: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume IsValidU64(a);
    __m := UpdateLocal(__m, __frame + 0, a);
    assume $DebugTrackLocal(0, 3, 0, 547, a);
    assume IsValidU64(b);
    __m := UpdateLocal(__m, __frame + 1, b);
    assume $DebugTrackLocal(0, 3, 1, 547, b);

    // increase the local counter
    __local_counter := __local_counter + 21;

    // bytecode translation starts here
    call __tmp := LdConst(6);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := LdConst(4);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 3), GetLocal(__m, __frame + 4));
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 3, 627);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 6, __tmp);

    call __tmp := Sub(GetLocal(__m, __frame + 5), GetLocal(__m, __frame + 6));
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 3, 627);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 7, __tmp);

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 8, __tmp);

    call __tmp := MulU64(GetLocal(__m, __frame + 7), GetLocal(__m, __frame + 8));
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 3, 626);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 9, __tmp);

    call __tmp := LdConst(3);
    __m := UpdateLocal(__m, __frame + 10, __tmp);

    call __tmp := Div(GetLocal(__m, __frame + 9), GetLocal(__m, __frame + 10));
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 3, 626);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 11, __tmp);

    call __tmp := LdConst(4);
    __m := UpdateLocal(__m, __frame + 12, __tmp);

    call __tmp := Mod(GetLocal(__m, __frame + 11), GetLocal(__m, __frame + 12));
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 3, 626);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 13, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 13));
    __m := UpdateLocal(__m, __frame + 2, __tmp);
    assume $DebugTrackLocal(0, 3, 2, 622, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 14, __tmp);

    call __tmp := LdConst(2);
    __m := UpdateLocal(__m, __frame + 15, __tmp);

    __tmp := Boolean(IsEqual(GetLocal(__m, __frame + 14), GetLocal(__m, __frame + 15)));
    __m := UpdateLocal(__m, __frame + 16, __tmp);

    call __tmp := Not(GetLocal(__m, __frame + 16));
    __m := UpdateLocal(__m, __frame + 17, __tmp);

    __tmp := GetLocal(__m, __frame + 17);
    if (!b#Boolean(__tmp)) { goto Label_19; }

    call __tmp := LdConst(42);
    __m := UpdateLocal(__m, __frame + 18, __tmp);

    goto Label_Abort;

Label_19:
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 19, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 20, __tmp);

    __ret0 := GetLocal(__m, __frame + 19);
    assume $DebugTrackLocal(0, 3, 3, 686, __ret0);
    __ret1 := GetLocal(__m, __frame + 20);
    assume $DebugTrackLocal(0, 3, 4, 686, __ret1);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    __ret1 := DefaultValue;
}

procedure TestArithmetic_arithmetic_ops_verify (a: Value, b: Value) returns (__ret0: Value, __ret1: Value)
{
    call InitVerification();
    call __ret0, __ret1 := TestArithmetic_arithmetic_ops(a, b);
}

procedure {:inline 1} TestArithmetic_overflow () returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var x: Value; // IntegerType()
    var y: Value; // IntegerType()
    var __t2: Value; // IntegerType()
    var __t3: Value; // IntegerType()
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

    // increase the local counter
    __local_counter := __local_counter + 6;

    // bytecode translation starts here
    call __tmp := LdConst(9223372036854775807);
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(0, 4, 0, 771, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := AddU64(GetLocal(__m, __frame + 3), GetLocal(__m, __frame + 4));
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 4, 803);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(0, 4, 1, 799, __tmp);

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure TestArithmetic_overflow_verify () returns ()
{
    call InitVerification();
    call TestArithmetic_overflow();
}

procedure {:inline 1} TestArithmetic_underflow () returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var x: Value; // IntegerType()
    var y: Value; // IntegerType()
    var __t2: Value; // IntegerType()
    var __t3: Value; // IntegerType()
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

    // increase the local counter
    __local_counter := __local_counter + 6;

    // bytecode translation starts here
    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(0, 5, 0, 887, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := Sub(GetLocal(__m, __frame + 3), GetLocal(__m, __frame + 4));
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 5, 901);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(0, 5, 1, 897, __tmp);

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure TestArithmetic_underflow_verify () returns ()
{
    call InitVerification();
    call TestArithmetic_underflow();
}

procedure {:inline 1} TestArithmetic_div_by_zero () returns ()
requires ExistsTxnSenderAccount(__m, __txn);
{
    // declare local variables
    var x: Value; // IntegerType()
    var y: Value; // IntegerType()
    var __t2: Value; // IntegerType()
    var __t3: Value; // IntegerType()
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

    // increase the local counter
    __local_counter := __local_counter + 6;

    // bytecode translation starts here
    call __tmp := LdConst(0);
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 2));
    __m := UpdateLocal(__m, __frame + 0, __tmp);
    assume $DebugTrackLocal(0, 6, 0, 987, __tmp);

    call __tmp := LdConst(1);
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    call __tmp := Div(GetLocal(__m, __frame + 3), GetLocal(__m, __frame + 4));
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 6, 1001);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 5, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 5));
    __m := UpdateLocal(__m, __frame + 1, __tmp);
    assume $DebugTrackLocal(0, 6, 1, 997, __tmp);

    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
}

procedure TestArithmetic_div_by_zero_verify () returns ()
{
    call InitVerification();
    call TestArithmetic_div_by_zero();
}

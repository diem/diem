

// ** synthetics of module TestMultiplication



// ** structs of module TestMultiplication



// ** functions of module TestMultiplication

procedure {:inline 1} TestMultiplication_overflow_u8_mul_bad (x: Value, y: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(false)))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(false))) ==> __abort_flag;

{
    // declare local variables
    var __t2: Value; // IntegerType()
    var __t3: Value; // IntegerType()
    var __t4: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume IsValidU8(x);
    __m := UpdateLocal(__m, __frame + 0, x);
    assume $DebugTrackLocal(0, 0, 0, 75, x);
    assume IsValidU8(y);
    __m := UpdateLocal(__m, __frame + 1, y);
    assume $DebugTrackLocal(0, 0, 1, 75, y);

    // increase the local counter
    __local_counter := __local_counter + 5;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := MulU8(GetLocal(__m, __frame + 2), GetLocal(__m, __frame + 3));
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 0, 198);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    __ret0 := GetLocal(__m, __frame + 4);
    assume $DebugTrackLocal(0, 0, 2, 191, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure TestMultiplication_overflow_u8_mul_bad_verify (x: Value, y: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := TestMultiplication_overflow_u8_mul_bad(x, y);
}

procedure {:inline 1} TestMultiplication_overflow_u8_mul_ok (x: Value, y: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(i#Integer(Integer(i#Integer(x) * i#Integer(y))) > i#Integer(Integer(255)))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(Integer(i#Integer(x) * i#Integer(y))) > i#Integer(Integer(255))))) ==> __abort_flag;

{
    // declare local variables
    var __t2: Value; // IntegerType()
    var __t3: Value; // IntegerType()
    var __t4: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume IsValidU8(x);
    __m := UpdateLocal(__m, __frame + 0, x);
    assume $DebugTrackLocal(0, 1, 0, 273, x);
    assume IsValidU8(y);
    __m := UpdateLocal(__m, __frame + 1, y);
    assume $DebugTrackLocal(0, 1, 1, 273, y);

    // increase the local counter
    __local_counter := __local_counter + 5;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := MulU8(GetLocal(__m, __frame + 2), GetLocal(__m, __frame + 3));
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 1, 380);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    __ret0 := GetLocal(__m, __frame + 4);
    assume $DebugTrackLocal(0, 1, 2, 373, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure TestMultiplication_overflow_u8_mul_ok_verify (x: Value, y: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := TestMultiplication_overflow_u8_mul_ok(x, y);
}

procedure {:inline 1} TestMultiplication_overflow_u64_mul_bad (x: Value, y: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(false)))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(false))) ==> __abort_flag;

{
    // declare local variables
    var __t2: Value; // IntegerType()
    var __t3: Value; // IntegerType()
    var __t4: Value; // IntegerType()
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
    assume $DebugTrackLocal(0, 2, 0, 456, x);
    assume IsValidU64(y);
    __m := UpdateLocal(__m, __frame + 1, y);
    assume $DebugTrackLocal(0, 2, 1, 456, y);

    // increase the local counter
    __local_counter := __local_counter + 5;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := MulU64(GetLocal(__m, __frame + 2), GetLocal(__m, __frame + 3));
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 2, 583);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    __ret0 := GetLocal(__m, __frame + 4);
    assume $DebugTrackLocal(0, 2, 2, 576, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure TestMultiplication_overflow_u64_mul_bad_verify (x: Value, y: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := TestMultiplication_overflow_u64_mul_bad(x, y);
}

procedure {:inline 1} TestMultiplication_overflow_u128_mul_bad (x: Value, y: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(false)))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(false))) ==> __abort_flag;

{
    // declare local variables
    var __t2: Value; // IntegerType()
    var __t3: Value; // IntegerType()
    var __t4: Value; // IntegerType()
    var __tmp: Value;
    var __frame: int;
    var __saved_m: Memory;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;

    // process and type check arguments
    assume IsValidU128(x);
    __m := UpdateLocal(__m, __frame + 0, x);
    assume $DebugTrackLocal(0, 3, 0, 858, x);
    assume IsValidU128(y);
    __m := UpdateLocal(__m, __frame + 1, y);
    assume $DebugTrackLocal(0, 3, 1, 858, y);

    // increase the local counter
    __local_counter := __local_counter + 5;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := MulU128(GetLocal(__m, __frame + 2), GetLocal(__m, __frame + 3));
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 3, 989);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    __ret0 := GetLocal(__m, __frame + 4);
    assume $DebugTrackLocal(0, 3, 2, 982, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure TestMultiplication_overflow_u128_mul_bad_verify (x: Value, y: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := TestMultiplication_overflow_u128_mul_bad(x, y);
}

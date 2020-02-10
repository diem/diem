

// ** synthetics of module CastBad



// ** structs of module CastBad



// ** functions of module CastBad

procedure {:inline 1} CastBad_aborting_u8_cast_bad (x: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(false)))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(false))) ==> __abort_flag;

{
    // declare local variables
    var __t1: Value; // IntegerType()
    var __t2: Value; // IntegerType()
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
    assume $DebugTrackLocal(0, 0, 0, 71, x);

    // increase the local counter
    __local_counter := __local_counter + 3;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := CastU8(GetLocal(__m, __frame + 1));
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 0, 189);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    __ret0 := GetLocal(__m, __frame + 2);
    assume $DebugTrackLocal(0, 0, 1, 182, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure CastBad_aborting_u8_cast_bad_verify (x: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := CastBad_aborting_u8_cast_bad(x);
}

procedure {:inline 1} CastBad_aborting_u8_cast_ok (x: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(i#Integer(x) > i#Integer(Integer(255)))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(x) > i#Integer(Integer(255))))) ==> __abort_flag;

{
    // declare local variables
    var __t1: Value; // IntegerType()
    var __t2: Value; // IntegerType()
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
    assume $DebugTrackLocal(0, 1, 0, 261, x);

    // increase the local counter
    __local_counter := __local_counter + 3;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := CastU8(GetLocal(__m, __frame + 1));
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 1, 357);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    __ret0 := GetLocal(__m, __frame + 2);
    assume $DebugTrackLocal(0, 1, 1, 350, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure CastBad_aborting_u8_cast_ok_verify (x: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := CastBad_aborting_u8_cast_ok(x);
}

procedure {:inline 1} CastBad_aborting_u64_cast_bad (x: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(false)))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(false))) ==> __abort_flag;

{
    // declare local variables
    var __t1: Value; // IntegerType()
    var __t2: Value; // IntegerType()
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
    assume $DebugTrackLocal(0, 2, 0, 433, x);

    // increase the local counter
    __local_counter := __local_counter + 3;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := CastU64(GetLocal(__m, __frame + 1));
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 2, 554);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    __ret0 := GetLocal(__m, __frame + 2);
    assume $DebugTrackLocal(0, 2, 1, 547, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure CastBad_aborting_u64_cast_bad_verify (x: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := CastBad_aborting_u64_cast_bad(x);
}

procedure {:inline 1} CastBad_aborting_u64_cast_ok (x: Value) returns (__ret0: Value)
requires ExistsTxnSenderAccount(__m, __txn);
ensures old(!(b#Boolean(Boolean(i#Integer(x) > i#Integer(Integer(9223372036854775807)))))) ==> !__abort_flag;
ensures old(b#Boolean(Boolean(i#Integer(x) > i#Integer(Integer(9223372036854775807))))) ==> __abort_flag;

{
    // declare local variables
    var __t1: Value; // IntegerType()
    var __t2: Value; // IntegerType()
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
    assume $DebugTrackLocal(0, 3, 0, 627, x);

    // increase the local counter
    __local_counter := __local_counter + 3;

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 1, __tmp);

    call __tmp := CastU64(GetLocal(__m, __frame + 1));
    if (__abort_flag) {
      assume $DebugTrackAbort(0, 3, 762);
      goto Label_Abort;
    }
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    __ret0 := GetLocal(__m, __frame + 2);
    assume $DebugTrackLocal(0, 3, 1, 755, __ret0);
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
}

procedure CastBad_aborting_u64_cast_ok_verify (x: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := CastBad_aborting_u64_cast_ok(x);
}

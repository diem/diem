

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
    var debug#TestMultiplication#overflow_u8_mul_bad#0#x: [Position]Value;
    var debug#TestMultiplication#overflow_u8_mul_bad#1#y: [Position]Value;
    var debug#TestMultiplication#overflow_u8_mul_bad#2#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 5;
    debug#TestMultiplication#overflow_u8_mul_bad#0#x := EmptyPositionMap;
    debug#TestMultiplication#overflow_u8_mul_bad#1#y := EmptyPositionMap;
    debug#TestMultiplication#overflow_u8_mul_bad#2#__ret := EmptyPositionMap;

    // process and type check arguments
    assume IsValidU8(x);
    __m := UpdateLocal(__m, __frame + 0, x);
    debug#TestMultiplication#overflow_u8_mul_bad#0#x := debug#TestMultiplication#overflow_u8_mul_bad#0#x[Position(75) := x];
    assume IsValidU8(y);
    __m := UpdateLocal(__m, __frame + 1, y);
    debug#TestMultiplication#overflow_u8_mul_bad#1#y := debug#TestMultiplication#overflow_u8_mul_bad#1#y[Position(75) := y];

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := MulU8(GetLocal(__m, __frame + 2), GetLocal(__m, __frame + 3));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    __ret0 := GetLocal(__m, __frame + 4);
    debug#TestMultiplication#overflow_u8_mul_bad#2#__ret := debug#TestMultiplication#overflow_u8_mul_bad#2#__ret[Position(191) := __ret0];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#TestMultiplication#overflow_u8_mul_bad#2#__ret := debug#TestMultiplication#overflow_u8_mul_bad#2#__ret[Position(221) := __ret0];
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
    var debug#TestMultiplication#overflow_u8_mul_ok#0#x: [Position]Value;
    var debug#TestMultiplication#overflow_u8_mul_ok#1#y: [Position]Value;
    var debug#TestMultiplication#overflow_u8_mul_ok#2#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 5;
    debug#TestMultiplication#overflow_u8_mul_ok#0#x := EmptyPositionMap;
    debug#TestMultiplication#overflow_u8_mul_ok#1#y := EmptyPositionMap;
    debug#TestMultiplication#overflow_u8_mul_ok#2#__ret := EmptyPositionMap;

    // process and type check arguments
    assume IsValidU8(x);
    __m := UpdateLocal(__m, __frame + 0, x);
    debug#TestMultiplication#overflow_u8_mul_ok#0#x := debug#TestMultiplication#overflow_u8_mul_ok#0#x[Position(273) := x];
    assume IsValidU8(y);
    __m := UpdateLocal(__m, __frame + 1, y);
    debug#TestMultiplication#overflow_u8_mul_ok#1#y := debug#TestMultiplication#overflow_u8_mul_ok#1#y[Position(273) := y];

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := MulU8(GetLocal(__m, __frame + 2), GetLocal(__m, __frame + 3));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    __ret0 := GetLocal(__m, __frame + 4);
    debug#TestMultiplication#overflow_u8_mul_ok#2#__ret := debug#TestMultiplication#overflow_u8_mul_ok#2#__ret[Position(373) := __ret0];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#TestMultiplication#overflow_u8_mul_ok#2#__ret := debug#TestMultiplication#overflow_u8_mul_ok#2#__ret[Position(403) := __ret0];
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
    var debug#TestMultiplication#overflow_u64_mul_bad#0#x: [Position]Value;
    var debug#TestMultiplication#overflow_u64_mul_bad#1#y: [Position]Value;
    var debug#TestMultiplication#overflow_u64_mul_bad#2#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 5;
    debug#TestMultiplication#overflow_u64_mul_bad#0#x := EmptyPositionMap;
    debug#TestMultiplication#overflow_u64_mul_bad#1#y := EmptyPositionMap;
    debug#TestMultiplication#overflow_u64_mul_bad#2#__ret := EmptyPositionMap;

    // process and type check arguments
    assume IsValidU64(x);
    __m := UpdateLocal(__m, __frame + 0, x);
    debug#TestMultiplication#overflow_u64_mul_bad#0#x := debug#TestMultiplication#overflow_u64_mul_bad#0#x[Position(456) := x];
    assume IsValidU64(y);
    __m := UpdateLocal(__m, __frame + 1, y);
    debug#TestMultiplication#overflow_u64_mul_bad#1#y := debug#TestMultiplication#overflow_u64_mul_bad#1#y[Position(456) := y];

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := MulU64(GetLocal(__m, __frame + 2), GetLocal(__m, __frame + 3));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    __ret0 := GetLocal(__m, __frame + 4);
    debug#TestMultiplication#overflow_u64_mul_bad#2#__ret := debug#TestMultiplication#overflow_u64_mul_bad#2#__ret[Position(576) := __ret0];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#TestMultiplication#overflow_u64_mul_bad#2#__ret := debug#TestMultiplication#overflow_u64_mul_bad#2#__ret[Position(606) := __ret0];
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
    var debug#TestMultiplication#overflow_u128_mul_bad#0#x: [Position]Value;
    var debug#TestMultiplication#overflow_u128_mul_bad#1#y: [Position]Value;
    var debug#TestMultiplication#overflow_u128_mul_bad#2#__ret: [Position]Value;

    // initialize function execution
    assume !__abort_flag;
    __saved_m := __m;
    __frame := __local_counter;
    __local_counter := __local_counter + 5;
    debug#TestMultiplication#overflow_u128_mul_bad#0#x := EmptyPositionMap;
    debug#TestMultiplication#overflow_u128_mul_bad#1#y := EmptyPositionMap;
    debug#TestMultiplication#overflow_u128_mul_bad#2#__ret := EmptyPositionMap;

    // process and type check arguments
    assume IsValidU128(x);
    __m := UpdateLocal(__m, __frame + 0, x);
    debug#TestMultiplication#overflow_u128_mul_bad#0#x := debug#TestMultiplication#overflow_u128_mul_bad#0#x[Position(858) := x];
    assume IsValidU128(y);
    __m := UpdateLocal(__m, __frame + 1, y);
    debug#TestMultiplication#overflow_u128_mul_bad#1#y := debug#TestMultiplication#overflow_u128_mul_bad#1#y[Position(858) := y];

    // bytecode translation starts here
    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 0));
    __m := UpdateLocal(__m, __frame + 2, __tmp);

    call __tmp := CopyOrMoveValue(GetLocal(__m, __frame + 1));
    __m := UpdateLocal(__m, __frame + 3, __tmp);

    call __tmp := MulU128(GetLocal(__m, __frame + 2), GetLocal(__m, __frame + 3));
    if (__abort_flag) { goto Label_Abort; }
    __m := UpdateLocal(__m, __frame + 4, __tmp);

    __ret0 := GetLocal(__m, __frame + 4);
    debug#TestMultiplication#overflow_u128_mul_bad#2#__ret := debug#TestMultiplication#overflow_u128_mul_bad#2#__ret[Position(982) := __ret0];
    return;

Label_Abort:
    __abort_flag := true;
    __m := __saved_m;
    __ret0 := DefaultValue;
    debug#TestMultiplication#overflow_u128_mul_bad#2#__ret := debug#TestMultiplication#overflow_u128_mul_bad#2#__ret[Position(1012) := __ret0];
}

procedure TestMultiplication_overflow_u128_mul_bad_verify (x: Value, y: Value) returns (__ret0: Value)
{
    call InitVerification();
    call __ret0 := TestMultiplication_overflow_u128_mul_bad(x, y);
}
